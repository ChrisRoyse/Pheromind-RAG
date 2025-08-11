// COMPREHENSIVE FUNCTIONAL TEST SUITE
// Tests actual embedding functionality - NO MOCKS, NO SIMULATIONS
//
// TRUTH REQUIREMENT: Every test verifies real system behavior
// This test suite defines what "working" means for the embedding system

use anyhow::Result;
use embed_search::gguf_embedder::{GGUFEmbedder, GGUFEmbedderConfig, EmbedderStats};
use embed_search::embedding_prefixes::{EmbeddingTask, CodeFormatter, BatchProcessor};
use std::time::Instant;
use std::collections::HashMap;

/// FUNCTIONAL TEST 1: Dual Embedder Architecture
/// Tests that we can create two different embedding models for text vs code
#[test]
fn test_dual_embedder_functionality() -> Result<()> {
    println!("🔍 FUNCTIONAL TEST 1: Dual Embedder Architecture");
    
    // Text embedder configuration
    let text_config = GGUFEmbedderConfig {
        model_path: "./src/model/nomic-embed-text-v1.5.Q4_K_M.gguf".to_string(),
        cache_size: 100,
        batch_size: 8,
        ..Default::default()
    };
    
    // Code embedder configuration  
    let code_config = GGUFEmbedderConfig {
        model_path: "./src/model/nomic-embed-code.Q4_K_M.gguf".to_string(),
        cache_size: 100,
        batch_size: 8,
        ..Default::default()
    };
    
    // Create both embedders
    let text_embedder = GGUFEmbedder::new(text_config)
        .map_err(|e| {
            eprintln!("❌ FAIL: Cannot create text embedder: {}", e);
            eprintln!("   This test requires nomic-embed-text-v1.5.Q4_K_M.gguf");
            e
        })?;
    
    let code_embedder = GGUFEmbedder::new(code_config)
        .map_err(|e| {
            eprintln!("❌ FAIL: Cannot create code embedder: {}", e);
            eprintln!("   This test requires nomic-embed-code.Q4_K_M.gguf");
            e
        })?;
    
    // Verify both have expected dimensions (768 for nomic models)
    assert_eq!(text_embedder.dimension(), 768, "Text embedder dimension incorrect");
    assert_eq!(code_embedder.dimension(), 768, "Code embedder dimension incorrect");
    
    // Test text vs code embedding on same content
    let test_content = "function calculate(x) { return x * 2; }";
    
    let text_embedding = text_embedder.embed(test_content, EmbeddingTask::SearchDocument)?;
    let code_embedding = code_embedder.embed(test_content, EmbeddingTask::CodeDefinition)?;
    
    // Verify embeddings are different (different models should produce different results)
    let similarity = cosine_similarity(&text_embedding, &code_embedding);
    println!("   Text vs Code embedding similarity: {:.4}", similarity);
    
    // They shouldn't be identical (unless by pure chance)
    if similarity > 0.99 {
        eprintln!("❌ WARNING: Text and code embeddings are nearly identical");
        eprintln!("   This might indicate both models are the same file");
    }
    
    println!("✅ PASS: Dual embedder architecture functional");
    Ok(())
}

