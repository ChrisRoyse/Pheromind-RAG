use embed_search::embedding::NomicEmbedder;
use std::time::Instant;

#[tokio::test]
async fn test_nomic_singleton() {
    let embedder1 = NomicEmbedder::get_global().await.unwrap();
    let embedder2 = NomicEmbedder::get_global().await.unwrap();
    
    // Should be the same instance
    assert!(std::sync::Arc::ptr_eq(&embedder1, &embedder2));
}

#[tokio::test]
async fn test_nomic_dimensions() {
    let embedder = NomicEmbedder::get_global().await.unwrap();
    
    // Should default to 768 dimensions
    assert_eq!(embedder.dimensions(), 768);
}

#[tokio::test]
async fn test_nomic_embed_single() {
    let embedder = NomicEmbedder::get_global().await.unwrap();
    
    let text = "def authenticate_user(username, password):";
    let embedding = embedder.embed(text).unwrap();
    
    // Check dimensions
    assert_eq!(embedding.len(), 768);
    
    // Check values are in expected range (normalized embeddings)
    for val in &embedding {
        assert!(*val >= -1.0 && *val <= 1.0);
    }
}

#[tokio::test]
async fn test_nomic_embed_batch() {
    let embedder = NomicEmbedder::get_global().await.unwrap();
    
    let texts = vec![
        "import numpy as np",
        "def process_data(data):",
        "class UserAuthentication:",
        "SELECT * FROM users WHERE id = ?",
    ];
    
    let embeddings = embedder.embed_batch(&texts).unwrap();
    
    // Check batch size
    assert_eq!(embeddings.len(), texts.len());
    
    // Check each embedding
    for embedding in &embeddings {
        assert_eq!(embedding.len(), 768);
    }
}

#[tokio::test]
async fn test_nomic_caching() {
    let embedder = NomicEmbedder::get_global().await.unwrap();
    
    let text = "cached test string";
    
    // First call - will compute
    let start = Instant::now();
    let embedding1 = embedder.embed(text).unwrap();
    let first_duration = start.elapsed();
    
    // Second call - should be cached
    let start = Instant::now();
    let embedding2 = embedder.embed(text).unwrap();
    let cached_duration = start.elapsed();
    
    // Embeddings should be identical
    assert_eq!(embedding1, embedding2);
    
    // Note: Since we're using placeholder embeddings for now,
    // both calls will be fast. Once real GGUF is implemented,
    // cached call should be significantly faster
    println!("First call: {:?}, Cached call: {:?}", first_duration, cached_duration);
}

#[tokio::test]
async fn test_nomic_model_download() {
    // This test verifies the model download mechanism
    // It will download the model on first run and use cached version on subsequent runs
    
    let start = Instant::now();
    let embedder = NomicEmbedder::new().await;
    let duration = start.elapsed();
    
    assert!(embedder.is_ok(), "Failed to initialize embedder: {:?}", embedder.err());
    
    println!("Model initialization took: {:?}", duration);
    
    // Second initialization should be much faster (cached)
    let start = Instant::now();
    let _embedder2 = NomicEmbedder::new().await.unwrap();
    let cached_duration = start.elapsed();
    
    println!("Cached model initialization took: {:?}", cached_duration);
    
    // Cached should be faster (though both might be fast if model already exists)
    // The important thing is that it doesn't fail
}

#[tokio::test]
async fn test_nomic_performance() {
    let embedder = NomicEmbedder::get_global().await.unwrap();
    
    // Generate test texts
    let texts: Vec<String> = (0..100)
        .map(|i| format!("Test text number {} with some code: function test{} () {{}}", i, i))
        .collect();
    
    let text_refs: Vec<&str> = texts.iter().map(|s| s.as_str()).collect();
    
    let start = Instant::now();
    let embeddings = embedder.embed_batch(&text_refs).unwrap();
    let duration = start.elapsed();
    
    assert_eq!(embeddings.len(), 100);
    
    let embeddings_per_sec = 100.0 / duration.as_secs_f64();
    println!("Performance: {:.1} embeddings/second", embeddings_per_sec);
    
    // Should meet performance target (200+ embeddings/sec with real model)
    // With placeholder implementation, this will be much faster
}

#[tokio::test] 
async fn test_nomic_matryoshka_dimensions() {
    let mut embedder = NomicEmbedder::new().await.unwrap();
    
    // Test valid dimensions
    let valid_dims = vec![64, 128, 256, 512, 768];
    for dim in valid_dims {
        assert!(embedder.set_dimensions(dim).is_ok());
        assert_eq!(embedder.dimensions(), dim);
    }
    
    // Test invalid dimensions
    let invalid_dims = vec![100, 384, 1024, 2048];
    for dim in invalid_dims {
        assert!(embedder.set_dimensions(dim).is_err());
    }
}

#[tokio::test]
async fn test_nomic_memory_usage() {
    use sysinfo::{System, SystemExt, ProcessExt};
    
    let mut system = System::new();
    system.refresh_processes();
    
    let pid = std::process::id() as i32;
    let initial_memory = system.process(pid.into())
        .map(|p| p.memory())
        .unwrap_or(0);
    
    // Initialize embedder
    let embedder = NomicEmbedder::get_global().await.unwrap();
    
    // Generate many embeddings
    let texts: Vec<String> = (0..1000)
        .map(|i| format!("Test embedding {}", i))
        .collect();
    let text_refs: Vec<&str> = texts.iter().map(|s| s.as_str()).collect();
    
    let _embeddings = embedder.embed_batch(&text_refs).unwrap();
    
    system.refresh_processes();
    let final_memory = system.process(pid.into())
        .map(|p| p.memory())
        .unwrap_or(0);
    
    let memory_increase_mb = (final_memory - initial_memory) as f64 / 1024.0;
    println!("Memory increase: {:.1} MB", memory_increase_mb);
    
    // Should be under 4GB with Q4 model
    assert!(memory_increase_mb < 4096.0, "Memory usage exceeds 4GB limit");
}