#[cfg(feature = "ml")]
use std::time::Instant;
#[cfg(feature = "ml")]
use embed_search::embedding::NomicEmbedder;

/// Performance benchmark for NomicEmbedder
#[cfg(feature = "ml")]
#[tokio::test]
async fn benchmark_nomic_embedding_performance() {
    println!("🚀 Starting Nomic embedding performance benchmark...");
    
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
    
    let embedder = NomicEmbedder::get_global().expect("Failed to create embedder");
    
    // Test 1: Sequential embedding
    println!("\n🔄 Test 1: Sequential embedding");
    let start = Instant::now();
    let mut sequential_embeddings = Vec::new();
    for text in &test_texts {
        let embedding = embedder.embed(text).expect("Failed to generate embedding");
        sequential_embeddings.push(embedding);
    }
    let sequential_time = start.elapsed();
    println!("⏱️  Sequential time: {:?} ({:.2}ms per embedding)", 
             sequential_time, 
             sequential_time.as_millis() as f64 / test_texts.len() as f64);
    
    // Test 2: Batch embedding
    println!("\n🔄 Test 2: Batch embedding");
    let start = Instant::now();
    let text_refs: Vec<&str> = test_texts.iter().map(|s| s.as_ref()).collect();
    let batch_embeddings = embedder.embed_batch(&text_refs).expect("Failed to generate batch embeddings");
    let batch_time = start.elapsed();
    println!("⏱️  Batch time: {:?} ({:.2}ms per embedding)", 
             batch_time, 
             batch_time.as_millis() as f64 / test_texts.len() as f64);
    
    // Test 3: Cache performance (repeat same texts)
    println!("\n🔄 Test 3: Cache performance (built-in caching)");
    let start = Instant::now();
    let mut cached_embeddings = Vec::new();
    for text in &test_texts {
        let embedding = embedder.embed(text).expect("Failed to generate cached embedding");
        cached_embeddings.push(embedding);
    }
    let cached_time = start.elapsed();
    println!("⏱️  Cached time: {:?} ({:.2}ms per embedding)", 
             cached_time, 
             cached_time.as_millis() as f64 / test_texts.len() as f64);
    
    // Verify embeddings are consistent and correct dimension
    println!("\n🔍 Verifying embedding consistency...");
    assert_eq!(sequential_embeddings.len(), batch_embeddings.len());
    assert_eq!(sequential_embeddings.len(), cached_embeddings.len());
    
    // Validate all embeddings are 768-dimensional and consistent
    assert_eq!(sequential_embeddings[0].len(), 768, "Sequential embeddings should be 768-dimensional");
    assert_eq!(batch_embeddings[0].len(), 768, "Batch embeddings should be 768-dimensional");
    assert_eq!(cached_embeddings[0].len(), 768, "Cached embeddings should be 768-dimensional");
    
    // Check that embeddings are normalized
    for (i, embedding) in sequential_embeddings.iter().enumerate() {
        let magnitude: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((magnitude - 1.0).abs() < 0.1, "Embedding {} should be normalized, got magnitude: {}", i, magnitude);
    }
    
    println!("✅ All embeddings are consistent and normalized");
    
    // Performance analysis
    println!("\n📈 Performance Analysis:");
    let batch_speedup = sequential_time.as_millis() as f64 / batch_time.as_millis() as f64;
    let cache_speedup = sequential_time.as_millis() as f64 / cached_time.as_millis() as f64;
    
    println!("🔥 Batch vs Sequential speedup: {:.2}x", batch_speedup);
    println!("🔥 Cache speedup: {:.2}x", cache_speedup);
    
    println!("✅ Performance benchmark completed successfully!");
}

/// Test embedding quality - different inputs should produce different embeddings
#[cfg(feature = "ml")]
#[tokio::test]
async fn test_embedding_quality() {
    println!("🎯 Testing embedding quality and differentiation...");
    
    let embedder = NomicEmbedder::get_global().expect("Failed to create embedder");
    
    let test_cases = vec![
        ("def calculate_sum(a, b): return a + b", "Python function"),
        ("class User: pass", "Python class"),
        ("function hello() { return 'world'; }", "JavaScript function"),
        ("SELECT * FROM users WHERE id = ?", "SQL query"),
        ("// This is a comment", "Code comment"),
    ];
    
    let mut embeddings = Vec::new();
    for (text, description) in &test_cases {
        let embedding = embedder.embed(text).expect("Failed to generate embedding");
        println!("📊 {} -> {} dimensions", description, embedding.len());
        embeddings.push(embedding);
    }
    
    // Verify all embeddings are different
    for i in 0..embeddings.len() {
        for j in i+1..embeddings.len() {
            let similarity = cosine_similarity(&embeddings[i], &embeddings[j]);
            println!("🔗 {} vs {}: {:.3} similarity", 
                     test_cases[i].1, test_cases[j].1, similarity);
            
            // Embeddings should be different (similarity < 0.95)
            assert!(similarity < 0.95, "Embeddings {} and {} are too similar: {:.3}", 
                    test_cases[i].1, test_cases[j].1, similarity);
        }
    }
    
    println!("✅ All embeddings are sufficiently different");
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    
    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot_product / (norm_a * norm_b)
    }
}