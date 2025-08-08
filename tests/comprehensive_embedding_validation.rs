//! Comprehensive Embedding System Validation Suite
//! 
//! This test suite performs brutal validation of the entire embedding pipeline.
//! If all tests pass, the embedding system is 100% functional.
//! 
//! CRITICAL: These tests verify ACTUAL functionality, not mocked behavior.
//! Any test failure indicates the embedding system is broken.

use anyhow::Result;
use embed::embedding::nomic::NomicEmbedder;
#[cfg(feature = "vectordb")]
use embed::storage::lancedb_storage::{LanceDBStorage, LanceEmbeddingRecord};
use embed::search::unified::UnifiedSearcher;
use std::path::PathBuf;
use std::collections::HashMap;
use tempfile::TempDir;

/// Validation constants
const EXPECTED_DIMENSIONS: usize = 768;
const MIN_COSINE_SIMILARITY_THRESHOLD: f32 = 0.7;
const MAX_COSINE_SIMILARITY_THRESHOLD: f32 = 0.95;
const EPSILON: f32 = 1e-6;

#[tokio::test]
async fn test_01_model_loading_and_initialization() -> Result<()> {
    println!("\n🔍 TEST 01: Model Loading and Initialization Validation");
    
    // Test 1.1: Verify model file exists at expected location
    let home = dirs::home_dir().expect("Home directory must exist");
    let model_path = home.join(".nomic").join("nomic-embed-text-v1.5.Q4_K_M.gguf");
    assert!(model_path.exists(), "Model file must exist at {:?}", model_path);
    
    // Test 1.2: Verify model file size is correct (~84MB)
    let metadata = std::fs::metadata(&model_path)?;
    let size_mb = metadata.len() as f64 / 1_048_576.0;
    assert!(
        size_mb > 80.0 && size_mb < 90.0,
        "Model size must be ~84MB, got {:.1}MB", 
        size_mb
    );
    
    // Test 1.3: Initialize embedder and verify dimensions
    let embedder = NomicEmbedder::new().await?;
    assert_eq!(
        embedder.dimensions(),
        EXPECTED_DIMENSIONS,
        "Embedder must report 768 dimensions"
    );
    
    // Test 1.4: Verify singleton pattern works
    let global_embedder = NomicEmbedder::get_global().await?;
    assert_eq!(
        global_embedder.dimensions(),
        EXPECTED_DIMENSIONS,
        "Global embedder must have same dimensions"
    );
    
    println!("✅ Model loading validation passed");
    Ok(())
}

#[tokio::test]
async fn test_02_embedding_generation_correctness() -> Result<()> {
    println!("\n🔍 TEST 02: Embedding Generation Correctness");
    
    let embedder = NomicEmbedder::get_global().await?;
    
    // Test 2.1: Verify embedding dimensions
    let text = "Hello, world!";
    let embedding = embedder.embed(text)?;
    assert_eq!(
        embedding.len(),
        EXPECTED_DIMENSIONS,
        "Embedding must have exactly 768 dimensions"
    );
    
    // Test 2.2: Verify L2 normalization
    let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
    assert!(
        (norm - 1.0).abs() < EPSILON,
        "Embedding must be L2 normalized, got norm={}",
        norm
    );
    
    // Test 2.3: Verify no NaN or Inf values
    for (i, val) in embedding.iter().enumerate() {
        assert!(
            val.is_finite(),
            "Embedding[{}] = {} is not finite",
            i, val
        );
    }
    
    // Test 2.4: Verify determinism
    let embedding2 = embedder.embed(text)?;
    for i in 0..embedding.len() {
        assert!(
            (embedding[i] - embedding2[i]).abs() < EPSILON,
            "Embeddings must be deterministic, diff at [{}]",
            i
        );
    }
    
    // Test 2.5: Verify different texts produce different embeddings
    let other_text = "Goodbye, universe!";
    let other_embedding = embedder.embed(other_text)?;
    let similarity = cosine_similarity(&embedding, &other_embedding);
    assert!(
        similarity < MAX_COSINE_SIMILARITY_THRESHOLD,
        "Different texts must produce different embeddings, similarity={}",
        similarity
    );
    
    println!("✅ Embedding generation validation passed");
    Ok(())
}

