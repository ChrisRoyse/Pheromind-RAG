# Task 018 - HIGH: Create LanceDB Table Schema for Embeddings

## Priority: HIGH
## Estimated Time: 10 minutes
## Dependencies: Task 017

## Objective
Define and implement the proper table schema for storing embeddings, metadata, and search indices.

## Current Issue
- Schema needs to be optimized for vector search
- Must support metadata filtering
- Proper indexing for performance

## Tasks
1. **Define embedding schema** (4 min)
   ```rust
   // Update src/storage/lancedb_store.rs
   use arrow_array::{
       Float32Array, StringArray, RecordBatch, Int64Array,
       FixedSizeListArray, UInt32Array
   };
   use arrow_schema::{DataType, Field, Schema, TimeUnit};
   use chrono::{DateTime, Utc};
   
   impl LanceDBStore {
       fn create_embedding_schema(embedding_dim: usize) -> Result<Schema> {
           let fields = vec![
               // Primary key
               Field::new("id", DataType::Utf8, false),
               
               // Vector embedding (fixed size list of f32)
               Field::new(
                   "embedding",
                   DataType::FixedSizeList(
                       Arc::new(Field::new("item", DataType::Float32, true)),
                       embedding_dim as i32,
                   ),
                   false,
               ),
               
               // Metadata as JSON string
               Field::new("metadata", DataType::Utf8, true),
               
               // Content hash for deduplication
               Field::new("content_hash", DataType::Utf8, true),
               
               // Source information
               Field::new("source_type", DataType::Utf8, true),
               Field::new("source_path", DataType::Utf8, true),
               
               // Timestamps
               Field::new(
                   "created_at",
                   DataType::Timestamp(TimeUnit::Millisecond, None),
                   false,
               ),
               Field::new(
                   "updated_at",
                   DataType::Timestamp(TimeUnit::Millisecond, None),
                   false,
               ),
               
               // Search optimization fields
               Field::new("embedding_norm", DataType::Float32, true),
               Field::new("chunk_size", DataType::UInt32, true),
               Field::new("language", DataType::Utf8, true),
           ];
           
           Ok(Schema::new(fields))
       }
   ```

2. **Add schema validation** (3 min)
   ```rust
   impl LanceDBStore {
       pub fn validate_schema(&self, batch: &RecordBatch) -> Result<()> {
           let expected_fields = [
               "id", "embedding", "metadata", "content_hash",
               "source_type", "source_path", "created_at", "updated_at",
               "embedding_norm", "chunk_size", "language"
           ];
           
           let schema = batch.schema();
           
           for field_name in &expected_fields {
               if schema.column_with_name(field_name).is_none() {
                   return Err(anyhow!("Missing required field: {}", field_name));
               }
           }
           
           // Validate embedding dimension
           if let Some((_, field)) = schema.column_with_name("embedding") {
               if let DataType::FixedSizeList(_, size) = field.data_type() {
                   if *size != self.embedding_dim as i32 {
                       return Err(anyhow!(
                           "Embedding dimension mismatch: expected {}, got {}",
                           self.embedding_dim,
                           size
                       ));
                   }
               } else {
                   return Err(anyhow!("Embedding field has wrong data type"));
               }
           }
           
           Ok(())
       }
   ```

3. **Add index configuration** (3 min)
   ```rust
   impl LanceDBStore {
       pub async fn create_vector_index(&self) -> Result<()> {
           let table = self.get_table().await?;
           
           // Create vector index for similarity search
           table
               .create_index(&["embedding"])
               .index_type(lancedb::index::IndexType::IVF_PQ)
               .num_partitions(256)
               .num_sub_vectors(16)
               .execute()
               .await?;
               
           println!("Created vector index on embedding column");
           
           // Create scalar indexes for filtering
           table
               .create_index(&["source_type"])
               .index_type(lancedb::index::IndexType::BTREE)
               .execute()
               .await?;
               
           table
               .create_index(&["created_at"])
               .index_type(lancedb::index::IndexType::BTREE)
               .execute()
               .await?;
               
           println!("Created scalar indexes");
           Ok(())
       }
       
       pub async fn optimize_table(&self) -> Result<()> {
           let table = self.get_table().await?;
           
           // Compact small files
           table.optimize().execute().await?;
           
           println!("Table optimized");
           Ok(())
       }
   }
   ```

## Success Criteria
- [ ] Schema compiles without errors
- [ ] All required fields defined
- [ ] Embedding dimension is configurable
- [ ] Schema validation works
- [ ] Index creation is implemented

## Files to Modify
- `src/storage/lancedb_store.rs`

## Schema Fields

### Required Fields
- `id`: Unique identifier (String)
- `embedding`: Vector (FixedSizeList<f32>)
- `metadata`: JSON metadata (String)
- `created_at`: Creation timestamp
- `updated_at`: Update timestamp

### Optional Fields
- `content_hash`: For deduplication
- `source_type`: File type or source
- `source_path`: Original file path
- `embedding_norm`: Cached L2 norm
- `chunk_size`: Text chunk size
- `language`: Detected language

## Validation
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_schema_creation() {
        let schema = LanceDBStore::create_embedding_schema(768).unwrap();
        assert_eq!(schema.fields().len(), 11);
        assert!(schema.column_with_name("embedding").is_some());
    }
}
```

## Next Task
→ Task 019: Implement add_embedding method for LanceDB