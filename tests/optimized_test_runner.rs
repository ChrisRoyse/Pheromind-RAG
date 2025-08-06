/// Optimized test runner that skips slow/unnecessary tests
/// Run with: cargo test --test optimized_test_runner

// Core functionality tests only
#[cfg(test)]
mod core_tests {
    use embed_lib::embedding::{CachedEmbedder, RealMiniLMEmbedder};
    use embed_lib::search::unified::UnifiedSearcher;
    use std::path::PathBuf;
    
    #[tokio::test]
    async fn test_singleton_embedder_performance() {
        // Test that singleton pattern works and is fast
        let start = std::time::Instant::now();
        
        // First call might download model
        let embedder1 = RealMiniLMEmbedder::get_global().await.unwrap();
        let time1 = start.elapsed();
        
        // Second call should be instant (cached)
        let start2 = std::time::Instant::now();
        let embedder2 = RealMiniLMEmbedder::get_global().await.unwrap();
        let time2 = start2.elapsed();
        
        // Verify they're the same instance
        assert!(std::ptr::eq(embedder1.as_ref(), embedder2.as_ref()));
        
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
        let embedder = CachedEmbedder::new_with_cache_size(100).await.unwrap();
        
        let test_text = "This is a test sentence for caching";
        
        // First embed (cache miss)
        let start = std::time::Instant::now();
        let embedding1 = embedder.embed(test_text).unwrap();
        let time_miss = start.elapsed();
        
        // Second embed (cache hit)
        let start = std::time::Instant::now();
        let embedding2 = embedder.embed(test_text).unwrap();
        let time_hit = start.elapsed();
        
        // Verify same embedding
        assert_eq!(embedding1, embedding2);
        
        // Cache hit should be at least 10x faster
        assert!(
            time_hit < time_miss / 10,
            "Cache not effective: hit {:?} vs miss {:?}",
            time_hit,
            time_miss
        );
        
        let stats = embedder.cache_stats();
        assert_eq!(stats.entries, 1);
        
        println!("✅ Cache working: miss {:?}, hit {:?} ({}x speedup)", 
                 time_miss, time_hit, time_miss.as_nanos() / time_hit.as_nanos().max(1));
    }
    
    #[tokio::test]
    async fn test_batch_embedding_performance() {
        let embedder = RealMiniLMEmbedder::get_global().await.unwrap();
        
        let texts = vec![
            "First sentence to embed",
            "Second sentence to embed",
            "Third sentence to embed",
            "Fourth sentence to embed",
        ];
        
        // Test optimized batch processing
        let start = std::time::Instant::now();
        let batch_embeddings = embedder.embed_batch_optimized(&texts.iter().map(|s| *s).collect::<Vec<_>>()).unwrap();
        let batch_time = start.elapsed();
        
        // Test sequential processing
        let start = std::time::Instant::now();
        let mut seq_embeddings = Vec::new();
        for text in &texts {
            seq_embeddings.push(embedder.embed(text).unwrap());
        }
        let seq_time = start.elapsed();
        
        // Verify same results
        assert_eq!(batch_embeddings.len(), seq_embeddings.len());
        for (batch, seq) in batch_embeddings.iter().zip(seq_embeddings.iter()) {
            let similarity: f32 = batch.iter().zip(seq.iter()).map(|(a, b)| a * b).sum();
            assert!(similarity > 0.99, "Embeddings don't match");
        }
        
        println!("✅ Batch processing: {:?} vs Sequential: {:?} ({}x speedup)",
                 batch_time, seq_time, seq_time.as_nanos() / batch_time.as_nanos().max(1));
    }
    
    #[tokio::test]
    async fn test_test_file_exclusion() {
        let project_root = std::env::current_dir().unwrap();
        let db_path = project_root.join("test_exclusion_db");
        
        // Clean up
        if db_path.exists() {
            std::fs::remove_dir_all(&db_path).ok();
        }
        
        // Create searcher with test files excluded
        let searcher = UnifiedSearcher::new_with_config(
            project_root.clone(),
            db_path.clone(),
            false // exclude test files
        ).await.unwrap();
        
        // Create a test directory structure
        let test_dir = project_root.join("test_exclusion_check");
        std::fs::create_dir_all(&test_dir).ok();
        std::fs::write(test_dir.join("main.rs"), "fn main() {}").ok();
        std::fs::write(test_dir.join("test_main.rs"), "fn test() {}").ok();
        std::fs::write(test_dir.join("main_test.rs"), "fn test() {}").ok();
        
        let stats = searcher.index_directory(&test_dir).await.unwrap();
        
        // Should only index main.rs, not test files
        assert_eq!(stats.files_indexed, 1, "Test files should be excluded");
        
        // Clean up
        std::fs::remove_dir_all(&test_dir).ok();
        std::fs::remove_dir_all(&db_path).ok();
        
        println!("✅ Test file exclusion working: indexed {} files (expected 1)", stats.files_indexed);
    }
}

// Run with: cargo test --test optimized_test_runner -- --nocapture
fn main() {
    println!("Run tests with: cargo test --test optimized_test_runner");
}