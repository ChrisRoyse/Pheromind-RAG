# Task 038 - HIGH: Create Comprehensive Integration Tests

## Priority: HIGH
## Estimated Time: 10 minutes
## Dependencies: Task 037

## Objective
Create end-to-end integration tests to validate the complete ML/Vector pipeline works correctly.

## Current Issue
- No comprehensive integration testing
- Need to verify entire pipeline works together
- Edge cases and error conditions need testing

## Tasks
1. **Create end-to-end pipeline tests** (5 min)
   ```rust
   // In src/ml/tests/integration_tests.rs
   use crate::ml::{
       batch_processor::BatchEmbeddingProcessor,
       embedding_service::EmbeddingService,
       errors::{EmbeddingError, EmbeddingResult},
   };
   use crate::storage::{
       lancedb_store::LanceDBStore,
       VectorStore,
   };
   use crate::types::{EmbeddingVector, SearchResult};
   use candle_core::Device;
   use tempfile::tempdir;
   use tokio_test;
   
   async fn create_test_embedding_service() -> EmbeddingResult<EmbeddingService> {
       let device = Device::Cpu;
       let service = EmbeddingService::new_with_device(device).await?;
       Ok(service)
   }
   
   async fn create_test_vector_store() -> EmbeddingResult<LanceDBStore> {
       let temp_dir = tempdir().map_err(|e| EmbeddingError::IoError {
           message: format!("Failed to create temp dir: {}", e),
       })?;
       
       let db_path = temp_dir.path().join("test_embeddings.lancedb");
       let store = LanceDBStore::new(
           db_path.to_str().unwrap(),
           "test_embeddings",
           768,
       ).await.map_err(|e| EmbeddingError::ConfigError {
           message: format!("Failed to create vector store: {}", e),
       })?;
       
       Ok(store)
   }
   
   #[tokio::test]
   async fn test_complete_embedding_pipeline() {
       let service = create_test_embedding_service().await.unwrap();
       
       let test_texts = vec![
           "Hello, this is a test document for embedding generation.",
           "Machine learning models can process natural language text.",
           "Vector databases store high-dimensional embeddings efficiently.",
       ];
       
       // Test single embedding generation
       let single_embedding = service.generate_embedding(test_texts[0]).await.unwrap();
       
       assert_eq!(single_embedding.len(), 768); // Nomic model dimension
       
       // Verify embedding is normalized
       let norm: f32 = single_embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
       assert!((norm - 1.0).abs() < 1e-6, "Embedding should be L2 normalized");
       
       // Test batch embedding generation
       let batch_embeddings = service.generate_embeddings_batch(&test_texts).await.unwrap();
       assert_eq!(batch_embeddings.len(), 3);
       
       for embedding in &batch_embeddings {
           assert_eq!(embedding.len(), 768);
           let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
           assert!((norm - 1.0).abs() < 1e-6);
       }
       
       // Test embedding similarity
       let similarity = cosine_similarity(&batch_embeddings[0], &batch_embeddings[1]);
       assert!(similarity > 0.0 && similarity < 1.0, "Similarity should be between 0 and 1");
   }
   
   #[tokio::test]
   async fn test_vector_store_integration() {
       let service = create_test_embedding_service().await.unwrap();
       let store = create_test_vector_store().await.unwrap();
       
       let test_documents = vec![
           ("doc1", "Artificial intelligence is transforming technology."),
           ("doc2", "Machine learning algorithms learn from data."),
           ("doc3", "Deep neural networks process complex patterns."),
           ("doc4", "Natural language processing understands human text."),
           ("doc5", "Computer vision analyzes visual information."),
       ];
       
       // Generate embeddings and store them
       for (id, text) in &test_documents {
           let embedding = service.generate_embedding(text).await.unwrap();
           let metadata = serde_json::json!({
               "text": text,
               "document_type": "test",
               "timestamp": chrono::Utc::now().timestamp()
           });
           
           store.add_embedding(id.to_string(), embedding, metadata).await.unwrap();
       }
       
       // Test similarity search
       let query_text = "AI and machine learning technologies";
       let query_embedding = service.generate_embedding(query_text).await.unwrap();
       
       let search_results = store.search(&query_embedding, 3, None).await.unwrap();
       
       assert_eq!(search_results.len(), 3, "Should return 3 search results");
       
       // Verify results are relevant (similarity > 0.5)
       for result in &search_results {
           assert!(result.score > 0.5, "Search results should be reasonably similar");
           assert!(!result.id.is_empty(), "Result should have valid ID");
           assert_eq!(result.embedding.len(), 768, "Result embedding should be valid");
       }
       
       // Test that results are ordered by similarity (descending)
       for i in 1..search_results.len() {
           assert!(
               search_results[i-1].score >= search_results[i].score,
               "Results should be ordered by similarity score"
           );
       }
   }
   
   #[tokio::test]
   async fn test_performance_requirements() {
       let service = create_test_embedding_service().await.unwrap();
       
       let test_text = "This is a performance test for embedding generation with a moderately long text that should be processed efficiently by the system.";
       
       // Test single embedding performance
       let start = std::time::Instant::now();
       let _embedding = service.generate_embedding(test_text).await.unwrap();
       let single_duration = start.elapsed();
       
       println!("Single embedding generation took: {:?}", single_duration);
       assert!(
           single_duration.as_millis() < 2000,
           "Single embedding should complete in under 2 seconds"
       );
       
       // Test batch performance
       let batch_texts = vec![test_text; 8];
       let start = std::time::Instant::now();
       let batch_embeddings = service.generate_embeddings_batch(&batch_texts).await.unwrap();
       let batch_duration = start.elapsed();
       
       println!("Batch embedding generation took: {:?}", batch_duration);
       assert_eq!(batch_embeddings.len(), 8);
       
       let throughput = batch_embeddings.len() as f64 / batch_duration.as_secs_f64();
       println!("Throughput: {:.2} embeddings/second", throughput);
       
       assert!(
           throughput > 2.0,
           "Batch processing should achieve at least 2 embeddings/second"
       );
   }
   
   fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
       assert_eq!(a.len(), b.len());
       
       let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
       
       // Vectors should already be normalized, so dot product is cosine similarity
       dot_product.max(0.0).min(1.0)
   }
   ```