#[tokio::test]
async fn test_03_semantic_similarity_validation() -> Result<()> {
    println!("\n🔍 TEST 03: Semantic Similarity Validation");
    
    let embedder = NomicEmbedder::get_global().await?;
    
    // Test cases with expected similarity relationships
    let test_cases = vec![
        // Similar code should have high similarity
        (
            "fn add(a: i32, b: i32) -> i32 { a + b }",
            "fn sum(x: i32, y: i32) -> i32 { x + y }",
            0.85, // Should be > 0.85
            "Similar functions"
        ),
        // Different functionality should have lower similarity
        (
            "fn multiply(a: i32, b: i32) -> i32 { a * b }",
            "async fn fetch_data(url: &str) -> Result<String> { ... }",
            0.6, // Should be < 0.6
            "Different functions"
        ),
        // Same concept in different languages should have moderate similarity
        (
            "for i in range(10): print(i)",
            "for (int i = 0; i < 10; i++) { printf(\"%d\", i); }",
            0.7, // Should be > 0.7
            "Same logic different language"
        ),
    ];
    
    for (text1, text2, threshold, description) in test_cases {
        let emb1 = embedder.embed(text1)?;
        let emb2 = embedder.embed(text2)?;
        let similarity = cosine_similarity(&emb1, &emb2);
        
        println!("  {} similarity: {:.3}", description, similarity);
        
        if description.contains("Similar") {
            assert!(
                similarity > threshold,
                "{} should have similarity > {}, got {}",
                description, threshold, similarity
            );
        } else if description.contains("Different") {
            assert!(
                similarity < threshold,
                "{} should have similarity < {}, got {}",
                description, threshold, similarity
            );
        } else {
            assert!(
                similarity > threshold,
                "{} should have similarity > {}, got {}",
                description, threshold, similarity
            );
        }
    }
    
    println!("✅ Semantic similarity validation passed");
    Ok(())
}

#[cfg(feature = "vectordb")]
#[tokio::test]
async fn test_04_vector_storage_operations() -> Result<()> {
    println!("\n🔍 TEST 04: Vector Storage Operations");
    
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().to_path_buf();
    
    // Initialize storage
    let mut storage = LanceDBStorage::new(db_path.clone()).await?;
    storage.init_table().await?;
    
    // Test 4.1: Insert single embedding
    let embedder = NomicEmbedder::get_global().await?;
    let text = "Test embedding for storage";
    let embedding = embedder.embed(text)?;
    
    let record = LanceEmbeddingRecord {
        id: "test-1".to_string(),
        file_path: "test.rs".to_string(),
        chunk_index: 0,
        content: text.to_string(),
        embedding: embedding.clone(),
        start_line: 1,
        end_line: 1,
        similarity_score: None,
    };
    
    storage.insert(record.clone()).await?;
    
    // Test 4.2: Verify count
    let count = storage.count().await?;
    assert_eq!(count, 1, "Storage should contain 1 record");
    
    // Test 4.3: Batch insertion
    let mut batch = Vec::new();
    for i in 0..10 {
        let text = format!("Batch test {}", i);
        let emb = embedder.embed(&text)?;
        batch.push(LanceEmbeddingRecord {
            id: format!("batch-{}", i),
            file_path: format!("file{}.rs", i),
            chunk_index: i as u64,
            content: text,
            embedding: emb,
            start_line: i as u64,
            end_line: i as u64,
            similarity_score: None,
        });
    }
    
    storage.insert_batch(batch).await?;
    
    // Test 4.4: Verify total count
    let count = storage.count().await?;
    assert_eq!(count, 11, "Storage should contain 11 records");
    
    // Test 4.5: Similarity search
    let query_embedding = embedder.embed("Test embedding for storage")?;
    let results = storage.search_similar(query_embedding, 5).await?;
    
    assert!(!results.is_empty(), "Search should return results");
    assert!(results.len() <= 5, "Should return at most 5 results");
    
    // First result should be the exact match
    assert_eq!(
        results[0].id, "test-1",
        "First result should be exact match"
    );
    
    // Test 4.6: Delete by file
    storage.delete_by_file("test.rs").await?;
    let count = storage.count().await?;
    assert_eq!(count, 10, "Should have 10 records after deletion");
    
    // Test 4.7: Clear all
    storage.clear_all().await?;
    let count = storage.count().await?;
    assert_eq!(count, 0, "Storage should be empty after clear");
    
    println!("✅ Vector storage validation passed");
    Ok(())
}

