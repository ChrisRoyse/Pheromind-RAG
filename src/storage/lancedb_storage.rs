use std::path::PathBuf;
use std::sync::Arc;
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use arrow_array::{RecordBatch, StringArray, UInt64Array, Float32Array, FixedSizeListArray};
use arrow_schema::{DataType, Field, Schema};
use lancedb::{Connection, Table};
use crate::chunking::Chunk;

#[derive(Debug)]
pub enum LanceStorageError {
    DatabaseError(String),
    SchemaError(String),
    InsertError(String), 
    SearchError(String),
    InvalidInput(String),
}

impl std::fmt::Display for LanceStorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LanceStorageError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            LanceStorageError::SchemaError(msg) => write!(f, "Schema error: {}", msg),
            LanceStorageError::InsertError(msg) => write!(f, "Insert error: {}", msg),
            LanceStorageError::SearchError(msg) => write!(f, "Search error: {}", msg),
            LanceStorageError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
        }
    }
}

impl std::error::Error for LanceStorageError {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanceEmbeddingRecord {
    pub id: String,
    pub file_path: String,
    pub chunk_index: u64,
    pub content: String,
    pub embedding: Vec<f32>,
    pub start_line: u64,
    pub end_line: u64,
}

/// Real LanceDB vector storage for 384-dimensional embeddings
pub struct LanceDBStorage {
    connection: Arc<Connection>,
    table_name: String,
    schema: Arc<Schema>,
}

impl LanceDBStorage {
    /// Create new LanceDB storage connection
    pub async fn new(db_path: PathBuf) -> Result<Self, LanceStorageError> {
        println!("🔄 Connecting to LanceDB at {:?}", db_path);
        
        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            tokio::fs::create_dir_all(parent).await
                .map_err(|e| LanceStorageError::DatabaseError(format!("Failed to create directory: {}", e)))?;
        }
        
        // Connect to LanceDB
        let uri = db_path.to_string_lossy().to_string();
        let connection = lancedb::connect(&uri).await
            .map_err(|e| LanceStorageError::DatabaseError(format!("Connection failed: {}", e)))?;
        
