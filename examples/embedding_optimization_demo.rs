use std::time::Instant;
use embed_search::embedding::{CachedEmbedder, RealMiniLMEmbedder};
use tempfile::TempDir;

/// Simple demonstration of embedding performance optimizations
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Embedding Performance Optimization Demo");
    println!("===========================================\n");

    // Sample code snippets that might be found in a codebase
    let code_samples = vec![
        "fn process_user_data(input: &str) -> String { input.trim().to_lowercase() }",
        "def authenticate_user(username, password): return check_credentials(username, password)",
        "const handleApiRequest = async (req, res) => { return await processRequest(req); }",
        "fn process_user_data(input: &str) -> String { input.trim().to_lowercase() }", // Duplicate
        "class DatabaseManager { public connect() { return this.pool.connect(); } }",
        "def authenticate_user(username, password): return check_credentials(username, password)", // Duplicate
        "async function validateInput(data) { return schema.validate(data); }",
        "fn process_user_data(input: &str) -> String { input.trim().to_lowercase() }", // Duplicate
    ];

    println!("📊 Test data: {} code snippets ({} unique)", 
             code_samples.len(), 
             code_samples.iter().collect::<std::collections::HashSet<_>>().len());

    // Test 1: Sequential processing without caching
    println!("\n1️⃣ Sequential Embedding (No Cache)");
    println!("   Processing each snippet individually...");
    
    let embedder = RealMiniLMEmbedder::get_global().await?;
    let start = Instant::now();
    
    let mut sequential_embeddings = Vec::new();
    for (i, code) in code_samples.iter().enumerate() {
        let embedding = embedder.embed(code)?;
        println!("   ✓ Embedded snippet {} ({} dims)", i + 1, embedding.len());
        sequential_embeddings.push(embedding);
    }
    
    let sequential_time = start.elapsed();
    println!("   ⏱️  Total time: {:?} ({:.1}ms per snippet)", 
             sequential_time, 
             sequential_time.as_millis() as f64 / code_samples.len() as f64);

    // Test 2: Batch processing (optimized tensor operations)
    println!("\n2️⃣ Batch Embedding (Tensor Optimization)");
    println!("   Processing all snippets in optimized batches...");
    
    let code_refs: Vec<&str> = code_samples.iter().map(|s| s.as_ref()).collect();
    let start = Instant::now();
    
    let batch_embeddings = embedder.embed_batch_optimized(&code_refs)?;
    let batch_time = start.elapsed();
    
    println!("   ✓ Embedded {} snippets in batches", batch_embeddings.len());
    println!("   ⏱️  Total time: {:?} ({:.1}ms per snippet)", 
             batch_time, 
             batch_time.as_millis() as f64 / code_samples.len() as f64);

    let batch_speedup = sequential_time.as_millis() as f64 / batch_time.as_millis() as f64;
    println!("   🚀 Speedup: {:.2}x faster than sequential", batch_speedup);

    // Test 3: Cached embedding (first run - cache population)
    println!("\n3️⃣ Cached Embedding (Cold Cache)");
    println!("   First run - populating cache...");
    
    let temp_dir = TempDir::new()?;
    let cached_embedder = CachedEmbedder::new_with_persistence(1000, temp_dir.path()).await?;
    
    let start = Instant::now();
    let cached_embeddings_cold = cached_embedder.embed_batch_cached(&code_refs)?;
    let cached_cold_time = start.elapsed();
    
    println!("   ✓ Embedded {} snippets with caching", cached_embeddings_cold.len());
    println!("   ⏱️  Total time: {:?} ({:.1}ms per snippet)", 
             cached_cold_time, 
             cached_cold_time.as_millis() as f64 / code_samples.len() as f64);

    let cache_stats = cached_embedder.cache_stats();
    println!("   📦 Cache now contains {} entries", cache_stats.entries);

    // Test 4: Cached embedding (second run - cache hits)
    println!("\n4️⃣ Cached Embedding (Warm Cache)");
    println!("   Second run - utilizing cached embeddings...");
    
    let start = Instant::now();
    let cached_embeddings_warm = cached_embedder.embed_batch_cached(&code_refs)?;
    let cached_warm_time = start.elapsed();
    
    println!("   ✓ Retrieved {} embeddings (mix of cached and new)", cached_embeddings_warm.len());
    println!("   ⏱️  Total time: {:?} ({:.1}ms per snippet)", 
             cached_warm_time, 
             cached_warm_time.as_millis() as f64 / code_samples.len() as f64);

    let cache_speedup = cached_cold_time.as_millis() as f64 / cached_warm_time.as_millis() as f64;
    println!("   🚀 Cache speedup: {:.2}x faster than cold cache", cache_speedup);

    // Performance Summary
    println!("\n📈 Performance Summary:");
    println!("======================");
    println!("Sequential:      {:>8.1}ms ({:.1}ms/item)", 
             sequential_time.as_millis(), 
             sequential_time.as_millis() as f64 / code_samples.len() as f64);
    println!("Batch:           {:>8.1}ms ({:.1}ms/item) - {:.1}x speedup", 
             batch_time.as_millis(), 
             batch_time.as_millis() as f64 / code_samples.len() as f64,
             batch_speedup);
    println!("Cached (cold):   {:>8.1}ms ({:.1}ms/item)", 
             cached_cold_time.as_millis(), 
             cached_cold_time.as_millis() as f64 / code_samples.len() as f64);
    println!("Cached (warm):   {:>8.1}ms ({:.1}ms/item) - {:.1}x speedup", 
             cached_warm_time.as_millis(), 
             cached_warm_time.as_millis() as f64 / code_samples.len() as f64,
             cache_speedup);

    // Verify embeddings are consistent
    println!("\n🔍 Verifying Consistency:");
    println!("==========================");
    
    // Check that all methods produce similar embeddings
    for i in 0..code_samples.len() {
        let seq = &sequential_embeddings[i];
        let batch = &batch_embeddings[i];
        let cached_cold = &cached_embeddings_cold[i];
        let cached_warm = &cached_embeddings_warm[i];
        
        let similarity_batch = cosine_similarity(seq, batch);
        let similarity_cached = cosine_similarity(seq, cached_cold);
        let similarity_warm = cosine_similarity(cached_cold, cached_warm);
        
        println!("Snippet {}: batch={:.4}, cached={:.4}, warm={:.4}", 
                 i + 1, similarity_batch, similarity_cached, similarity_warm);
        
        assert!(similarity_batch > 0.999, "Batch embedding inconsistent");
        assert!(similarity_cached > 0.999, "Cached embedding inconsistent");
        assert!(similarity_warm > 0.999, "Warm cache embedding inconsistent");
    }
    
    println!("✅ All embeddings are consistent!");

    // Show cache benefits for repeated content
    println!("\n📦 Cache Benefits Analysis:");
    println!("============================");
    let unique_samples: Vec<&str> = code_samples.iter()
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .cloned()
        .collect();
    
    println!("Total snippets: {} (includes duplicates)", code_samples.len());
    println!("Unique snippets: {} (cached after first occurrence)", unique_samples.len());
    println!("Cache hits: {} ({}% of requests)", 
             code_samples.len() - unique_samples.len(),
             ((code_samples.len() - unique_samples.len()) * 100) / code_samples.len());

    let final_cache_stats = cached_embedder.cache_stats();
    println!("Final cache state: {} entries", final_cache_stats.entries);

    println!("\n🎉 Optimization Benefits Demonstrated:");
    println!("   • Batch processing: {:.1}x faster than sequential", batch_speedup);
    println!("   • Caching: {:.1}x faster for repeated content", cache_speedup);
    println!("   • Memory efficient: LRU cache with configurable size");
    println!("   • Persistent: cache survives application restarts");
    println!("   • Thread-safe: suitable for concurrent access");

    Ok(())
}

/// Calculate cosine similarity between two embeddings
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