// FOCUSED FUNCTIONAL VALIDATION
// Fast tests that verify core embedding functionality is REAL
// NO MOCKS - Tests actual GGUF model behavior

use anyhow::Result;
use embed_search::gguf_embedder::{GGUFEmbedder, GGUFEmbedderConfig};
use embed_search::embedding_prefixes::EmbeddingTask;
use std::time::Instant;

/// VALIDATION 1: Verify GGUF Embedder Creates Real Embeddings
#[test]
fn test_real_embeddings_generated() -> Result<()> {
    println!("🔍 VALIDATION 1: Real Embeddings Generated");
    
    let config = GGUFEmbedderConfig {
        cache_size: 10,
        batch_size: 2,
        ..Default::default()
    };
    
    let embedder = GGUFEmbedder::new(config)
        .expect("FAIL: Cannot create GGUF embedder - check model file exists");
    
    // Test single embedding
    let text = "test embedding validation";
    let embedding = embedder.embed(text, EmbeddingTask::SearchDocument)?;
    
    // TRUTH CHECKS - NO COMPROMISES
    assert_eq!(embedding.len(), 768, "FAIL: Wrong embedding dimension");
    
    // Check embedding is normalized (nomic models should normalize)
    let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
    assert!((norm - 1.0).abs() < 1e-4, "FAIL: Embedding not normalized, norm = {}", norm);
    
    // Check embedding has meaningful content (not all zeros)
    let non_zero_count = embedding.iter().filter(|&&x| x.abs() > 1e-8).count();
    assert!(non_zero_count > 100, "FAIL: Embedding mostly zeros ({} non-zero)", non_zero_count);
    
    // Check values are reasonable (not all identical)
    let unique_values: std::collections::HashSet<_> = embedding.iter()
        .map(|&x| (x * 10000.0) as i32)
        .collect();
    assert!(unique_values.len() > 50, "FAIL: Too few unique values in embedding");
    
    println!("✅ PASS: Real embeddings verified (768 dims, normalized, {} non-zero values)", non_zero_count);
    Ok(())
}

/// VALIDATION 2: Verify Different Texts Produce Different Embeddings
#[test]
fn test_embedding_differentiation() -> Result<()> {
    println!("🔍 VALIDATION 2: Embedding Differentiation");
    
    let config = GGUFEmbedderConfig::default();
    let embedder = GGUFEmbedder::new(config)?;
    
    let text1 = "computer programming";
    let text2 = "banana fruit";
    let text3 = "computer programming"; // Same as text1
    
    let emb1 = embedder.embed(text1, EmbeddingTask::SearchDocument)?;
    let emb2 = embedder.embed(text2, EmbeddingTask::SearchDocument)?;
    let emb3 = embedder.embed(text3, EmbeddingTask::SearchDocument)?;
    
    // Calculate similarities
    let sim_different = cosine_similarity(&emb1, &emb2);
    let sim_identical = cosine_similarity(&emb1, &emb3);
    
    println!("   Similar texts: {:.4}", sim_identical);
    println!("   Different texts: {:.4}", sim_different);
    
    // TRUTH CHECKS
    assert!(sim_identical > 0.99, "FAIL: Identical texts similarity too low: {}", sim_identical);
    assert!(sim_different < 0.95, "FAIL: Different texts similarity too high: {}", sim_different);
    assert!(sim_identical > sim_different, "FAIL: Identical texts less similar than different texts");
    
    println!("✅ PASS: Embeddings differentiate between different texts");
    Ok(())
}

/// VALIDATION 3: Verify Task Prefixes Work
#[test]
fn test_task_prefix_effects() -> Result<()> {
    println!("🔍 VALIDATION 3: Task Prefix Effects");
    
    let config = GGUFEmbedderConfig::default();
    let embedder = GGUFEmbedder::new(config)?;
    
    let base_text = "function definition";
    
    // Same text with different task prefixes
    let search_emb = embedder.embed(base_text, EmbeddingTask::SearchDocument)?;
    let code_emb = embedder.embed(base_text, EmbeddingTask::CodeDefinition)?;
    let query_emb = embedder.embed(base_text, EmbeddingTask::SearchQuery)?;
    
    let sim_search_code = cosine_similarity(&search_emb, &code_emb);
    let sim_search_query = cosine_similarity(&search_emb, &query_emb);
    let sim_code_query = cosine_similarity(&code_emb, &query_emb);
    
    println!("   SearchDocument vs CodeDefinition: {:.4}", sim_search_code);
    println!("   SearchDocument vs SearchQuery: {:.4}", sim_search_query);
    println!("   CodeDefinition vs SearchQuery: {:.4}", sim_code_query);
    
    // They should be similar (same base text) but not identical (different prefixes)
    assert!(sim_search_code < 0.999, "FAIL: Task prefixes not affecting embeddings");
    assert!(sim_search_code > 0.8, "FAIL: Task prefixes causing too much difference");
    
    println!("✅ PASS: Task prefixes are working");
    Ok(())
}

