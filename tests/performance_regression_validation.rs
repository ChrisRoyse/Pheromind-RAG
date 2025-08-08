//! Performance Regression Tests for Transformer Attention and Embedding Pipeline
//!
//! These tests verify that attention computation and the complete embedding pipeline
//! meet performance requirements and detect regressions. CRITICAL: These tests will
//! expose performance issues caused by broken quantization or inefficient attention.

#[cfg(feature = "ml")]
use embed_search::embedding::NomicEmbedder;
#[cfg(feature = "ml")]
use std::time::{Instant, Duration};
#[cfg(feature = "ml")]
use std::collections::HashMap;

/// Performance benchmark result
#[cfg(feature = "ml")]
#[derive(Debug, Clone)]
struct PerformanceBenchmark {
    test_name: String,
    duration: Duration,
    embeddings_per_second: f64,
    memory_usage_mb: f64,
    success: bool,
    error_message: Option<String>,
}

/// Test single embedding performance
/// CRITICAL: Broken quantization can cause significant slowdowns
#[cfg(feature = "ml")]
#[tokio::test]
async fn test_single_embedding_performance() {
    let embedder = NomicEmbedder::get_global().await.unwrap();
    
    // Test cases with different complexities
    let performance_tests = vec![
        (
            "short_text",
            "hello",
            Duration::from_millis(100), // Max 100ms for short text
        ),
        (
            "medium_text", 
            "def authenticate_user(username, password): return bcrypt.checkpw(password, stored_hash)",
            Duration::from_millis(200), // Max 200ms for medium text
        ),
        (
            "long_text",
            "class DatabaseManager { constructor(config) { this.config = config; this.pool = createConnectionPool(config); } async query(sql, params) { const conn = await this.pool.getConnection(); try { const result = await conn.execute(sql, params); return result; } finally { conn.release(); } } }",
            Duration::from_millis(500), // Max 500ms for long text
        ),
        (
            "very_long_text",
            "function processComplexData(input) { const results = []; for (let i = 0; i < input.length; i++) { const item = input[i]; if (item.type === 'user') { results.push(processUser(item)); } else if (item.type === 'order') { results.push(processOrder(item)); } else if (item.type === 'product') { results.push(processProduct(item)); } else { results.push(processGeneric(item)); } } return results.sort((a, b) => a.priority - b.priority); }".repeat(3),
            Duration::from_millis(1000), // Max 1s for very long text
        ),
    ];
    
    let mut results = Vec::new();
    
    for (test_name, text, max_duration) in performance_tests {
        println!("Running performance test: {}", test_name);
        
        // Warm-up (model loading, cache warming)
        let _ = embedder.embed("warmup").unwrap();
        
        // Measure performance
        let start = Instant::now();
        let embedding_result = embedder.embed(text);
        let duration = start.elapsed();
        
        let benchmark = match embedding_result {
            Ok(embedding) => {
                // Validate the embedding
                assert_eq!(embedding.len(), 768, "Wrong embedding dimensions for {}", test_name);
                
                let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
                assert!((norm - 1.0).abs() < 0.01, "Poor normalization for {}: {}", test_name, norm);
                
                let embeddings_per_second = 1.0 / duration.as_secs_f64();
                
                PerformanceBenchmark {
                    test_name: test_name.to_string(),
                    duration,
                    embeddings_per_second,
                    memory_usage_mb: 0.0, // TODO: measure actual memory
                    success: true,
                    error_message: None,
                }
            }
            Err(e) => {
                PerformanceBenchmark {
                    test_name: test_name.to_string(),
                    duration,
                    embeddings_per_second: 0.0,
                    memory_usage_mb: 0.0,
                    success: false,
                    error_message: Some(e.to_string()),
                }
            }
        };
        
        // CRITICAL PERFORMANCE TEST
        assert!(duration <= max_duration,
            "❌ PERFORMANCE REGRESSION: {} took {}ms, expected <= {}ms\n  \
             This indicates performance degradation in attention computation or quantization.\n  \
             Text length: {} characters",
            test_name, duration.as_millis(), max_duration.as_millis(), text.len());
        
        assert!(benchmark.success,
            "❌ FAILED: {} failed with error: {:?}",
            test_name, benchmark.error_message);
        
        results.push(benchmark);
        
        println!("  ✓ {}: {}ms ({:.1} emb/sec)", test_name, duration.as_millis(), 1.0 / duration.as_secs_f64());
    }
    
    println!("✅ Single embedding performance tests passed!");
    
    // Performance summary
    for result in &results {
        if result.success {
            println!("  {}: {}ms ({:.1} emb/sec)",
                    result.test_name, result.duration.as_millis(), result.embeddings_per_second);
        }
    }
}

