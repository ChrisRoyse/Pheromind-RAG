//! Concurrency and Stress Testing Validation
//! 
//! This module tests the system under concurrent load conditions to validate
//! thread safety, performance under stress, and proper resource management.

use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};
use std::time::{Instant, Duration};
use tokio::sync::{RwLock, Semaphore};
use anyhow::Result;
use futures::future::join_all;

#[cfg(feature = "ml")]
use embed_search::embedding::NomicEmbedder;
#[cfg(feature = "vectordb")]
use embed_search::storage::lancedb_storage::{LanceDBStorage, LanceEmbeddingRecord};
use embed_search::search::bm25::{BM25Engine, BM25Document, Token};
use embed_search::search::text_processor::CodeTextProcessor;
use embed_search::chunking::SimpleRegexChunker;
use tempfile::TempDir;

/// Stress test configuration
#[derive(Debug, Clone)]
pub struct StressTestConfig {
    pub concurrent_tasks: usize,
    pub operations_per_task: usize,
    pub max_duration_secs: u64,
    pub memory_limit_mb: f64,
}

impl Default for StressTestConfig {
    fn default() -> Self {
        Self {
            concurrent_tasks: 10,
            operations_per_task: 50,
            max_duration_secs: 60,
            memory_limit_mb: 500.0,
        }
    }
}

/// Thread-safe metrics collector
#[derive(Debug)]
pub struct MetricsCollector {
    pub operations_completed: AtomicUsize,
    pub operations_failed: AtomicUsize,
    pub total_duration: Arc<RwLock<Duration>>,
    pub peak_memory_mb: Arc<RwLock<f64>>,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            operations_completed: AtomicUsize::new(0),
            operations_failed: AtomicUsize::new(0),
            total_duration: Arc::new(RwLock::new(Duration::ZERO)),
            peak_memory_mb: Arc::new(RwLock::new(0.0)),
        }
    }
    
    pub async fn record_operation(&self, duration: Duration, success: bool) {
        if success {
            self.operations_completed.fetch_add(1, Ordering::Relaxed);
        } else {
            self.operations_failed.fetch_add(1, Ordering::Relaxed);
        }
        
        let mut total = self.total_duration.write().await;
        *total += duration;
        
        // Update peak memory if needed
        let current_memory = get_current_memory_usage_mb();
        let mut peak = self.peak_memory_mb.write().await;
        if current_memory > *peak {
            *peak = current_memory;
        }
    }
    
    pub async fn get_summary(&self) -> StressTestSummary {
        let completed = self.operations_completed.load(Ordering::Relaxed);
        let failed = self.operations_failed.load(Ordering::Relaxed);
        let total_duration = *self.total_duration.read().await;
        let peak_memory = *self.peak_memory_mb.read().await;
        
        StressTestSummary {
            operations_completed: completed,
            operations_failed: failed,
            success_rate: if completed + failed > 0 {
                completed as f64 / (completed + failed) as f64
            } else { 0.0 },
            average_operation_time: if completed > 0 {
                total_duration / completed as u32
            } else { Duration::ZERO },
            peak_memory_mb: peak_memory,
        }
    }
}

#[derive(Debug)]
pub struct StressTestSummary {
    pub operations_completed: usize,
    pub operations_failed: usize,
    pub success_rate: f64,
    pub average_operation_time: Duration,
    pub peak_memory_mb: f64,
}

