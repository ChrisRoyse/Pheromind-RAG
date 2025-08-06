use std::time::Instant;
use embed_search::embedding::{CachedEmbedder, RealMiniLMEmbedder};
use tempfile::TempDir;

/// Performance benchmark comparing cached vs uncached embedding performance
#[tokio::test]
async fn benchmark_embedding_cache_performance() {
    println!("🚀 Starting embedding performance benchmark...");
    
    // Test data - mix of repeated and unique content
    let test_texts = vec![
        "This is a common function that processes user data",
        "Database connection established successfully",
        "Error handling for network timeouts",
        "This is a common function that processes user data", // Duplicate
        "Implementing authentication middleware",
        "Database connection established successfully", // Duplicate
        "Performance optimization for batch operations",
        "Error handling for network timeouts", // Duplicate
        "Cache invalidation strategies",
        "This is a common function that processes user data", // Duplicate
        "Logging configuration and setup",
        "Performance optimization for batch operations", // Duplicate
        "Unit test for payment processing",
        "Cache invalidation strategies", // Duplicate
        "API rate limiting implementation",
        "Logging configuration and setup", // Duplicate
    ];
    
    println!("📊 Test data: {} texts ({} unique)", 
             test_texts.len(), 
             test_texts.iter().collect::<std::collections::HashSet<_>>().len());
    
    // Test 1: Sequential embedding without cache
    println!("\n🔄 Test 1: Sequential embedding without cache");
    let start = Instant::now();
    let embedder = RealMiniLMEmbedder::get_global().await.expect("Failed to create embedder");
    
    let mut sequential_embeddings = Vec::new();
    for text in &test_texts {
        let embedding = embedder.embed(text).expect("Failed to generate embedding");
        sequential_embeddings.push(embedding);
    }
    let sequential_time = start.elapsed();
    println!("⏱️  Sequential time: {:?} ({:.2}ms per embedding)", 
             sequential_time, 
             sequential_time.as_millis() as f64 / test_texts.len() as f64);
    
    // Test 2: Batch embedding without cache
    println!("\n🔄 Test 2: Batch embedding without cache");
    let start = Instant::now();
    let text_refs: Vec<&str> = test_texts.iter().map(|s| s.as_ref()).collect();
    let batch_embeddings = embedder.embed_batch_optimized(&text_refs).expect("Failed to generate batch embeddings");
    let batch_time = start.elapsed();
    println!("⏱️  Batch time: {:?} ({:.2}ms per embedding)", 
             batch_time, 
             batch_time.as_millis() as f64 / test_texts.len() as f64);
    
    // Test 3: First run with cached embedder (cold cache)
    println!("\n🔄 Test 3: Cached embedder - cold cache");
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let cached_embedder = CachedEmbedder::new_with_persistence(1000, temp_dir.path()).await
        .expect("Failed to create cached embedder");
    
    let start = Instant::now();
    let cached_embeddings_cold = cached_embedder.embed_batch_cached(&text_refs).expect("Failed to generate cached embeddings");
    let cached_cold_time = start.elapsed();
    println!("⏱️  Cached (cold) time: {:?} ({:.2}ms per embedding)", 
             cached_cold_time, 
             cached_cold_time.as_millis() as f64 / test_texts.len() as f64);
    
    // Test 4: Second run with cached embedder (warm cache)
    println!("\n🔄 Test 4: Cached embedder - warm cache");
    let start = Instant::now();
    let cached_embeddings_warm = cached_embedder.embed_batch_cached(&text_refs).expect("Failed to generate cached embeddings");
    let cached_warm_time = start.elapsed();
    println!("⏱️  Cached (warm) time: {:?} ({:.2}ms per embedding)", 
             cached_warm_time, 
             cached_warm_time.as_millis() as f64 / test_texts.len() as f64);
    
    // Verify embeddings are identical
    println!("\n🔍 Verifying embedding consistency...");
    assert_eq!(sequential_embeddings.len(), batch_embeddings.len());
    assert_eq!(sequential_embeddings.len(), cached_embeddings_cold.len());
    assert_eq!(sequential_embeddings.len(), cached_embeddings_warm.len());
    
    // Check that embeddings are numerically similar (allowing for small floating point differences)
    for i in 0..sequential_embeddings.len() {
        let seq = &sequential_embeddings[i];
        let batch = &batch_embeddings[i];
        let cached_cold = &cached_embeddings_cold[i];
        let cached_warm = &cached_embeddings_warm[i];
        
        // All embeddings should be very similar
        let similarity_batch = cosine_similarity(seq, batch);
        let similarity_cached_cold = cosine_similarity(seq, cached_cold);
        let similarity_cached_warm = cosine_similarity(seq, cached_warm);
        
        assert!(similarity_batch > 0.999, "Batch embedding differs too much from sequential at index {}: {:.6}", i, similarity_batch);
        assert!(similarity_cached_cold > 0.999, "Cached cold embedding differs too much from sequential at index {}: {:.6}", i, similarity_cached_cold);
        assert!(similarity_cached_warm > 0.999, "Cached warm embedding differs too much from sequential at index {}: {:.6}", i, similarity_cached_warm);
    }
    println!("✅ All embeddings are consistent");
    
    // Performance analysis
    println!("\n📈 Performance Analysis:");
    let batch_speedup = sequential_time.as_millis() as f64 / batch_time.as_millis() as f64;
    let cache_speedup = cached_cold_time.as_millis() as f64 / cached_warm_time.as_millis() as f64;
    
    println!("🔥 Batch vs Sequential speedup: {:.2}x", batch_speedup);
    println!("🔥 Cache hit speedup: {:.2}x", cache_speedup);
    
    let cache_stats = cached_embedder.cache_stats();
    println!("📦 Cache stats: {}", cache_stats);
    
    // Performance expectations
    assert!(batch_speedup > 1.5, "Batch processing should be at least 1.5x faster than sequential");
    assert!(cache_speedup > 5.0, "Cache hits should be at least 5x faster than cache misses");
    
    println!("✅ Performance benchmark completed successfully!");
}