/// Test batch embedding performance and throughput
/// CRITICAL: Broken implementation may have poor batch performance
#[cfg(feature = "ml")]
#[tokio::test]
async fn test_batch_embedding_performance() {
    let embedder = NomicEmbedder::get_global().await.unwrap();
    
    // Test different batch sizes
    let batch_tests = vec![
        (10, Duration::from_secs(2)),   // 10 embeddings in <= 2s (5 emb/sec minimum)
        (50, Duration::from_secs(8)),   // 50 embeddings in <= 8s (6.25 emb/sec minimum)  
        (100, Duration::from_secs(15)), // 100 embeddings in <= 15s (6.67 emb/sec minimum)
    ];
    
    // Generate test texts
    let base_texts = vec![
        "def calculate_sum(a, b): return a + b",
        "function multiply(x, y) { return x * y; }",
        "SELECT * FROM users WHERE active = true",
        "class User { constructor(name) { this.name = name; } }",
        "for i in range(10): print(i)",
    ];
    
    for (batch_size, max_duration) in batch_tests {
        println!("Running batch performance test: {} embeddings", batch_size);
        
        // Create batch by repeating and varying base texts
        let batch_texts: Vec<String> = (0..batch_size)
            .map(|i| format!("{} // variant {}", base_texts[i % base_texts.len()], i))
            .collect();
        let batch_refs: Vec<&str> = batch_texts.iter().map(|s| s.as_str()).collect();
        
        // Measure batch performance
        let start = Instant::now();
        let batch_result = embedder.embed_batch(&batch_refs);
        let duration = start.elapsed();
        
        assert!(batch_result.is_ok(),
            "❌ FAILED: Batch embedding failed with {} texts: {:?}",
            batch_size, batch_result.err());
        
        let embeddings = batch_result.unwrap();
        
        // Validate batch results
        assert_eq!(embeddings.len(), batch_size,
            "Batch size mismatch: got {}, expected {}", embeddings.len(), batch_size);
        
        for (i, embedding) in embeddings.iter().enumerate() {
            assert_eq!(embedding.len(), 768,
                "Wrong dimensions for embedding {}: got {}, expected 768", i, embedding.len());
            
            let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
            assert!((norm - 1.0).abs() < 0.01,
                "Poor normalization for embedding {}: norm = {}", i, norm);
        }
        
        // CRITICAL PERFORMANCE TEST  
        assert!(duration <= max_duration,
            "❌ BATCH PERFORMANCE REGRESSION: {} embeddings took {}ms, expected <= {}ms\n  \
             This indicates poor batch processing performance.\n  \
             Throughput: {:.2} emb/sec (expected >= {:.2} emb/sec)",
            batch_size, duration.as_millis(), max_duration.as_millis(),
            batch_size as f64 / duration.as_secs_f64(),
            batch_size as f64 / max_duration.as_secs_f64());
        
        let throughput = batch_size as f64 / duration.as_secs_f64();
        println!("  ✓ {} embeddings: {}ms ({:.2} emb/sec)", 
                batch_size, duration.as_millis(), throughput);
    }
    
    println!("✅ Batch embedding performance tests passed!");
}

