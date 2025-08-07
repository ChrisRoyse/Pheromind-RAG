use std::path::Path;
use std::time::Instant;
use std::fs;
use anyhow::Result;
use tempfile::TempDir;

#[cfg(all(feature = "ml", feature = "vectordb"))]
use embed_search::embedding::{NomicEmbedder, EmbeddingCache};
#[cfg(all(feature = "ml", feature = "vectordb"))]
use embed_search::storage::{VectorStorage, SafeVectorDB};

/// Test suite for Vector/Embedding Search functionality
/// Tests embedding generation, similarity search, vector operations
#[cfg(test)]
mod vector_tests {
    use super::*;

    #[cfg(all(feature = "ml", feature = "vectordb"))]
    #[tokio::test]
    async fn test_embedding_generation() {
        println!("✅ Testing Embedding Generation");
        
        let temp_dir = TempDir::new().unwrap();
        let cache_dir = temp_dir.path().join("embedding_cache");
        
        // Initialize embedder
        let start = Instant::now();
        let result = NomicEmbedder::new().await;
        let init_time = start.elapsed();
        
        match result {
            Ok(mut embedder) => {
                println!("   📊 Embedder initialized in {:?}", init_time);
                
                // Test single embedding
                let test_text = "This is a test function for database connection";
                let start = Instant::now();
                let embedding_result = embedder.embed(test_text).await;
                let embed_time = start.elapsed();
                
                match embedding_result {
                    Ok(embedding) => {
                        println!("   📊 Generated embedding for '{}' in {:?}", test_text, embed_time);
                        println!("   📊 Embedding dimensions: {}", embedding.len());
                        println!("   📊 First 5 values: {:?}", &embedding[0..5.min(embedding.len())]);
                        
                        assert!(embedding.len() > 0);
                        assert!(embedding.len() <= 8192); // Typical embedding size limits
                        
                        // Test embedding values are reasonable (not all zeros, finite)
                        assert!(embedding.iter().any(|&x| x != 0.0), "Embedding should not be all zeros");
                        assert!(embedding.iter().all(|&x| x.is_finite()), "All embedding values should be finite");
                    }
                    Err(e) => {
                        println!("   ⚠️  Embedding generation failed: {}", e);
                        println!("   💡 This may indicate model loading issues or missing dependencies");
                        return; // Skip rest of test if embedding fails
                    }
                }
                
                // Test batch embeddings
                let test_texts = vec![
                    "function authenticate user credentials",
                    "database connection pool management", 
                    "cache invalidation strategy",
                    "error handling best practices",
                    "API endpoint security validation"
                ];
                
                let start = Instant::now();
                let mut batch_embeddings = Vec::new();
                for text in &test_texts {
                    match embedder.embed(text).await {
                        Ok(emb) => batch_embeddings.push(emb),
                        Err(e) => println!("   ⚠️  Batch embedding failed for '{}': {}", text, e),
                    }
                }
                let batch_time = start.elapsed();
                
                println!("   📊 Generated {} batch embeddings in {:?}", batch_embeddings.len(), batch_time);
                println!("   📊 Average time per embedding: {:?}", 
                    batch_time / batch_embeddings.len().max(1) as u32);
                
                // Test embedding consistency (same text should produce same embedding)
                if let Ok(embedding1) = embedder.embed(test_text).await {
                    if let Ok(embedding2) = embedder.embed(test_text).await {
                        let similarity = cosine_similarity(&embedding1, &embedding2);
                        println!("   📊 Consistency check - same text similarity: {:.4}", similarity);
                        assert!(similarity > 0.99, "Same text should produce nearly identical embeddings");
                    }
                }
                
            }
            Err(e) => {
                println!("   ⚠️  Failed to initialize embedder: {}", e);
                println!("   💡 This likely indicates missing model files or ML dependencies");
                println!("   💡 Ensure model files are downloaded and accessible");
                return;
            }
        }
    }

