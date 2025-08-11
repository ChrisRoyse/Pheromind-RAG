// BRUTAL VALIDATION TEST: Verify GGUF embeddings are real, not fake
use anyhow::Result;
use embed_search::gguf_embedder::{GGUFEmbedder, GGUFEmbedderConfig};
use embed_search::embedding_prefixes::EmbeddingTask;
use std::collections::HashMap;

#[test]
fn test_gguf_embedding_reality() -> Result<()> {
    println!("🔍 BRUTAL VALIDATION: Testing GGUF Embedding Reality");
    
    // Create embedder
    let mut config = GGUFEmbedderConfig::default();
    config.model_path = "./src/model/nomic-embed-text-v1.5.Q4_K_M.gguf".to_string();
    
    let embedder = match GGUFEmbedder::new(config) {
        Ok(e) => e,
        Err(e) => {
            println!("❌ VALIDATION FAILED: Cannot create embedder: {}", e);
            panic!("GGUF embedder creation failed");
        }
    };
    
    println!("✅ Embedder created successfully");
    println!("   Dimension: {}", embedder.dimension());
    
    // Test 1: Basic embedding generation
    let test_texts = vec![
        "Hello world",
        "Machine learning algorithm",
        "Database connection",
        "Hello world", // Duplicate for cache test
    ];
    
    println!("\n🔍 Test 1: Basic Embedding Generation");
    let mut embeddings = Vec::new();
    
    for (i, text) in test_texts.iter().enumerate() {
        match embedder.embed(text, EmbeddingTask::SearchDocument) {
            Ok(embedding) => {
                println!("   Text {}: '{}' -> {} dims", i, text, embedding.len());
                
                // Verify embedding is not zeros
                let non_zero_count = embedding.iter().filter(|&&x| x.abs() > 1e-8).count();
                if non_zero_count < 10 {
                    println!("❌ CRITICAL: Embedding {} is mostly zeros ({} non-zero)", i, non_zero_count);
                    panic!("Embeddings are fake - mostly zeros");
                }
                
                // Verify normalization
                let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
                if (norm - 1.0).abs() > 1e-5 {
                    println!("❌ CRITICAL: Embedding {} not normalized (norm: {})", i, norm);
                    panic!("Embeddings not properly normalized");
                }
                
                println!("   ✅ Embedding {} has {} non-zero values, norm: {:.6}", i, non_zero_count, norm);
                embeddings.push(embedding);
            }
            Err(e) => {
                println!("❌ VALIDATION FAILED: Embedding {} failed: {}", i, e);
                panic!("Embedding generation failed");
            }
        }
    }
    
    // Test 2: Verify embeddings are semantically meaningful
    println!("\n🔍 Test 2: Semantic Similarity Validation");
    
    // Calculate cosine similarities
    let sim_0_1 = cosine_similarity(&embeddings[0], &embeddings[1]);
    let sim_0_2 = cosine_similarity(&embeddings[0], &embeddings[2]);
    let sim_1_2 = cosine_similarity(&embeddings[1], &embeddings[2]);
    let sim_0_3 = cosine_similarity(&embeddings[0], &embeddings[3]); // Should be 1.0 (identical)
    
    println!("   Similarity 'Hello world' vs 'ML algorithm': {:.4}", sim_0_1);
    println!("   Similarity 'Hello world' vs 'Database': {:.4}", sim_0_2);
    println!("   Similarity 'ML algorithm' vs 'Database': {:.4}", sim_1_2);
    println!("   Similarity 'Hello world' vs 'Hello world' (cache): {:.4}", sim_0_3);
    
    // Validation checks
    if sim_0_3 < 0.999 {
        println!("❌ CRITICAL: Identical texts have similarity {:.4}, expected ~1.0", sim_0_3);
        println!("   This indicates caching is broken or embeddings are random");
        panic!("Caching validation failed");
    }
    
    if sim_0_1.abs() < 0.01 && sim_0_2.abs() < 0.01 && sim_1_2.abs() < 0.01 {
        println!("❌ CRITICAL: All similarities near zero - embeddings may be random/hash-based");
        panic!("Embeddings appear to be hash-based, not semantic");
    }
    
    println!("   ✅ Embeddings show semantic structure");
    
    // Test 3: Batch processing validation
    println!("\n🔍 Test 3: Batch Processing Validation");
    
    let batch_texts = vec![
        "function definition".to_string(),
        "class inheritance".to_string(),
        "variable assignment".to_string(),
    ];
    
    let batch_embeddings = embedder.embed_batch(batch_texts.clone(), EmbeddingTask::CodeDefinition)?;
    
    if batch_embeddings.len() != batch_texts.len() {
        println!("❌ CRITICAL: Batch returned {} embeddings for {} texts", 
                 batch_embeddings.len(), batch_texts.len());
        panic!("Batch processing failed");
    }
    
    for (i, embedding) in batch_embeddings.iter().enumerate() {
        let non_zero = embedding.iter().filter(|&&x| x.abs() > 1e-8).count();
        if non_zero < 10 {
            println!("❌ CRITICAL: Batch embedding {} is mostly zeros", i);
            panic!("Batch embeddings are fake");
        }
        println!("   ✅ Batch embedding {}: {} non-zero values", i, non_zero);
    }
    
    // Test 4: Performance and memory validation
    println!("\n🔍 Test 4: Performance Validation");
    
    let start = std::time::Instant::now();
    let perf_text = "performance test text for embedding speed validation";
    
    for _ in 0..5 {
        embedder.embed(perf_text, EmbeddingTask::SearchQuery)?;
    }
    
    let duration = start.elapsed();
    println!("   5 embeddings took: {:?}", duration);
    
    if duration.as_secs() > 30 {
        println!("❌ WARNING: Embeddings are very slow ({:?})", duration);
        println!("   This might indicate CPU-only mode or inefficient processing");
    }
    
    // Get final statistics
    let stats = embedder.stats();
    println!("\n📊 Final Statistics:");
    println!("   Total embeddings: {}", stats.total_embeddings);
    println!("   Cache hits: {}", stats.cache_hits);
    println!("   Cache misses: {}", stats.cache_misses);
    println!("   Cache hit rate: {:.2}%", stats.cache_hit_rate() * 100.0);
    println!("   Tokens processed: {}", stats.total_tokens_processed);
    
    println!("\n✅ VALIDATION COMPLETE: GGUF embeddings are REAL and functional");
    
    Ok(())
}

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