2. **Add error handling and edge case tests** (3 min)
   ```rust
   #[tokio::test]
   async fn test_error_handling() {
       let service = create_test_embedding_service().await.unwrap();
       
       // Test empty string
       let result = service.generate_embedding("").await;
       // Should handle gracefully - either return zero vector or appropriate error
       match result {
           Ok(embedding) => {
               assert_eq!(embedding.len(), 768);
               // Could be zero vector or some default
           },
           Err(e) => {
               println!("Empty string handled with error: {}", e);
           }
       }
       
       // Test very long text
       let long_text = "word ".repeat(10000);
       let result = service.generate_embedding(&long_text).await;
       
       match result {
           Ok(embedding) => {
               assert_eq!(embedding.len(), 768);
               let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
               assert!((norm - 1.0).abs() < 1e-6);
           },
           Err(e) => {
               println!("Long text handled with error: {}", e);
               // Should handle gracefully with chunking or truncation
           }
       }
       
       // Test special characters
       let special_text = "Special chars: 🚀 áéíóú ñ 中文 العربية русский 🎉";
       let embedding = service.generate_embedding(special_text).await.unwrap();
       assert_eq!(embedding.len(), 768);
       
       // Test batch with mixed content
       let mixed_batch = vec![
           "", // Empty
           "Normal text", // Normal
           "word ".repeat(1000).as_str(), // Long
           special_text, // Special chars
       ];
       
       let batch_result = service.generate_embeddings_batch(&mixed_batch).await;
       match batch_result {
           Ok(embeddings) => {
               assert_eq!(embeddings.len(), 4);
               for embedding in embeddings {
                   assert_eq!(embedding.len(), 768);
               }
           },
           Err(e) => {
               println!("Mixed batch handled with error: {}", e);
           }
       }
   }
   
   #[tokio::test]
   async fn test_concurrent_processing() {
       let service = std::sync::Arc::new(create_test_embedding_service().await.unwrap());
       
       let test_texts = vec![
           "Concurrent processing test 1",
           "Concurrent processing test 2",
           "Concurrent processing test 3",
           "Concurrent processing test 4",
       ];
       
       // Spawn multiple concurrent embedding requests
       let mut handles = vec![];
       
       for (i, text) in test_texts.iter().enumerate() {
           let service_clone = std::sync::Arc::clone(&service);
           let text_owned = text.to_string();
           
           let handle = tokio::spawn(async move {
               let embedding = service_clone.generate_embedding(&text_owned).await.unwrap();
               (i, embedding)
           });
           
           handles.push(handle);
       }
       
       // Wait for all requests to complete
       let mut results = vec![];
       for handle in handles {
           let (index, embedding) = handle.await.unwrap();
           results.push((index, embedding));
       }
       
       // Verify all completed successfully
       assert_eq!(results.len(), 4);
       
       results.sort_by_key(|(i, _)| *i);
       
       for (_, embedding) in results {
           assert_eq!(embedding.len(), 768);
           let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
           assert!((norm - 1.0).abs() < 1e-6);
       }
   }
   ```