/// Test performance scaling with input length
/// Performance should scale reasonably with sequence length
#[cfg(feature = "ml")]
#[tokio::test]
async fn test_performance_scaling_with_length() {
    let embedder = NomicEmbedder::get_global().await.unwrap();
    
    // Generate texts of different lengths
    let base_code = "function processItem(item) { return item.value * 2; }";
    let scaling_tests = vec![
        (1, base_code.to_string()),
        (2, base_code.repeat(2)),
        (4, base_code.repeat(4)),
        (8, base_code.repeat(8)),
    ];
    
    let mut performance_data = Vec::new();
    
    for (multiplier, text) in scaling_tests {
        let token_count_estimate = text.len() / 4; // Rough estimate: 4 chars per token
        
        println!("Testing scaling: {}x length (~{} tokens)", multiplier, token_count_estimate);
        
        let start = Instant::now();
        let result = embedder.embed(&text);
        let duration = start.elapsed();
        
        assert!(result.is_ok(),
            "❌ FAILED: Embedding failed for {}x length: {:?}", multiplier, result.err());
        
        let embedding = result.unwrap();
        
        // Validate result
        assert_eq!(embedding.len(), 768);
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 0.01, "Poor normalization for {}x: {}", multiplier, norm);
        
        performance_data.push((multiplier, token_count_estimate, duration));
        
        println!("  {}x length: {}ms (~{} tokens)", 
                multiplier, duration.as_millis(), token_count_estimate);
    }
    
    // Analyze scaling behavior
    // Performance should not degrade exponentially (O(n^2) attention is acceptable)
    for i in 1..performance_data.len() {
        let (mult_curr, tokens_curr, dur_curr) = &performance_data[i];
        let (mult_prev, tokens_prev, dur_prev) = &performance_data[i - 1];
        
        let length_ratio = *tokens_curr as f64 / *tokens_prev as f64;
        let time_ratio = dur_curr.as_secs_f64() / dur_prev.as_secs_f64();
        
        // Time should not scale worse than O(n^2.5)
        let max_acceptable_ratio = length_ratio.powf(2.5);
        
        assert!(time_ratio <= max_acceptable_ratio,
            "❌ PERFORMANCE SCALING ISSUE: {}x -> {}x length\n  \
             Length ratio: {:.2}x, Time ratio: {:.2}x (max acceptable: {:.2}x)\n  \
             This indicates poor attention scaling or quantization inefficiency.",
            mult_prev, mult_curr, length_ratio, time_ratio, max_acceptable_ratio);
        
        println!("  Scaling {}x -> {}x: length {:.2}x, time {:.2}x ✓", 
                mult_prev, mult_curr, length_ratio, time_ratio);
    }
    
    println!("✅ Performance scaling tests passed!");
}

/// Test memory usage efficiency
/// Memory usage should be reasonable and not leak
#[cfg(feature = "ml")]
#[tokio::test]
async fn test_memory_usage_efficiency() {
    use sysinfo::{System, Pid};
    
    let mut system = System::new();
    system.refresh_processes();
    
    let pid = Pid::from_u32(std::process::id());
    let get_memory = || {
        system.refresh_processes();
        system.process(pid)
            .map(|p| p.memory() as f64 / 1024.0) // Convert to MB
            .unwrap_or(0.0)
    };
    
    let initial_memory = get_memory();
    println!("Initial memory usage: {:.1} MB", initial_memory);
    
    let embedder = NomicEmbedder::get_global().await.unwrap();
    let after_init_memory = get_memory();
    let init_overhead = after_init_memory - initial_memory;
    
    println!("Memory after embedder init: {:.1} MB (+{:.1} MB)", after_init_memory, init_overhead);
    
    // Test memory usage during embedding generation
    let test_texts: Vec<String> = (0..1000)
        .map(|i| format!("function test{} () {{ return {}; }}", i, i))
        .collect();
    let text_refs: Vec<&str> = test_texts.iter().map(|s| s.as_str()).collect();
    
    let before_embeddings_memory = get_memory();
    
    let start = Instant::now();
    let embeddings_result = embedder.embed_batch(&text_refs);
    let embedding_duration = start.elapsed();
    
    assert!(embeddings_result.is_ok(),
        "❌ FAILED: Batch embedding failed during memory test: {:?}", embeddings_result.err());
    
    let embeddings = embeddings_result.unwrap();
    let after_embeddings_memory = get_memory();
    let embedding_overhead = after_embeddings_memory - before_embeddings_memory;
    
    println!("Memory after 1000 embeddings: {:.1} MB (+{:.1} MB)", 
            after_embeddings_memory, embedding_overhead);
    println!("Embedding generation took: {}ms", embedding_duration.as_millis());
    
    // Validate embeddings were generated correctly
    assert_eq!(embeddings.len(), 1000);
    for (i, embedding) in embeddings.iter().enumerate() {
        assert_eq!(embedding.len(), 768);
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 0.01, "Poor normalization for embedding {}: {}", i, norm);
    }
    
    // CRITICAL MEMORY TESTS
    assert!(init_overhead < 2048.0,
        "❌ EXCESSIVE MEMORY: Embedder initialization used {:.1} MB, expected < 2048 MB\n  \
         This suggests memory inefficiency in model loading or quantization.",
        init_overhead);
    
    assert!(embedding_overhead < 1024.0,
        "❌ MEMORY LEAK: Embedding 1000 texts used {:.1} MB additional memory, expected < 1024 MB\n  \
         This suggests memory leaks in the embedding pipeline.",
        embedding_overhead);
    
    // Test memory efficiency per embedding
    let memory_per_embedding = embedding_overhead / 1000.0;
    assert!(memory_per_embedding < 0.5,
        "❌ INEFFICIENT MEMORY: {:.3} MB per embedding, expected < 0.5 MB\n  \
         This suggests inefficient memory usage in the embedding process.",
        memory_per_embedding);
    
    // Force garbage collection and check for memory cleanup
    drop(embeddings);
    std::thread::sleep(Duration::from_millis(100)); // Give GC a chance
    
    let after_cleanup_memory = get_memory();
    let cleanup_reduction = after_embeddings_memory - after_cleanup_memory;
    
    println!("Memory after cleanup: {:.1} MB (-{:.1} MB)", 
            after_cleanup_memory, cleanup_reduction);
    
    println!("✅ Memory usage efficiency tests passed!");
    println!("  - Init overhead: {:.1} MB", init_overhead);
    println!("  - Embedding overhead: {:.1} MB ({:.3} MB per embedding)", 
            embedding_overhead, memory_per_embedding);
    println!("  - Memory cleanup: {:.1} MB recovered", cleanup_reduction);
}