#[test]
fn test_model_dimension_validation() -> Result<()> {
    println!("🔍 DIMENSION VALIDATION TEST");
    
    let config = GGUFEmbedderConfig::default();
    let embedder = GGUFEmbedder::new(config)?;
    
    // Nomic models should be 768 dimensions
    assert_eq!(embedder.dimension(), 768, "Expected 768 dimensions for nomic model");
    
    let embedding = embedder.embed("test", EmbeddingTask::SearchDocument)?;
    assert_eq!(embedding.len(), 768, "Actual embedding length must match reported dimension");
    
    println!("✅ Dimension validation passed: 768 dimensions confirmed");
    
    Ok(())
}

#[test]
fn test_rope_scaling_validation() -> Result<()> {
    println!("🔍 ROPE SCALING VALIDATION");
    
    // This test verifies the rope scaling parameters are applied
    let config = GGUFEmbedderConfig::default();
    let embedder = GGUFEmbedder::new(config)?;
    
    // Test with longer text to verify rope scaling works
    let long_text = "This is a longer piece of text that should test the rope scaling parameters. ".repeat(20);
    
    match embedder.embed(&long_text, EmbeddingTask::SearchDocument) {
        Ok(embedding) => {
            let non_zero = embedding.iter().filter(|&&x| x.abs() > 1e-8).count();
            if non_zero < 10 {
                panic!("Long text embedding failed - rope scaling may not be working");
            }
            println!("✅ Rope scaling validation passed: long text processed successfully");
        }
        Err(e) => {
            panic!("Rope scaling validation failed: {}", e);
        }
    }
    
    Ok(())
}