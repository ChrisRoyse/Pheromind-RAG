# Task 017 - HIGH: Setup LanceDB Connection and Configuration

## Priority: HIGH
## Estimated Time: 10 minutes
## Dependencies: Task 016

## Objective
Setup LanceDB as the new vector database backend with proper connection and configuration.

## Current Issue
- No vector database after removing Sled
- Need LanceDB integration for embeddings
- Configuration and connection setup required

## Tasks
1. **Add LanceDB dependency** (2 min)
   ```toml
   # Add to Cargo.toml
   [dependencies]
   lancedb = "0.4"
   arrow = "51.0"
   arrow-array = "51.0"
   arrow-schema = "51.0"
   tokio = { version = "1.0", features = ["full"] }
   ```

2. **Create LanceDB connection manager** (6 min)
   ```rust
   // In src/storage/lancedb_store.rs
   use lancedb::{Connection, Table};
   use arrow_array::{Float32Array, StringArray, RecordBatch};
   use arrow_schema::{DataType, Field, Schema};
   use anyhow::{Result, anyhow};
   use std::sync::Arc;
   use crate::types::{EmbeddingVector, SearchResult};
   use super::VectorStore;
   
   pub struct LanceDBStore {
       connection: Connection,
       table_name: String,
       embedding_dim: usize,
   }
   
   impl LanceDBStore {
       pub async fn new(db_path: &str, table_name: &str, embedding_dim: usize) -> Result<Self> {
           // Connect to LanceDB
           let connection = lancedb::connect(db_path).execute().await?;
           
           let store = Self {
               connection,
               table_name: table_name.to_string(),
               embedding_dim,
           };
           
           // Create table if it doesn't exist
           store.ensure_table_exists().await?;
           
           Ok(store)
       }
       
       async fn ensure_table_exists(&self) -> Result<()> {
           // Check if table exists
           let table_names = self.connection.table_names().execute().await?;
           
           if !table_names.contains(&self.table_name) {
               // Create schema for embeddings table
               let schema = Self::create_schema(self.embedding_dim)?;
               
               // Create empty table
               let empty_batch = RecordBatch::new_empty(Arc::new(schema));
               
               self.connection
                   .create_table(&self.table_name, vec![empty_batch])
                   .execute()
                   .await?;
                   
               println!("Created LanceDB table: {}", self.table_name);
           }
           
           Ok(())
       }
       
       fn create_schema(embedding_dim: usize) -> Result<Schema> {
           let mut fields = vec![
               Field::new("id", DataType::Utf8, false),
               Field::new(
                   "embedding",
                   DataType::FixedSizeList(
                       Arc::new(Field::new("item", DataType::Float32, true)),
                       embedding_dim as i32,
                   ),
                   false,
               ),
               Field::new("metadata", DataType::Utf8, true),
               Field::new("timestamp", DataType::Int64, false),
           ];
           
           Ok(Schema::new(fields))
       }
       
       async fn get_table(&self) -> Result<Table> {
           self.connection
               .open_table(&self.table_name)
               .execute()
               .await
               .map_err(|e| anyhow!("Failed to open table {}: {}", self.table_name, e))
       }
   }
   ```

3. **Add configuration** (2 min)
   ```rust
   // In src/config.rs
   #[derive(Debug, Clone)]
   pub struct DatabaseConfig {
       pub lancedb_path: String,
       pub table_name: String,
       pub embedding_dim: usize,
   }
   
   impl Default for DatabaseConfig {
       fn default() -> Self {
           Self {
               lancedb_path: "./data/embeddings.lancedb".to_string(),
               table_name: "embeddings".to_string(),
               embedding_dim: 768,
           }
       }
   }
   
   impl DatabaseConfig {
       pub fn from_env() -> Self {
           Self {
               lancedb_path: std::env::var("LANCEDB_PATH")
                   .unwrap_or_else(|_| "./data/embeddings.lancedb".to_string()),
               table_name: std::env::var("LANCEDB_TABLE")
                   .unwrap_or_else(|_| "embeddings".to_string()),
               embedding_dim: std::env::var("EMBEDDING_DIM")
                   .unwrap_or_else(|_| "768".to_string())
                   .parse()
                   .unwrap_or(768),
           }
       }
   }
   ```

## Success Criteria
- [ ] LanceDB dependency added
- [ ] Connection manager compiles
- [ ] Table creation works
- [ ] Schema is properly defined
- [ ] Configuration is flexible

## Files to Create
- `src/storage/lancedb_store.rs`

## Files to Modify
- `Cargo.toml`
- `src/config.rs`
- `src/storage/mod.rs`

## Validation
```bash
# Test compilation
cargo check

# Test basic connection (if data directory exists)
mkdir -p data
cargo test storage::lancedb_store::test_connection
```

## Next Task
→ Task 018: Create LanceDB table schema for embeddings