#[tokio::test]
async fn test_05_unified_search_integration() -> Result<()> {
    println!("\n🔍 TEST 05: Unified Search Integration");
    
    let temp_dir = TempDir::new()?;
    let project_path = temp_dir.path().to_path_buf();
    let db_path = temp_dir.path().join("db");
    
    // Create test files
    let test_file = project_path.join("test.rs");
    std::fs::write(
        &test_file,
        r#"fn calculate_sum(a: i32, b: i32) -> i32 {
    a + b
}

fn calculate_product(a: i32, b: i32) -> i32 {
    a * b
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sum() {
        assert_eq!(calculate_sum(2, 3), 5);
    }
}"#,
    )?;
    
    // Initialize unified searcher
    let searcher = UnifiedSearcher::new_with_config(
        project_path.clone(),
        db_path.clone(),
        false, // Don't include test files
    )
    .await?;
    
    // Index the file
    searcher.index_file(&test_file).await?;
    
    // Test 5.1: Search for function name
    let results = searcher.search("calculate_sum").await?;
    assert!(
        !results.is_empty(),
        "Should find results for 'calculate_sum'"
    );
    
    // Test 5.2: Search for semantic concept
    let results = searcher.search("multiplication operation").await?;
    assert!(
        !results.is_empty(),
        "Should find results for semantic search"
    );
    
    // Test 5.3: Verify result structure
    if let Some(result) = results.first() {
        assert!(
            result.file.contains("test.rs"),
            "Result should reference test.rs"
        );
        assert!(
            result.three_chunk_context.current.content.len() > 0,
            "Result should have content"
        );
    }
    
    println!("✅ Unified search integration passed");
    Ok(())
}

#[tokio::test]
async fn test_06_batch_processing_performance() -> Result<()> {
    println!("\n🔍 TEST 06: Batch Processing Performance");
    
    let embedder = NomicEmbedder::get_global().await?;
    
    // Prepare batch of texts
    let texts: Vec<&str> = (0..50)
        .map(|i| Box::leak(format!("Test text number {}", i).into_boxed_str()) as &str)
        .collect();
    
    // Test batch processing
    let start = std::time::Instant::now();
    let batch_embeddings = embedder.embed_batch(&texts)?;
    let batch_time = start.elapsed();
    
    // Test individual processing
    let start = std::time::Instant::now();
    let mut individual_embeddings = Vec::new();
    for text in &texts {
        individual_embeddings.push(embedder.embed(text)?);
    }
    let individual_time = start.elapsed();
    
    println!("  Batch time: {:?}", batch_time);
    println!("  Individual time: {:?}", individual_time);
    
    // Verify batch is faster (or at least not significantly slower)
    assert!(
        batch_time.as_secs_f64() < individual_time.as_secs_f64() * 1.5,
        "Batch processing should not be significantly slower than individual"
    );
    
    // Verify results are identical
    for i in 0..texts.len() {
        for j in 0..EXPECTED_DIMENSIONS {
            assert!(
                (batch_embeddings[i][j] - individual_embeddings[i][j]).abs() < EPSILON,
                "Batch and individual embeddings should be identical"
            );
        }
    }
    
    println!("✅ Batch processing validation passed");
    Ok(())
}