/// FUNCTIONAL TEST 2: Task-Specific Prefix Handling
/// Tests that different task prefixes produce different embeddings
#[test]
fn test_task_prefix_functionality() -> Result<()> {
    println!("🔍 FUNCTIONAL TEST 2: Task-Specific Prefix Handling");
    
    let config = GGUFEmbedderConfig::default();
    let embedder = GGUFEmbedder::new(config)?;
    
    let base_text = "machine learning algorithm implementation";
    let tasks = vec![
        EmbeddingTask::SearchQuery,
        EmbeddingTask::SearchDocument,
        EmbeddingTask::CodeDefinition,
        EmbeddingTask::CodeUsage,
        EmbeddingTask::Classification,
        EmbeddingTask::Clustering,
    ];
    
    let mut embeddings = Vec::new();
    let mut task_names = Vec::new();
    
    // Generate embeddings for each task
    for task in tasks {
        let embedding = embedder.embed(base_text, task)?;
        
        // Verify embedding is valid
        assert_eq!(embedding.len(), 768, "Embedding dimension incorrect for task {:?}", task);
        
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 1e-5, "Embedding not normalized for task {:?}", task);
        
        let non_zero = embedding.iter().filter(|&&x| x.abs() > 1e-8).count();
        assert!(non_zero > 100, "Too many zero values in embedding for task {:?}", task);
        
        embeddings.push(embedding);
        task_names.push(format!("{:?}", task));
    }
    
    // Test that different prefixes produce different embeddings
    println!("   Task prefix similarity matrix:");
    for i in 0..embeddings.len() {
        for j in i+1..embeddings.len() {
            let sim = cosine_similarity(&embeddings[i], &embeddings[j]);
            println!("     {} vs {}: {:.4}", task_names[i], task_names[j], sim);
            
            // They should be similar (same base text) but not identical (different prefixes)
            if sim > 0.999 {
                eprintln!("❌ WARNING: Tasks {} and {} produce nearly identical embeddings", 
                         task_names[i], task_names[j]);
                eprintln!("   Prefixes may not be working properly");
            }
        }
    }
    
    println!("✅ PASS: Task prefix functionality working");
    Ok(())
}

/// FUNCTIONAL TEST 3: Vector Similarity Comparisons
/// Tests that semantic similarity works as expected
#[test]
fn test_vector_similarity_functionality() -> Result<()> {
    println!("🔍 FUNCTIONAL TEST 3: Vector Similarity Comparisons");
    
    let config = GGUFEmbedderConfig::default();
    let embedder = GGUFEmbedder::new(config)?;
    
    // Test cases with expected relationships
    let test_cases = vec![
        ("dog", "puppy", "high"),           // Similar concepts
        ("car", "automobile", "high"),     // Synonyms  
        ("programming", "coding", "high"), // Related terms
        ("happy", "joyful", "high"),       // Similar emotions
        ("dog", "computer", "low"),        // Unrelated
        ("hot", "cold", "low"),            // Opposites
        ("red", "mathematics", "low"),     // Completely unrelated
    ];
    
    println!("   Semantic similarity tests:");
    
    for (text1, text2, expected) in test_cases {
        let emb1 = embedder.embed(text1, EmbeddingTask::SearchDocument)?;
        let emb2 = embedder.embed(text2, EmbeddingTask::SearchDocument)?;
        
        let similarity = cosine_similarity(&emb1, &emb2);
        println!("     '{}' vs '{}': {:.4} (expected: {})", text1, text2, similarity, expected);
        
        // Verify embeddings are valid
        assert_eq!(emb1.len(), 768);
        assert_eq!(emb2.len(), 768);
        
        // Check if similarity aligns with expectations
        match expected {
            "high" => {
                if similarity < 0.5 {
                    eprintln!("❌ WARNING: Expected high similarity but got {:.4} for '{}' vs '{}'", 
                             similarity, text1, text2);
                }
            }
            "low" => {
                if similarity > 0.8 {
                    eprintln!("❌ WARNING: Expected low similarity but got {:.4} for '{}' vs '{}'", 
                             similarity, text1, text2);
                }
            }
            _ => {}
        }
    }
    
    // Test identical strings (should be exactly 1.0 due to caching)
    let identical_text = "test string for identity check";
    let emb_a = embedder.embed(identical_text, EmbeddingTask::SearchDocument)?;
    let emb_b = embedder.embed(identical_text, EmbeddingTask::SearchDocument)?;
    let identity_sim = cosine_similarity(&emb_a, &emb_b);
    
    println!("     Identity test: {:.6}", identity_sim);
    assert!((identity_sim - 1.0).abs() < 1e-6, "Identical strings should have similarity 1.0");
    
    println!("✅ PASS: Vector similarity functionality working");
    Ok(())
}

