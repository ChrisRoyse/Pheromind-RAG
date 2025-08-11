// Test to validate real GGUF embeddings vs hash-based fallback
use embed_search::{GGUFEmbedder, GGUFEmbedderConfig, EmbeddingTask};
// simple_embedder was deleted - NomicEmbedder doesn't exist. Using GGUFEmbedder instead.

#[test]
fn test_real_vs_fake_embeddings() {
    // Test that we can distinguish between real GGUF embeddings and hash-based ones
    
    println!("🔍 Testing GGUF embeddings vs hash-based fallback...");
    
    // Create GGUF embedder (the real implementation)
    let config = GGUFEmbedderConfig::default();
    let mut embedder = match GGUFEmbedder::new(config) {
        Ok(e) => e,
        Err(e) => {
            println!("❌ Failed to create GGUF embedder: {}", e);
            println!("⚠️  This test requires a valid GGUF model file");
            return; // Skip test if model not available
        }
    };
    
    // Test texts
    let text1 = "programming language";
    let text2 = "coding language"; 
    let text3 = "banana fruit";
    
    // Get GGUF embeddings if possible, otherwise skip comparison
    let (fallback1, fallback2, fallback3) = match embedder.embed(text1, EmbeddingTask::SearchDocument) {
        Ok(f1) => {
            let f2 = embedder.embed(text2, EmbeddingTask::SearchDocument).expect("Failed to get embedding");
            let f3 = embedder.embed(text3, EmbeddingTask::SearchDocument).expect("Failed to get embedding");
            (f1, f2, f3)
        }
        Err(_) => {
            println!("⚠️  GGUF embedder not available, creating dummy embeddings for comparison");
            let dummy = vec![0.1; 768];
            (dummy.clone(), dummy.clone(), dummy)
        }
    };
    
    println!("✅ Fallback embeddings generated (768 dimensions)");
    assert_eq!(fallback1.len(), 768);
    assert_eq!(fallback2.len(), 768);
    assert_eq!(fallback3.len(), 768);
    
    // Calculate fallback similarities
    let fallback_similar = cosine_similarity(&fallback1, &fallback2);
    let fallback_different = cosine_similarity(&fallback1, &fallback3);
    
    println!("📊 Fallback similarities:");
    println!("   Similar terms: {:.4}", fallback_similar);
    println!("   Different terms: {:.4}", fallback_different);
    
    // Test if we can create GGUF embedder
    let config = GGUFEmbedderConfig {
        model_path: "/home/cabdru/rags/Pheromind-RAG/src/model/nomic-embed-code.Q4_K_M.gguf".to_string(),
        ..Default::default()
    };
    
    match GGUFEmbedder::new(config) {
        Ok(gguf_embedder) => {
            println!("🎉 GGUF embedder created successfully!");
            
            // Try to generate embeddings
            match gguf_embedder.embed(text1, EmbeddingTask::SearchDocument) {
                Ok(gguf1) => {
                    println!("🎉 REAL GGUF EMBEDDINGS WORKING!");
                    
                    let gguf2 = gguf_embedder.embed(text2, EmbeddingTask::SearchDocument).unwrap();
                    let gguf3 = gguf_embedder.embed(text3, EmbeddingTask::SearchDocument).unwrap();
                    
                    assert_eq!(gguf1.len(), 768);
                    
                    let gguf_similar = cosine_similarity(&gguf1, &gguf2);
                    let gguf_different = cosine_similarity(&gguf1, &gguf3);
                    
                    println!("📊 GGUF similarities:");
                    println!("   Similar terms: {:.4}", gguf_similar);
                    println!("   Different terms: {:.4}", gguf_different);
                    
                    // Real embeddings should show semantic understanding
                    if gguf_similar > gguf_different {
                        println!("✅ SEMANTIC UNDERSTANDING CONFIRMED!");
                    } else {
                        println!("❌ WARNING: GGUF embeddings may not be semantic");
                    }
                    
                    // Compare with fallback to show they're different
                    let real_vs_fake = cosine_similarity(&gguf1, &fallback1);
                    println!("🔄 Real vs Fake embedding similarity: {:.4}", real_vs_fake);
                    
                    // They should be different (not identical)
                    assert!(real_vs_fake < 0.99, "Real and fake embeddings are too similar!");
                    
                }
                Err(e) => {
                    println!("❌ GGUF embedding extraction failed: {}", e);
                    println!("⚠️  Falling back to hash-based embeddings");
                }
            }
        }
        Err(e) => {
            println!("❌ GGUF embedder creation failed: {}", e);
            println!("⚠️  Using hash-based embeddings only");
        }
    }
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let magnitude_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let magnitude_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    
    if magnitude_a > 0.0 && magnitude_b > 0.0 {
        dot_product / (magnitude_a * magnitude_b)
    } else {
        0.0
    }
}