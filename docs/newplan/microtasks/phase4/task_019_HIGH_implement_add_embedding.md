# Task 019 - HIGH: Implement add_embedding Method for LanceDB

## Priority: HIGH
## Estimated Time: 10 minutes
## Dependencies: Task 018

## Objective
Implement the add_embedding method to store embeddings in LanceDB with proper data conversion.

## Current Issue
- VectorStore trait method not implemented
- Need Arrow data conversion for LanceDB
- Proper error handling required

## Tasks
1. **Implement add_embedding method** (6 min)
   ```rust
   // In src/storage/lancedb_store.rs
   #[async_trait::async_trait]
   impl VectorStore for LanceDBStore {
       async fn add_embedding(
           &self,
           id: String,
           embedding: EmbeddingVector,
           metadata: serde_json::Value,
       ) -> Result<()> {
           let table = self.get_table().await?;
           
           // Validate embedding dimension
           if embedding.len() != self.embedding_dim {
               return Err(anyhow!(
                   "Embedding dimension mismatch: expected {}, got {}",
                   self.embedding_dim,
                   embedding.len()
               ));
           }
           
           // Create record batch
           let batch = self.create_embedding_batch(
               vec![id],
               vec![embedding],
               vec![metadata],
           )?;
           
           // Insert into table
           table.add(vec![batch]).execute().await?;
           
           Ok(())
       }
   }
   
   impl LanceDBStore {
       fn create_embedding_batch(
           &self,
           ids: Vec<String>,
           embeddings: Vec<EmbeddingVector>,
           metadata_list: Vec<serde_json::Value>,
       ) -> Result<RecordBatch> {
           let num_rows = ids.len();
           let now = Utc::now().timestamp_millis();
           
           // Create arrays
           let id_array = StringArray::from(ids);
           
           // Convert embeddings to FixedSizeListArray
           let embedding_array = self.create_embedding_array(embeddings)?;
           
           // Convert metadata to JSON strings
           let metadata_strings: Vec<String> = metadata_list
               .into_iter()
               .map(|m| serde_json::to_string(&m).unwrap_or_else(|_| "{}".to_string()))
               .collect();
           let metadata_array = StringArray::from(metadata_strings);
           
           // Create timestamp arrays
           let created_at_array = Int64Array::from(vec![now; num_rows]);
           let updated_at_array = Int64Array::from(vec![now; num_rows]);
           
           // Create optional fields with None values for now
           let content_hash_array = StringArray::from(vec![None::<String>; num_rows]);
           let source_type_array = StringArray::from(vec![None::<String>; num_rows]);
           let source_path_array = StringArray::from(vec![None::<String>; num_rows]);
           let embedding_norm_array = Float32Array::from(vec![None::<f32>; num_rows]);
           let chunk_size_array = UInt32Array::from(vec![None::<u32>; num_rows]);
           let language_array = StringArray::from(vec![None::<String>; num_rows]);
           
           // Create schema and batch
           let schema = Arc::new(Self::create_embedding_schema(self.embedding_dim)?);
           
           let batch = RecordBatch::try_new(
               schema,
               vec![
                   Arc::new(id_array),
                   Arc::new(embedding_array),
                   Arc::new(metadata_array),
                   Arc::new(content_hash_array),
                   Arc::new(source_type_array),
                   Arc::new(source_path_array),
                   Arc::new(created_at_array),
                   Arc::new(updated_at_array),
                   Arc::new(embedding_norm_array),
                   Arc::new(chunk_size_array),
                   Arc::new(language_array),
               ],
           )?;
           
           self.validate_schema(&batch)?;
           Ok(batch)
       }
   ```

2. **Add embedding array conversion** (3 min)
   ```rust
   impl LanceDBStore {
       fn create_embedding_array(
           &self,
           embeddings: Vec<EmbeddingVector>,
       ) -> Result<FixedSizeListArray> {
           let mut all_values = Vec::new();
           
           for embedding in &embeddings {
               if embedding.len() != self.embedding_dim {
                   return Err(anyhow!(
                       "Embedding dimension mismatch: expected {}, got {}",
                       self.embedding_dim,
                       embedding.len()
                   ));
               }
               all_values.extend_from_slice(embedding);
           }
           
           let values_array = Float32Array::from(all_values);
           
           let list_array = FixedSizeListArray::new(
               Arc::new(Field::new("item", DataType::Float32, true)),
               self.embedding_dim as i32,
               Arc::new(values_array),
               None, // No null values
           );
           
           Ok(list_array)
       }
   }
   ```

3. **Add batch insertion support** (1 min)
   ```rust
   impl LanceDBStore {
       pub async fn add_embeddings_batch(
           &self,
           data: Vec<(String, EmbeddingVector, serde_json::Value)>,
       ) -> Result<()> {
           if data.is_empty() {
               return Ok(());
           }
           
           let (ids, embeddings, metadata_list): (Vec<_>, Vec<_>, Vec<_>) = 
               data.into_iter().map(|(id, emb, meta)| (id, emb, meta)).multiunzip();
           
           let batch = self.create_embedding_batch(ids, embeddings, metadata_list)?;
           
           let table = self.get_table().await?;
           table.add(vec![batch]).execute().await?;
           
           Ok(())
       }
   }
   ```

## Success Criteria
- [ ] add_embedding method compiles
- [ ] Embedding validation works
- [ ] Arrow data conversion succeeds
- [ ] Data is properly inserted
- [ ] Batch insertion supported

## Files to Modify
- `src/storage/lancedb_store.rs`
- `Cargo.toml` (add itertools for multiunzip)

## Dependencies to Add
```toml
[dependencies]
itertools = "0.12"
chrono = { version = "0.4", features = ["serde"] }
```

## Validation
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[tokio::test]
    async fn test_add_embedding() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.lancedb");
        
        let store = LanceDBStore::new(
            db_path.to_str().unwrap(),
            "test_table",
            768,
        ).await.unwrap();
        
        let embedding = vec![0.1; 768];
        let metadata = serde_json::json!({"test": "value"});
        
        let result = store.add_embedding(
            "test_id".to_string(),
            embedding,
            metadata,
        ).await;
        
        assert!(result.is_ok());
    }
}
```

## Next Task
→ Task 020: Implement search method for similarity queries