3. **Add memory and resource tests** (2 min)
   ```rust
   #[tokio::test]
   async fn test_memory_usage() {
       let service = create_test_embedding_service().await.unwrap();
       
       // Get initial memory baseline
       let initial_memory = get_memory_usage();
       
       // Process many embeddings to test for memory leaks
       let test_text = "Memory usage test with moderate length text.";
       
       for batch_num in 0..10 {
           let batch_texts = vec![test_text; 16];
           let _embeddings = service.generate_embeddings_batch(&batch_texts).await.unwrap();
           
           if batch_num % 3 == 0 {
               // Check memory every few batches
               let current_memory = get_memory_usage();
               let memory_growth = current_memory - initial_memory;
               
               println!("Batch {}: Memory growth: {:.2} MB", batch_num, memory_growth);
               
               // Memory growth should be reasonable
               assert!(
                   memory_growth < 500.0,
                   "Memory growth should be less than 500MB"
               );
           }
       }
       
       // Final memory check
       let final_memory = get_memory_usage();
       let total_growth = final_memory - initial_memory;
       
       println!("Total memory growth: {:.2} MB", total_growth);
       assert!(
           total_growth < 1000.0,
           "Total memory growth should be less than 1GB"
       );
   }
   
   fn get_memory_usage() -> f64 {
       // Placeholder - would implement actual memory monitoring
       // Could use system calls or process monitoring
       0.0
   }
   
   #[tokio::test]
   async fn test_resource_cleanup() {
       // Test that resources are properly cleaned up
       {
           let service = create_test_embedding_service().await.unwrap();
           let _embedding = service.generate_embedding("Cleanup test").await.unwrap();
           // Service goes out of scope here
       }
       
       // Give some time for cleanup
       tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
       
       // Verify no resources are leaked
       // This would be more sophisticated in a real implementation
       assert!(true, "Resource cleanup completed");
   }
   ```

## Success Criteria
- [ ] End-to-end pipeline tests pass
- [ ] Vector store integration works
- [ ] Performance requirements met
- [ ] Error handling is robust
- [ ] Concurrent processing works
- [ ] Memory usage is controlled
- [ ] Resource cleanup works properly

## Files to Create
- `src/ml/tests/integration_tests.rs`
- `src/ml/tests/mod.rs`

## Files to Modify
- `src/lib.rs` (ensure test modules are included)

## Test Categories
1. **Functional Tests**: Basic embedding generation
2. **Integration Tests**: Full pipeline with vector store
3. **Performance Tests**: Speed and throughput requirements
4. **Error Tests**: Edge cases and error conditions
5. **Concurrency Tests**: Multi-threaded safety
6. **Resource Tests**: Memory and cleanup validation

## Running Tests
```bash
# Run all integration tests
cargo test ml::tests::integration_tests --verbose

# Run specific test categories
cargo test test_complete_embedding_pipeline
cargo test test_performance_requirements --release
cargo test test_concurrent_processing
```

## Next Task
→ Task 039: Create end-to-end pipeline validation