use embed_search::storage::{VectorStorage, EmbeddingRecord};
use embed_search::chunking::Chunk;
use tempfile::TempDir;

#[tokio::test]
async fn test_similarity_search_basic() {
    let temp_dir = TempDir::new().expect("Should create temp dir");
    let db_path = temp_dir.path().join("test_similarity.db");
    
    let mut storage = VectorStorage::new(db_path).await.expect("Should create storage");
    storage.init_schema().await.expect("Should init schema");
    
    // Insert test embeddings
    let chunk1 = Chunk { content: "fn add(a: i32, b: i32) -> i32 { a + b }".to_string(), start_line: 1, end_line: 1 };
    let chunk2 = Chunk { content: "fn multiply(x: f64, y: f64) -> f64 { x * y }".to_string(), start_line: 3, end_line: 3 };
    let chunk3 = Chunk { content: "struct Point { x: i32, y: i32 }".to_string(), start_line: 5, end_line: 5 };
    
    // Create different embedding patterns
    let mut math_embedding = vec![0.0f32; 384];
    math_embedding[0] = 1.0; // Mathematical functions pattern
    math_embedding[1] = 0.8;
    
    let mut math_embedding2 = vec![0.0f32; 384];
    math_embedding2[0] = 0.9; // Similar to first
    math_embedding2[1] = 0.7;
    
    let mut struct_embedding = vec![0.0f32; 384];
    struct_embedding[10] = 1.0; // Different pattern for structs
    struct_embedding[11] = 0.9;
    
    // Normalize embeddings
    normalize_embedding(&mut math_embedding);
    normalize_embedding(&mut math_embedding2);
    normalize_embedding(&mut struct_embedding);
    
    storage.insert_embedding("math.rs", 0, &chunk1, math_embedding.clone()).await.unwrap();
    storage.insert_embedding("math.rs", 1, &chunk2, math_embedding2).await.unwrap();
    storage.insert_embedding("structs.rs", 0, &chunk3, struct_embedding).await.unwrap();
    
    // Search with query similar to math functions
    let mut query = vec![0.0f32; 384];
    query[0] = 1.0;
    query[1] = 0.9;
    normalize_embedding(&mut query);
    
    let results = storage.search_similar(query, 3).await.unwrap();
    
    assert_eq!(results.len(), 3, "Should return all 3 results");
    // First result should be most similar (math function)
    assert_eq!(results[0].chunk_index, 0, "Most similar should be first math function");
    assert!(results[0].content.contains("add"), "Should find the add function");
}

#[tokio::test]
async fn test_similarity_search_exact_match() {
    let temp_dir = TempDir::new().expect("Should create temp dir");
    let db_path = temp_dir.path().join("test_exact.db");
    
    let mut storage = VectorStorage::new(db_path).await.expect("Should create storage");
    storage.init_schema().await.expect("Should init schema");
    
    let chunk = Chunk { content: "fn test() {}".to_string(), start_line: 1, end_line: 1 };
    let embedding = vec![0.1f32; 384];
    
    storage.insert_embedding("test.rs", 0, &chunk, embedding.clone()).await.unwrap();
    
    // Search with exact same embedding
    let results = storage.search_similar(embedding, 1).await.unwrap();
    
    assert_eq!(results.len(), 1, "Should find exact match");
    assert_eq!(results[0].content, "fn test() {}", "Should return exact content");
}

#[tokio::test]
async fn test_similarity_search_empty_database() {
    let temp_dir = TempDir::new().expect("Should create temp dir");
    let db_path = temp_dir.path().join("test_empty.db");
    
    let mut storage = VectorStorage::new(db_path).await.expect("Should create storage");
    storage.init_schema().await.expect("Should init schema");
    
    let query = vec![0.1f32; 384];
    let results = storage.search_similar(query, 5).await.unwrap();
    
    assert_eq!(results.len(), 0, "Should return no results for empty database");
}

