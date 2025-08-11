/// COMPONENT BOUNDARY INTEGRATION TESTS
/// Tests critical integration points between system components
/// No tolerance for loose coupling that breaks under real usage

use anyhow::Result;
use embed_search::{
    gguf_embedder::{GGUFEmbedder, GGUFEmbedderConfig},
    embedding_prefixes::{EmbeddingTask, CodeFormatter, BatchProcessor},
    simple_storage::{VectorStorage, SearchResult as VectorResult},
    chunking::{SimpleRegexChunker, MarkdownRegexChunker, Chunk, MarkdownChunk},
    Config,
    llama_wrapper_working::{GGUFModel, GGUFContext},
};
use tempfile::tempdir;
use std::time::Instant;
use std::sync::{Arc, Mutex};
use std::thread;

/// Test GGUF model loading and context creation boundary
#[test]
fn test_gguf_model_context_boundary() -> Result<()> {
    println!("🔥 BOUNDARY TEST: GGUF Model ↔ Context Integration");
    
    let model_path = "./src/model/nomic-embed-text-v1.5.Q4_K_M.gguf";
    
    // Test boundary: Model loading
    println!("🔄 Testing model loading boundary");
    let load_start = Instant::now();
    let model = GGUFModel::load_from_file(model_path, 0)
        .map_err(|e| anyhow::anyhow!("Model loading failed: {}", e))?;
    let load_time = load_start.elapsed();
    
    println!("✅ Model loaded in {:?}", load_time);
    assert!(load_time.as_secs() < 30, "Model loading took too long: {:?}", load_time);
    
    // Validate model properties
    assert!(model.embedding_dim() > 0, "Invalid embedding dimension: {}", model.embedding_dim());
    println!("📐 Model embedding dimension: {}", model.embedding_dim());
    
    // Test boundary: Context creation from model
    println!("🔄 Testing context creation boundary");
    let context_sizes = vec![512, 1024, 2048, 4096];
    
    for context_size in context_sizes {
        let ctx_start = Instant::now();
        let mut context = GGUFContext::new_with_model(&model, context_size)
            .map_err(|e| anyhow::anyhow!("Context creation failed for size {}: {}", context_size, e))?;
        let ctx_time = ctx_start.elapsed();
        
        println!("✅ Context created (size: {}) in {:?}", context_size, ctx_time);
        assert!(ctx_time.as_secs() < 5, "Context creation took too long: {:?}", ctx_time);
        
        // Test embedding generation at boundary
        let test_text = "Test boundary integration";
        let embed_start = Instant::now();
        let embedding = context.embed(test_text)
            .map_err(|e| anyhow::anyhow!("Embedding generation failed: {}", e))?;
        let embed_time = embed_start.elapsed();
        
        assert_eq!(embedding.len(), model.embedding_dim(), 
                  "Embedding dimension mismatch: {} vs {}", embedding.len(), model.embedding_dim());
        assert!(embed_time.as_millis() < 1000, "Embedding generation too slow: {:?}", embed_time);
        
        // Validate embedding quality at boundary
        validate_embedding_at_boundary(&embedding, &format!("context_size_{}", context_size))?;
    }
    
    println!("✅ GGUF Model ↔ Context boundary test passed");
    Ok(())
}

