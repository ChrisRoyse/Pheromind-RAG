use std::path::PathBuf;
use std::collections::HashMap;
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use crate::chunking::Chunk;

#[derive(Debug)]
pub enum StorageError {
    DatabaseError(String),
    SchemaError(String),
    InsertError(String),
    SearchError(String),
    InvalidInput(String),
}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StorageError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            StorageError::SchemaError(msg) => write!(f, "Schema error: {}", msg),
            StorageError::InsertError(msg) => write!(f, "Insert error: {}", msg),
            StorageError::SearchError(msg) => write!(f, "Search error: {}", msg),
            StorageError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
        }
    }
}

impl std::error::Error for StorageError {}

impl From<sled::Error> for StorageError {
    fn from(err: sled::Error) -> Self {
        StorageError::DatabaseError(err.to_string())
    }
}

impl From<serde_json::Error> for StorageError {
    fn from(err: serde_json::Error) -> Self {
        StorageError::DatabaseError(err.to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingRecord {
    pub id: String,
    pub file_path: String,
    pub chunk_index: usize,
    pub content: String,
    pub embedding: Vec<f32>,
    pub start_line: usize,
    pub end_line: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorSchema {
    pub version: u32,
    pub embedding_dim: usize,
    pub created_at: String,
}

pub struct VectorStorage {
    db: sled::Db,
    schema: Option<VectorSchema>,
}

impl VectorStorage {
    pub async fn new(db_path: PathBuf) -> Result<Self, StorageError> {
        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            tokio::fs::create_dir_all(parent).await
                .map_err(|e| StorageError::DatabaseError(format!("Failed to create directory: {}", e)))?;
        }
        
        let db = sled::open(db_path)?;
        
        Ok(Self {
            db,
            schema: None,
        })
    }
    
    pub async fn init_schema(&mut self) -> Result<(), StorageError> {
        let schema = VectorSchema {
            version: 1,
            embedding_dim: 384, // MiniLM dimensions
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        
        let schema_json = serde_json::to_vec(&schema)?;
        self.db.insert(b"__schema__", schema_json)?;
        self.schema = Some(schema);
        
        Ok(())
    }
    
    pub fn get_schema(&self) -> Option<&VectorSchema> {
        self.schema.as_ref()
    }
    
    pub async fn insert_embedding(
        &mut self,
        file_path: &str,
        chunk_index: usize,
        chunk: &Chunk,
        embedding: Vec<f32>
    ) -> Result<(), StorageError> {
        if embedding.len() != 384 {
            return Err(StorageError::InvalidInput(
                format!("Embedding must be 384-dimensional, got {}", embedding.len())
            ));
        }
        
        let record = EmbeddingRecord {
            id: format!("{}-{}", file_path, chunk_index),
            file_path: file_path.to_string(),
            chunk_index,
            content: chunk.content.clone(),
            embedding,
            start_line: chunk.start_line,
            end_line: chunk.end_line,
        };
        
        let record_json = serde_json::to_vec(&record)?;
        let key = format!("embedding:{}", record.id);
        
        self.db.insert(key.as_bytes(), record_json)?;
        
        Ok(())
    }
    
    pub async fn insert_batch(
        &mut self,
        embeddings_data: Vec<(&str, usize, Chunk, Vec<f32>)>
    ) -> Result<(), StorageError> {
        if embeddings_data.is_empty() {
            return Ok(());
        }
        
        // Validate all embeddings are 384-dimensional
        for (_, _, _, embedding) in &embeddings_data {
            if embedding.len() != 384 {
                return Err(StorageError::InvalidInput(
                    format!("All embeddings must be 384-dimensional, got {}", embedding.len())
                ));
            }
        }
        
        // Use transaction for batch insert
        let mut batch = sled::Batch::default();
        
        for (file_path, chunk_index, chunk, embedding) in embeddings_data {
            let record = EmbeddingRecord {
                id: format!("{}-{}", file_path, chunk_index),
                file_path: file_path.to_string(),
                chunk_index,
                content: chunk.content,
                embedding,
                start_line: chunk.start_line,
                end_line: chunk.end_line,
            };
            
            let record_json = serde_json::to_vec(&record)?;
            let key = format!("embedding:{}", record.id);
            batch.insert(key.as_bytes(), record_json);
        }
        
        self.db.apply_batch(batch)?;
        Ok(())
    }
    
    pub async fn delete_by_file(&mut self, file_path: &str) -> Result<(), StorageError> {
        let mut batch = sled::Batch::default();
        
        // Find all keys for this file
        for result in self.db.scan_prefix(b"embedding:") {
            let (key, value) = result?;
            let record: EmbeddingRecord = serde_json::from_slice(&value)?;
            
            if record.file_path == file_path {
                batch.remove(key);
            }
        }
        
        self.db.apply_batch(batch)?;
        Ok(())
    }
    
    pub async fn clear_all(&mut self) -> Result<(), StorageError> {
        let mut batch = sled::Batch::default();
        
        // Remove all embedding records but keep schema
        for result in self.db.scan_prefix(b"embedding:") {
            let (key, _) = result?;
            batch.remove(key);
        }
        
        self.db.apply_batch(batch)?;
        Ok(())
    }
    
    pub async fn count(&self) -> Result<usize, StorageError> {
        let count = self.db.scan_prefix(b"embedding:").count();
        Ok(count)
    }
    
    pub async fn prepare_for_search(&mut self) -> Result<(), StorageError> {
        // For now, this is a no-op since we don't have complex indexing
        // In a real implementation, this would create vector indexes
        Ok(())
    }
    
    pub async fn get_all_embeddings(&self) -> Result<Vec<EmbeddingRecord>, StorageError> {
        let mut embeddings = Vec::new();
        
        for result in self.db.scan_prefix(b"embedding:") {
            let (_, value) = result?;
            let record: EmbeddingRecord = serde_json::from_slice(&value)?;
            embeddings.push(record);
        }
        
        Ok(embeddings)
    }
    
    pub async fn search_similar(&self, query_embedding: Vec<f32>, limit: usize) -> Result<Vec<EmbeddingRecord>, StorageError> {
        if query_embedding.len() != 384 {
            return Err(StorageError::InvalidInput(
                format!("Query embedding must be 384-dimensional, got {}", query_embedding.len())
            ));
        }
        
        let mut similarities = Vec::new();
        
        // Calculate cosine similarity with all embeddings (brute force for now)
        for result in self.db.scan_prefix(b"embedding:") {
            let (_, value) = result?;
            let record: EmbeddingRecord = serde_json::from_slice(&value)?;
            
            let similarity = cosine_similarity(&query_embedding, &record.embedding);
            similarities.push((similarity, record));
        }
        
        // Sort by similarity (descending)
        similarities.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        
        // Take top results
        let results = similarities.into_iter()
            .take(limit)
            .map(|(_, record)| record)
            .collect();
        
        Ok(results)
    }
}

// Helper function for cosine similarity
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }
    
    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    
    dot_product / (norm_a * norm_b)
}

// Thread-safe implementation
unsafe impl Send for VectorStorage {}
unsafe impl Sync for VectorStorage {}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_basic_storage_creation() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        
        let storage = VectorStorage::new(db_path).await;
        assert!(storage.is_ok());
    }
    
