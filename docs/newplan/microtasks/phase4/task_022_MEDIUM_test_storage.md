# Task 022 - MEDIUM: Test LanceDB Storage Operations

## Priority: MEDIUM
## Estimated Time: 10 minutes
## Dependencies: Task 021

## Objective
Create comprehensive tests for LanceDB storage operations to verify functionality.

## Current Issue
- No tests for LanceDB integration
- Need to verify all VectorStore operations
- Performance testing required

## Tasks
1. **Create storage integration tests** (6 min)
   ```rust
   // In src/storage/tests/lancedb_integration.rs
   use super::*;
   use tempfile::tempdir;
   use tokio_test;
   
   async fn create_test_store() -> LanceDBStore {
       let temp_dir = tempdir().unwrap();
       let db_path = temp_dir.path().join("test.lancedb");
       
       LanceDBStore::new(
           db_path.to_str().unwrap(),
           "test_embeddings",
           768,
       ).await.unwrap()
   }
   
   #[tokio::test]
   async fn test_add_and_retrieve_embedding() {
       let store = create_test_store().await;
       
       let id = "test_embedding_1".to_string();
       let embedding = vec![0.1, 0.2, 0.3].repeat(256); // 768 dimensions
       let metadata = serde_json::json!({
           "source": "test",
           "type": "document"
       });
       
       // Add embedding
       let result = store.add_embedding(id.clone(), embedding.clone(), metadata.clone()).await;
       assert!(result.is_ok(), "Failed to add embedding: {:?}", result);
       
       // Search for the embedding
       let search_results = store.search(&embedding, 5, None).await.unwrap();
       assert!(!search_results.is_empty(), "No search results found");
       assert_eq!(search_results[0].id, id);
       assert!(search_results[0].score > 0.99, "Score too low: {}", search_results[0].score);
   }
   
   #[tokio::test]
   async fn test_batch_operations() {
       let store = create_test_store().await;
       
       // Prepare batch data
       let batch_data: Vec<_> = (0..10)
           .map(|i| {
               let id = format!("batch_test_{}", i);
               let embedding = vec![i as f32 / 10.0; 768];
               let metadata = serde_json::json!({"batch_id": i});
               (id, embedding, metadata)
           })
           .collect();
       
       // Add batch
       let result = store.add_embeddings_batch(batch_data.clone()).await;
       assert!(result.is_ok(), "Batch insert failed: {:?}", result);
       
       // Search and verify count
       let query = vec![0.5; 768];
       let results = store.search(&query, 20, None).await.unwrap();
       assert_eq!(results.len(), 10, "Expected 10 results, got {}", results.len());
   }
   ```

2. **Add performance tests** (3 min)
   ```rust
   #[tokio::test]
   async fn test_search_performance() {
       let store = create_test_store().await;
       
       // Add many embeddings
       let mut batch_data = Vec::new();
       for i in 0..1000 {
           let id = format!("perf_test_{}", i);
           let embedding: Vec<f32> = (0..768)
               .map(|j| (i as f32 + j as f32) / 1000.0)
               .collect();
           let metadata = serde_json::json!({"index": i});
           batch_data.push((id, embedding, metadata));
       }
       
       let start = std::time::Instant::now();
       store.add_embeddings_batch(batch_data).await.unwrap();
       let insert_time = start.elapsed();
       println!("Batch insert (1000 items) took: {:?}", insert_time);
       assert!(insert_time.as_secs() < 30, "Insert too slow: {:?}", insert_time);
       
       // Test search performance
       let query = vec![0.5; 768];
       let start = std::time::Instant::now();
       let results = store.search(&query, 10, None).await.unwrap();
       let search_time = start.elapsed();
       
       println!("Search took: {:?}", search_time);
       assert!(search_time.as_millis() < 1000, "Search too slow: {:?}", search_time);
       assert_eq!(results.len(), 10);
   }
   
   #[tokio::test]
   async fn test_error_conditions() {
       let store = create_test_store().await;
       
       // Test dimension mismatch
       let wrong_embedding = vec![0.1; 512]; // Wrong dimension
       let result = store.add_embedding(
           "test".to_string(),
           wrong_embedding,
           serde_json::json!({}),
       ).await;
       assert!(result.is_err());
       
       // Test empty ID
       let embedding = vec![0.1; 768];
       let result = store.add_embedding(
           "".to_string(),
           embedding,
           serde_json::json!({}),
       ).await;
       assert!(result.is_err());
       
       // Test invalid search params
       let query = vec![0.1; 512]; // Wrong dimension
       let result = store.search(&query, 10, None).await;
       assert!(result.is_err());
   }
   ```

3. **Add cleanup and resource tests** (1 min)
   ```rust
   #[tokio::test]
   async fn test_table_operations() {
       let store = create_test_store().await;
       
       // Test table creation
       assert!(store.ensure_table_exists().await.is_ok());
       
       // Test count operation
       let initial_count = store.count().await.unwrap();
       assert_eq!(initial_count, 0);
       
       // Add some data
       let embedding = vec![0.1; 768];
       store.add_embedding(
           "count_test".to_string(),
           embedding,
           serde_json::json!({}),
       ).await.unwrap();
       
       let final_count = store.count().await.unwrap();
       assert_eq!(final_count, 1);
   }
   ```

## Success Criteria
- [ ] All integration tests pass
- [ ] Performance tests complete within limits
- [ ] Error conditions properly handled
- [ ] Batch operations work correctly
- [ ] Resource management verified

## Files to Create
- `src/storage/tests/lancedb_integration.rs`
- `src/storage/tests/mod.rs`

## Files to Modify
- `Cargo.toml` (add test dependencies)
- `src/storage/mod.rs`

## Test Dependencies
```toml
[dev-dependencies]
tempfile = "3.8"
tokio-test = "0.4"
```

## Validation
```bash
# Run storage tests
cargo test storage::tests::lancedb_integration --verbose

# Run performance tests
cargo test storage::tests::lancedb_integration::test_search_performance --release

# Expected: All tests pass, performance within limits
```

## Next Task
→ Task 023: Optimize LanceDB performance settings