/// Test embedder configuration to model loading boundary
#[test]
fn test_embedder_config_boundary() -> Result<()> {
    println!("🔥 BOUNDARY TEST: EmbedderConfig ↔ GGUFEmbedder Integration");
    
    // Test various configuration combinations
    let test_configs = vec![
        ("minimal", GGUFEmbedderConfig {
            model_path: "./src/model/nomic-embed-text-v1.5.Q4_K_M.gguf".to_string(),
            context_size: 512,
            batch_size: 1,
            cache_size: 10,
            ..Default::default()
        }),
        ("standard", GGUFEmbedderConfig {
            model_path: "./src/model/nomic-embed-text-v1.5.Q4_K_M.gguf".to_string(),
            context_size: 2048,
            batch_size: 8,
            cache_size: 1000,
            ..Default::default()
        }),
        ("large", GGUFEmbedderConfig {
            model_path: "./src/model/nomic-embed-text-v1.5.Q4_K_M.gguf".to_string(),
            context_size: 4096,
            batch_size: 16,
            cache_size: 5000,
            ..Default::default()
        }),
    ];
    
    for (config_name, config) in test_configs {
        println!("🔄 Testing {} configuration boundary", config_name);
        
        let init_start = Instant::now();
        let embedder = GGUFEmbedder::new(config.clone())
            .map_err(|e| anyhow::anyhow!("Embedder initialization failed for {}: {}", config_name, e))?;
        let init_time = init_start.elapsed();
        
        println!("✅ {} embedder initialized in {:?}", config_name, init_time);
        assert!(init_time.as_secs() < 30, "Initialization too slow for {}: {:?}", config_name, init_time);
        
        // Test configuration effects on embedding
        let test_texts = vec![
            "Short text",
            "Medium length text with more words to test configuration effects",
            "Very long text that contains many words and should test the configuration limits including context size handling and batch processing capabilities of the embedder system".to_string(),
        ];
        
        for (i, text) in test_texts.iter().enumerate() {
            let embedding = embedder.embed(text, EmbeddingTask::SearchDocument)
                .map_err(|e| anyhow::anyhow!("Embedding failed for {} with text {}: {}", config_name, i, e))?;
            
            assert_eq!(embedding.len(), 768, "Wrong embedding dimension for {} config", config_name);
            validate_embedding_at_boundary(&embedding, &format!("{}_text_{}", config_name, i))?;
        }
        
        // Test batch processing boundary
        if config.batch_size > 1 {
            println!("🔄 Testing batch boundary for {} config", config_name);
            let batch_texts: Vec<String> = (0..config.batch_size + 2)
                .map(|i| format!("Batch text number {} for testing", i))
                .collect();
            
            let batch_start = Instant::now();
            let batch_embeddings = embedder.embed_batch(batch_texts.clone(), EmbeddingTask::SearchDocument)
                .map_err(|e| anyhow::anyhow!("Batch embedding failed for {}: {}", config_name, e))?;
            let batch_time = batch_start.elapsed();
            
            assert_eq!(batch_embeddings.len(), batch_texts.len(), 
                      "Batch size mismatch for {} config", config_name);
            println!("✅ Batch processing completed for {} in {:?}", config_name, batch_time);
        }
    }
    
    println!("✅ EmbedderConfig ↔ GGUFEmbedder boundary test passed");
    Ok(())
}