/// Test concurrent embedding performance
/// Multiple simultaneous embedding requests should be handled efficiently
#[cfg(feature = "ml")]
#[tokio::test]
async fn test_concurrent_embedding_performance() {
    let embedder = NomicEmbedder::get_global().await.unwrap();
    
    // Test concurrent embedding requests
    let concurrent_tests = vec![2, 4, 8];
    
    for num_concurrent in concurrent_tests {
        println!("Testing {} concurrent embedding requests", num_concurrent);
        
        let test_texts: Vec<String> = (0..num_concurrent)
            .map(|i| format!("async function process{}(data) {{ return await transform(data); }}", i))
            .collect();
        
        let start = Instant::now();
        
        // Create concurrent tasks
        let mut tasks = Vec::new();
        for text in &test_texts {
            let embedder_ref = embedder.clone();
            let text_clone = text.clone();
            
            let task = tokio::spawn(async move {
                embedder_ref.embed(&text_clone)
            });
            
            tasks.push(task);
        }
        
        // Wait for all tasks to complete
        let mut results = Vec::new();
        for task in tasks {
            let result = task.await.expect("Task should not panic");
            results.push(result);
        }
        
        let total_duration = start.elapsed();
        
        // Validate all results
        for (i, result) in results.iter().enumerate() {
            assert!(result.is_ok(),
                "❌ FAILED: Concurrent embedding {} failed: {:?}", i, result.as_ref().err());
            
            let embedding = result.as_ref().unwrap();
            assert_eq!(embedding.len(), 768);
            
            let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
            assert!((norm - 1.0).abs() < 0.01,
                "Poor normalization for concurrent embedding {}: {}", i, norm);
        }
        
        let concurrent_throughput = num_concurrent as f64 / total_duration.as_secs_f64();
        
        // Compare with sequential performance
        let sequential_start = Instant::now();
        for text in &test_texts {
            let _ = embedder.embed(text).unwrap();
        }
        let sequential_duration = sequential_start.elapsed();
        let sequential_throughput = num_concurrent as f64 / sequential_duration.as_secs_f64();
        
        // Concurrent should not be dramatically slower than sequential
        // (some overhead is acceptable, but not more than 50% slower)
        let efficiency = concurrent_throughput / sequential_throughput;
        
        assert!(efficiency > 0.5,
            "❌ POOR CONCURRENT PERFORMANCE: {} concurrent requests\n  \
             Concurrent: {:.2} emb/sec, Sequential: {:.2} emb/sec\n  \
             Efficiency: {:.1}% (expected >= 50%)\n  \
             This suggests poor concurrent handling or resource contention.",
            num_concurrent, concurrent_throughput, sequential_throughput, efficiency * 100.0);
        
        println!("  ✓ {} concurrent: {:.2} emb/sec ({:.1}% of sequential efficiency)",
                num_concurrent, concurrent_throughput, efficiency * 100.0);
    }
    
    println!("✅ Concurrent embedding performance tests passed!");
}