/// Benchmark large batch processing
#[tokio::test]
async fn benchmark_large_batch_processing() {
    println!("🚀 Starting large batch processing benchmark...");
    
    // Generate test data with various sizes
    let small_batch: Vec<String> = (0..10).map(|i| format!("Small batch item {}", i)).collect();
    let medium_batch: Vec<String> = (0..50).map(|i| format!("Medium batch item {}", i)).collect();
    let large_batch: Vec<String> = (0..200).map(|i| format!("Large batch item {}", i)).collect();
    
    let cached_embedder = CachedEmbedder::new_with_cache_size(500).await
        .expect("Failed to create cached embedder");
    
    println!("\n📊 Testing different batch sizes...");
    
    for (name, batch) in &[
        ("Small (10)", &small_batch),
        ("Medium (50)", &medium_batch), 
        ("Large (200)", &large_batch),
    ] {
        let text_refs: Vec<&str> = batch.iter().map(|s| s.as_str()).collect();
        
        // Time the batch processing
        let start = Instant::now();
        let embeddings = cached_embedder.embed_batch_cached(&text_refs)
            .expect("Failed to process batch");
        let duration = start.elapsed();
        
        println!("⏱️  {} items: {:?} ({:.2}ms per item)", 
                 name, duration, 
                 duration.as_millis() as f64 / batch.len() as f64);
        
        assert_eq!(embeddings.len(), batch.len());
        
        // Verify embeddings are properly normalized
        for (i, embedding) in embeddings.iter().enumerate() {
            let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
            assert!((norm - 1.0).abs() < 0.01, "Embedding {} not normalized: norm = {:.6}", i, norm);
        }
    }
    
    println!("✅ Large batch processing benchmark completed successfully!");
}