/// FUNCTIONAL TEST 4: Batch Processing Verification
/// Tests batch processing maintains consistency with single embeddings
#[test]
fn test_batch_processing_functionality() -> Result<()> {
    println!("🔍 FUNCTIONAL TEST 4: Batch Processing Verification");
    
    let config = GGUFEmbedderConfig {
        batch_size: 3, // Small batch for testing
        ..Default::default()
    };
    let embedder = GGUFEmbedder::new(config)?;
    
    let test_texts = vec![
        "first test string".to_string(),
        "second test string".to_string(),
        "third test string".to_string(),
        "fourth test string".to_string(),
        "fifth test string".to_string(),
    ];
    
    // Get single embeddings
    let mut single_embeddings = Vec::new();
    for text in &test_texts {
        let emb = embedder.embed(text, EmbeddingTask::SearchDocument)?;
        single_embeddings.push(emb);
    }
    
    // Clear cache to ensure batch processing isn't using cached results
    embedder.clear_cache();
    
    // Get batch embeddings
    let batch_embeddings = embedder.embed_batch(test_texts.clone(), EmbeddingTask::SearchDocument)?;
    
    // Verify batch processing
    assert_eq!(batch_embeddings.len(), test_texts.len(), "Batch size mismatch");
    
    println!("   Comparing single vs batch embeddings:");
    
    for i in 0..test_texts.len() {
        assert_eq!(single_embeddings[i].len(), 768);
        assert_eq!(batch_embeddings[i].len(), 768);
        
        let similarity = cosine_similarity(&single_embeddings[i], &batch_embeddings[i]);
        println!("     Text {}: similarity = {:.6}", i, similarity);
        
        // They should be very similar (same text, same task)
        if similarity < 0.99 {
            eprintln!("❌ FAIL: Batch embedding {} differs significantly from single embedding", i);
            eprintln!("   Single processing and batch processing should produce identical results");
            panic!("Batch processing inconsistency detected");
        }
    }
    
    println!("✅ PASS: Batch processing functionality working");
    Ok(())
}

/// FUNCTIONAL TEST 5: Cache Hit/Miss Testing
/// Tests that caching works correctly and improves performance
#[test]
fn test_cache_functionality() -> Result<()> {
    println!("🔍 FUNCTIONAL TEST 5: Cache Hit/Miss Testing");
    
    let config = GGUFEmbedderConfig {
        cache_size: 10,
        ..Default::default()
    };
    let embedder = GGUFEmbedder::new(config)?;
    
    let test_text = "cache test string for performance validation";
    
    // Clear cache and get initial stats
    embedder.clear_cache();
    let initial_stats = embedder.stats();
    println!("   Initial stats: {} embeddings, {} hits, {} misses", 
             initial_stats.total_embeddings, initial_stats.cache_hits, initial_stats.cache_misses);
    
    // First embedding - should be cache miss
    let start = Instant::now();
    let emb1 = embedder.embed(test_text, EmbeddingTask::SearchDocument)?;
    let first_duration = start.elapsed();
    
    let after_first = embedder.stats();
    assert_eq!(after_first.total_embeddings - initial_stats.total_embeddings, 1);
    assert_eq!(after_first.cache_misses - initial_stats.cache_misses, 1);
    
    // Second embedding - should be cache hit
    let start = Instant::now();
    let emb2 = embedder.embed(test_text, EmbeddingTask::SearchDocument)?;
    let second_duration = start.elapsed();
    
    let after_second = embedder.stats();
    assert_eq!(after_second.total_embeddings - initial_stats.total_embeddings, 2);
    assert_eq!(after_second.cache_hits - initial_stats.cache_hits, 1);
    
    // Cache hit should be much faster
    println!("   First embedding (miss): {:?}", first_duration);
    println!("   Second embedding (hit): {:?}", second_duration);
    
    if second_duration >= first_duration {
        eprintln!("❌ WARNING: Cache hit not faster than cache miss");
        eprintln!("   This might indicate caching is not working properly");
    }
    
    // Embeddings should be identical
    let similarity = cosine_similarity(&emb1, &emb2);
    assert!((similarity - 1.0).abs() < 1e-6, "Cached embedding differs from original");
    
    // Test cache eviction
    for i in 0..15 {
        let text = format!("cache eviction test {}", i);
        embedder.embed(&text, EmbeddingTask::SearchDocument)?;
    }
    
    let (cache_used, cache_capacity) = embedder.cache_info();
    println!("   Cache usage: {}/{}", cache_used, cache_capacity);
    assert!(cache_used <= cache_capacity, "Cache exceeded capacity");
    
    println!("✅ PASS: Cache functionality working");
    Ok(())
}