/// Test embedding task prefix application boundary
#[test]
fn test_embedding_task_prefix_boundary() -> Result<()> {
    println!("🔥 BOUNDARY TEST: EmbeddingTask ↔ Prefix Application");
    
    let config = GGUFEmbedderConfig {
        model_path: "./src/model/nomic-embed-text-v1.5.Q4_K_M.gguf".to_string(),
        ..Default::default()
    };
    let embedder = GGUFEmbedder::new(config)?;
    
    let base_text = "Fibonacci algorithm implementation";
    
    // Test all embedding tasks
    let tasks = vec![
        EmbeddingTask::SearchQuery,
        EmbeddingTask::SearchDocument,
        EmbeddingTask::CodeDefinition,
        EmbeddingTask::CodeDocumentation,
    ];
    
    let mut embeddings = Vec::new();
    
    for task in &tasks {
        println!("🔄 Testing task boundary: {:?}", task);
        
        // Test prefix application
        let prefixed_text = task.apply_prefix(base_text);
        println!("📝 Prefixed text: {}", prefixed_text);
        
        // Generate embedding with task
        let embedding = embedder.embed(base_text, *task)
            .map_err(|e| anyhow::anyhow!("Embedding failed for task {:?}: {}", task, e))?;
        
        validate_embedding_at_boundary(&embedding, &format!("task_{:?}", task))?;
        embeddings.push((task, embedding));
    }
    
    // Test that different tasks produce different embeddings
    println!("🔄 Testing task differentiation boundary");
    for i in 0..embeddings.len() {
        for j in (i + 1)..embeddings.len() {
            let (task_i, emb_i) = &embeddings[i];
            let (task_j, emb_j) = &embeddings[j];
            
            let similarity = cosine_similarity(emb_i, emb_j);
            println!("📊 Similarity {:?} ↔ {:?}: {:.4}", task_i, task_j, similarity);
            
            // Tasks should produce meaningfully different embeddings
            assert!(similarity < 0.95, 
                   "Tasks {:?} and {:?} produce too similar embeddings: {:.4}", 
                   task_i, task_j, similarity);
        }
    }
    
    // Test code formatting boundary
    println!("🔄 Testing code formatting boundary");
    let rust_code = "fn fibonacci(n: u32) -> u64 { if n <= 1 { n as u64 } else { fibonacci(n-1) + fibonacci(n-2) } }";
    let python_code = "def fibonacci(n): return n if n <= 1 else fibonacci(n-1) + fibonacci(n-2)";
    
    let rust_formatted = CodeFormatter::format_code(rust_code, "rust");
    let python_formatted = CodeFormatter::format_code(python_code, "python");
    
    println!("📝 Rust formatted: {}", rust_formatted);
    println!("📝 Python formatted: {}", python_formatted);
    
    let rust_embedding = embedder.embed(&rust_formatted, EmbeddingTask::CodeDefinition)?;
    let python_embedding = embedder.embed(&python_formatted, EmbeddingTask::CodeDefinition)?;
    
    validate_embedding_at_boundary(&rust_embedding, "rust_formatted")?;
    validate_embedding_at_boundary(&python_embedding, "python_formatted")?;
    
    println!("✅ EmbeddingTask ↔ Prefix Application boundary test passed");
    Ok(())
}

/// Test storage input/output boundary validation
#[test]
fn test_storage_io_boundary() -> Result<()> {
    println!("🔥 BOUNDARY TEST: Storage Input/Output Validation");
    
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("boundary_test.db").to_str().unwrap().to_string();
    let mut storage = VectorStorage::new(&db_path)?;
    
    // Generate test embeddings
    let config = GGUFEmbedderConfig {
        model_path: "./src/model/nomic-embed-text-v1.5.Q4_K_M.gguf".to_string(),
        ..Default::default()
    };
    let embedder = GGUFEmbedder::new(config)?;
    
    // Test boundary: Storage input validation
    println!("🔄 Testing storage input boundary");
    
    // Test 1: Normal case
    let contents = vec!["Normal content".to_string(), "Another content".to_string()];
    let paths = vec!["file1.txt".to_string(), "file2.txt".to_string()];
    let embeddings = embedder.embed_batch(contents.clone(), EmbeddingTask::SearchDocument)?;
    
    storage.store(contents.clone(), embeddings.clone(), paths.clone())?;
    assert_eq!(storage.len(), 2, "Storage length mismatch");
    
    // Test 2: Empty arrays (boundary case)
    let empty_result = storage.store(vec![], vec![], vec![]);
    assert!(empty_result.is_ok(), "Empty storage should succeed");
    
    // Test 3: Single item
    let single_content = vec!["Single item".to_string()];
    let single_path = vec!["single.txt".to_string()];
    let single_embedding = embedder.embed_batch(single_content.clone(), EmbeddingTask::SearchDocument)?;
    
    storage.store(single_content, single_embedding, single_path)?;
    assert_eq!(storage.len(), 3, "Storage should contain 3 items");
    
    // Test boundary: Search input/output validation
    println!("🔄 Testing search boundary");
    
    let query_embedding = embedder.embed("search query", EmbeddingTask::SearchQuery)?;
    
    // Test various search limits
    let limits = vec![0, 1, 5, 10, 100];
    for limit in limits {
        let results = storage.search(query_embedding.clone(), limit)?;
        
        if limit == 0 {
            assert!(results.is_empty(), "Search with limit 0 should return empty results");
        } else {
            let expected_len = std::cmp::min(limit, storage.len());
            assert_eq!(results.len(), expected_len, 
                      "Search limit {} not respected: got {} results", limit, results.len());
        }
        
        // Validate result structure
        for result in &results {
            assert!(!result.content.is_empty(), "Result content should not be empty");
            assert!(!result.file_path.is_empty(), "Result file path should not be empty");
            assert!(result.score >= 0.0 && result.score <= 1.0, 
                   "Invalid similarity score: {}", result.score);
        }
        
        // Validate result ordering
        for i in 1..results.len() {
            assert!(results[i-1].score >= results[i].score, 
                   "Results not ordered by score: {} > {}", results[i-1].score, results[i].score);
        }
    }
    
    // Test boundary: Clear operation
    println!("🔄 Testing clear boundary");
    storage.clear()?;
    assert_eq!(storage.len(), 0, "Storage should be empty after clear");
    assert!(storage.is_empty(), "Storage should report as empty");
    
    let post_clear_results = storage.search(query_embedding, 10)?;
    assert!(post_clear_results.is_empty(), "Search after clear should return no results");
    
    println!("✅ Storage Input/Output boundary test passed");
    Ok(())
}

