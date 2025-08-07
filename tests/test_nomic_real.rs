//! BRUTAL REALITY CHECK: Does Nomic Embedding Actually Work?
//! 
//! This test attempts to load the Nomic model and generate real embeddings.
//! If this fails, it means the embedding system is phantom code.
//! 
//! Run with: cargo test test_nomic_real --features ml -- --nocapture

#[cfg(feature = "ml")]
use embed_search::embedding::NomicEmbedder;
#[cfg(feature = "ml")]
use std::time::Instant;

/// CRITICAL TEST: Can we actually load the Nomic model?
#[cfg(feature = "ml")]
#[tokio::test]
async fn test_nomic_model_loading() {
    println!("\n=== BRUTAL REALITY CHECK: NOMIC MODEL LOADING ===");
    
    let start = Instant::now();
    println!("Attempting to load Nomic embedder...");
    
    match NomicEmbedder::get_global().await {
        Ok(embedder) => {
            let load_time = start.elapsed();
            println!("✅ SUCCESS: Model loaded in {:?}", load_time);
            println!("📊 Dimensions: {}", embedder.dimensions());
        }
        Err(e) => {
            println!("❌ FAILURE: Cannot load model: {:?}", e);
            panic!("PHANTOM CODE DETECTED: Model loading failed - {}", e);
        }
    }
}

/// CRITICAL TEST: Can we actually generate embeddings?
#[cfg(feature = "ml")]
#[tokio::test]
async fn test_nomic_embedding_generation() {
    println!("\n=== BRUTAL REALITY CHECK: EMBEDDING GENERATION ===");
    
    let embedder = match NomicEmbedder::get_global().await {
        Ok(e) => e,
        Err(e) => {
            panic!("PHANTOM CODE: Cannot get embedder - {}", e);
        }
    };
    
    let test_text = "hello world";
    println!("Generating embedding for: '{}'", test_text);
    
    let start = Instant::now();
    
    match embedder.embed(test_text) {
        Ok(embedding) => {
            let generation_time = start.elapsed();
            println!("✅ SUCCESS: Embedding generated in {:?}", generation_time);
            
            // Verify dimensions
            let actual_dims = embedding.len();
            println!("📊 Actual dimensions: {}", actual_dims);
            
            if actual_dims != 768 {
                panic!("DIMENSION MISMATCH: Expected 768, got {}", actual_dims);
            }
            
            // Print first 5 values for verification
            println!("🔢 First 5 values: {:?}", &embedding[0..5]);
            
            // Verify values are not all zeros (phantom check)
            let non_zero_count = embedding.iter().filter(|&&x| x != 0.0).count();
            println!("📈 Non-zero values: {}/{}", non_zero_count, actual_dims);
            
            if non_zero_count == 0 {
                panic!("PHANTOM CODE: All embedding values are zero!");
            }
            
            // Verify values are reasonable (not NaN or infinite)
            let invalid_count = embedding.iter()
                .filter(|&&x| x.is_nan() || x.is_infinite())
                .count();
            
            if invalid_count > 0 {
                panic!("INVALID VALUES: {} NaN/infinite values detected", invalid_count);
            }
            
            println!("✅ REALITY CHECK PASSED: Embeddings are real and valid");
            
        }
        Err(e) => {
            println!("❌ FAILURE: Cannot generate embedding: {:?}", e);
            panic!("PHANTOM CODE DETECTED: Embedding generation failed - {}", e);
        }
    }
}

/// CRITICAL TEST: Can we generate different embeddings for different texts?
#[cfg(feature = "ml")]
#[tokio::test]
async fn test_nomic_embedding_uniqueness() {
    println!("\n=== BRUTAL REALITY CHECK: EMBEDDING UNIQUENESS ===");
    
    let embedder = match NomicEmbedder::get_global().await {
        Ok(e) => e,
        Err(e) => {
            panic!("PHANTOM CODE: Cannot get embedder - {}", e);
        }
    };
    
    let text1 = "hello world";
    let text2 = "goodbye universe";
    
    println!("Generating embeddings for different texts...");
    
    let embedding1 = match embedder.embed(text1) {
        Ok(e) => e,
        Err(e) => panic!("Failed to embed text1: {}", e),
    };
    
    let embedding2 = match embedder.embed(text2) {
        Ok(e) => e,
        Err(e) => panic!("Failed to embed text2: {}", e),
    };
    
    // Calculate cosine similarity to ensure they're different
    let dot_product: f32 = embedding1.iter()
        .zip(embedding2.iter())
        .map(|(a, b)| a * b)
        .sum();
    
    let norm1: f32 = embedding1.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm2: f32 = embedding2.iter().map(|x| x * x).sum::<f32>().sqrt();
    
    let cosine_similarity = dot_product / (norm1 * norm2);
    
    println!("🔍 Cosine similarity: {:.4}", cosine_similarity);
    
    // They shouldn't be identical (similarity = 1.0) or opposite (similarity = -1.0)
    if (cosine_similarity - 1.0).abs() < 0.001 {
        panic!("PHANTOM CODE: Embeddings are identical for different texts!");
    }
    
    if (cosine_similarity + 1.0).abs() < 0.001 {
        panic!("PHANTOM CODE: Embeddings are exact opposites!");
    }
    
    println!("✅ REALITY CHECK PASSED: Embeddings are unique for different texts");
}

/// CRITICAL TEST: Performance check - can we generate multiple embeddings?
#[cfg(feature = "ml")]
#[tokio::test]
async fn test_nomic_batch_performance() {
    println!("\n=== BRUTAL REALITY CHECK: BATCH PERFORMANCE ===");
    
    let embedder = match NomicEmbedder::get_global().await {
        Ok(e) => e,
        Err(e) => {
            panic!("PHANTOM CODE: Cannot get embedder - {}", e);
        }
    };
    
    let test_texts = vec![
        "function hello() { return 'world'; }",
        "def goodbye(): print('universe')",
        "public static void main(String[] args)",
        "use std::collections::HashMap;",
        "import numpy as np",
    ];
    
    println!("Generating {} embeddings for performance test...", test_texts.len());
    
    let start = Instant::now();
    let mut embeddings = Vec::new();
    
    for (i, text) in test_texts.iter().enumerate() {
        match embedder.embed(text) {
            Ok(embedding) => {
                embeddings.push(embedding);
                println!("  ✅ Embedding {} generated", i + 1);
            }
            Err(e) => {
                panic!("PHANTOM CODE: Failed to generate embedding {}: {}", i + 1, e);
            }
        }
    }
    
    let total_time = start.elapsed();
    let avg_time = total_time / test_texts.len() as u32;
    
    println!("📊 Total time: {:?}", total_time);
    println!("📊 Average time per embedding: {:?}", avg_time);
    
    // Verify all embeddings have correct dimensions
    for (i, embedding) in embeddings.iter().enumerate() {
        if embedding.len() != 768 {
            panic!("DIMENSION ERROR: Embedding {} has {} dimensions, expected 768", 
                   i + 1, embedding.len());
        }
    }
    
    println!("✅ REALITY CHECK PASSED: Batch embedding generation works");
}

/// Fallback test for when ML feature is not enabled
#[cfg(not(feature = "ml"))]
#[test]
fn test_ml_feature_disabled() {
    println!("\n=== ML FEATURE NOT ENABLED ===");
    println!("Run with: cargo test test_nomic_real --features ml -- --nocapture");
    println!("This test suite requires the 'ml' feature to be enabled.");
}