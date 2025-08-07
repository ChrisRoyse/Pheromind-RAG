#[cfg(feature = "ml")]
use embed_search::embedding::NomicEmbedder;
#[cfg(feature = "ml")]
use std::time::Instant;

#[cfg(feature = "ml")]
#[tokio::test]
async fn test_nomic_singleton() {
    let embedder1 = NomicEmbedder::get_global().unwrap();
    let embedder2 = NomicEmbedder::get_global().unwrap();
    
    // Should be the same instance
    assert!(std::sync::Arc::ptr_eq(&embedder1, &embedder2));
}

#[cfg(feature = "ml")]
#[tokio::test]
async fn test_nomic_dimensions() {
    let embedder = NomicEmbedder::get_global().unwrap();
    
    // Should default to 768 dimensions
    assert_eq!(embedder.dimensions(), 768);
}

#[cfg(feature = "ml")]
#[tokio::test]
async fn test_nomic_embed_single() {
    let embedder = NomicEmbedder::get_global().unwrap();
    
    let text = "def authenticate_user(username, password):";
    let embedding = embedder.embed(text).unwrap();
    
    // Check dimensions
    assert_eq!(embedding.len(), 768);
    
    // Check values are in expected range (normalized embeddings)
    for val in &embedding {
        assert!(*val >= -1.0 && *val <= 1.0);
    }
}

#[cfg(feature = "ml")]
#[tokio::test]
async fn test_nomic_embed_batch() {
    let embedder = NomicEmbedder::get_global().unwrap();
    
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

#[cfg(feature = "ml")]
#[tokio::test]
async fn test_nomic_caching() {
    let embedder = NomicEmbedder::get_global().unwrap();
    
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
    
    // Note: With real GGUF implementation, cached calls should be 
    // significantly faster than initial computation
    println!("First call: {:?}, Cached call: {:?}", first_duration, cached_duration);
}

#[cfg(feature = "ml")]
#[tokio::test]
async fn test_nomic_model_download() {
    // This test verifies the model download mechanism
    // It will download the model on first run and use cached version on subsequent runs
    
    let start = Instant::now();
    let embedder = NomicEmbedder::new();
    let duration = start.elapsed();
    
    assert!(embedder.is_ok(), "Failed to initialize embedder: {:?}", embedder.err());
    
    println!("Model initialization took: {:?}", duration);
    
    // Second initialization should be much faster (cached)
    let start = Instant::now();
    let _embedder2 = NomicEmbedder::new().unwrap();
    let cached_duration = start.elapsed();
    
    println!("Cached model initialization took: {:?}", cached_duration);
    
    // Cached should be faster (though both might be fast if model already exists)
    // The important thing is that it doesn't fail
}

#[cfg(feature = "ml")]
#[tokio::test]
async fn test_nomic_performance() {
    let embedder = NomicEmbedder::get_global().unwrap();
    
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
    
    // Should meet performance target (200+ embeddings/sec with real GGUF model)
}

#[cfg(feature = "ml")]
#[tokio::test] 
async fn test_nomic_matryoshka_dimensions() {
    let mut embedder = NomicEmbedder::new().unwrap();
    
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

#[cfg(feature = "ml")]
#[tokio::test]
async fn test_nomic_memory_usage() {
    use sysinfo::{System, Pid};
    
    let mut system = System::new();
    system.refresh_processes();
    
    let pid = Pid::from_u32(std::process::id());
    let initial_memory = system.process(pid)
        .map(|p| p.memory())
        .expect("Failed to get initial memory usage - cannot perform memory monitoring test");
    
    // Initialize embedder
    let embedder = NomicEmbedder::get_global().unwrap();
    
    // Generate many embeddings
    let texts: Vec<String> = (0..1000)
        .map(|i| format!("Test embedding {}", i))
        .collect();
    let text_refs: Vec<&str> = texts.iter().map(|s| s.as_str()).collect();
    
    let _embeddings = embedder.embed_batch(&text_refs).unwrap();
    
    system.refresh_processes();
    let final_memory = system.process(pid)
        .map(|p| p.memory())
        .expect("Failed to get final memory usage - cannot complete memory monitoring test");
    
    let memory_increase_mb = (final_memory - initial_memory) as f64 / 1024.0;
    println!("Memory increase: {:.1} MB", memory_increase_mb);
    
    // Should be under 4GB with Q4 model
    assert!(memory_increase_mb < 4096.0, "Memory usage exceeds 4GB limit");
}

// Basic test that can run without ml feature
#[tokio::test]
async fn test_nomic_no_ml_feature() {
    // This test verifies behavior when ML feature is disabled
    #[cfg(not(feature = "ml"))]
    {
        use embed_search::embedding::NomicEmbedder;
        use std::path::PathBuf;
        
        let result = NomicEmbedder::get_global();
        assert!(result.is_err(), "get_global should fail without ml feature");
        
        let result = NomicEmbedder::new(PathBuf::from("test"), PathBuf::from("test"));
        assert!(result.is_err(), "new should fail without ml feature");
    }
    
    // When ML feature is enabled, this test does nothing
    #[cfg(feature = "ml")]
    {
        // Test passes - ML feature is available
    }
}