/// Test chunker output to embedder input boundary
#[test]
fn test_chunker_embedder_boundary() -> Result<()> {
    println!("🔥 BOUNDARY TEST: Chunker Output ↔ Embedder Input");
    
    let simple_chunker = SimpleRegexChunker::with_chunk_size(150).expect("Failed to create simple chunker");
    let markdown_chunker = MarkdownRegexChunker::with_options(200, true).expect("Failed to create markdown chunker");
    
    let config = GGUFEmbedderConfig {
        model_path: "./src/model/nomic-embed-text-v1.5.Q4_K_M.gguf".to_string(),
        ..Default::default()
    };
    let text_embedder = GGUFEmbedder::new(config.clone())?;
    
    let code_config = GGUFEmbedderConfig {
        model_path: "./src/model/nomic-embed-code.Q4_K_M.gguf".to_string(),
        ..Default::default()
    };
    let code_embedder = GGUFEmbedder::new(code_config)?;
    
    // Test Rust code chunking to embedding boundary
    println!("🔄 Testing Rust code chunking ↔ embedding boundary");
    let rust_code = r#"
use std::collections::HashMap;

/// A simple cache implementation
pub struct Cache<K, V> {
    data: HashMap<K, V>,
    capacity: usize,
}

impl<K, V> Cache<K, V>
where
    K: Clone + Eq + std::hash::Hash,
    V: Clone,
{
    pub fn new(capacity: usize) -> Self {
        Self {
            data: HashMap::new(),
            capacity,
        }
    }
    
    pub fn insert(&mut self, key: K, value: V) {
        if self.data.len() >= self.capacity {
            // Simple eviction strategy
            if let Some(first_key) = self.data.keys().next().cloned() {
                self.data.remove(&first_key);
            }
        }
        self.data.insert(key, value);
    }
    
    pub fn get(&self, key: &K) -> Option<&V> {
        self.data.get(key)
    }
}
"#;
    
    let rust_chunks = simple_chunker.chunk_file(rust_code);
    println!("📊 Generated {} Rust code chunks", rust_chunks.len());
    
    // Process each chunk through embedder
    for (i, chunk) in rust_chunks.iter().enumerate() {
        println!("🔄 Processing chunk {} ({} chars)", i + 1, chunk.content.len());
        
        // Test boundary: chunk content to embedder
        let embedding = code_embedder.embed(&chunk.content, EmbeddingTask::CodeDefinition)
            .map_err(|e| anyhow::anyhow!("Failed to embed chunk {}: {}", i, e))?;
        
        validate_embedding_at_boundary(&embedding, &format!("rust_chunk_{}", i))?;
        
        // Test chunk metadata preservation
        assert!(chunk.start_offset < chunk.end_offset, "Invalid chunk offsets");
        assert!(!chunk.content.trim().is_empty(), "Chunk content should not be empty");
    }
    
    // Test markdown chunking to embedding boundary
    println!("🔄 Testing markdown chunking ↔ embedding boundary");
    let markdown_content = r#"
# Cache Implementation Guide

This document explains how to implement a simple cache in Rust.

## Overview

A cache is a data structure that stores recently accessed data for fast retrieval.

### Key Components

1. **Storage**: HashMap for key-value pairs
2. **Capacity**: Maximum number of items
3. **Eviction**: Strategy for removing old items

## Implementation

```rust
pub struct Cache<K, V> {
    data: HashMap<K, V>,
    capacity: usize,
}
```

The cache uses a HashMap for O(1) average-case lookup and insertion.

### Methods

- `new(capacity)`: Create a new cache
- `insert(key, value)`: Add or update an item
- `get(key)`: Retrieve an item

## Performance Considerations

- Memory usage grows with capacity
- Eviction overhead on full cache
- Hash collision handling
"#;
    
    let md_chunks = markdown_chunker.chunk_markdown(markdown_content);
    println!("📊 Generated {} markdown chunks", md_chunks.len());
    
    // Process markdown chunks
    for (i, chunk) in md_chunks.iter().enumerate() {
        println!("🔄 Processing markdown chunk {} ({} chars)", i + 1, chunk.content.len());
        
        let embedding = text_embedder.embed(&chunk.content, EmbeddingTask::SearchDocument)
            .map_err(|e| anyhow::anyhow!("Failed to embed markdown chunk {}: {}", i, e))?;
        
        validate_embedding_at_boundary(&embedding, &format!("markdown_chunk_{}", i))?;
        
        // Test markdown-specific metadata
        assert!(chunk.start_line < chunk.end_line, "Invalid markdown chunk line numbers");
        println!("  📝 Chunk type: {:?}", chunk.chunk_type);
    }
    
    // Test batch processing boundary
    println!("🔄 Testing batch processing boundary");
    let chunk_contents: Vec<String> = rust_chunks.iter()
        .map(|c| c.content.clone())
        .collect();
    
    if chunk_contents.len() > 1 {
        let batch_embeddings = code_embedder.embed_batch(chunk_contents.clone(), EmbeddingTask::CodeDefinition)
            .map_err(|e| anyhow::anyhow!("Batch embedding failed: {}", e))?;
        
        assert_eq!(batch_embeddings.len(), chunk_contents.len(), 
                  "Batch embedding count mismatch");
        
        for (i, embedding) in batch_embeddings.iter().enumerate() {
            validate_embedding_at_boundary(embedding, &format!("batch_chunk_{}", i))?;
        }
    }
    
    println!("✅ Chunker Output ↔ Embedder Input boundary test passed");
    Ok(())
}