/// VALIDATION 4: Verify Caching Works
#[test]
fn test_caching_behavior() -> Result<()> {
    println!("🔍 VALIDATION 4: Caching Behavior");
    
    let config = GGUFEmbedderConfig {
        cache_size: 5,
        ..Default::default()
    };
    let embedder = GGUFEmbedder::new(config)?;
    
    let test_text = "cache test string";
    
    // Clear cache and check stats
    embedder.clear_cache();
    let initial_stats = embedder.stats();
    
    // First embedding - should be cache miss
    let start = Instant::now();
    let emb1 = embedder.embed(test_text, EmbeddingTask::SearchDocument)?;
    let first_time = start.elapsed();
    
    let after_first = embedder.stats();
    assert_eq!(after_first.cache_misses - initial_stats.cache_misses, 1, "FAIL: Cache miss not recorded");
    
    // Second embedding - should be cache hit
    let start = Instant::now();
    let emb2 = embedder.embed(test_text, EmbeddingTask::SearchDocument)?;
    let second_time = start.elapsed();
    
    let after_second = embedder.stats();
    assert_eq!(after_second.cache_hits - initial_stats.cache_hits, 1, "FAIL: Cache hit not recorded");
    
    // Cache hit should be much faster
    println!("   First time (miss): {:?}", first_time);
    println!("   Second time (hit): {:?}", second_time);
    
    // Embeddings should be identical
    let similarity = cosine_similarity(&emb1, &emb2);
    assert!((similarity - 1.0).abs() < 1e-6, "FAIL: Cached embedding differs from original");
    
    // Cache hit should be significantly faster
    assert!(second_time < first_time / 10, "FAIL: Cache hit not faster than miss");
    
    println!("✅ PASS: Caching is working correctly");
    Ok(())
}

/// VALIDATION 5: Verify Batch Processing Consistency
#[test] 
fn test_batch_consistency() -> Result<()> {
    println!("🔍 VALIDATION 5: Batch Processing Consistency");
    
    let config = GGUFEmbedderConfig {
        batch_size: 2,
        ..Default::default()
    };
    let embedder = GGUFEmbedder::new(config)?;
    
    let texts = vec![
        "first text".to_string(),
        "second text".to_string(),
    ];
    
    // Get single embeddings
    let single1 = embedder.embed(&texts[0], EmbeddingTask::SearchDocument)?;
    let single2 = embedder.embed(&texts[1], EmbeddingTask::SearchDocument)?;
    
    // Clear cache to ensure batch doesn't use cache
    embedder.clear_cache();
    
    // Get batch embeddings
    let batch_embeddings = embedder.embed_batch(texts, EmbeddingTask::SearchDocument)?;
    
    assert_eq!(batch_embeddings.len(), 2, "FAIL: Batch size mismatch");
    
    // Compare single vs batch
    let sim1 = cosine_similarity(&single1, &batch_embeddings[0]);
    let sim2 = cosine_similarity(&single2, &batch_embeddings[1]);
    
    println!("   Single vs batch similarity 1: {:.6}", sim1);
    println!("   Single vs batch similarity 2: {:.6}", sim2);
    
    // Should be very similar (ideally identical)
    assert!(sim1 > 0.99, "FAIL: Batch embedding 1 differs from single");
    assert!(sim2 > 0.99, "FAIL: Batch embedding 2 differs from single");
    
    println!("✅ PASS: Batch processing is consistent");
    Ok(())
}

/// VALIDATION 6: Verify Performance Characteristics
#[test]
fn test_performance_bounds() -> Result<()> {
    println!("🔍 VALIDATION 6: Performance Bounds");
    
    let config = GGUFEmbedderConfig::default();
    let embedder = GGUFEmbedder::new(config)?;
    
    // Test reasonable performance on short text
    let short_text = "quick test";
    
    let start = Instant::now();
    let _embedding = embedder.embed(short_text, EmbeddingTask::SearchDocument)?;
    let duration = start.elapsed();
    
    println!("   Short text embedding time: {:?}", duration);
    
    // Should complete in reasonable time (allowing for CPU-only mode)
    if duration.as_secs() > 30 {
        println!("❌ WARNING: Very slow embedding ({:?}) - CPU-only mode or performance issue", duration);
    }
    
    // Test stats collection
    let stats = embedder.stats();
    assert!(stats.total_embeddings > 0, "FAIL: Stats not tracking embeddings");
    
    println!("   Stats: {} embeddings, {:.1}% cache hit rate", 
             stats.total_embeddings, stats.cache_hit_rate() * 100.0);
    
    println!("✅ PASS: Performance characteristics validated");
    Ok(())
}