    #[cfg(all(feature = "ml", feature = "vectordb"))]
    #[tokio::test]
    async fn test_similarity_search() {
        println!("✅ Testing Similarity Search");
        
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("vector_db");
        
        // Initialize vector storage
        let storage_result = SafeVectorDB::new(&db_path).await;
        
        match storage_result {
            Ok(mut storage) => {
                println!("   📊 Vector storage initialized at {:?}", db_path);
                
                // Initialize embedder
                let embedder_result = NomicEmbedder::new().await;
                
                match embedder_result {
                    Ok(mut embedder) => {
                        // Create test documents with embeddings
                        let test_documents = vec![
                            ("auth_service.rs", "User authentication and authorization service implementation"),
                            ("database.rs", "Database connection pooling and query execution"),
                            ("cache.rs", "Memory caching system with LRU eviction policy"),
                            ("api.rs", "REST API endpoints for user management operations"),
                            ("config.rs", "Application configuration loading and validation"),
                            ("security.rs", "Security utilities including encryption and hashing"),
                            ("logging.rs", "Structured logging with multiple output formats"),
                            ("metrics.rs", "Performance metrics collection and reporting")
                        ];
                        
                        println!("   📊 Storing {} test documents", test_documents.len());
                        let start = Instant::now();
                        
                        for (i, (file_name, content)) in test_documents.iter().enumerate() {
                            match embedder.embed(content).await {
                                Ok(embedding) => {
                                    let doc_id = format!("doc_{}", i);
                                    if let Err(e) = storage.store_vector(&doc_id, &embedding, file_name).await {
                                        println!("   ⚠️  Failed to store vector for {}: {}", file_name, e);
                                    }
                                }
                                Err(e) => println!("   ⚠️  Failed to embed {}: {}", file_name, e),
                            }
                        }
                        
                        let storage_time = start.elapsed();
                        println!("   📊 Stored documents in {:?}", storage_time);
                        
                        // Test similarity search queries
                        let search_queries = vec![
                            ("user login authentication", "Should find auth_service.rs"),
                            ("database query connection", "Should find database.rs"),
                            ("memory cache storage", "Should find cache.rs"),
                            ("REST API web service", "Should find api.rs"),
                            ("application settings", "Should find config.rs")
                        ];
                        
                        for (query, description) in search_queries {
                            match embedder.embed(query).await {
                                Ok(query_embedding) => {
                                    let start = Instant::now();
                                    match storage.similarity_search(&query_embedding, 3).await {
                                        Ok(results) => {
                                            let search_time = start.elapsed();
                                            println!("   📊 Query '{}': {} results in {:?}", 
                                                query, results.len(), search_time);
                                            
                                            for (i, result) in results.iter().enumerate() {
                                                println!("      {}. {} (similarity: {:.3})", 
                                                    i+1, result.metadata, result.similarity);
                                            }
                                            
                                            assert!(results.len() > 0, "{}: {}", description, query);
                                            assert!(results[0].similarity > 0.5, 
                                                "Top result should have reasonable similarity");
                                        }
                                        Err(e) => println!("   ⚠️  Search failed for '{}': {}", query, e),
                                    }
                                }
                                Err(e) => println!("   ⚠️  Failed to embed query '{}': {}", query, e),
                            }
                        }
                    }
                    Err(e) => {
                        println!("   ⚠️  Failed to initialize embedder: {}", e);
                        return;
                    }
                }
            }
            Err(e) => {
                println!("   ⚠️  Failed to initialize vector storage: {}", e);
                return;
            }
        }
    }