/// Test thread safety at component boundaries
#[test]
fn test_thread_safety_boundaries() -> Result<()> {
    println!("🔥 BOUNDARY TEST: Thread Safety Across Components");
    
    let config = GGUFEmbedderConfig {
        model_path: "./src/model/nomic-embed-text-v1.5.Q4_K_M.gguf".to_string(),
        cache_size: 100,
        ..Default::default()
    };
    
    let embedder = Arc::new(GGUFEmbedder::new(config)?);
    let results = Arc::new(Mutex::new(Vec::new()));
    
    // Test concurrent embedding across threads
    println!("🔄 Testing concurrent embedding boundary");
    let mut handles = vec![];
    
    for i in 0..5 {
        let embedder_clone = embedder.clone();
        let results_clone = results.clone();
        
        let handle = thread::spawn(move || {
            let text = format!("Test text number {} for concurrent processing", i);
            
            // Test embedding boundary under concurrency
            match embedder_clone.embed(&text, EmbeddingTask::SearchDocument) {
                Ok(embedding) => {
                    let mut results_guard = results_clone.lock().unwrap();
                    results_guard.push((i, embedding));
                    Ok(())
                }
                Err(e) => Err(anyhow::anyhow!("Thread {} embedding failed: {}", i, e))
            }
        });
        
        handles.push(handle);
    }
    
    // Wait for all threads and collect results
    for handle in handles {
        handle.join().unwrap()?;
    }
    
    let final_results = results.lock().unwrap();
    assert_eq!(final_results.len(), 5, "Not all threads completed successfully");
    
    // Validate thread safety didn't corrupt embeddings
    for (thread_id, embedding) in final_results.iter() {
        validate_embedding_at_boundary(embedding, &format!("thread_{}", thread_id))?;
    }
    
    // Test cache consistency under concurrency
    println!("🔄 Testing cache consistency boundary");
    let cache_test_text = "Cache consistency test";
    
    // Generate same embedding from multiple threads
    let mut cache_handles = vec![];
    let cache_results = Arc::new(Mutex::new(Vec::new()));
    
    for i in 0..3 {
        let embedder_clone = embedder.clone();
        let cache_results_clone = cache_results.clone();
        
        let handle = thread::spawn(move || {
            // Each thread requests the same embedding (should hit cache after first)
            match embedder_clone.embed(cache_test_text, EmbeddingTask::SearchDocument) {
                Ok(embedding) => {
                    let mut results_guard = cache_results_clone.lock().unwrap();
                    results_guard.push(embedding);
                    Ok(())
                }
                Err(e) => Err(anyhow::anyhow!("Cache test thread {} failed: {}", i, e))
            }
        });
        
        cache_handles.push(handle);
    }
    
    for handle in cache_handles {
        handle.join().unwrap()?;
    }
    
    let cache_final_results = cache_results.lock().unwrap();
    assert_eq!(cache_final_results.len(), 3, "Cache test threads failed");
    
    // All embeddings should be identical (cache hit)
    for i in 1..cache_final_results.len() {
        let diff: f32 = cache_final_results[0].iter()
            .zip(cache_final_results[i].iter())
            .map(|(a, b)| (a - b).abs())
            .sum();
        
        assert!(diff < 1e-6, "Cache inconsistency detected: difference = {}", diff);
    }
    
    println!("✅ Thread Safety boundary test passed");
    Ok(())
}