#[tokio::test]
#[cfg(feature = "ml")]
async fn test_concurrent_embedding_operations() -> Result<()> {
    println!("🔄 Testing Concurrent Embedding Operations");
    
    let embedder = match NomicEmbedder::get_global().await {
        Ok(e) => e,
        Err(_) => {
            println!("⚠️ Skipping concurrent embedding tests - model not available");
            return Ok(());
        }
    };
    
    let config = StressTestConfig::default();
    let metrics = Arc::new(MetricsCollector::new());
    let semaphore = Arc::new(Semaphore::new(config.concurrent_tasks));
    
    println!("📊 Starting {} concurrent tasks, {} operations each", 
             config.concurrent_tasks, config.operations_per_task);
    
    let test_texts = create_stress_test_texts(100);
    let start_time = Instant::now();
    
    let mut tasks = Vec::new();
    
    for task_id in 0..config.concurrent_tasks {
        let embedder_clone = embedder.clone();
        let metrics_clone = metrics.clone();
        let semaphore_clone = semaphore.clone();
        let texts_clone = test_texts.clone();
        
        let task = tokio::spawn(async move {
            let _permit = semaphore_clone.acquire().await.unwrap();
            
            for operation_id in 0..config.operations_per_task {
                let text_idx = (task_id * config.operations_per_task + operation_id) % texts_clone.len();
                let text = &texts_clone[text_idx];
                
                let op_start = Instant::now();
                let success = match embedder_clone.embed(text) {
                    Ok(embedding) => {
                        // Validate embedding quality
                        embedding.len() == 768 && 
                        embedding.iter().all(|&x| x.is_finite()) &&
                        embedding.iter().any(|&x| x != 0.0)
                    }
                    Err(_) => false,
                };
                let op_duration = op_start.elapsed();
                
                metrics_clone.record_operation(op_duration, success).await;
                
                // Brief pause to simulate real-world usage
                tokio::time::sleep(Duration::from_millis(1)).await;
            }
        });
        
        tasks.push(task);
    }
    
    // Wait for all tasks to complete or timeout
    let timeout_future = tokio::time::sleep(Duration::from_secs(config.max_duration_secs));
    tokio::select! {
        _ = join_all(tasks) => {
            println!("✅ All concurrent tasks completed successfully");
        }
        _ = timeout_future => {
            println!("⚠️ Test timed out after {} seconds", config.max_duration_secs);
        }
    }
    
    let total_duration = start_time.elapsed();
    let summary = metrics.get_summary().await;
    
    println!("📈 Concurrent Embedding Test Results:");
    println!("  ✅ Operations completed: {}", summary.operations_completed);
    println!("  ❌ Operations failed: {}", summary.operations_failed);
    println!("  📊 Success rate: {:.2}%", summary.success_rate * 100.0);
    println!("  ⏱️  Average operation time: {:?}", summary.average_operation_time);
    println!("  💾 Peak memory usage: {:.2} MB", summary.peak_memory_mb);
    println!("  🚀 Total test time: {:?}", total_duration);
    
    // Performance assertions
    assert!(summary.success_rate > 0.95, "Success rate should be > 95%, got {:.2}%", summary.success_rate * 100.0);
    assert!(summary.average_operation_time.as_millis() < 1000, "Average operation time should be < 1s");
    assert!(summary.peak_memory_mb < config.memory_limit_mb, "Memory usage should stay under limit");
    
    println!("✅ Concurrent embedding operations test PASSED");
    Ok(())
}