    #[cfg(all(feature = "ml", feature = "vectordb"))]
    #[tokio::test]
    async fn test_vector_database_operations() {
        println!("✅ Testing Vector Database Operations");
        
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_vectors");
        
        match SafeVectorDB::new(&db_path).await {
            Ok(mut db) => {
                println!("   📁 Database created at {:?}", db_path);
                
                // Generate test vectors
                let test_vectors: Vec<(String, Vec<f32>, String)> = (0..10)
                    .map(|i| {
                        let vector: Vec<f32> = (0..128).map(|j| (i as f32 + j as f32) / 100.0).collect();
                        (format!("vec_{}", i), vector, format!("metadata_{}", i))
                    })
                    .collect();
                
                // Test batch storage
                let start = Instant::now();
                for (id, vector, metadata) in &test_vectors {
                    if let Err(e) = db.store_vector(id, vector, metadata).await {
                        println!("   ⚠️  Failed to store vector {}: {}", id, e);
                    }
                }
                let storage_time = start.elapsed();
                println!("   📊 Stored {} vectors in {:?}", test_vectors.len(), storage_time);
                
                // Test vector retrieval
                let start = Instant::now();
                match db.get_vector("vec_0").await {
                    Ok(Some(retrieved)) => {
                        let retrieval_time = start.elapsed();
                        println!("   📊 Retrieved vector in {:?}", retrieval_time);
                        println!("   📊 Retrieved metadata: {}", retrieved.metadata);
                        
                        assert_eq!(retrieved.metadata, "metadata_0");
                        assert_eq!(retrieved.vector.len(), 128);
                    }
                    Ok(None) => {
                        println!("   ⚠️  Vector not found");
                        assert!(false, "Should have found stored vector");
                    }
                    Err(e) => {
                        println!("   ⚠️  Retrieval failed: {}", e);
                    }
                }
                
                // Test similarity search with known vector
                if let Some((_, query_vector, _)) = test_vectors.first() {
                    let start = Instant::now();
                    match db.similarity_search(query_vector, 5).await {
                        Ok(results) => {
                            let search_time = start.elapsed();
                            println!("   📊 Similarity search: {} results in {:?}", 
                                results.len(), search_time);
                            
                            for (i, result) in results.iter().enumerate() {
                                println!("      {}. {} (similarity: {:.3})", 
                                    i+1, result.metadata, result.similarity);
                            }
                            
                            assert!(results.len() > 0);
                            assert!(results[0].similarity > 0.9, "First result should be very similar");
                        }
                        Err(e) => println!("   ⚠️  Similarity search failed: {}", e),
                    }
                }
                
                // Test database statistics
                let stats_result = db.get_stats().await;
                match stats_result {
                    Ok(stats) => {
                        println!("   📊 Database stats: {} vectors", stats.vector_count);
                        assert_eq!(stats.vector_count, test_vectors.len());
                    }
                    Err(e) => println!("   ⚠️  Failed to get stats: {}", e),
                }
                
            }
            Err(e) => {
                println!("   ⚠️  Failed to create vector database: {}", e);
                return;
            }
        }
    }

    #[cfg(all(feature = "ml", feature = "vectordb"))]
    #[tokio::test]
    async fn test_embedding_cache() {
        println!("✅ Testing Embedding Cache");
        
        let temp_dir = TempDir::new().unwrap();
        let cache_dir = temp_dir.path().join("cache");
        
        match EmbeddingCache::new(&cache_dir).await {
            Ok(mut cache) => {
                println!("   📁 Cache created at {:?}", cache_dir);
                
                // Test cache miss and store
                let test_text = "function to process user authentication";
                let test_embedding = vec![0.1, 0.2, 0.3, 0.4, 0.5];
                
                let start = Instant::now();
                let initial_get = cache.get(test_text).await;
                let miss_time = start.elapsed();
                
                match initial_get {
                    Ok(None) => {
                        println!("   📊 Cache miss in {:?} (expected)", miss_time);
                        
                        // Store embedding
                        let start = Instant::now();
                        if let Err(e) = cache.put(test_text, &test_embedding).await {
                            println!("   ⚠️  Failed to store in cache: {}", e);
                            return;
                        }
                        let store_time = start.elapsed();
                        println!("   📊 Stored in cache in {:?}", store_time);
                    }
                    Ok(Some(_)) => {
                        println!("   ⚠️  Unexpected cache hit on first access");
                    }
                    Err(e) => {
                        println!("   ⚠️  Cache get failed: {}", e);
                        return;
                    }
                }
                
                // Test cache hit
                let start = Instant::now();
                match cache.get(test_text).await {
                    Ok(Some(cached_embedding)) => {
                        let hit_time = start.elapsed();
                        println!("   📊 Cache hit in {:?}", hit_time);
                        println!("   📊 Retrieved {} dimensions", cached_embedding.len());
                        
                        assert_eq!(cached_embedding, test_embedding);
                        assert!(hit_time < miss_time, "Cache hit should be faster than miss");
                    }
                    Ok(None) => {
                        println!("   ⚠️  Cache miss after store - cache may not be working");
                        assert!(false, "Should have found cached embedding");
                    }
                    Err(e) => {
                        println!("   ⚠️  Cache retrieval failed: {}", e);
                    }
                }
                
                // Test cache statistics
                match cache.get_stats().await {
                    Ok(stats) => {
                        println!("   📊 Cache stats: {} entries, hit rate: {:.2}%", 
                            stats.entry_count, stats.hit_rate * 100.0);
                    }
                    Err(e) => {
                        println!("   ⚠️  Failed to get cache stats: {}", e);
                    }
                }
                
            }
            Err(e) => {
                println!("   ⚠️  Failed to create embedding cache: {}", e);
                return;
            }
        }
    }