/// Validate embedding quality at component boundaries
fn validate_embedding_at_boundary(embedding: &[f32], context: &str) -> Result<()> {
    // Dimension check
    assert_eq!(embedding.len(), 768, "Wrong embedding dimension at {}", context);
    
    // Non-zero content check
    let non_zero_count = embedding.iter().filter(|&&x| x.abs() > 1e-8).count();
    let non_zero_ratio = non_zero_count as f64 / embedding.len() as f64;
    assert!(non_zero_ratio > 0.3, "Too many zero values at {}: {:.1}%", 
            context, non_zero_ratio * 100.0);
    
    // Normalization check
    let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
    assert!((norm - 1.0).abs() < 0.2, "Poor normalization at {}: norm = {:.6}", context, norm);
    
    // Value distribution check
    let mean = embedding.iter().sum::<f32>() / embedding.len() as f32;
    let variance: f32 = embedding.iter().map(|x| (x - mean).powi(2)).sum::<f32>() / embedding.len() as f32;
    let std_dev = variance.sqrt();
    assert!(std_dev > 0.001, "Too uniform values at {}: std_dev = {:.6}", context, std_dev);
    
    // Range check
    let min_val = embedding.iter().fold(f32::INFINITY, |a, &b| a.min(b));
    let max_val = embedding.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
    assert!(min_val > -10.0 && max_val < 10.0, "Values out of range at {}: [{:.3}, {:.3}]", 
            context, min_val, max_val);
    
    Ok(())
}

/// Calculate cosine similarity between embeddings
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }
    
    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    
    dot_product / (norm_a * norm_b)
}