/// Performance regression detection with historical baselines
/// Compare current performance against expected baselines
#[cfg(feature = "ml")]
#[tokio::test]
async fn test_performance_regression_detection() {
    let embedder = NomicEmbedder::get_global().await.unwrap();
    
    // Baseline performance expectations (adjust based on hardware)
    let performance_baselines = HashMap::from([
        ("single_short", 50.0),      // >= 50 emb/sec for short text
        ("single_medium", 20.0),     // >= 20 emb/sec for medium text
        ("single_long", 8.0),        // >= 8 emb/sec for long text
        ("batch_small", 30.0),       // >= 30 emb/sec for small batches
        ("batch_medium", 25.0),      // >= 25 emb/sec for medium batches
    ]);
    
    let test_cases = vec![
        ("single_short", vec!["test"], 1),
        ("single_medium", vec!["def authenticate_user(username, password): return validate(username, password)"], 1),
        ("single_long", vec!["class UserManager { constructor(db) { this.db = db; this.cache = new Map(); } async getUser(id) { if (this.cache.has(id)) return this.cache.get(id); const user = await this.db.query('SELECT * FROM users WHERE id = ?', [id]); this.cache.set(id, user); return user; } }"], 1),
        ("batch_small", vec!["test1", "test2", "test3", "test4", "test5"], 5),
        ("batch_medium", vec!["function test() { return 'result'; }"; 20], 20),
    ];
    
    let mut performance_results = HashMap::new();
    
    for (test_name, texts, expected_count) in test_cases {
        let expected_baseline = performance_baselines.get(test_name).copied().unwrap_or(1.0);
        
        println!("Running performance regression test: {}", test_name);
        
        let start = Instant::now();
        
        let results = if texts.len() == 1 {
            // Single embedding
            vec![embedder.embed(texts[0]).unwrap()]
        } else {
            // Batch embedding
            embedder.embed_batch(&texts).unwrap()
        };
        
        let duration = start.elapsed();
        
        // Validate results
        assert_eq!(results.len(), expected_count);
        for (i, embedding) in results.iter().enumerate() {
            assert_eq!(embedding.len(), 768);
            let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
            assert!((norm - 1.0).abs() < 0.01, "Poor normalization for {}, embedding {}: {}", 
                    test_name, i, norm);
        }
        
        let throughput = expected_count as f64 / duration.as_secs_f64();
        performance_results.insert(test_name, throughput);
        
        // CRITICAL REGRESSION TEST
        assert!(throughput >= expected_baseline,
            "❌ PERFORMANCE REGRESSION DETECTED: {}\n  \
             Current: {:.2} emb/sec, Expected: >= {:.2} emb/sec\n  \
             Regression: {:.1}%\n  \
             This indicates a significant performance degradation.",
            test_name, throughput, expected_baseline, 
            (1.0 - throughput / expected_baseline) * 100.0);
        
        let performance_ratio = throughput / expected_baseline;
        println!("  ✓ {}: {:.2} emb/sec ({:.1}% of baseline)", 
                test_name, throughput, performance_ratio * 100.0);
    }
    
    println!("✅ Performance regression detection passed!");
    
    // Summary report
    println!("\n📊 Performance Summary:");
    for (test_name, throughput) in &performance_results {
        let baseline = performance_baselines.get(test_name).copied().unwrap_or(1.0);
        let ratio = throughput / baseline;
        let status = if ratio >= 1.0 { "✓" } else { "⚠" };
        
        println!("  {} {}: {:.2} emb/sec ({:.1}% of baseline)",
                status, test_name, throughput, ratio * 100.0);
    }
}

#[cfg(not(feature = "ml"))]
mod no_ml_tests {
    #[test]
    fn test_performance_regression_requires_ml_feature() {
        println!("Performance regression tests require 'ml' feature to be enabled");
        println!("Run with: cargo test --features ml");
    }
}