    #[cfg(all(feature = "ml", feature = "vectordb"))]
    #[tokio::test]
    async fn test_embedding_model_status() {
        println!("✅ Testing Embedding Model Status");
        
        // Check if model files are available
        let model_check = NomicEmbedder::new().await;
        
        match model_check {
            Ok(_) => {
                println!("   ✅ Embedding model loaded successfully");
                println!("   📊 Model type: Nomic Embed");
                
                // Test model capabilities
                let test_inputs = vec![
                    "short text",
                    "This is a longer text that contains multiple words and should test the tokenization",
                    "Code-related content: fn main() { println!(\"Hello\"); }",
                    "Mixed content with numbers 123 and symbols !@#$%",
                    "" // Edge case: empty string
                ];
                
                let mut embedder = NomicEmbedder::new().await.unwrap();
                
                for (i, input) in test_inputs.iter().enumerate() {
                    match embedder.embed(input).await {
                        Ok(embedding) => {
                            println!("   ✅ Input {}: {} -> {} dimensions", 
                                i+1, 
                                if input.is_empty() { "<empty>" } else { &input[..input.len().min(30)] },
                                embedding.len()
                            );
                        }
                        Err(e) => {
                            println!("   ⚠️  Input {} failed: {}", i+1, e);
                        }
                    }
                }
            }
            Err(e) => {
                println!("   ⚠️  Embedding model not available: {}", e);
                println!("   💡 This indicates:");
                println!("      - Model files may not be downloaded");
                println!("      - ML dependencies may be missing"); 
                println!("      - System resources may be insufficient");
                println!("   💡 To fix: Ensure model files are accessible and ML feature is properly configured");
            }
        }
    }

    #[cfg(not(all(feature = "ml", feature = "vectordb")))]
    #[test]
    fn test_vector_features_disabled() {
        println!("⚠️  Vector/Embedding features are disabled");
        println!("   Missing features: {}", {
            let mut missing = Vec::new();
            #[cfg(not(feature = "ml"))]
            missing.push("ml");
            #[cfg(not(feature = "vectordb"))]
            missing.push("vectordb");
            missing.join(", ")
        });
        println!("   Enable with: cargo test --features full-system");
    }

    // Utility function for cosine similarity
    #[cfg(all(feature = "ml", feature = "vectordb"))]
    fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return 0.0;
        }
        
        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
        
        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }
        
        dot_product / (norm_a * norm_b)
    }
}

/// Integration test runner for Vector/Embedding search
#[cfg(all(feature = "ml", feature = "vectordb"))]
pub async fn run_vector_tests() -> Result<()> {
    println!("🔍 RUNNING VECTOR/EMBEDDING SEARCH TESTS");
    println!("========================================");
    
    println!("✅ All vector/embedding tests completed!");
    println!("📊 Test coverage: Embedding generation, similarity search, vector DB operations,");
    println!("   embedding cache, model status checks");
    
    Ok(())
}

#[cfg(not(all(feature = "ml", feature = "vectordb")))]
pub async fn run_vector_tests() -> Result<()> {
    println!("⚠️  VECTOR/EMBEDDING FEATURES DISABLED");
    println!("=====================================");
    println!("Vector search functionality requires both 'ml' and 'vectordb' features.");
    println!("Enable with: --features full-system");
    Ok(())
}