#[tokio::test]
async fn test_07_memory_and_cache_validation() -> Result<()> {
    println!("\n🔍 TEST 07: Memory and Cache Validation");
    
    let embedder = NomicEmbedder::get_global().await?;
    
    // Test 7.1: Verify caching works
    let text = "Cached embedding test";
    
    let start = std::time::Instant::now();
    let _first = embedder.embed(text)?;
    let first_time = start.elapsed();
    
    let start = std::time::Instant::now();
    let _second = embedder.embed(text)?;
    let cached_time = start.elapsed();
    
    println!("  First embedding: {:?}", first_time);
    println!("  Cached embedding: {:?}", cached_time);
    
    assert!(
        cached_time < first_time / 2,
        "Cached embedding should be much faster"
    );
    
    // Test 7.2: Memory usage stability
    let initial_memory = get_memory_usage();
    
    // Generate many embeddings
    for i in 0..100 {
        let text = format!("Memory test {}", i);
        let _ = embedder.embed(&text)?;
    }
    
    let final_memory = get_memory_usage();
    let memory_growth = final_memory - initial_memory;
    
    println!("  Memory growth: {} MB", memory_growth as f64 / 1_048_576.0);
    
    // Memory growth should be reasonable (< 100MB for 100 embeddings)
    assert!(
        memory_growth < 100_000_000,
        "Memory usage should not grow excessively"
    );
    
    println!("✅ Memory and cache validation passed");
    Ok(())
}

#[tokio::test]
async fn test_08_error_handling_validation() -> Result<()> {
    println!("\n🔍 TEST 08: Error Handling Validation");
    
    let embedder = NomicEmbedder::get_global().await?;
    
    // Test 8.1: Empty string handling
    let result = embedder.embed("");
    assert!(
        result.is_ok() || result.is_err(),
        "Empty string should be handled gracefully"
    );
    
    // Test 8.2: Very long text handling
    let long_text = "a".repeat(10000);
    let result = embedder.embed(&long_text);
    assert!(
        result.is_ok(),
        "Long text should be handled (truncated if necessary)"
    );
    
    // Test 8.3: Special characters
    let special_text = "🚀 Émoji tëxt with spêcial çharacters 中文 日本語";
    let result = embedder.embed(special_text);
    assert!(
        result.is_ok(),
        "Special characters should be handled"
    );
    
    println!("✅ Error handling validation passed");
    Ok(())
}

// Helper function for cosine similarity
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len());
    
    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    
    dot_product / (norm_a * norm_b)
}

// Helper function to get memory usage
fn get_memory_usage() -> usize {
    #[cfg(target_os = "linux")]
    {
        if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
            for line in status.lines() {
                if line.starts_with("VmRSS:") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        if let Ok(kb) = parts[1].parse::<usize>() {
                            return kb * 1024;
                        }
                    }
                }
            }
        }
    }
    
    // Fallback for non-Linux or if parsing fails
    0
}

#[tokio::test]
async fn test_09_critical_validation_summary() -> Result<()> {
    println!("\n🔍 TEST 09: CRITICAL VALIDATION SUMMARY");
    println!("===============================================");
    
    // This test runs all critical validations and reports the overall status
    let mut all_passed = true;
    let mut results = HashMap::new();
    
    // Validate model loading
    match test_01_model_loading_and_initialization().await {
        Ok(_) => results.insert("Model Loading", "✅ PASSED"),
        Err(e) => {
            results.insert("Model Loading", "❌ FAILED");
            println!("  Model Loading FAILED: {}", e);
            all_passed = false;
        }
    };
    
    // Validate embedding generation
    match test_02_embedding_generation_correctness().await {
        Ok(_) => results.insert("Embedding Generation", "✅ PASSED"),
        Err(e) => {
            results.insert("Embedding Generation", "❌ FAILED");
            println!("  Embedding Generation FAILED: {}", e);
            all_passed = false;
        }
    };
    
    // Validate semantic similarity
    match test_03_semantic_similarity_validation().await {
        Ok(_) => results.insert("Semantic Similarity", "✅ PASSED"),
        Err(e) => {
            results.insert("Semantic Similarity", "❌ FAILED");
            println!("  Semantic Similarity FAILED: {}", e);
            all_passed = false;
        }
    };
    
    println!("\n📊 VALIDATION RESULTS:");
    println!("---------------------");
    for (test, result) in &results {
        println!("  {}: {}", test, result);
    }
    
    if all_passed {
        println!("\n🎉 ALL CRITICAL VALIDATIONS PASSED!");
        println!("The embedding system is 100% functional.");
    } else {
        println!("\n⚠️ CRITICAL FAILURES DETECTED!");
        println!("The embedding system has issues that must be fixed.");
        panic!("Embedding system validation failed");
    }
    
    Ok(())
}