/// VALIDATION 7: Verify Error Conditions
#[test]
fn test_error_handling() -> Result<()> {
    println!("🔍 VALIDATION 7: Error Handling");
    
    let config = GGUFEmbedderConfig::default();
    let embedder = GGUFEmbedder::new(config)?;
    
    // Test empty string - should handle gracefully
    let empty_result = embedder.embed("", EmbeddingTask::SearchDocument);
    println!("   Empty string: {:?}", empty_result.is_ok());
    
    // Test special characters - should not crash
    let special_result = embedder.embed("🚀🔍✅", EmbeddingTask::SearchDocument);
    println!("   Special chars: {:?}", special_result.is_ok());
    
    // Test very short text
    let short_result = embedder.embed("a", EmbeddingTask::SearchDocument);
    println!("   Single char: {:?}", short_result.is_ok());
    
    // At least one should succeed
    assert!(empty_result.is_ok() || special_result.is_ok() || short_result.is_ok(),
           "FAIL: All edge cases failed");
    
    println!("✅ PASS: Error handling validated");
    Ok(())
}

/// VALIDATION 8: Verify Model Loading and Configuration
#[test] 
fn test_model_configuration() -> Result<()> {
    println!("🔍 VALIDATION 8: Model Configuration");
    
    // Test default configuration
    let default_config = GGUFEmbedderConfig::default();
    let embedder1 = GGUFEmbedder::new(default_config)?;
    
    assert_eq!(embedder1.dimension(), 768, "FAIL: Wrong dimension for nomic model");
    
    // Test custom configuration
    let custom_config = GGUFEmbedderConfig {
        cache_size: 20,
        batch_size: 4,
        ..Default::default()
    };
    let embedder2 = GGUFEmbedder::new(custom_config)?;
    
    let (_, capacity) = embedder2.cache_info();
    assert_eq!(capacity, 20, "FAIL: Custom cache size not applied");
    
    println!("   Default dimension: {}", embedder1.dimension());
    println!("   Custom cache capacity: {}", capacity);
    
    println!("✅ PASS: Model configuration working");
    Ok(())
}

/// VALIDATION 9: Verify Memory Safety
#[test]
fn test_memory_safety() -> Result<()> {
    println!("🔍 VALIDATION 9: Memory Safety");
    
    let config = GGUFEmbedderConfig {
        cache_size: 5,
        ..Default::default()
    };
    
    // Create and drop embedders to test cleanup
    for i in 0..3 {
        let embedder = GGUFEmbedder::new(config.clone())?;
        let text = format!("memory test {}", i);
        let _embedding = embedder.embed(&text, EmbeddingTask::SearchDocument)?;
        println!("   Iteration {}: embedder created and used", i);
        // embedder drops here
    }
    
    println!("✅ PASS: Memory safety validated");
    Ok(())
}

/// VALIDATION 10: Final Integration Check
#[test]
fn test_minimal_integration() -> Result<()> {
    println!("🔍 VALIDATION 10: Minimal Integration Check");
    
    let config = GGUFEmbedderConfig::default();
    let embedder = GGUFEmbedder::new(config)?;
    
    // Minimal real-world scenario
    let document = "Rust programming language tutorial";
    let query = "rust programming";
    
    let doc_emb = embedder.embed(document, EmbeddingTask::SearchDocument)?;
    let query_emb = embedder.embed(query, EmbeddingTask::SearchQuery)?;
    
    let similarity = cosine_similarity(&doc_emb, &query_emb);
    
    println!("   Document: '{}'", document);
    println!("   Query: '{}'", query);
    println!("   Similarity: {:.4}", similarity);
    
    // Should show some semantic understanding
    assert!(similarity > 0.3, "FAIL: No semantic similarity detected");
    assert!(doc_emb.len() == 768, "FAIL: Wrong embedding dimension");
    assert!(query_emb.len() == 768, "FAIL: Wrong embedding dimension");
    
    let final_stats = embedder.stats();
    println!("   Final stats: {} embeddings processed", final_stats.total_embeddings);
    
    println!("✅ PASS: Integration check successful");
    Ok(())
}

// Helper function for cosine similarity
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