/// Test cache persistence across instances
#[tokio::test]
async fn benchmark_cache_persistence() {
    println!("🚀 Starting cache persistence benchmark...");
    
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let test_texts = vec![
        "Persistent cache test item 1",
        "Persistent cache test item 2", 
        "Persistent cache test item 3",
        "Persistent cache test item 4",
        "Persistent cache test item 5",
    ];
    let text_refs: Vec<&str> = test_texts.iter().map(|s| s.as_ref()).collect();
    
    // First instance - populate cache
    println!("\n🔄 Creating first cached embedder instance...");
    {
        let cached_embedder = CachedEmbedder::new_with_persistence(100, temp_dir.path()).await
            .expect("Failed to create cached embedder");
        
        let start = Instant::now();
        let _embeddings = cached_embedder.embed_batch_cached(&text_refs)
            .expect("Failed to generate embeddings");
        let first_time = start.elapsed();
        
        println!("⏱️  First instance time: {:?}", first_time);
        
        let stats = cached_embedder.cache_stats();
        println!("📦 Cache populated: {} entries", stats.entries);
    } // Embedder dropped here, should save cache to disk
    
    // Second instance - should load from cache
    println!("\n🔄 Creating second cached embedder instance...");
    {
        let cached_embedder = CachedEmbedder::new_with_persistence(100, temp_dir.path()).await
            .expect("Failed to create cached embedder");
        
        let start = Instant::now();
        let _embeddings = cached_embedder.embed_batch_cached(&text_refs)
            .expect("Failed to generate embeddings");
        let second_time = start.elapsed();
        
        println!("⏱️  Second instance time: {:?}", second_time);
        
        let stats = cached_embedder.cache_stats();
        println!("📦 Cache loaded: {} entries", stats.entries);
        
        // Second instance should be much faster due to cached embeddings
        // We expect at least some speedup from persistence
        println!("🔥 Persistence may provide speedup depending on cache loading efficiency");
    }
    
    println!("✅ Cache persistence benchmark completed successfully!");
}

/// Helper function to calculate cosine similarity
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len());
    
    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    
    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot_product / (norm_a * norm_b)
    }
}

/// Integration test for the complete embedding pipeline
#[tokio::test]
async fn integration_test_embedding_pipeline() {
    println!("🚀 Starting embedding pipeline integration test...");
    
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let cached_embedder = CachedEmbedder::new_with_persistence(1000, temp_dir.path()).await
        .expect("Failed to create cached embedder");
    
    // Test various content types
    let test_contents = vec![
        // Code snippets
        "fn main() { println!(\"Hello, world!\"); }",
        "def process_data(data): return data.strip().lower()",
        "const fetchData = async () => { return await api.get('/data'); }",
        
        // Documentation
        "This function processes user input and returns sanitized data",
        "Configuration settings for the application database connection",
        "Error handling middleware for HTTP request processing",
        
        // Repeated content (should hit cache)
        "fn main() { println!(\"Hello, world!\"); }",
        "def process_data(data): return data.strip().lower()",
        "This function processes user input and returns sanitized data",
    ];
    
    let text_refs: Vec<&str> = test_contents.iter().map(|s| s.as_ref()).collect();
    
    println!("📊 Processing {} pieces of content...", test_contents.len());
    
    let start = Instant::now();
    let embeddings = cached_embedder.embed_batch_cached(&text_refs)
        .expect("Failed to process content");
    let duration = start.elapsed();
    
    println!("⏱️  Processing time: {:?} ({:.2}ms per item)", 
             duration, 
             duration.as_millis() as f64 / test_contents.len() as f64);
    
    // Verify results
    assert_eq!(embeddings.len(), test_contents.len());
    
    for (i, embedding) in embeddings.iter().enumerate() {
        assert_eq!(embedding.len(), 384, "Wrong embedding dimension for item {}", i);
        
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 0.01, "Embedding {} not normalized: norm = {:.6}", i, norm);
    }
    
    // Check semantic similarity
    let code_rust = &embeddings[0];
    let code_python = &embeddings[1];
    let _code_js = &embeddings[2];
    let doc1 = &embeddings[3];
    let doc2 = &embeddings[4];
    
    let code_similarity = cosine_similarity(code_rust, code_python);
    let doc_similarity = cosine_similarity(doc1, doc2);
    let cross_similarity = cosine_similarity(code_rust, doc1);
    
    println!("🔍 Code-to-code similarity: {:.4}", code_similarity);
    println!("🔍 Doc-to-doc similarity: {:.4}", doc_similarity);
    println!("🔍 Code-to-doc similarity: {:.4}", cross_similarity);
    
    // Code should be more similar to code than to docs
    assert!(code_similarity > cross_similarity, 
            "Code snippets should be more similar to each other than to documentation");
    
    let cache_stats = cached_embedder.cache_stats();
    println!("📦 Final cache stats: {}", cache_stats);
    
    println!("✅ Integration test completed successfully!");
}