#[tokio::test]
#[cfg(all(feature = "ml", feature = "vectordb"))]
async fn test_concurrent_vector_storage_operations() -> Result<()> {
    println!("🗄️ Testing Concurrent Vector Storage Operations");
    
    let embedder = match NomicEmbedder::get_global().await {
        Ok(e) => e,
        Err(_) => {
            println!("⚠️ Skipping concurrent storage tests - model not available");
            return Ok(());
        }
    };
    
    let temp_dir = TempDir::new()?;
    let storage = Arc::new(RwLock::new(
        LanceDBStorage::new(&temp_dir.path().join("concurrent_test")).await?
    ));
    
    let config = StressTestConfig { 
        concurrent_tasks: 5,  // Fewer tasks for storage operations
        operations_per_task: 20,
        ..StressTestConfig::default()
    };
    let metrics = Arc::new(MetricsCollector::new());
    
    println!("📊 Testing concurrent vector storage with {} tasks", config.concurrent_tasks);
    
    let test_texts = create_stress_test_texts(50);
    let mut tasks = Vec::new();
    
    for task_id in 0..config.concurrent_tasks {
        let embedder_clone = embedder.clone();
        let storage_clone = storage.clone();
        let metrics_clone = metrics.clone();
        let texts_clone = test_texts.clone();
        
        let task = tokio::spawn(async move {
            for operation_id in 0..config.operations_per_task {
                let text_idx = (task_id * config.operations_per_task + operation_id) % texts_clone.len();
                let text = &texts_clone[text_idx];
                
                let op_start = Instant::now();
                let success = match embedder_clone.embed(text) {
                    Ok(embedding) => {
                        let record = LanceEmbeddingRecord {
                            id: format!("task_{}_{}", task_id, operation_id),
                            text: text.clone(),
                            embedding,
                            file_path: format!("test_file_{}.rs", task_id),
                            line_number: operation_id as i32,
                            chunk_index: 0,
                        };
                        
                        match storage_clone.write().await.add_embedding(record).await {
                            Ok(_) => true,
                            Err(_) => false,
                        }
                    }
                    Err(_) => false,
                };
                let op_duration = op_start.elapsed();
                
                metrics_clone.record_operation(op_duration, success).await;
                
                // Small delay between operations
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        });
        
        tasks.push(task);
    }
    
    let start_time = Instant::now();
    join_all(tasks).await;
    let total_duration = start_time.elapsed();
    
    let summary = metrics.get_summary().await;
    
    println!("📈 Concurrent Vector Storage Test Results:");
    println!("  ✅ Operations completed: {}", summary.operations_completed);
    println!("  ❌ Operations failed: {}", summary.operations_failed);
    println!("  📊 Success rate: {:.2}%", summary.success_rate * 100.0);
    println!("  ⏱️  Average operation time: {:?}", summary.average_operation_time);
    println!("  💾 Peak memory usage: {:.2} MB", summary.peak_memory_mb);
    println!("  🚀 Total test time: {:?}", total_duration);
    
    // Test search functionality after concurrent writes
    println!("🔍 Testing search after concurrent writes");
    let query = "function test example";
    let query_embedding = embedder.embed(query)?;
    let search_results = storage.read().await.search_similar(&query_embedding, 5).await?;
    
    println!("  🎯 Search returned {} results", search_results.len());
    
    // Assertions
    assert!(summary.success_rate > 0.90, "Success rate should be > 90% for storage operations");
    assert!(!search_results.is_empty(), "Search should return results after concurrent writes");
    
    println!("✅ Concurrent vector storage operations test PASSED");
    Ok(())
}

#[tokio::test]
async fn test_concurrent_bm25_operations() -> Result<()> {
    println!("📝 Testing Concurrent BM25 Operations");
    
    let bm25_engine = Arc::new(RwLock::new(BM25Engine::new()));
    let text_processor = CodeTextProcessor::new();
    
    let config = StressTestConfig {
        concurrent_tasks: 8,
        operations_per_task: 25,
        ..StressTestConfig::default()
    };
    let metrics = Arc::new(MetricsCollector::new());
    
    let test_texts = create_stress_test_texts(200);
    
    println!("📊 Testing concurrent BM25 indexing and searching");
    
    // Phase 1: Concurrent indexing
    println!("🏗️ Phase 1: Concurrent indexing");
    let mut indexing_tasks = Vec::new();
    
    for task_id in 0..config.concurrent_tasks {
        let engine_clone = bm25_engine.clone();
        let metrics_clone = metrics.clone();
        let texts_clone = test_texts.clone();
        let processor = text_processor.clone();
        
        let task = tokio::spawn(async move {
            for operation_id in 0..config.operations_per_task {
                let text_idx = (task_id * config.operations_per_task + operation_id) % texts_clone.len();
                let text = &texts_clone[text_idx];
                
                let op_start = Instant::now();
                let success = {
                    let tokens = processor.tokenize_code(text, Some("rust"));
                    let bm25_tokens: Vec<Token> = tokens.into_iter()
                        .map(|t| Token { 
                            text: t.text, 
                            position: t.position, 
                            importance_weight: 1.0 
                        })
                        .collect();
                    
                    let doc = BM25Document {
                        id: format!("doc_{}_{}", task_id, operation_id),
                        file_path: format!("test_{}.rs", task_id),
                        chunk_index: 0,
                        tokens: bm25_tokens,
                        start_line: operation_id + 1,
                        end_line: operation_id + 1,
                        language: Some("rust".to_string()),
                    };
                    
                    match engine_clone.write().await.add_document(doc) {
                        Ok(_) => true,
                        Err(_) => false,
                    }
                };
                let op_duration = op_start.elapsed();
                
                metrics_clone.record_operation(op_duration, success).await;
                
                tokio::time::sleep(Duration::from_millis(2)).await;
            }
        });
        
        indexing_tasks.push(task);
    }
    
    join_all(indexing_tasks).await;
    
    // Phase 2: Concurrent searching
    println!("🔍 Phase 2: Concurrent searching");
    let search_metrics = Arc::new(MetricsCollector::new());
    let mut searching_tasks = Vec::new();
    
    let search_queries = vec![
        "function", "class", "async", "await", "error", "data", "user", "test", "api", "result"
    ];
    
    for task_id in 0..config.concurrent_tasks {
        let engine_clone = bm25_engine.clone();
        let metrics_clone = search_metrics.clone();
        let queries_clone = search_queries.clone();
        
        let task = tokio::spawn(async move {
            for operation_id in 0..config.operations_per_task {
                let query_idx = operation_id % queries_clone.len();
                let query = &queries_clone[query_idx];
                
                let op_start = Instant::now();
                let success = match engine_clone.read().await.search(query, 10) {
                    Ok(results) => {
                        // Validate results are properly formed
                        results.iter().all(|r| !r.doc_id.is_empty() && r.score.is_finite())
                    }
                    Err(_) => false,
                };
                let op_duration = op_start.elapsed();
                
                metrics_clone.record_operation(op_duration, success).await;
                
                tokio::time::sleep(Duration::from_millis(1)).await;
            }
        });
        
        searching_tasks.push(task);
    }
    
    join_all(searching_tasks).await;
    
    let indexing_summary = metrics.get_summary().await;
    let searching_summary = search_metrics.get_summary().await;
    
    println!("📈 BM25 Indexing Results:");
    println!("  ✅ Operations completed: {}", indexing_summary.operations_completed);
    println!("  📊 Success rate: {:.2}%", indexing_summary.success_rate * 100.0);
    println!("  ⏱️  Average time: {:?}", indexing_summary.average_operation_time);
    
    println!("📈 BM25 Searching Results:");
    println!("  ✅ Operations completed: {}", searching_summary.operations_completed);
    println!("  📊 Success rate: {:.2}%", searching_summary.success_rate * 100.0);
    println!("  ⏱️  Average time: {:?}", searching_summary.average_operation_time);
    
    // Assertions
    assert!(indexing_summary.success_rate > 0.95, "BM25 indexing success rate should be > 95%");
    assert!(searching_summary.success_rate > 0.95, "BM25 searching success rate should be > 95%");
    assert!(searching_summary.average_operation_time.as_millis() < 50, "BM25 searches should be fast");
    
    println!("✅ Concurrent BM25 operations test PASSED");
    Ok(())
}

#[tokio::test]
async fn test_memory_leak_detection() -> Result<()> {
    println!("🔍 Testing Memory Leak Detection");
    
    let initial_memory = get_current_memory_usage_mb();
    println!("📊 Initial memory usage: {:.2} MB", initial_memory);
    
    #[cfg(feature = "ml")]
    {
        let embedder = match NomicEmbedder::get_global().await {
            Ok(e) => e,
            Err(_) => {
                println!("⚠️ Skipping memory leak tests - model not available");
                return Ok(());
            }
        };
        
        let mut memory_samples = Vec::new();
        let test_text = "This is a test string for memory leak detection.";
        
        // Perform many operations and sample memory usage
        for iteration in 0..100 {
            // Generate embeddings
            for _ in 0..10 {
                let _embedding = embedder.embed(test_text)?;
            }
            
            // Sample memory every 10 iterations
            if iteration % 10 == 0 {
                let current_memory = get_current_memory_usage_mb();
                memory_samples.push(current_memory);
                println!("📈 Iteration {}: {:.2} MB", iteration, current_memory);
            }
            
            // Force garbage collection opportunity
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        
        let final_memory = get_current_memory_usage_mb();
        let memory_growth = final_memory - initial_memory;
        
        println!("📊 Final memory usage: {:.2} MB", final_memory);
        println!("📈 Memory growth: {:.2} MB", memory_growth);
        
        // Calculate memory growth trend
        let first_half_avg = memory_samples[0..memory_samples.len()/2].iter().sum::<f64>() / (memory_samples.len()/2) as f64;
        let second_half_avg = memory_samples[memory_samples.len()/2..].iter().sum::<f64>() / (memory_samples.len()/2) as f64;
        let growth_trend = second_half_avg - first_half_avg;
        
        println!("📊 Memory growth trend: {:.2} MB", growth_trend);
        
        // Assertions
        assert!(memory_growth < 100.0, "Memory growth should be less than 100 MB, got {:.2} MB", memory_growth);
        assert!(growth_trend < 50.0, "Memory growth trend should be minimal, got {:.2} MB", growth_trend);
        
        println!("✅ No significant memory leaks detected");
    }
    
    Ok(())
}

/// Create stress test texts with various patterns
fn create_stress_test_texts(count: usize) -> Vec<String> {
    let mut texts = Vec::new();
    let patterns = vec![
        "function test{id}() {{ return 'result{id}'; }}",
        "class Test{id} {{ constructor() {{ this.value = {id}; }} }}",
        "const data{id} = await fetch('/api/endpoint{id}');",
        "if (condition{id}) {{ process{id}(); }}",
        "for (let i = 0; i < {id}; i++) {{ console.log(i); }}",
        "try {{ operation{id}(); }} catch (error) {{ handleError{id}(error); }}",
        "async function processData{id}(input) {{ return input * {id}; }}",
        "const result{id} = items.filter(item => item.id === {id});",
    ];
    
    for i in 0..count {
        let pattern_idx = i % patterns.len();
        let text = patterns[pattern_idx].replace("{id}", &i.to_string());
        texts.push(text);
    }
    
    texts
}

/// Get current memory usage (simplified implementation)
fn get_current_memory_usage_mb() -> f64 {
    use sysinfo::{System, Process};
    
    let mut system = System::new_all();
    system.refresh_memory();
    system.refresh_processes();
    
    if let Some(process) = system.process((std::process::id() as usize).into()) {
        process.memory() as f64 / 1024.0 / 1024.0
    } else {
        system.used_memory() as f64 / 1024.0 / 1024.0
    }
}