    #[tokio::test]
    async fn test_schema_creation() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_schema.db");
        
        let mut storage = VectorStorage::new(db_path).await.unwrap();
        let result = storage.init_schema().await;
        assert!(result.is_ok());
        
        let schema = storage.get_schema().unwrap();
        assert_eq!(schema.embedding_dim, 384);
        assert_eq!(schema.version, 1);
    }
    
    #[tokio::test]
    async fn test_embedding_insertion() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_insert.db");
        
        let mut storage = VectorStorage::new(db_path).await.unwrap();
        storage.init_schema().await.unwrap();
        
        let chunk = Chunk {
            content: "fn test() {}".to_string(),
            start_line: 1,
            end_line: 1,
        };
        
        let embedding = vec![0.1f32; 384];
        let result = storage.insert_embedding("test.rs", 0, &chunk, embedding).await;
        assert!(result.is_ok());
        
        let count = storage.count().await.unwrap();
        assert_eq!(count, 1);
    }
    
    #[tokio::test]
    async fn test_similarity_search() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_search.db");
        
        let mut storage = VectorStorage::new(db_path).await.unwrap();
        storage.init_schema().await.unwrap();
        
        // Insert test embeddings
        let chunk1 = Chunk { content: "fn test1() {}".to_string(), start_line: 1, end_line: 1 };
        let chunk2 = Chunk { content: "fn test2() {}".to_string(), start_line: 3, end_line: 3 };
        
        let embedding1 = vec![1.0f32; 384];
        let embedding2 = vec![0.5f32; 384];
        
        storage.insert_embedding("test.rs", 0, &chunk1, embedding1.clone()).await.unwrap();
        storage.insert_embedding("test.rs", 1, &chunk2, embedding2).await.unwrap();
        
        // Search with query similar to embedding1
        let query = vec![1.0f32; 384];
        let results = storage.search_similar(query, 2).await.unwrap();
        
        assert_eq!(results.len(), 2);
        // First result should be more similar (embedding1)
        assert_eq!(results[0].chunk_index, 0);
    }
}