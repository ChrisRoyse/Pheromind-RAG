use embed::storage::lancedb_storage::{LanceDBStorage, LanceEmbeddingRecord};
use std::path::PathBuf;
use tempfile::TempDir;

/// Quick validation test to verify basic LanceDB functionality works
#[tokio::test]
async fn quick_lancedb_validation() {
    println!("🔥 QUICK BRUTAL TEST: Basic LanceDB Validation");
    
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("quick_test.db");
    
    // Test 1: Storage creation
    let storage = LanceDBStorage::new(db_path).await;
    assert!(storage.is_ok(), "FAILED: Storage creation failed - {:?}", storage);
    let storage = storage.unwrap();
    
    // Test 2: Table initialization  
    let init_result = storage.init_table().await;
    assert!(init_result.is_ok(), "FAILED: Table initialization failed - {:?}", init_result);
    
    // Test 3: Create and insert a single record
    let test_record = LanceEmbeddingRecord {
        id: "test-1".to_string(),
        file_path: "test.rs".to_string(),
        chunk_index: 0,
        content: "fn main() { println!(\"test\"); }".to_string(),
        embedding: vec![0.1; 768],
        start_line: 1,
        end_line: 1,
        similarity_score: None,
        checksum: None,
    };
    
    let insert_result = storage.insert_batch(vec![test_record.clone()]).await;
    assert!(insert_result.is_ok(), "FAILED: Insert failed - {:?}", insert_result);
    
    // Test 4: Verify count
    let count = storage.count().await.unwrap();
    assert_eq!(count, 1, "FAILED: Count mismatch - expected 1, got {}", count);
    
    // Test 5: Search functionality
    let search_result = storage.search_similar(test_record.embedding.clone(), 1).await;
    assert!(search_result.is_ok(), "FAILED: Search failed - {:?}", search_result);
    
    let results = search_result.unwrap();
    assert!(!results.is_empty(), "FAILED: Search returned no results");
    assert_eq!(results[0].content, test_record.content, "FAILED: Content mismatch");
    
    println!("✅ QUICK VALIDATION PASSED: Basic LanceDB functionality works");
}

/// Test the exact integration path used in unified.rs
#[tokio::test] 
async fn validate_unified_integration_path() {
    println!("🔥 BRUTAL TEST: Unified Integration Path Validation");
    
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("unified_test.db");
    
    let storage = LanceDBStorage::new(db_path).await.unwrap();
    storage.init_table().await.unwrap();
    
    // Simulate the exact record creation from unified.rs lines 545-556
    let chunks = vec!["fn test() {}", "println!(\"hello\");", "}"];
    let embeddings = vec![vec![0.1; 768], vec![0.2; 768], vec![0.3; 768]];
    let file_path = std::path::Path::new("test_file.rs");
    
    // Create records exactly as unified.rs does
    let mut records = Vec::with_capacity(chunks.len());
    for ((idx, chunk), embedding) in chunks.iter().enumerate().zip(embeddings.into_iter()) {
        records.push(LanceEmbeddingRecord {
            id: format!("{}-{}", file_path.to_string_lossy(), idx),
            file_path: file_path.to_string_lossy().to_string(),
            chunk_index: idx as u64,
            content: chunk.to_string(),
            checksum: None, // Will be computed during storage
            embedding,
            start_line: idx as u64 + 1,
            end_line: idx as u64 + 2,
            similarity_score: None,
        });
    }
    
    // Test the exact insert_batch call from unified.rs line 560
    let insert_result = storage.insert_batch(records.clone()).await;
    assert!(insert_result.is_ok(), "FAILED: Unified integration insert_batch failed - {:?}", insert_result);
    
    // Verify all records were stored
    let count = storage.count().await.unwrap();
    assert_eq!(count, 3, "FAILED: Expected 3 records from unified integration, got {}", count);
    
    // Test retrieval matches stored data
    for record in &records {
        let search_results = storage.search_similar(record.embedding.clone(), 1).await.unwrap();
        assert!(!search_results.is_empty(), "FAILED: Could not retrieve stored record");
        
        let retrieved = &search_results[0];
        assert_eq!(retrieved.file_path, record.file_path, "FAILED: File path corruption");
        assert_eq!(retrieved.content, record.content, "FAILED: Content corruption"); 
        assert_eq!(retrieved.chunk_index, record.chunk_index, "FAILED: Chunk index corruption");
    }
    
    println!("✅ UNIFIED INTEGRATION VALIDATION PASSED");
}