        // Define schema for 384-dimensional embeddings
        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Utf8, false),
            Field::new("file_path", DataType::Utf8, false),
            Field::new("chunk_index", DataType::UInt64, false),
            Field::new("content", DataType::Utf8, false),
            Field::new("embedding", DataType::FixedSizeList(
                Arc::new(Field::new("item", DataType::Float32, true)), 384
            ), false),
            Field::new("start_line", DataType::UInt64, false),
            Field::new("end_line", DataType::UInt64, false),
        ]));
        
        println!("✅ Connected to LanceDB with 384-dimensional embedding schema");
        
        Ok(Self {
            connection: Arc::new(connection),
            table_name: "embeddings".to_string(),
            schema,
        })
    }
    
    /// Initialize the embeddings table
    pub async fn init_table(&self) -> Result<(), LanceStorageError> {
        // Check if table already exists
        let table_names = self.connection.table_names().await
            .map_err(|e| LanceStorageError::SchemaError(format!("Failed to list tables: {}", e)))?;
        
        if table_names.contains(&self.table_name) {
            println!("📋 Table '{}' already exists", self.table_name);
            return Ok(());
        }
        
        // Create empty table with schema
        let empty_batch = RecordBatch::new_empty(self.schema.clone());
        
        self.connection.create_table(&self.table_name, empty_batch).await
            .map_err(|e| LanceStorageError::SchemaError(format!("Failed to create table: {}", e)))?;
        
        println!("✅ Created LanceDB table '{}'", self.table_name);
        Ok(())
    }
    
    /// Insert a single embedding record
    pub async fn insert_embedding(
        &self,
        file_path: &str,
        chunk_index: usize,
        chunk: &Chunk,
        embedding: Vec<f32>
    ) -> Result<(), LanceStorageError> {
        if embedding.len() != 384 {
            return Err(LanceStorageError::InvalidInput(
                format!("Embedding must be 384-dimensional, got {}", embedding.len())
            ));
        }
        
        let record = LanceEmbeddingRecord {
            id: format!("{}-{}", file_path, chunk_index),
            file_path: file_path.to_string(),
            chunk_index: chunk_index as u64,
            content: chunk.content.clone(),
            embedding,
            start_line: chunk.start_line as u64,
            end_line: chunk.end_line as u64,
        };
        
        self.insert_batch(vec![record]).await
    }
    
    /// Insert multiple embedding records efficiently
    pub async fn insert_batch(&self, records: Vec<LanceEmbeddingRecord>) -> Result<(), LanceStorageError> {
        if records.is_empty() {
            return Ok(());
        }
        
        // Validate all embeddings are 384-dimensional
        for record in &records {
            if record.embedding.len() != 384 {
                return Err(LanceStorageError::InvalidInput(
                    format!("All embeddings must be 384-dimensional, got {}", record.embedding.len())
                ));
            }
        }
        
        // Convert records to Arrow arrays
        let ids: Vec<String> = records.iter().map(|r| r.id.clone()).collect();
        let file_paths: Vec<String> = records.iter().map(|r| r.file_path.clone()).collect();
        let chunk_indices: Vec<u64> = records.iter().map(|r| r.chunk_index).collect();
        let contents: Vec<String> = records.iter().map(|r| r.content.clone()).collect();
        let start_lines: Vec<u64> = records.iter().map(|r| r.start_line).collect();
        let end_lines: Vec<u64> = records.iter().map(|r| r.end_line).collect();
        
        // Flatten embeddings for FixedSizeListArray
        let mut flat_embeddings = Vec::with_capacity(records.len() * 384);
        for record in &records {
            flat_embeddings.extend_from_slice(&record.embedding);
        }
        
        // Create Arrow arrays
        let id_array = StringArray::from(ids);
        let file_path_array = StringArray::from(file_paths);
        let chunk_index_array = UInt64Array::from(chunk_indices);
        let content_array = StringArray::from(contents);
        let start_line_array = UInt64Array::from(start_lines);
        let end_line_array = UInt64Array::from(end_lines);
        
        let embedding_values = Float32Array::from(flat_embeddings);
        let embedding_array = FixedSizeListArray::new(
            Arc::new(Field::new("item", DataType::Float32, true)),
            384,
            Arc::new(embedding_values),
            None,
        );
        
        // Create RecordBatch
        let batch = RecordBatch::try_new(
            self.schema.clone(),
            vec![
                Arc::new(id_array),
                Arc::new(file_path_array), 
                Arc::new(chunk_index_array),
                Arc::new(content_array),
                Arc::new(embedding_array),
                Arc::new(start_line_array),
                Arc::new(end_line_array),
            ],
        ).map_err(|e| LanceStorageError::InsertError(format!("RecordBatch creation failed: {}", e)))?;
        
        // Get table and insert
        let table = self.connection.open_table(&self.table_name).await
            .map_err(|e| LanceStorageError::InsertError(format!("Failed to open table: {}", e)))?;
        
        table.add(batch).await
            .map_err(|e| LanceStorageError::InsertError(format!("Insert failed: {}", e)))?;
        
        println!("✅ Inserted {} records into LanceDB", records.len());
        Ok(())
    }
    
    /// Perform vector similarity search
    pub async fn search_similar(&self, query_embedding: Vec<f32>, limit: usize) -> Result<Vec<LanceEmbeddingRecord>, LanceStorageError> {
        if query_embedding.len() != 384 {
            return Err(LanceStorageError::InvalidInput(
                format!("Query embedding must be 384-dimensional, got {}", query_embedding.len())
            ));
        }
        
        // Get table
        let table = self.connection.open_table(&self.table_name).await
            .map_err(|e| LanceStorageError::SearchError(format!("Failed to open table: {}", e)))?;
        
        // Perform vector search
        let results = table.vector_search(query_embedding)
            .map_err(|e| LanceStorageError::SearchError(format!("Vector search failed: {}", e)))?
            .limit(limit)
            .execute().await
            .map_err(|e| LanceStorageError::SearchError(format!("Search execution failed: {}", e)))?;
        
        // Convert results back to records
        let mut records = Vec::new();
        
        // Extract data from RecordBatch
        let id_array = results.column(0).as_any().downcast_ref::<StringArray>()
            .ok_or_else(|| LanceStorageError::SearchError("Failed to extract id column".to_string()))?;
        let file_path_array = results.column(1).as_any().downcast_ref::<StringArray>()
            .ok_or_else(|| LanceStorageError::SearchError("Failed to extract file_path column".to_string()))?;
        let chunk_index_array = results.column(2).as_any().downcast_ref::<UInt64Array>()
            .ok_or_else(|| LanceStorageError::SearchError("Failed to extract chunk_index column".to_string()))?;
        let content_array = results.column(3).as_any().downcast_ref::<StringArray>()
            .ok_or_else(|| LanceStorageError::SearchError("Failed to extract content column".to_string()))?;
        let embedding_array = results.column(4).as_any().downcast_ref::<FixedSizeListArray>()
            .ok_or_else(|| LanceStorageError::SearchError("Failed to extract embedding column".to_string()))?;
        let start_line_array = results.column(5).as_any().downcast_ref::<UInt64Array>()
            .ok_or_else(|| LanceStorageError::SearchError("Failed to extract start_line column".to_string()))?;
        let end_line_array = results.column(6).as_any().downcast_ref::<UInt64Array>()
            .ok_or_else(|| LanceStorageError::SearchError("Failed to extract end_line column".to_string()))?;
        
        for i in 0..results.num_rows() {
            let id = id_array.value(i).to_string();
            let file_path = file_path_array.value(i).to_string();
            let chunk_index = chunk_index_array.value(i);
            let content = content_array.value(i).to_string();
            let start_line = start_line_array.value(i);
            let end_line = end_line_array.value(i);
            
            // Extract embedding vector
            let embedding_list = embedding_array.value(i);
            let embedding_values = embedding_list.as_any().downcast_ref::<Float32Array>()
                .ok_or_else(|| LanceStorageError::SearchError("Failed to extract embedding values".to_string()))?;
            let embedding: Vec<f32> = (0..384).map(|j| embedding_values.value(j)).collect();
            
            records.push(LanceEmbeddingRecord {
                id,
                file_path,
                chunk_index,
                content,
                embedding,
                start_line,
                end_line,
            });
        }
        
        Ok(records)
    }
    
    /// Count total records in the table
    pub async fn count(&self) -> Result<usize, LanceStorageError> {
        let table = self.connection.open_table(&self.table_name).await
            .map_err(|e| LanceStorageError::DatabaseError(format!("Failed to open table: {}", e)))?;
        
        let count = table.count_rows(None).await
            .map_err(|e| LanceStorageError::DatabaseError(format!("Count failed: {}", e)))?;
        
        Ok(count)
    }
    
    /// Delete all records from the table
    pub async fn clear_all(&self) -> Result<(), LanceStorageError> {
        // Drop and recreate table
        let _ = self.connection.drop_table(&self.table_name).await;
        self.init_table().await?;
        
        println!("🧹 Cleared all records from LanceDB table");
        Ok(())
    }
    
    /// Delete records by file path
    pub async fn delete_by_file(&self, file_path: &str) -> Result<(), LanceStorageError> {
        let table = self.connection.open_table(&self.table_name).await
            .map_err(|e| LanceStorageError::DatabaseError(format!("Failed to open table: {}", e)))?;
        
        let predicate = format!("file_path = '{}'", file_path);
        table.delete(&predicate).await
            .map_err(|e| LanceStorageError::DatabaseError(format!("Delete failed: {}", e)))?;
        
        println!("🗑️  Deleted records for file: {}", file_path);
        Ok(())
    }
    
    /// Get storage info
    pub fn storage_info(&self) -> String {
        format!("LanceDB vector storage (384-dimensional embeddings)")
    }
}

