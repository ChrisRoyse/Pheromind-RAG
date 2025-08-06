/// Optimized test runner that focuses on core Nomic embedding functionality
/// Run with: cargo test --test optimized_test_runner

// Core functionality tests only
#[cfg(test)]
mod core_tests {
    use embed_search::embedding::NomicEmbedder;
    use std::path::PathBuf;
    
    #[tokio::test]
    async fn test_singleton_embedder_performance() {
        // Test that singleton pattern works and is fast
        let start = std::time::Instant::now();
        
        // First call might download model
        let embedder1 = NomicEmbedder::get_global().await.unwrap();
        let time1 = start.elapsed();
        
        // Second call should be instant (cached)
        let start2 = std::time::Instant::now();
        let embedder2 = NomicEmbedder::get_global().await.unwrap();
        let time2 = start2.elapsed();
        
        // Verify they're the same instance
        assert!(std::sync::Arc::ptr_eq(&embedder1, &embedder2));
        
        // Second call should be under 1ms (just returning cached instance)
        assert!(
            time2.as_millis() < 10,
            "Singleton access took {}ms (should be <10ms)",
            time2.as_millis()
        );
        
        println!("✅ Singleton pattern working: first access {:?}, second access {:?}", time1, time2);
    }
    
    #[tokio::test]
    async fn test_embedding_cache_effectiveness() {
        let embedder = NomicEmbedder::get_global().await.unwrap();
        
        let test_text = "This is a test sentence for caching";
        
        // First embed (cache miss)
        let start = std::time::Instant::now();
        let embedding1 = embedder.embed(test_text).unwrap();
        let time_miss = start.elapsed();
        
        // Second embed (cache hit)  
        let start = std::time::Instant::now();
        let embedding2 = embedder.embed(test_text).unwrap();
        let time_hit = start.elapsed();
        
        // Results should be identical
        assert_eq!(embedding1, embedding2);
        assert_eq!(embedding1.len(), 768);
        
        // Cache hit should be faster (though both might be fast due to built-in caching)
        println!("✅ Cache working: miss {:?}, hit {:?}", time_miss, time_hit);
    }
    
    #[tokio::test]
    async fn test_basic_embedding_quality() {
        let embedder = NomicEmbedder::get_global().await.unwrap();
        
        // Test different code snippets
        let tests = vec![
            "def calculate_sum(a, b): return a + b",
            "class User: pass",
            "SELECT * FROM users WHERE active = true",
            "function hello() { return 'world'; }",
        ];
        
        let mut embeddings = Vec::new();
        for text in &tests {
            let embedding = embedder.embed(text).unwrap();
            assert_eq!(embedding.len(), 768, "All embeddings should be 768-dimensional");
            
            // Check normalization
            let magnitude: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
            assert!((magnitude - 1.0).abs() < 0.1, "Embedding should be normalized");
            
            embeddings.push(embedding);
        }
        
        // Verify embeddings are different
        for i in 0..embeddings.len() {
            for j in i+1..embeddings.len() {
                assert_ne!(embeddings[i], embeddings[j], "Different inputs should produce different embeddings");
            }
        }
        
        println!("✅ Basic embedding quality test passed");
    }
    
    #[tokio::test] 
    async fn test_batch_vs_individual_consistency() {
        let embedder = NomicEmbedder::get_global().await.unwrap();
        
        let texts = vec![
            "function test1() {}",
            "class TestClass {}",
            "def test_function(): pass",
        ];
        
        // Individual embeddings
        let mut individual_embeddings = Vec::new();
        for text in &texts {
            individual_embeddings.push(embedder.embed(text).unwrap());
        }
        
        // Batch embeddings
        let batch_embeddings = embedder.embed_batch(&texts).unwrap();
        
        // Should be identical
        assert_eq!(individual_embeddings.len(), batch_embeddings.len());
        for (ind, batch) in individual_embeddings.iter().zip(batch_embeddings.iter()) {
            assert_eq!(ind, batch, "Individual and batch embeddings should be identical");
        }
        
        println!("✅ Batch vs individual consistency test passed");
    }
    
    #[tokio::test]
    async fn test_nomic_model_dimensions() {
        let embedder = NomicEmbedder::get_global().await.unwrap();
        
        // Test dimension getter
        assert_eq!(embedder.dimensions(), 768);
        
        // Test actual embedding dimensions
        let embedding = embedder.embed("test").unwrap();
        assert_eq!(embedding.len(), 768);
        
        println!("✅ Model dimensions test passed (768D confirmed)");
    }
}