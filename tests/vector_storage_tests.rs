use embed_search::storage::{VectorStorage, StorageError, VectorSchema};
use embed_search::chunking::Chunk;
use std::path::PathBuf;
use tempfile::TempDir;

#[tokio::test]
async fn test_vector_storage_creation() {
    // Test that VectorStorage can be created without panic
    let temp_dir = TempDir::new().expect("Should create temp dir");
    let db_path = temp_dir.path().join("test.db");
    
    let storage = VectorStorage::new(db_path).await;
    assert!(storage.is_ok(), "Storage creation should succeed");
}

#[tokio::test]
async fn test_schema_initialization() {
    // Test that schema includes all required fields
    let temp_dir = TempDir::new().expect("Should create temp dir");
    let db_path = temp_dir.path().join("test_schema.db");
    
    let mut storage = VectorStorage::new(db_path).await.expect("Should create storage");
    let result = storage.init_schema().await;
    assert!(result.is_ok(), "Schema initialization should succeed");
    
    // Verify schema has required configuration
    let schema = storage.get_schema().expect("Should have schema after init");
    assert_eq!(schema.embedding_dim, 384, "Schema should specify 384-dimensional embeddings");
    assert_eq!(schema.version, 1, "Schema should have version 1");
    assert!(!schema.created_at.is_empty(), "Schema should have creation timestamp");
}

#[tokio::test]
async fn test_embedding_field_dimensions() {
    // Test that embeddings are validated for 384 dimensions
    let temp_dir = TempDir::new().expect("Should create temp dir");
    let db_path = temp_dir.path().join("test_embedding.db");
    
    let mut storage = VectorStorage::new(db_path).await.expect("Should create storage");
    storage.init_schema().await.expect("Should init schema");
    
    let chunk = Chunk {
        content: "fn test() {}".to_string(),
        start_line: 1,
        end_line: 1,
    };
    
    // Test correct dimensions
    let correct_embedding = vec![0.1f32; 384];
    let result = storage.insert_embedding("test.rs", 0, &chunk, correct_embedding).await;
    assert!(result.is_ok(), "Should accept 384-dimensional embedding");
    
    // Test incorrect dimensions
    let wrong_embedding = vec![0.1f32; 128];
    let result = storage.insert_embedding("test.rs", 1, &chunk, wrong_embedding).await;
    assert!(result.is_err(), "Should reject non-384-dimensional embedding");
}

#[tokio::test]
async fn test_insert_single_embedding() {
    // Test inserting a single embedding
    let temp_dir = TempDir::new().expect("Should create temp dir");
    let db_path = temp_dir.path().join("test_insert.db");
    
    let mut storage = VectorStorage::new(db_path).await.expect("Should create storage");
    storage.init_schema().await.expect("Should init schema");
    
    let chunk = Chunk {
        content: "fn test() {}".to_string(),
        start_line: 1,
        end_line: 1,
    };
    
    let embedding = vec![0.1f32; 384]; // 384-dimensional test embedding
    
    let result = storage.insert_embedding("test.rs", 0, &chunk, embedding).await;
    assert!(result.is_ok(), "Should insert embedding successfully");
    
    let count = storage.count().await.expect("Should get count");
    assert_eq!(count, 1, "Should have 1 embedding stored");
}

#[tokio::test]
async fn test_insert_multiple_embeddings() {
    // Test inserting multiple embeddings
    let temp_dir = TempDir::new().expect("Should create temp dir");
    let db_path = temp_dir.path().join("test_multi_insert.db");
    
    let mut storage = VectorStorage::new(db_path).await.expect("Should create storage");
    storage.init_schema().await.expect("Should init schema");
    
    let chunks = vec![
        Chunk { content: "fn test1() {}".to_string(), start_line: 1, end_line: 1 },
        Chunk { content: "fn test2() {}".to_string(), start_line: 3, end_line: 3 },
        Chunk { content: "fn test3() {}".to_string(), start_line: 5, end_line: 5 },
    ];
    
    for (i, chunk) in chunks.iter().enumerate() {
        let embedding = vec![0.1f32 * (i as f32 + 1.0); 384]; // Different embeddings
        let result = storage.insert_embedding("test.rs", i, chunk, embedding).await;
        assert!(result.is_ok(), "Should insert embedding {}", i);
    }
    
    let count = storage.count().await.expect("Should get count");
    assert_eq!(count, 3, "Should have 3 embeddings stored");
}