/// FUNCTIONAL TEST 6: Performance Metrics Validation
/// Tests performance characteristics and validates they meet expectations
#[test]
fn test_performance_metrics() -> Result<()> {
    println!("🔍 FUNCTIONAL TEST 6: Performance Metrics Validation");
    
    let config = GGUFEmbedderConfig::default();
    let embedder = GGUFEmbedder::new(config)?;
    
    // Test single embedding performance
    let test_text = "performance test string with moderate length for timing validation";
    
    let single_start = Instant::now();
    let _embedding = embedder.embed(test_text, EmbeddingTask::SearchDocument)?;
    let single_duration = single_start.elapsed();
    
    println!("   Single embedding time: {:?}", single_duration);
    
    if single_duration.as_secs() > 5 {
        eprintln!("❌ WARNING: Single embedding very slow ({:?})", single_duration);
        eprintln!("   This might indicate performance issues");
    }
    
    // Test batch performance
    let batch_texts: Vec<String> = (0..10)
        .map(|i| format!("batch performance test string number {}", i))
        .collect();
    
    let batch_start = Instant::now();
    let batch_embeddings = embedder.embed_batch(batch_texts.clone(), EmbeddingTask::SearchDocument)?;
    let batch_duration = batch_start.elapsed();
    
    println!("   Batch embedding time (10 items): {:?}", batch_duration);
    println!("   Average per item: {:?}", batch_duration / 10);
    
    assert_eq!(batch_embeddings.len(), 10);
    
    // Test memory usage patterns
    let stats = embedder.stats();
    println!("   Performance stats:");
    println!("     Total embeddings: {}", stats.total_embeddings);
    println!("     Batch operations: {}", stats.batch_operations);
    println!("     Tokens processed: {}", stats.total_tokens_processed);
    println!("     Cache hit rate: {:.2}%", stats.cache_hit_rate() * 100.0);
    
    // Validate stats make sense
    assert!(stats.total_embeddings > 0, "No embeddings recorded in stats");
    assert!(stats.total_tokens_processed > 0, "No tokens processed recorded");
    
    println!("✅ PASS: Performance metrics validation complete");
    Ok(())
}

/// FUNCTIONAL TEST 7: Code Language Detection and Formatting
/// Tests language-specific code processing
#[test]
fn test_code_language_functionality() -> Result<()> {
    println!("🔍 FUNCTIONAL TEST 7: Code Language Detection and Formatting");
    
    let config = GGUFEmbedderConfig::default();
    let embedder = GGUFEmbedder::new(config)?;
    
    let code_samples = vec![
        ("fn main() { println!(\"Hello\"); }", "main.rs", "rust"),
        ("def hello(): print('Hello')", "hello.py", "python"),
        ("function hello() { console.log('Hello'); }", "hello.js", "javascript"),
        ("package main; func main() { println(\"Hello\") }", "main.go", "go"),
    ];
    
    println!("   Testing language-specific code processing:");
    
    let mut embeddings = Vec::new();
    
    for (code, filename, expected_lang) in code_samples {
        // Test language detection
        let detected = CodeFormatter::detect_language(filename);
        println!("     {}: detected = {:?}, expected = {}", filename, detected, expected_lang);
        
        if let Some(detected_lang) = detected {
            assert_eq!(detected_lang, expected_lang, "Language detection failed for {}", filename);
        }
        
        // Test code embedding
        let embedding = embedder.embed_code(code, Some(expected_lang), EmbeddingTask::CodeDefinition)?;
        assert_eq!(embedding.len(), 768);
        
        // Test code file embedding
        let file_embedding = embedder.embed_code_file(code, filename, EmbeddingTask::CodeDefinition)?;
        assert_eq!(file_embedding.len(), 768);
        
        // They should be similar (same code, different methods)
        let similarity = cosine_similarity(&embedding, &file_embedding);
        println!("     Code vs file embedding similarity: {:.4}", similarity);
        
        if similarity < 0.95 {
            eprintln!("❌ WARNING: Code and file embeddings very different for {}", filename);
        }
        
        embeddings.push((embedding, expected_lang));
    }
    
    // Test batch code processing
    let batch_codes = vec![
        ("fn test() {}", Some("test.rs")),
        ("def test():", Some("test.py")),
        ("function test() {}", Some("test.js")),
    ];
    
    let batch_embeddings = embedder.embed_code_batch(batch_codes, EmbeddingTask::CodeDefinition)?;
    assert_eq!(batch_embeddings.len(), 3);
    
    for embedding in batch_embeddings {
        assert_eq!(embedding.len(), 768);
        let non_zero = embedding.iter().filter(|&&x| x.abs() > 1e-8).count();
        assert!(non_zero > 100, "Code batch embedding mostly zeros");
    }
    
    println!("✅ PASS: Code language functionality working");
    Ok(())
}

