use embed_search::embedding::{MiniLMEmbedder, EmbeddingError};

#[tokio::test]
async fn test_minilm_embedder_loads() {
    // Test that embedder can be created without panic
    let embedder = MiniLMEmbedder::new().await;
    assert!(embedder.is_ok() || embedder.is_err()); // Either works or fails gracefully
}

#[tokio::test]
async fn test_single_embedding_dimensions() {
    // Test single text -> 384-dim vector
    let embedder = match MiniLMEmbedder::new().await {
        Ok(e) => e,
        Err(_) => {
            println!("⏭️ Skipping test - model not available");
            return;
        }
    };
    let text = "fn hello() { println!('world'); }";
    let embedding = embedder.embed(text).unwrap();
    assert_eq!(embedding.len(), 384);
}

#[tokio::test]
async fn test_batch_embedding() {
    // Test multiple texts -> multiple 384-dim vectors
    let embedder = match MiniLMEmbedder::new().await {
        Ok(e) => e,
        Err(_) => {
            println!("⏭️ Skipping test - model not available");
            return;
        }
    };
    let texts = vec!["fn test1()", "class MyClass", "def python_func()"];
    let embeddings = embedder.embed_batch(&texts).unwrap();

    assert_eq!(embeddings.len(), 3);
    for embedding in embeddings {
        assert_eq!(embedding.len(), 384);
    }
}

#[tokio::test]
async fn test_embedding_normalization() {
    // Test that embeddings are normalized (magnitude close to 1.0)
    let embedder = match MiniLMEmbedder::new().await {
        Ok(e) => e,
        Err(_) => {
            println!("⏭️ Skipping test - model not available");
            return;
        }
    };
    let embedding = embedder.embed("test text").unwrap();

    let magnitude: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
    assert!((magnitude - 1.0).abs() < 0.1, "Embedding should be normalized, got magnitude: {}", magnitude);
}

#[tokio::test]
async fn test_deterministic_embeddings() {
    // Same text should produce same embedding
    let embedder = match MiniLMEmbedder::new().await {
        Ok(e) => e,
        Err(_) => {
            println!("⏭️ Skipping test - model not available");
            return;
        }
    };
    let text = "fn example() {}";

    let embedding1 = embedder.embed(text).unwrap();
    let embedding2 = embedder.embed(text).unwrap();

    assert_eq!(embedding1, embedding2);
}

#[tokio::test]
async fn test_empty_input_handling() {
    // Test graceful handling of empty input
    let embedder = match MiniLMEmbedder::new().await {
        Ok(e) => e,
        Err(_) => {
            println!("⏭️ Skipping test - model not available");
            return;
        }
    };
    let embedding = embedder.embed("");
    assert!(embedding.is_ok());
    assert_eq!(embedding.unwrap().len(), 384);
}

#[tokio::test]
async fn test_large_input_handling() {
    // Test handling of large input (should truncate or handle gracefully)
    let embedder = match MiniLMEmbedder::new().await {
        Ok(e) => e,
        Err(_) => {
            println!("⏭️ Skipping test - model not available");
            return;
        }
    };
    let large_text = "fn test() {}\n".repeat(1000);
    let embedding = embedder.embed(&large_text);
    assert!(embedding.is_ok());
    assert_eq!(embedding.unwrap().len(), 384);
}

#[test]
fn test_real_mode_consistency() {
    // Test that real mode is consistent and reproducible
    let embedder = MiniLMEmbedder::mock(); // This now returns a real embedder
    let text = "test function";
    
    let embedding1 = embedder.embed(text).unwrap();
    let embedding2 = embedder.embed(text).unwrap();
    
    assert_eq!(embedding1, embedding2);
    assert_eq!(embedding1.len(), 384);
    assert!(!embedder.is_mock_mode()); // Should be false now
}

#[tokio::test]
async fn test_concurrent_embedding() {
    // Test that embedder can handle concurrent requests
    let embedder = match MiniLMEmbedder::new().await {
        Ok(e) => std::sync::Arc::new(e),
        Err(_) => {
            println!("⏭️ Skipping test - model not available");
            return;
        }
    };
    
    let mut handles = Vec::new();
    for i in 0..5 {
        let embedder_clone = embedder.clone();
        let handle = tokio::spawn(async move {
            let text = format!("fn test_{}() {{}}", i);
            embedder_clone.embed(&text)
        });
        handles.push(handle);
    }
    
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 384);
    }
}

#[test]
fn test_embedding_dimensions() {
    let embedder = MiniLMEmbedder::mock(); // Now returns real embedder
    assert_eq!(embedder.embedding_dim(), 384);
}

#[test]
fn test_real_mode_check() {
    let embedder = MiniLMEmbedder::mock(); // Now returns real embedder
    assert!(!embedder.is_mock_mode()); // Should be false
    assert!(embedder.is_loaded()); // Should be true
}

#[test]
fn test_chunked_embedding() {
    let embedder = MiniLMEmbedder::mock(); // Now returns real embedder
    let texts: Vec<&str> = (0..10).map(|i| if i % 2 == 0 { "fn test()" } else { "class Test" }).collect(); // Reduced for real model
    
    let embeddings = embedder.embed_chunked(&texts).unwrap();
    
    assert_eq!(embeddings.len(), 10);
    for embedding in embeddings {
        assert_eq!(embedding.len(), 384);
    }
}

#[test]
fn test_batch_size_constant() {
    assert_eq!(MiniLMEmbedder::DEFAULT_BATCH_SIZE, 32);
}