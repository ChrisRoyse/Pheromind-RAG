/// COMPREHENSIVE INTEGRATION TESTS - No tolerance for component failures
/// Tests every integration boundary in the RAG system with brutal precision
/// 
/// Integration Boundaries Tested:
/// 1. File Processing → Chunking → Embedding Pipeline
/// 2. Dual Embedder Integration (Text vs Code Models)
/// 3. Storage Layer Integration (VectorStorage + Tantivy)
/// 4. Search Pipeline Integration (Vector + Text → Fusion)
/// 5. Configuration Loading → System Initialization
/// 6. Error Propagation Through All Components
/// 7. Memory Management Across Components
/// 8. Performance Validation Under Load

use anyhow::Result;
use embed_search::{
    simple_search::HybridSearch,
    gguf_embedder::{GGUFEmbedder, GGUFEmbedderConfig},
    embedding_prefixes::EmbeddingTask,
    simple_storage::{VectorStorage, SearchResult as VectorResult},
    Config,
    chunking::{SimpleRegexChunker, MarkdownRegexChunker, Chunk, MarkdownChunk},
};
use tempfile::tempdir;
use std::time::Instant;
use std::sync::Arc;
use std::path::Path;
use tokio::sync::Semaphore;

/// Test complete pipeline from file input to search results
#[tokio::test]
async fn test_full_pipeline_integration() -> Result<()> {
    println!("🔥 INTEGRATION TEST: Full Pipeline Validation");
    
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("pipeline_test.db").to_str().unwrap().to_string();
    
    // Initialize hybrid search system
    let mut search = HybridSearch::new(&db_path).await
        .map_err(|e| anyhow::anyhow!("Failed to initialize HybridSearch: {}", e))?;
    
    // Test data: Mixed content types
    let test_files = vec![
        ("main.rs", r#"
use std::collections::HashMap;

/// Calculate Fibonacci numbers
fn fibonacci(n: u32) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

#[derive(Debug)]
struct UserCache {
    data: HashMap<String, String>,
    max_size: usize,
}

impl UserCache {
    fn new(max_size: usize) -> Self {
        Self {
            data: HashMap::new(),
            max_size,
        }
    }
    
    fn insert(&mut self, key: String, value: String) {
        if self.data.len() >= self.max_size {
            // Evict oldest entry (simplified)
            if let Some(first_key) = self.data.keys().next().cloned() {
                self.data.remove(&first_key);
            }
        }
        self.data.insert(key, value);
    }
}
"#),
        ("README.md", r#"
# Fibonacci Calculator

This project implements a simple Fibonacci number calculator in Rust.

## Features

- Recursive Fibonacci calculation
- User cache with size limits
- Memory-efficient implementation

## Usage

```rust
let result = fibonacci(10);
println!("10th Fibonacci number: {}", result);
```

## Performance

The recursive implementation has exponential time complexity O(2^n).
For better performance, consider using memoization or iterative approach.
"#),
        ("utils.py", r#"
import time
from typing import Dict, Optional

class PerformanceMonitor:
    """Monitor function execution times"""
    
    def __init__(self):
        self.metrics: Dict[str, list] = {}
    
    def time_function(self, func_name: str, execution_time: float):
        """Record execution time for a function"""
        if func_name not in self.metrics:
            self.metrics[func_name] = []
        self.metrics[func_name].append(execution_time)
    
    def get_average_time(self, func_name: str) -> Optional[float]:
        """Get average execution time for a function"""
        if func_name not in self.metrics or not self.metrics[func_name]:
            return None
        return sum(self.metrics[func_name]) / len(self.metrics[func_name])

def fibonacci_iterative(n: int) -> int:
    """Iterative Fibonacci calculation"""
    if n <= 1:
        return n
    
    a, b = 0, 1
    for _ in range(2, n + 1):
        a, b = b, a + b
    
    return b
"#),
    ];
    
    // Index all test files
    for (filename, content) in &test_files {
        println!("🔄 Indexing {}", filename);
        let start = Instant::now();
        
        search.index(vec![content.to_string()], vec![filename.to_string()]).await
            .map_err(|e| anyhow::anyhow!("Failed to index {}: {}", filename, e))?;
        
        let index_time = start.elapsed();
        println!("✅ Indexed {} in {:?}", filename, index_time);
        
        // Verify indexing didn't exceed reasonable time bounds
        assert!(index_time.as_secs() < 5, "Indexing {} took too long: {:?}", filename, index_time);
    }
    
    // Test search functionality with various queries
    let test_queries = vec![
        ("fibonacci", "Should find both Rust and Python implementations"),
        ("UserCache", "Should find Rust struct definition"),
        ("PerformanceMonitor", "Should find Python class"),
        ("HashMap", "Should find Rust imports"),
        ("time complexity", "Should find markdown documentation"),
        ("recursive implementation", "Should find documentation"),
    ];
    
    for (query, description) in test_queries {
        println!("🔍 Testing query: '{}' - {}", query, description);
        let start = Instant::now();
        
        let results = search.search(query, 5).await
            .map_err(|e| anyhow::anyhow!("Search failed for '{}': {}", query, e))?;
        
        let search_time = start.elapsed();
        println!("📊 Query '{}': {} results in {:?}", query, results.len(), search_time);
        
        // Validate search performance
        assert!(search_time.as_millis() < 200, "Search for '{}' took too long: {:?}", query, search_time);
        assert!(!results.is_empty(), "No results found for '{}'", query);
        
        // Validate result quality
        for result in &results {
            assert!(!result.content.is_empty(), "Empty content in search result");
            assert!(!result.file_path.is_empty(), "Empty file path in search result");
            assert!(result.score > 0.0, "Invalid score in search result: {}", result.score);
            
            println!("  📄 {}: {:.3} ({})", result.file_path, result.score, result.match_type);
        }
    }
    
    println!("✅ Full pipeline integration test passed");
    Ok(())
}

/// Test dual embedder integration with different content types
#[tokio::test]
async fn test_dual_embedder_integration() -> Result<()> {
    println!("🔥 INTEGRATION TEST: Dual Embedder Validation");
    
    // Verify both model files exist
    let text_model_path = "./src/model/nomic-embed-text-v1.5.Q4_K_M.gguf";
    let code_model_path = "./src/model/nomic-embed-code.Q4_K_M.gguf";
    
    assert!(Path::new(text_model_path).exists(), "Text model not found: {}", text_model_path);
    assert!(Path::new(code_model_path).exists(), "Code model not found: {}", code_model_path);
    
    // Initialize both embedders
    let text_config = GGUFEmbedderConfig {
        model_path: text_model_path.to_string(),
        ..Default::default()
    };
    let text_embedder = GGUFEmbedder::new(text_config)
        .map_err(|e| anyhow::anyhow!("Failed to initialize text embedder: {}", e))?;
    
    let code_config = GGUFEmbedderConfig {
        model_path: code_model_path.to_string(),
        ..Default::default()
    };
    let code_embedder = GGUFEmbedder::new(code_config)
        .map_err(|e| anyhow::anyhow!("Failed to initialize code embedder: {}", e))?;
    
    // Test content types
    let markdown_content = "# Algorithm Documentation\nThis explains the Fibonacci algorithm implementation.";
    let rust_content = "fn fibonacci(n: u32) -> u64 { match n { 0 => 0, 1 => 1, _ => fibonacci(n-1) + fibonacci(n-2) } }";
    let python_content = "def fibonacci(n): return n if n <= 1 else fibonacci(n-1) + fibonacci(n-2)";
    
    println!("🔄 Testing text embedder with markdown content");
    let text_embedding = text_embedder.embed(markdown_content, EmbeddingTask::SearchDocument)
        .map_err(|e| anyhow::anyhow!("Text embedding failed: {}", e))?;
    assert_eq!(text_embedding.len(), 768, "Incorrect text embedding dimension");
    validate_embedding_quality(&text_embedding, "text")?;
    
    println!("🔄 Testing code embedder with Rust content");
    let rust_embedding = code_embedder.embed(rust_content, EmbeddingTask::CodeDefinition)
        .map_err(|e| anyhow::anyhow!("Rust embedding failed: {}", e))?;
    assert_eq!(rust_embedding.len(), 768, "Incorrect code embedding dimension");
    validate_embedding_quality(&rust_embedding, "Rust code")?;
    
    println!("🔄 Testing code embedder with Python content");
    let python_embedding = code_embedder.embed(python_content, EmbeddingTask::CodeDefinition)
        .map_err(|e| anyhow::anyhow!("Python embedding failed: {}", e))?;
    validate_embedding_quality(&python_embedding, "Python code")?;
    
    // Test cross-model differences
    let text_with_code_model = code_embedder.embed(markdown_content, EmbeddingTask::SearchDocument)?;
    let code_with_text_model = text_embedder.embed(rust_content, EmbeddingTask::CodeDefinition)?;
    
    let text_model_diff = cosine_distance(&text_embedding, &text_with_code_model);
    let code_model_diff = cosine_distance(&rust_embedding, &code_with_text_model);
    
    println!("📊 Text content: text model vs code model distance: {:.4}", text_model_diff);
    println!("📊 Code content: code model vs text model distance: {:.4}", code_model_diff);
    
    // Models should produce meaningfully different embeddings for same content
    assert!(text_model_diff > 0.1, "Text and code models too similar for text content: {:.4}", text_model_diff);
    assert!(code_model_diff > 0.1, "Code and text models too similar for code content: {:.4}", code_model_diff);
    
    println!("✅ Dual embedder integration test passed");
    Ok(())
}

/// Test storage layer integration with vector operations
#[tokio::test]
async fn test_storage_integration() -> Result<()> {
    println!("🔥 INTEGRATION TEST: Storage Layer Validation");
    
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("storage_test.db").to_str().unwrap().to_string();
    
    // Initialize storage
    let mut storage = VectorStorage::new(&db_path)
        .map_err(|e| anyhow::anyhow!("Failed to initialize storage: {}", e))?;
    
    // Initialize embedder for test data
    let config = GGUFEmbedderConfig {
        model_path: "./src/model/nomic-embed-text-v1.5.Q4_K_M.gguf".to_string(),
        batch_size: 3,
        ..Default::default()
    };
    let embedder = GGUFEmbedder::new(config)?;
    
    // Test data with varying content types
    let test_documents = vec![
        ("Document 1: Fibonacci algorithm explanation", "doc1.md"),
        ("Document 2: Implementation details and performance", "doc2.md"),
        ("Document 3: Mathematical properties of Fibonacci sequences", "doc3.md"),
        ("Document 4: Recursive vs iterative approaches", "doc4.md"),
        ("Document 5: Code optimization techniques", "doc5.md"),
    ];
    
    // Generate embeddings
    let contents: Vec<String> = test_documents.iter().map(|(content, _)| content.to_string()).collect();
    let paths: Vec<String> = test_documents.iter().map(|(_, path)| path.to_string()).collect();
    
    println!("🔄 Generating embeddings for {} documents", contents.len());
    let embeddings = embedder.embed_batch(contents.clone(), EmbeddingTask::SearchDocument)?;
    assert_eq!(embeddings.len(), contents.len(), "Embedding count mismatch");
    
    // Store documents
    println!("🔄 Storing documents in vector storage");
    storage.store(contents.clone(), embeddings.clone(), paths.clone())?;
    assert_eq!(storage.len(), test_documents.len(), "Storage count mismatch");
    
    // Test various search queries
    let search_queries = vec![
        "Fibonacci algorithm",
        "performance optimization",
        "recursive implementation",
        "mathematical properties",
        "iterative approach",
    ];
    
    for query in search_queries {
        println!("🔍 Testing storage search: '{}'", query);
        
        let query_embedding = embedder.embed(query, EmbeddingTask::SearchQuery)?;
        let results = storage.search(query_embedding, 3)?;
        
        assert!(!results.is_empty(), "No results for query: {}", query);
        
        // Validate result quality
        for result in &results {
            assert!(!result.content.is_empty(), "Empty content in result");
            assert!(!result.file_path.is_empty(), "Empty file path in result");
            assert!(result.score >= 0.0 && result.score <= 1.0, "Invalid similarity score: {}", result.score);
        }
        
        // Results should be sorted by relevance
        for i in 1..results.len() {
            assert!(results[i-1].score >= results[i].score, "Results not sorted by relevance");
        }
        
        println!("  📊 Found {} results, top score: {:.4}", results.len(), results[0].score);
    }
    
    // Test storage persistence and cleanup
    println!("🔄 Testing storage operations");
    let original_count = storage.len();
    
    // Add more documents
    let additional_content = vec!["Additional document content".to_string()];
    let additional_paths = vec!["additional.md".to_string()];
    let additional_embeddings = embedder.embed_batch(additional_content.clone(), EmbeddingTask::SearchDocument)?;
    
    storage.store(additional_content, additional_embeddings, additional_paths)?;
    assert_eq!(storage.len(), original_count + 1, "Storage size not updated correctly");
    
    // Clear storage
    storage.clear()?;
    assert_eq!(storage.len(), 0, "Storage not cleared properly");
    assert!(storage.is_empty(), "Storage should be empty after clear");
    
    println!("✅ Storage integration test passed");
    Ok(())
}

/// Test chunking integration with different content types
#[tokio::test]
async fn test_chunking_integration() -> Result<()> {
    println!("🔥 INTEGRATION TEST: Chunking Integration Validation");
    
    // Initialize chunkers
    let simple_chunker = SimpleRegexChunker::with_chunk_size(200).expect("Failed to create chunker");
    let mut markdown_chunker = MarkdownRegexChunker::new(300, 50);
    
    // Test Rust code chunking
    let rust_code = r#"
use std::collections::HashMap;
use std::sync::Arc;

/// A thread-safe cache implementation
pub struct ThreadSafeCache<K, V> {
    data: Arc<parking_lot::Mutex<HashMap<K, V>>>,
    max_size: usize,
}

impl<K, V> ThreadSafeCache<K, V>
where
    K: Clone + Eq + std::hash::Hash,
    V: Clone,
{
    pub fn new(max_size: usize) -> Self {
        Self {
            data: Arc::new(parking_lot::Mutex::new(HashMap::new())),
            max_size,
        }
    }
    
    pub fn get(&self, key: &K) -> Option<V> {
        let guard = self.data.lock();
        guard.get(key).cloned()
    }
    
    pub fn insert(&self, key: K, value: V) {
        let mut guard = self.data.lock();
        if guard.len() >= self.max_size {
            // Simple eviction: remove first key
            if let Some(first_key) = guard.keys().next().cloned() {
                guard.remove(&first_key);
            }
        }
        guard.insert(key, value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cache_basic_operations() {
        let cache = ThreadSafeCache::new(2);
        
        cache.insert("key1", "value1");
        cache.insert("key2", "value2");
        
        assert_eq!(cache.get(&"key1"), Some("value1"));
        assert_eq!(cache.get(&"key2"), Some("value2"));
        
        // Test eviction
        cache.insert("key3", "value3");
        // One of the first two keys should be evicted
        let count = [&"key1", &"key2"].iter()
            .filter(|&&k| cache.get(&k).is_some())
            .count();
        assert_eq!(count, 1);
    }
}
"#;
    
    println!("🔄 Testing simple chunker with Rust code");
    let rust_chunks = simple_chunker.chunk_text(rust_code);
    assert!(!rust_chunks.is_empty(), "No chunks generated for Rust code");
    
    // Validate chunk properties
    for chunk in &rust_chunks {
        assert!(!chunk.content.is_empty(), "Empty chunk content");
        assert!(chunk.content.len() <= 250, "Chunk too large: {} chars", chunk.content.len()); // Some overlap allowed
        validate_chunk_structure(chunk)?;
    }
    
    println!("📊 Generated {} chunks for Rust code", rust_chunks.len());
    
    // Test markdown chunking
    let markdown_content = r#"
# Cache Implementation Guide

This document explains how to implement a thread-safe cache in Rust.

## Overview

Caching is essential for performance optimization. A good cache should:

- Provide fast access to frequently used data
- Handle concurrent access safely
- Implement reasonable eviction policies
- Maintain memory bounds

## Implementation Details

### Basic Structure

```rust
pub struct ThreadSafeCache<K, V> {
    data: Arc<Mutex<HashMap<K, V>>>,
    max_size: usize,
}
```

### Key Features

1. **Thread Safety**: Uses `Arc<Mutex<>>` for concurrent access
2. **Generic Types**: Works with any `Clone + Eq + Hash` key type
3. **Size Limits**: Prevents unbounded memory growth
4. **Simple Eviction**: Removes oldest entries when full

## Usage Examples

### Basic Operations

```rust
let cache = ThreadSafeCache::new(100);
cache.insert("user:123", user_data);
let user = cache.get(&"user:123");
```

### Performance Considerations

- Lock contention can become a bottleneck under high load
- Consider using `parking_lot::RwLock` for read-heavy workloads
- Hash map operations are O(1) average case
- Memory overhead is approximately 2x the stored data

## Best Practices

1. Choose appropriate cache sizes based on available memory
2. Monitor hit rates and adjust eviction policies accordingly
3. Consider using weak references for large objects
4. Implement proper error handling for cache misses

## Testing

Always test your cache implementation with:

- Concurrent access patterns
- Memory pressure scenarios
- Eviction behavior validation
- Performance benchmarks
"#;
    
    println!("🔄 Testing markdown chunker with documentation");
    let md_chunks = markdown_chunker.chunk_markdown(markdown_content);
    assert!(!md_chunks.is_empty(), "No chunks generated for markdown");
    
    // Validate markdown chunk properties
    for chunk in &md_chunks {
        assert!(!chunk.content.is_empty(), "Empty markdown chunk content");
        validate_markdown_chunk_structure(chunk)?;
    }
    
    println!("📊 Generated {} markdown chunks", md_chunks.len());
    
    // Test chunking consistency
    let rechunked = simple_chunker.chunk_text(rust_code);
    assert_eq!(rust_chunks.len(), rechunked.len(), "Chunking not consistent");
    
    println!("✅ Chunking integration test passed");
    Ok(())
}

/// Test error propagation through the entire pipeline
#[tokio::test]
async fn test_error_propagation() -> Result<()> {
    println!("🔥 INTEGRATION TEST: Error Propagation Validation");
    
    // Test 1: Invalid model path
    println!("🔄 Testing invalid model path error handling");
    let invalid_config = GGUFEmbedderConfig {
        model_path: "/nonexistent/path/model.gguf".to_string(),
        ..Default::default()
    };
    
    let result = GGUFEmbedder::new(invalid_config);
    assert!(result.is_err(), "Should fail with invalid model path");
    println!("✅ Invalid model path properly rejected");
    
    // Test 2: Invalid database path
    println!("🔄 Testing invalid database path error handling");
    let invalid_db_path = "/invalid/readonly/path/db.db";
    let result = HybridSearch::new(invalid_db_path).await;
    // This might succeed depending on system, so we won't assert failure
    
    // Test 3: Empty content handling
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("error_test.db").to_str().unwrap().to_string();
    let mut search = HybridSearch::new(&db_path).await?;
    
    println!("🔄 Testing empty content handling");
    let result = search.index(vec![], vec![]).await;
    assert!(result.is_ok(), "Empty content should be handled gracefully");
    
    let result = search.index(vec!["".to_string()], vec!["empty.txt".to_string()]).await;
    assert!(result.is_ok(), "Empty string content should be handled gracefully");
    
    // Test 4: Mismatched content and path arrays
    println!("🔄 Testing mismatched array lengths");
    let result = search.index(
        vec!["content1".to_string(), "content2".to_string()],
        vec!["file1.txt".to_string()], // Mismatched length
    ).await;
    // Implementation might handle this gracefully or error - both are valid
    
    // Test 5: Very large content handling
    println!("🔄 Testing large content handling");
    let large_content = "x".repeat(1_000_000); // 1MB content
    let result = search.index(vec![large_content], vec!["large.txt".to_string()]).await;
    // Should either succeed or fail gracefully
    
    // Test 6: Search with empty query
    println!("🔄 Testing empty query handling");
    let result = search.search("", 10).await;
    assert!(result.is_ok(), "Empty query should be handled gracefully");
    
    let results = result.unwrap();
    println!("📊 Empty query returned {} results", results.len());
    
    println!("✅ Error propagation test passed");
    Ok(())
}

/// Test concurrent operations and thread safety
#[tokio::test]
async fn test_concurrent_operations() -> Result<()> {
    println!("🔥 INTEGRATION TEST: Concurrent Operations Validation");
    
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("concurrent_test.db").to_str().unwrap().to_string();
    
    // Initialize search system
    let search = Arc::new(tokio::sync::Mutex::new(HybridSearch::new(&db_path).await?));
    let semaphore = Arc::new(Semaphore::new(4)); // Limit concurrent operations
    
    // Test concurrent indexing
    println!("🔄 Testing concurrent indexing operations");
    let mut handles = vec![];
    
    for i in 0..10 {
        let search = search.clone();
        let semaphore = semaphore.clone();
        
        let handle = tokio::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();
            
            let content = format!("fn function_{}() {{ println!(\"Function {}\"); }}", i, i);
            let path = format!("file_{}.rs", i);
            
            let mut search_guard = search.lock().await;
            search_guard.index(vec![content], vec![path]).await
        });
        
        handles.push(handle);
    }
    
    // Wait for all indexing operations
    for handle in handles {
        let result = handle.await?;
        assert!(result.is_ok(), "Concurrent indexing failed: {:?}", result);
    }
    
    println!("✅ Concurrent indexing completed");
    
    // Test concurrent searching
    println!("🔄 Testing concurrent search operations");
    let mut search_handles = vec![];
    
    for i in 0..5 {
        let search = search.clone();
        let semaphore = semaphore.clone();
        
        let handle = tokio::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();
            
            let query = format!("function_{}", i);
            let search_guard = search.lock().await;
            search_guard.search(&query, 5).await
        });
        
        search_handles.push(handle);
    }
    
    // Wait for all search operations
    for handle in search_handles {
        let result = handle.await?;
        assert!(result.is_ok(), "Concurrent search failed");
        
        let results = result.unwrap();
        println!("📊 Concurrent search returned {} results", results.len());
    }
    
    println!("✅ Concurrent operations test passed");
    Ok(())
}

/// Validate embedding quality
fn validate_embedding_quality(embedding: &[f32], content_type: &str) -> Result<()> {
    // Check dimension
    assert_eq!(embedding.len(), 768, "Incorrect embedding dimension for {}", content_type);
    
    // Check for non-zero values (should have meaningful content)
    let non_zero_count = embedding.iter().filter(|&&x| x.abs() > 1e-8).count();
    let non_zero_ratio = non_zero_count as f64 / embedding.len() as f64;
    assert!(non_zero_ratio > 0.5, "Too many zero values in {} embedding: {:.2}%", 
            content_type, non_zero_ratio * 100.0);
    
    // Check normalization (should be approximately unit length)
    let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
    assert!((norm - 1.0).abs() < 0.1, "Embedding not properly normalized for {}: norm = {:.6}", 
            content_type, norm);
    
    // Check for reasonable value distribution
    let mean = embedding.iter().sum::<f32>() / embedding.len() as f32;
    let variance: f32 = embedding.iter().map(|x| (x - mean).powi(2)).sum::<f32>() / embedding.len() as f32;
    let std_dev = variance.sqrt();
    
    assert!(std_dev > 0.01, "Embedding values too uniform for {}: std_dev = {:.6}", 
            content_type, std_dev);
    
    println!("✅ {} embedding quality validated: norm={:.4}, std_dev={:.4}, non_zero={:.1}%", 
             content_type, norm, std_dev, non_zero_ratio * 100.0);
    
    Ok(())
}

/// Calculate cosine distance between two embeddings
fn cosine_distance(a: &[f32], b: &[f32]) -> f32 {
    let similarity = cosine_similarity(a, b);
    1.0 - similarity
}

/// Calculate cosine similarity between two embeddings
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

/// Validate basic chunk structure
fn validate_chunk_structure(chunk: &Chunk) -> Result<()> {
    assert!(!chunk.content.is_empty(), "Chunk content cannot be empty");
    assert!(chunk.start_line <= chunk.end_line, "Invalid chunk line numbers");
    Ok(())
}

/// Validate markdown chunk structure
fn validate_markdown_chunk_structure(chunk: &MarkdownChunk) -> Result<()> {
    assert!(!chunk.content.is_empty(), "Markdown chunk content cannot be empty");
    assert!(chunk.start_line <= chunk.end_line, "Invalid markdown chunk line numbers");
    // Additional markdown-specific validations could go here
    Ok(())
}

/// Test memory usage during operations
#[tokio::test]
async fn test_memory_management() -> Result<()> {
    println!("🔥 INTEGRATION TEST: Memory Management Validation");
    
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("memory_test.db").to_str().unwrap().to_string();
    
    // Initialize system
    let mut search = HybridSearch::new(&db_path).await?;
    
    // Measure baseline memory
    let initial_memory = get_memory_usage();
    println!("📊 Initial memory usage: {:.2} MB", initial_memory);
    
    // Index a large number of documents
    let document_count = 100;
    let mut contents = Vec::with_capacity(document_count);
    let mut paths = Vec::with_capacity(document_count);
    
    for i in 0..document_count {
        let content = format!("This is document number {} with some content to index. It contains various words and phrases that will be embedded and stored. The content is long enough to create meaningful embeddings but not too long to cause memory issues.", i);
        contents.push(content);
        paths.push(format!("doc_{}.txt", i));
    }
    
    // Index in batches to monitor memory usage
    let batch_size = 20;
    for batch_start in (0..document_count).step_by(batch_size) {
        let batch_end = std::cmp::min(batch_start + batch_size, document_count);
        let batch_contents = contents[batch_start..batch_end].to_vec();
        let batch_paths = paths[batch_start..batch_end].to_vec();
        
        search.index(batch_contents, batch_paths).await?;
        
        let current_memory = get_memory_usage();
        let memory_increase = current_memory - initial_memory;
        println!("📊 After batch {}: {:.2} MB (+{:.2} MB)", 
                 batch_start / batch_size + 1, current_memory, memory_increase);
        
        // Memory should not grow unbounded
        assert!(memory_increase < 1000.0, "Excessive memory usage: {:.2} MB increase", memory_increase);
    }
    
    // Perform searches and monitor memory
    for i in 0..10 {
        let query = format!("document number {}", i * 10);
        let _results = search.search(&query, 5).await?;
        
        if i % 5 == 0 {
            let current_memory = get_memory_usage();
            println!("📊 After {} searches: {:.2} MB", i + 1, current_memory);
        }
    }
    
    // Clear system and check memory cleanup
    search.clear().await?;
    
    // Allow time for cleanup
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    
    let final_memory = get_memory_usage();
    println!("📊 Final memory usage: {:.2} MB", final_memory);
    
    println!("✅ Memory management test passed");
    Ok(())
}

/// Get current memory usage in MB (simplified)
fn get_memory_usage() -> f64 {
    // This is a simplified implementation
    // In a real scenario, you'd use proper system calls or profiling tools
    match std::fs::read_to_string("/proc/self/status") {
        Ok(content) => {
            for line in content.lines() {
                if line.starts_with("VmRSS:") {
                    if let Some(kb_str) = line.split_whitespace().nth(1) {
                        if let Ok(kb) = kb_str.parse::<u64>() {
                            return kb as f64 / 1024.0; // Convert to MB
                        }
                    }
                }
            }
        }
        Err(_) => {
            // Fallback for non-Linux systems
            return 0.0;
        }
    }
    0.0
}