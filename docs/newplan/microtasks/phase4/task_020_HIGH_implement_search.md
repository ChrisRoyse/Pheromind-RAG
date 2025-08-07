# Task 020 - HIGH: Implement Search Method for Similarity Queries

## Priority: HIGH
## Estimated Time: 10 minutes
## Dependencies: Task 019

## Objective
Implement vector similarity search using LanceDB's vector search capabilities.

## Current Issue
- Search method not implemented in VectorStore trait
- Need to convert query embeddings to LanceDB format
- Result processing and scoring required

## Tasks
1. **Implement search method** (6 min)
   ```rust
   // In src/storage/lancedb_store.rs
   use arrow_array::Array;
   use lancedb::query::{QueryBase, VectorQuery};
   
   #[async_trait::async_trait]
   impl VectorStore for LanceDBStore {
       async fn search(
           &self,
           query_embedding: &EmbeddingVector,
           limit: usize,
           threshold: Option<f32>,
       ) -> Result<Vec<SearchResult>> {
           // Validate query embedding
           if query_embedding.len() != self.embedding_dim {
               return Err(anyhow!(
                   "Query embedding dimension mismatch: expected {}, got {}",
                   self.embedding_dim,
                   query_embedding.len()
               ));
           }
           
           let table = self.get_table().await?;
           
           // Perform vector search
           let mut query = table
               .vector_search(query_embedding.clone())
               .limit(limit)
               .distance_type(lancedb::query::DistanceType::Cosine);
               
           // Add distance threshold if specified
           if let Some(thresh) = threshold {
               query = query.where_clause(&format!("_distance <= {}", thresh));
           }
           
           let results = query.execute().await?;
           
           // Convert results to SearchResult format
           self.process_search_results(results).await
       }
   }
   
   impl LanceDBStore {
       async fn process_search_results(
           &self,
           results: RecordBatch,
       ) -> Result<Vec<SearchResult>> {
           let mut search_results = Vec::new();
           let num_rows = results.num_rows();
           
           if num_rows == 0 {
               return Ok(search_results);
           }
           
           // Extract columns
           let id_column = results
               .column_by_name("id")
               .ok_or_else(|| anyhow!("Missing id column in search results"))?;
           let id_array = id_column
               .as_any()
               .downcast_ref::<StringArray>()
               .ok_or_else(|| anyhow!("Invalid id column type"))?;
           
           let embedding_column = results
               .column_by_name("embedding")
               .ok_or_else(|| anyhow!("Missing embedding column in search results"))?;
           let embedding_array = embedding_column
               .as_any()
               .downcast_ref::<FixedSizeListArray>()
               .ok_or_else(|| anyhow!("Invalid embedding column type"))?;
           
           let metadata_column = results.column_by_name("metadata");
           let metadata_array = metadata_column
               .map(|col| col.as_any().downcast_ref::<StringArray>())
               .flatten();
           
           // Extract distance scores if available
           let distance_column = results.column_by_name("_distance");
           let distance_array = distance_column
               .map(|col| col.as_any().downcast_ref::<Float32Array>())
               .flatten();
           
           // Process each row
           for i in 0..num_rows {
               let id = id_array.value(i).to_string();
               
               // Extract embedding
               let embedding_list = embedding_array.value(i);
               let embedding_values = embedding_list
                   .as_any()
                   .downcast_ref::<Float32Array>()
                   .ok_or_else(|| anyhow!("Invalid embedding values type"))?;
               
               let embedding: Vec<f32> = (0..embedding_values.len())
                   .map(|j| embedding_values.value(j))
                   .collect();
               
               // Extract metadata
               let metadata = if let Some(meta_array) = metadata_array {
                   if !meta_array.is_null(i) {
                       let meta_str = meta_array.value(i);
                       serde_json::from_str(meta_str).unwrap_or_else(|_| serde_json::json!({}))
                   } else {
                       serde_json::json!({})
                   }
               } else {
                   serde_json::json!({})
               };
               
               // Extract distance score (convert to similarity)
               let score = if let Some(dist_array) = distance_array {
                   if !dist_array.is_null(i) {
                       let distance = dist_array.value(i);
                       // Convert cosine distance to similarity (1 - distance)
                       1.0 - distance
                   } else {
                       0.0
                   }
               } else {
                   0.0
               };
               
               search_results.push(SearchResult {
                   id,
                   embedding,
                   metadata,
                   score,
               });
           }
           
           Ok(search_results)
       }
   }
   ```

2. **Add search filtering** (3 min)
   ```rust
   impl LanceDBStore {
       pub async fn search_with_filter(
           &self,
           query_embedding: &EmbeddingVector,
           limit: usize,
           threshold: Option<f32>,
           filter: Option<&str>,
       ) -> Result<Vec<SearchResult>> {
           let table = self.get_table().await?;
           
           let mut query = table
               .vector_search(query_embedding.clone())
               .limit(limit)
               .distance_type(lancedb::query::DistanceType::Cosine);
           
           // Add filters
           if let Some(thresh) = threshold {
               query = query.where_clause(&format!("_distance <= {}", thresh));
           }
           
           if let Some(filter_clause) = filter {
               query = query.where_clause(filter_clause);
           }
           
           let results = query.execute().await?;
           self.process_search_results(results).await
       }
       
       pub async fn search_by_metadata(
           &self,
           query_embedding: &EmbeddingVector,
           limit: usize,
           source_type: Option<&str>,
           language: Option<&str>,
       ) -> Result<Vec<SearchResult>> {
           let mut filters = Vec::new();
           
           if let Some(src_type) = source_type {
               filters.push(format!("source_type = '{}'", src_type));
           }
           
           if let Some(lang) = language {
               filters.push(format!("language = '{}'", lang));
           }
           
           let filter_clause = if !filters.is_empty() {
               Some(filters.join(" AND ").as_str())
           } else {
               None
           };
           
           self.search_with_filter(query_embedding, limit, None, filter_clause).await
       }
   }
   ```

3. **Add search validation** (1 min)
   ```rust
   impl LanceDBStore {
       fn validate_search_params(
           &self,
           query_embedding: &EmbeddingVector,
           limit: usize,
       ) -> Result<()> {
           if query_embedding.len() != self.embedding_dim {
               return Err(anyhow!(
                   "Query embedding dimension mismatch: expected {}, got {}",
                   self.embedding_dim,
                   query_embedding.len()
               ));
           }
           
           if limit == 0 {
               return Err(anyhow!("Search limit must be greater than 0"));
           }
           
           if limit > 10000 {
               return Err(anyhow!("Search limit too large: max 10000, got {}", limit));
           }
           
           Ok(())
       }
   }
   ```

## Success Criteria
- [ ] Search method compiles successfully
- [ ] Vector search query executes
- [ ] Results are properly converted
- [ ] Distance scores calculated correctly
- [ ] Filtering works as expected

## Files to Modify
- `src/storage/lancedb_store.rs`
- `src/types.rs` (ensure SearchResult is properly defined)

## Validation
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_search() {
        let store = create_test_store().await;
        
        // Add test embedding
        let embedding = vec![0.1; 768];
        store.add_embedding(
            "test_id".to_string(),
            embedding.clone(),
            serde_json::json!({"test": "value"}),
        ).await.unwrap();
        
        // Search for similar embeddings
        let results = store.search(&embedding, 10, None).await.unwrap();
        
        assert!(!results.is_empty());
        assert_eq!(results[0].id, "test_id");
        assert!(results[0].score > 0.9); // Should be very similar
    }
}
```

## Next Task
→ Task 021: Handle LanceDB errors properly