#[tokio::test]
async fn test_batch_insert_embeddings() {
    // Test batch insertion for efficiency
    let temp_dir = TempDir::new().expect("Should create temp dir");
    let db_path = temp_dir.path().join("test_batch.db");
    
    let mut storage = VectorStorage::new(db_path).await.expect("Should create storage");
    storage.init_schema().await.expect("Should init schema");
    
    let embeddings_data = vec![
        ("test1.rs", 0, Chunk { content: "fn a() {}".to_string(), start_line: 1, end_line: 1 }, vec![0.1f32; 384]),
        ("test1.rs", 1, Chunk { content: "fn b() {}".to_string(), start_line: 3, end_line: 3 }, vec![0.2f32; 384]),
        ("test2.rs", 0, Chunk { content: "fn c() {}".to_string(), start_line: 1, end_line: 1 }, vec![0.3f32; 384]),
    ];
    
    let result = storage.insert_batch(embeddings_data).await;
    assert!(result.is_ok(), "Batch insert should succeed");
    
    let count = storage.count().await.expect("Should get count");
    assert_eq!(count, 3, "Should have 3 embeddings from batch insert");
}

#[tokio::test]
async fn test_delete_by_file() {
    // Test removing all embeddings for a specific file
    let temp_dir = TempDir::new().expect("Should create temp dir");
    let db_path = temp_dir.path().join("test_delete.db");
    
    let mut storage = VectorStorage::new(db_path).await.expect("Should create storage");
    storage.init_schema().await.expect("Should init schema");
    
    // Insert embeddings for multiple files
    let chunk = Chunk { content: "fn test() {}".to_string(), start_line: 1, end_line: 1 };
    storage.insert_embedding("file1.rs", 0, &chunk, vec![0.1f32; 384]).await.expect("Should insert");
    storage.insert_embedding("file2.rs", 0, &chunk, vec![0.2f32; 384]).await.expect("Should insert");
    storage.insert_embedding("file1.rs", 1, &chunk, vec![0.3f32; 384]).await.expect("Should insert");
    
    let count_before = storage.count().await.expect("Should get count");
    assert_eq!(count_before, 3, "Should have 3 embeddings");
    
    // Delete all embeddings for file1.rs
    let result = storage.delete_by_file("file1.rs").await;
    assert!(result.is_ok(), "Should delete by file");
    
    let count_after = storage.count().await.expect("Should get count");
    assert_eq!(count_after, 1, "Should have 1 embedding left");
}

#[tokio::test]
async fn test_clear_all() {
    // Test clearing entire database
    let temp_dir = TempDir::new().expect("Should create temp dir");
    let db_path = temp_dir.path().join("test_clear.db");
    
    let mut storage = VectorStorage::new(db_path).await.expect("Should create storage");
    storage.init_schema().await.expect("Should init schema");
    
    // Insert some data
    let chunk = Chunk { content: "fn test() {}".to_string(), start_line: 1, end_line: 1 };
    storage.insert_embedding("test.rs", 0, &chunk, vec![0.1f32; 384]).await.expect("Should insert");
    
    let count_before = storage.count().await.expect("Should get count");
    assert!(count_before > 0, "Should have data before clear");
    
    let result = storage.clear_all().await;
    assert!(result.is_ok(), "Should clear all data");
    
    let count_after = storage.count().await.expect("Should get count");
    assert_eq!(count_after, 0, "Should have no data after clear");
}

#[tokio::test]
async fn test_search_preparation() {
    // Test that storage is ready for similarity search
    let temp_dir = TempDir::new().expect("Should create temp dir");
    let db_path = temp_dir.path().join("test_search_prep.db");
    
    let mut storage = VectorStorage::new(db_path).await.expect("Should create storage");
    storage.init_schema().await.expect("Should init schema");
    
    // Insert some test data
    let chunk = Chunk { content: "fn test() {}".to_string(), start_line: 1, end_line: 1 };
    let embedding = vec![0.1f32; 384];
    storage.insert_embedding("test.rs", 0, &chunk, embedding).await.expect("Should insert");
    
    // Verify we can prepare for search (this would create indexes in a real implementation)
    let result = storage.prepare_for_search().await;
    assert!(result.is_ok(), "Should prepare for search");
}

#[tokio::test]
async fn test_error_handling() {
    // Test various error conditions
    
    // Invalid path
    let invalid_path = PathBuf::from("/invalid/path/that/does/not/exist");
    let storage_result = VectorStorage::new(invalid_path).await;
    // Should either succeed (creating path) or fail gracefully
    
    // Wrong embedding dimensions
    let temp_dir = TempDir::new().expect("Should create temp dir");
    let db_path = temp_dir.path().join("test_errors.db");
    
    let mut storage = VectorStorage::new(db_path).await.expect("Should create storage");
    storage.init_schema().await.expect("Should init schema");
    
    let chunk = Chunk { content: "fn test() {}".to_string(), start_line: 1, end_line: 1 };
    let wrong_embedding = vec![0.1f32; 128]; // Wrong dimensions
    
    let result = storage.insert_embedding("test.rs", 0, &chunk, wrong_embedding).await;
    assert!(result.is_err(), "Should fail with wrong embedding dimensions");
}