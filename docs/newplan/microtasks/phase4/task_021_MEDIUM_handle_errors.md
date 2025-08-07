# Task 021 - MEDIUM: Handle LanceDB Errors Properly

## Priority: MEDIUM
## Estimated Time: 10 minutes
## Dependencies: Task 020

## Objective
Implement comprehensive error handling for LanceDB operations with proper error types and recovery.

## Current Issue
- Generic error handling not informative
- No error recovery mechanisms
- Database connection errors not handled

## Tasks
1. **Define custom error types** (4 min)
   ```rust
   // In src/storage/errors.rs
   use thiserror::Error;
   
   #[derive(Error, Debug)]
   pub enum VectorStoreError {
       #[error("Database connection failed: {0}")]
       ConnectionFailed(String),
       
       #[error("Table not found: {0}")]
       TableNotFound(String),
       
       #[error("Schema validation failed: {0}")]
       SchemaValidation(String),
       
       #[error("Embedding dimension mismatch: expected {expected}, got {actual}")]
       DimensionMismatch { expected: usize, actual: usize },
       
       #[error("Invalid query parameters: {0}")]
       InvalidQuery(String),
       
       #[error("Search failed: {0}")]
       SearchFailed(String),
       
       #[error("Insert failed: {0}")]
       InsertFailed(String),
       
       #[error("Serialization error: {0}")]
       SerializationError(#[from] serde_json::Error),
       
       #[error("Arrow error: {0}")]
       ArrowError(#[from] arrow_schema::ArrowError),
       
       #[error("LanceDB error: {0}")]
       LanceDBError(#[from] lancedb::Error),
       
       #[error("IO error: {0}")]
       IoError(#[from] std::io::Error),
       
       #[error("Database corrupted: {0}")]
       DatabaseCorrupted(String),
       
       #[error("Operation timeout after {seconds} seconds")]
       Timeout { seconds: u64 },
   }
   
   pub type VectorStoreResult<T> = Result<T, VectorStoreError>;
   ```

2. **Add error handling wrapper** (4 min)
   ```rust
   // Update src/storage/lancedb_store.rs
   use super::errors::{VectorStoreError, VectorStoreResult};
   use std::time::{Duration, Instant};
   
   impl LanceDBStore {
       async fn execute_with_retry<F, T, Fut>(
           &self,
           operation: F,
           max_retries: u32,
           operation_name: &str,
       ) -> VectorStoreResult<T>
       where
           F: Fn() -> Fut,
           Fut: std::future::Future<Output = Result<T, lancedb::Error>>,
       {
           let mut last_error = None;
           
           for attempt in 0..=max_retries {
               match operation().await {
                   Ok(result) => return Ok(result),
                   Err(e) => {
                       last_error = Some(e);
                       
                       if attempt < max_retries {
                           let delay = Duration::from_millis(100 * (2_u64.pow(attempt)));
                           tokio::time::sleep(delay).await;
                           eprintln!("Retrying {} (attempt {}/{})", operation_name, attempt + 1, max_retries + 1);
                       }
                   }
               }
           }
           
           Err(VectorStoreError::LanceDBError(
               last_error.unwrap_or_else(|| lancedb::Error::Generic("Unknown error".to_string()))
           ))
       }
       
       async fn execute_with_timeout<F, Fut, T>(
           &self,
           operation: F,
           timeout_seconds: u64,
           operation_name: &str,
       ) -> VectorStoreResult<T>
       where
           F: Fn() -> Fut,
           Fut: std::future::Future<Output = VectorStoreResult<T>>,
       {
           let timeout = Duration::from_secs(timeout_seconds);
           
           match tokio::time::timeout(timeout, operation()).await {
               Ok(result) => result,
               Err(_) => Err(VectorStoreError::Timeout { seconds: timeout_seconds }),
           }
       }
   }
   ```

3. **Update methods with proper error handling** (2 min)
   ```rust
   // Update VectorStore implementation
   #[async_trait::async_trait]
   impl VectorStore for LanceDBStore {
       async fn add_embedding(
           &self,
           id: String,
           embedding: EmbeddingVector,
           metadata: serde_json::Value,
       ) -> VectorStoreResult<()> {
           // Validate input
           if embedding.len() != self.embedding_dim {
               return Err(VectorStoreError::DimensionMismatch {
                   expected: self.embedding_dim,
                   actual: embedding.len(),
               });
           }
           
           if id.is_empty() {
               return Err(VectorStoreError::InvalidQuery(
                   "ID cannot be empty".to_string()
               ));
           }
           
           // Execute with retry and timeout
           self.execute_with_timeout(
               || async {
                   self.execute_with_retry(
                       || async {
                           let table = self.get_table().await?;
                           let batch = self.create_embedding_batch(
                               vec![id.clone()],
                               vec![embedding.clone()],
                               vec![metadata.clone()],
                           ).map_err(|e| lancedb::Error::Generic(e.to_string()))?;
                           
                           table.add(vec![batch]).execute().await
                       },
                       3,
                       "add_embedding",
                   ).await
               },
               30,
               "add_embedding",
           ).await.map_err(|e| {
               VectorStoreError::InsertFailed(format!("Failed to add embedding '{}': {}", id, e))
           })?;
           
           Ok(())
       }
       
       async fn search(
           &self,
           query_embedding: &EmbeddingVector,
           limit: usize,
           threshold: Option<f32>,
       ) -> VectorStoreResult<Vec<SearchResult>> {
           // Validate input
           self.validate_search_params(query_embedding, limit)?;
           
           self.execute_with_timeout(
               || async {
                   self.execute_with_retry(
                       || async {
                           let table = self.get_table().await?;
                           
                           let mut query = table
                               .vector_search(query_embedding.clone())
                               .limit(limit)
                               .distance_type(lancedb::query::DistanceType::Cosine);
                           
                           if let Some(thresh) = threshold {
                               query = query.where_clause(&format!("_distance <= {}", thresh));
                           }
                           
                           query.execute().await
                       },
                       3,
                       "vector_search",
                   ).await
               },
               60,
               "vector_search",
           ).await
           .and_then(|results| {
               self.process_search_results(results)
                   .map_err(|e| VectorStoreError::SearchFailed(e.to_string()))
           })
       }
   }
   ```

## Success Criteria
- [ ] Custom error types defined
- [ ] Retry mechanism implemented
- [ ] Timeout handling works
- [ ] All operations use proper error handling
- [ ] Error messages are informative

## Files to Create
- `src/storage/errors.rs`

## Files to Modify
- `src/storage/lancedb_store.rs`
- `src/storage/mod.rs`
- `Cargo.toml` (add thiserror dependency)

## Dependencies to Add
```toml
[dependencies]
thiserror = "1.0"
```

## Validation
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_dimension_mismatch_error() {
        let store = create_test_store().await;
        
        let wrong_embedding = vec![0.1; 512]; // Wrong dimension
        let result = store.add_embedding(
            "test_id".to_string(),
            wrong_embedding,
            serde_json::json!({}),
        ).await;
        
        match result {
            Err(VectorStoreError::DimensionMismatch { expected, actual }) => {
                assert_eq!(expected, 768);
                assert_eq!(actual, 512);
            },
            _ => panic!("Expected DimensionMismatch error"),
        }
    }
}
```

## Next Task
→ Task 022: Test LanceDB storage operations