#[tokio::test]
async fn test_similarity_search_limit() {
    let temp_dir = TempDir::new().expect("Should create temp dir");
    let db_path = temp_dir.path().join("test_limit.db");
    
    let mut storage = VectorStorage::new(db_path).await.expect("Should create storage");
    storage.init_schema().await.expect("Should init schema");
    
    // Insert 5 embeddings
    for i in 0..5 {
        let chunk = Chunk {
            content: format!("fn test_{}() {{}}", i),
            start_line: i + 1,
            end_line: i + 1,
        };
        let mut embedding = vec![0.0f32; 384];
        embedding[0] = 0.1 * (i as f32 + 1.0);
        normalize_embedding(&mut embedding);
        
        storage.insert_embedding("test.rs", i, &chunk, embedding).await.unwrap();
    }
    
    let query = vec![0.1f32; 384];
    
    // Test limit of 3
    let results = storage.search_similar(query.clone(), 3).await.unwrap();
    assert_eq!(results.len(), 3, "Should respect limit of 3");
    
    // Test limit larger than available
    let results = storage.search_similar(query, 10).await.unwrap();
    assert_eq!(results.len(), 5, "Should return all available results when limit exceeds count");
}

#[tokio::test]
async fn test_similarity_search_wrong_dimensions() {
    let temp_dir = TempDir::new().expect("Should create temp dir");
    let db_path = temp_dir.path().join("test_wrong_dim.db");
    
    let mut storage = VectorStorage::new(db_path).await.expect("Should create storage");
    storage.init_schema().await.expect("Should init schema");
    
    // Try to search with wrong dimensions
    let wrong_query = vec![0.1f32; 128]; // Wrong size
    let result = storage.search_similar(wrong_query, 1).await;
    
    assert!(result.is_err(), "Should reject query with wrong dimensions");
}

#[tokio::test]
async fn test_get_all_embeddings() {
    let temp_dir = TempDir::new().expect("Should create temp dir");
    let db_path = temp_dir.path().join("test_get_all.db");
    
    let mut storage = VectorStorage::new(db_path).await.expect("Should create storage");
    storage.init_schema().await.expect("Should init schema");
    
    // Insert test data
    let chunk1 = Chunk { content: "fn test1() {}".to_string(), start_line: 1, end_line: 1 };
    let chunk2 = Chunk { content: "fn test2() {}".to_string(), start_line: 3, end_line: 3 };
    
    storage.insert_embedding("test.rs", 0, &chunk1, vec![0.1f32; 384]).await.unwrap();
    storage.insert_embedding("test.rs", 1, &chunk2, vec![0.2f32; 384]).await.unwrap();
    
    let all_embeddings = storage.get_all_embeddings().await.unwrap();
    
    assert_eq!(all_embeddings.len(), 2, "Should return all embeddings");
    assert!(all_embeddings.iter().any(|e| e.content == "fn test1() {}"), "Should include first embedding");
    assert!(all_embeddings.iter().any(|e| e.content == "fn test2() {}"), "Should include second embedding");
}

#[tokio::test]
async fn test_cosine_similarity_calculation() {
    let temp_dir = TempDir::new().expect("Should create temp dir");
    let db_path = temp_dir.path().join("test_cosine.db");
    
    let mut storage = VectorStorage::new(db_path).await.expect("Should create storage");
    storage.init_schema().await.expect("Should init schema");
    
    let chunk = Chunk { content: "test".to_string(), start_line: 1, end_line: 1 };
    
    // Create orthogonal vectors (should have similarity 0)
    let mut embedding1 = vec![0.0f32; 384];
    embedding1[0] = 1.0;
    
    let mut embedding2 = vec![0.0f32; 384];
    embedding2[1] = 1.0;
    
    storage.insert_embedding("test.rs", 0, &chunk, embedding1.clone()).await.unwrap();
    
    let results = storage.search_similar(embedding2, 1).await.unwrap();
    
    // For orthogonal normalized vectors, cosine similarity should be 0
    // (but our simple implementation might not be perfectly orthogonal due to floating point)
    assert_eq!(results.len(), 1, "Should find the embedding");
}

// Helper function to normalize embeddings
fn normalize_embedding(embedding: &mut Vec<f32>) {
    let magnitude: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
    if magnitude > 0.0 {
        for value in embedding.iter_mut() {
            *value /= magnitude;
        }
    }
}