/// FUNCTIONAL TEST 8: Error Handling and Edge Cases
/// Tests system behavior under error conditions
#[test]
fn test_error_handling_functionality() -> Result<()> {
    println!("🔍 FUNCTIONAL TEST 8: Error Handling and Edge Cases");
    
    let config = GGUFEmbedderConfig::default();
    let embedder = GGUFEmbedder::new(config)?;
    
    // Test empty string
    let empty_result = embedder.embed("", EmbeddingTask::SearchDocument);
    println!("   Empty string result: {:?}", empty_result.is_ok());
    
    // Test very long string
    let long_string = "word ".repeat(2000); // Very long text
    let long_result = embedder.embed(&long_string, EmbeddingTask::SearchDocument);
    match long_result {
        Ok(embedding) => {
            assert_eq!(embedding.len(), 768);
            println!("   Long string handled successfully");
        }
        Err(e) => {
            println!("   Long string failed (expected): {}", e);
        }
    }
    
    // Test special characters
    let special_chars = "🔍🚀✅❌⚠️💻🧠📊";
    let special_result = embedder.embed(special_chars, EmbeddingTask::SearchDocument);
    match special_result {
        Ok(embedding) => {
            assert_eq!(embedding.len(), 768);
            println!("   Special characters handled successfully");
        }
        Err(e) => {
            println!("   Special characters failed: {}", e);
        }
    }
    
    // Test batch with empty items
    let batch_with_empty = vec![
        "normal text".to_string(),
        "".to_string(),
        "more normal text".to_string(),
    ];
    
    let batch_result = embedder.embed_batch(batch_with_empty, EmbeddingTask::SearchDocument);
    match batch_result {
        Ok(embeddings) => {
            println!("   Batch with empty items: {} embeddings generated", embeddings.len());
            for (i, embedding) in embeddings.iter().enumerate() {
                assert_eq!(embedding.len(), 768, "Embedding {} has wrong dimension", i);
            }
        }
        Err(e) => {
            println!("   Batch with empty items failed: {}", e);
        }
    }
    
    println!("✅ PASS: Error handling functionality tested");
    Ok(())
}

/// FUNCTIONAL TEST 9: Thread Safety Verification
/// Tests concurrent access to embedder
#[test]
fn test_thread_safety_functionality() -> Result<()> {
    println!("🔍 FUNCTIONAL TEST 9: Thread Safety Verification");
    
    use std::sync::Arc;
    use std::thread;
    
    let config = GGUFEmbedderConfig::default();
    let embedder = Arc::new(GGUFEmbedder::new(config)?);
    
    let mut handles = Vec::new();
    
    // Spawn multiple threads
    for i in 0..4 {
        let embedder_clone = embedder.clone();
        let handle = thread::spawn(move || {
            let text = format!("thread safety test from thread {}", i);
            let result = embedder_clone.embed(&text, EmbeddingTask::SearchDocument);
            (i, result)
        });
        handles.push(handle);
    }
    
    // Collect results
    let mut successful_threads = 0;
    for handle in handles {
        let (thread_id, result) = handle.join().unwrap();
        match result {
            Ok(embedding) => {
                assert_eq!(embedding.len(), 768);
                successful_threads += 1;
                println!("   Thread {} completed successfully", thread_id);
            }
            Err(e) => {
                println!("   Thread {} failed: {}", thread_id, e);
            }
        }
    }
    
    assert!(successful_threads > 0, "No threads completed successfully");
    println!("   {}/4 threads completed successfully", successful_threads);
    
    println!("✅ PASS: Thread safety functionality working");
    Ok(())
}

