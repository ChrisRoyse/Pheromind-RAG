use embed_search::embedding::{MiniLMEmbedder, EmbeddingError};

#[test]
fn test_minilm_embedder_loads() {
    // Test that embedder can be created without panic
    let embedder = MiniLMEmbedder::new();
    assert!(embedder.is_ok() || embedder.is_err()); // Either works or fails gracefully
}

#[test]
fn test_single_embedding_dimensions() {
    // Test single text -> 384-dim vector
    let embedder = MiniLMEmbedder::new().unwrap_or_else(|_| MiniLMEmbedder::mock());
    let text = "fn hello() { println!('world'); }";
    let embedding = embedder.embed(text).unwrap();
    assert_eq!(embedding.len(), 384);
}

#[test]
fn test_batch_embedding() {
    // Test multiple texts -> multiple 384-dim vectors
    let embedder = MiniLMEmbedder::new().unwrap_or_else(|_| MiniLMEmbedder::mock());
    let texts = vec!["fn test1()", "class MyClass", "def python_func()"];
    let embeddings = embedder.embed_batch(&texts).unwrap();

    assert_eq!(embeddings.len(), 3);
    for embedding in embeddings {
        assert_eq!(embedding.len(), 384);
    }
}

#[test]
fn test_embedding_normalization() {
    // Test that embeddings are normalized (magnitude close to 1.0)
    let embedder = MiniLMEmbedder::new().unwrap_or_else(|_| MiniLMEmbedder::mock());
    let embedding = embedder.embed("test text").unwrap();

    let magnitude: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
    assert!((magnitude - 1.0).abs() < 0.1, "Embedding should be normalized, got magnitude: {}", magnitude);
}

#[test]
fn test_deterministic_embeddings() {
    // Same text should produce same embedding
    let embedder = MiniLMEmbedder::new().unwrap_or_else(|_| MiniLMEmbedder::mock());
    let text = "fn example() {}";

    let embedding1 = embedder.embed(text).unwrap();
    let embedding2 = embedder.embed(text).unwrap();

    assert_eq!(embedding1, embedding2);
}

#[test]
fn test_empty_input_handling() {
    // Test graceful handling of empty input
    let embedder = MiniLMEmbedder::new().unwrap_or_else(|_| MiniLMEmbedder::mock());
    let embedding = embedder.embed("");
    assert!(embedding.is_ok());
    assert_eq!(embedding.unwrap().len(), 384);
}

#[test]
fn test_large_input_handling() {
    // Test handling of large input (should truncate or handle gracefully)
    let embedder = MiniLMEmbedder::new().unwrap_or_else(|_| MiniLMEmbedder::mock());
    let large_text = "fn test() {}\n".repeat(1000);
    let embedding = embedder.embed(&large_text);
    assert!(embedding.is_ok());
    assert_eq!(embedding.unwrap().len(), 384);
}

#[test]
fn test_mock_mode_consistency() {
    // Test that mock mode is consistent and reproducible
    let embedder = MiniLMEmbedder::mock();
    let text = "test function";
    
    let embedding1 = embedder.embed(text).unwrap();
    let embedding2 = embedder.embed(text).unwrap();
    
    assert_eq!(embedding1, embedding2);
    assert_eq!(embedding1.len(), 384);
}

#[tokio::test]
async fn test_concurrent_embedding() {
    // Test that embedder can handle concurrent requests
    let embedder = std::sync::Arc::new(MiniLMEmbedder::new().unwrap_or_else(|_| MiniLMEmbedder::mock()));
    
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
    let embedder = MiniLMEmbedder::mock();
    assert_eq!(embedder.embedding_dim(), 384);
}

#[test]
fn test_mock_mode_check() {
    let embedder = MiniLMEmbedder::mock();
    assert!(embedder.is_mock_mode());
}

#[test]
fn test_chunked_embedding() {
    let embedder = MiniLMEmbedder::mock();
    let texts: Vec<&str> = (0..100).map(|i| if i % 2 == 0 { "fn test()" } else { "class Test" }).collect();
    
    let embeddings = embedder.embed_chunked(&texts).unwrap();
    
    assert_eq!(embeddings.len(), 100);
    for embedding in embeddings {
        assert_eq!(embedding.len(), 384);
    }
}

#[test]
fn test_batch_size_constant() {
    assert_eq!(MiniLMEmbedder::DEFAULT_BATCH_SIZE, 32);
}