// Thread safety
unsafe impl Send for LanceDBStorage {}
unsafe impl Sync for LanceDBStorage {}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_lancedb_storage_creation() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_lance.db");
        
        let storage = LanceDBStorage::new(db_path).await;
        assert!(storage.is_ok(), "LanceDB storage creation should succeed");
        
        let storage = storage.unwrap();
        let init_result = storage.init_table().await;
        assert!(init_result.is_ok(), "Table initialization should succeed");
    }
    
    #[tokio::test]
    async fn test_real_vector_operations() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_vector.db");
        
        let storage = LanceDBStorage::new(db_path).await.unwrap();
        storage.init_table().await.unwrap();
        
        // Create test chunk
        let chunk = Chunk {
            content: "fn test() { println!(\"hello\"); }".to_string(),
            start_line: 1,
            end_line: 1,
        };
        
        // Create real-looking embedding (normalized)
        let mut embedding = vec![0.1f32; 384];
        let norm = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        for val in &mut embedding {
            *val /= norm;
        }
        
        // Test insert
        let insert_result = storage.insert_embedding("test.rs", 0, &chunk, embedding.clone()).await;
        assert!(insert_result.is_ok(), "Insert should succeed");
        
        // Test count
        let count = storage.count().await.unwrap();
        assert_eq!(count, 1, "Should have 1 record");
        
        // Test search
        let search_results = storage.search_similar(embedding, 5).await.unwrap();
        assert_eq!(search_results.len(), 1, "Should find 1 result");
        assert_eq!(search_results[0].content, "fn test() { println!(\"hello\"); }", "Content should match");
        
        println!("✅ LanceDB real vector operations test passed");
    }
}