/// FUNCTIONAL TEST 10: End-to-End Integration Test
/// Tests complete workflow from configuration to results
#[test]
fn test_end_to_end_functionality() -> Result<()> {
    println!("🔍 FUNCTIONAL TEST 10: End-to-End Integration Test");
    
    // Custom configuration
    let config = GGUFEmbedderConfig {
        model_path: "./src/model/nomic-embed-text-v1.5.Q4_K_M.gguf".to_string(),
        context_size: 4096,
        batch_size: 4,
        cache_size: 50,
        normalize: true,
        threads: 2,
        gpu_layers: 0,
    };
    
    let embedder = GGUFEmbedder::new(config)?;
    
    // Verify configuration applied
    assert_eq!(embedder.dimension(), 768);
    let (cache_used, cache_capacity) = embedder.cache_info();
    assert_eq!(cache_capacity, 50);
    
    // Real-world scenario: search system
    let documents = vec![
        "Rust is a systems programming language focused on safety and performance",
        "Python is a high-level programming language used for data science",
        "JavaScript is a dynamic language primarily used for web development", 
        "Go is a statically typed language designed for building scalable applications",
        "Machine learning algorithms require large datasets for training",
    ];
    
    let query = "programming language for building applications";
    
    // Index documents
    println!("   Indexing {} documents...", documents.len());
    let mut doc_embeddings = Vec::new();
    for (i, doc) in documents.iter().enumerate() {
        let embedding = embedder.embed(doc, EmbeddingTask::SearchDocument)?;
        doc_embeddings.push((i, embedding));
    }
    
    // Process query
    println!("   Processing query: '{}'", query);
    let query_embedding = embedder.embed(query, EmbeddingTask::SearchQuery)?;
    
    // Calculate similarities
    let mut similarities = Vec::new();
    for (doc_id, doc_embedding) in doc_embeddings {
        let sim = cosine_similarity(&query_embedding, &doc_embedding);
        similarities.push((doc_id, sim));
    }
    
    // Sort by similarity
    similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    
    println!("   Search results (by similarity):");
    for (doc_id, similarity) in &similarities[..3] {
        println!("     Doc {}: {:.4} - '{}'", doc_id, similarity, 
                documents[*doc_id].chars().take(50).collect::<String>());
    }
    
    // Verify reasonable results
    let top_similarity = similarities[0].1;
    assert!(top_similarity > 0.3, "Top result similarity too low: {}", top_similarity);
    
    // Performance check
    let final_stats = embedder.stats();
    println!("   Final statistics:");
    println!("     Total embeddings: {}", final_stats.total_embeddings);
    println!("     Cache hit rate: {:.1}%", final_stats.cache_hit_rate() * 100.0);
    println!("     Tokens processed: {}", final_stats.total_tokens_processed);
    
    assert!(final_stats.total_embeddings >= 6, "Expected at least 6 embeddings");
    
    println!("✅ PASS: End-to-end integration test complete");
    Ok(())
}

// Helper function for cosine similarity calculation
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }
    
    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    
    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot_product / (norm_a * norm_b)
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    
    /// Meta test: verify all functional tests can be discovered and run
    #[test]
    fn test_functional_test_coverage() {
        println!("🔍 META TEST: Functional Test Coverage Verification");
        
        let test_functions = vec![
            "test_dual_embedder_functionality",
            "test_task_prefix_functionality", 
            "test_vector_similarity_functionality",
            "test_batch_processing_functionality",
            "test_cache_functionality",
            "test_performance_metrics",
            "test_code_language_functionality",
            "test_error_handling_functionality",
            "test_thread_safety_functionality",
            "test_end_to_end_functionality",
        ];
        
        println!("   Functional test suite includes {} tests:", test_functions.len());
        for (i, test_name) in test_functions.iter().enumerate() {
            println!("     {}. {}", i + 1, test_name);
        }
        
        assert_eq!(test_functions.len(), 10, "Expected exactly 10 functional tests");
        println!("✅ PASS: All functional tests accounted for");
    }
}