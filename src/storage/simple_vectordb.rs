#[cfg(feature = "vectordb")]
use std::path::PathBuf;
#[cfg(feature = "vectordb")]
use std::sync::Arc;
#[cfg(feature = "vectordb")]
use parking_lot::RwLock;
#[cfg(feature = "vectordb")]
use anyhow::Result;
#[cfg(feature = "vectordb")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "vectordb")]
use crate::chunking::Chunk;
#[cfg(feature = "vectordb")]
#[derive(Debug)]
pub enum StorageError {
    DatabaseError(String),
    SchemaError(String),
    InsertError(String),
    SearchError(String),
    InvalidInput(String),
    InvalidVector {
        reason: String,
    },
}

#[cfg(feature = "vectordb")]
impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StorageError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            StorageError::SchemaError(msg) => write!(f, "Schema error: {}", msg),
            StorageError::InsertError(msg) => write!(f, "Insert error: {}", msg),
            StorageError::SearchError(msg) => write!(f, "Search error: {}", msg),
            StorageError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            StorageError::InvalidVector { reason } => write!(f, "Invalid vector: {}", reason),
        }
    }
}

#[cfg(feature = "vectordb")]
impl std::error::Error for StorageError {}

#[cfg(feature = "vectordb")]
impl From<sled::Error> for StorageError {
    fn from(err: sled::Error) -> Self {
        StorageError::DatabaseError(err.to_string())
    }
}

#[cfg(feature = "vectordb")]
impl From<serde_json::Error> for StorageError {
    fn from(err: serde_json::Error) -> Self {
        StorageError::DatabaseError(err.to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg(feature = "vectordb")]
pub struct EmbeddingRecord {
    pub id: String,
    pub file_path: String,
    pub chunk_index: u32,
    pub content: String,
    pub embedding: Vec<f32>,
    pub start_line: usize,
    pub end_line: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg(feature = "vectordb")]
pub struct VectorSchema {
    pub version: u32,
    pub embedding_dim: usize,
    pub created_at: String,
}

#[cfg(feature = "vectordb")]
pub struct VectorStorage {
    db: sled::Db,
    schema: Arc<RwLock<Option<VectorSchema>>>,
}

#[cfg(feature = "vectordb")]
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
            schema: Arc::new(RwLock::new(None)),
        })
    }
    
    pub async fn init_schema(&self) -> Result<(), StorageError> {
        let schema = VectorSchema {
            version: 1,
            embedding_dim: 768,
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        
        let schema_json = serde_json::to_vec(&schema)?;
        self.db.insert(b"__schema__", schema_json)?;
        *self.schema.write() = Some(schema);
        
        Ok(())
    }
    
    pub fn get_schema(&self) -> Option<VectorSchema> {
        self.schema.read().clone()
    }
    
    pub async fn insert_embedding(
        &self,
        file_path: &str,
        chunk_index: usize,
        chunk: &Chunk,
        embedding: Vec<f32>
    ) -> Result<(), StorageError> {
        let expected_dim = 768;
        if embedding.len() != expected_dim {
            return Err(StorageError::InvalidInput(
                format!("Embedding must be {}-dimensional, got {}", expected_dim, embedding.len())
            ));
        }
        
        let record = EmbeddingRecord {
            id: format!("{}-{}", file_path, chunk_index),
            file_path: file_path.to_string(),
            chunk_index: chunk_index as u32,
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
        &self,
        embeddings_data: Vec<(&str, usize, Chunk, Vec<f32>)>
    ) -> Result<(), StorageError> {
        if embeddings_data.is_empty() {
            return Ok(());
        }
        
        // Validate all embeddings match expected dimensions
        let expected_dim = 768;
        for (_, _, _, embedding) in &embeddings_data {
            if embedding.len() != expected_dim {
                return Err(StorageError::InvalidInput(
                    format!("All embeddings must be {}-dimensional, got {}", expected_dim, embedding.len())
                ));
            }
        }
        
        // Use transaction for batch insert - explicitly create new batch
        let mut batch = sled::Batch::default();
        
        for (file_path, chunk_index, chunk, embedding) in embeddings_data {
            let record = EmbeddingRecord {
                id: format!("{}-{}", file_path, chunk_index),
                file_path: file_path.to_string(),
                chunk_index: chunk_index as u32,
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
    
    pub async fn delete_by_file(&self, file_path: &str) -> Result<(), StorageError> {
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
    
    pub async fn clear_all(&self) -> Result<(), StorageError> {
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
    
    pub async fn prepare_for_search(&self) -> Result<(), StorageError> {
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
        let expected_dim = 768;
        if query_embedding.len() != expected_dim {
            return Err(StorageError::InvalidInput(
                format!("Query embedding must be {}-dimensional, got {}", expected_dim, query_embedding.len())
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
        
        // Validate all similarity scores are finite before sorting - PRINCIPLE 0: No NaN fallbacks
        for (idx, (similarity, _)) in similarities.iter().enumerate() {
            if !similarity.is_finite() {
                return Err(StorageError::InvalidVector {
                    reason: format!("Similarity calculation produced invalid result (NaN or infinite) at index {}. Score: {}. This indicates corrupted similarity computation and cannot be recovered from.", idx, similarity),
                });
            }
        }
        
        // Sort by similarity (descending) - safe after validation
        similarities.sort_by(|a, b| {
            b.0.partial_cmp(&a.0).unwrap() // Safe after validation
        });
        
        // Take top results
        let results = similarities.into_iter()
            .take(limit)
            .map(|(_, record)| record)
            .collect();
        
        Ok(results)
    }
}

// Helper function for cosine similarity
#[allow(dead_code)]
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

// Thread safety is automatically provided by Arc<RwLock<>> and sled::Db

#[cfg(all(test, feature = "vectordb"))]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use crate::chunking::Chunk;
    use crate::Config;
    
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
        
        let storage = VectorStorage::new(db_path).await.unwrap();
        let result = storage.init_schema().await;
        assert!(result.is_ok());
        
        let schema = storage.get_schema().unwrap();
        assert_eq!(schema.embedding_dim, 768);
        assert_eq!(schema.version, 1);
    }
    
    #[tokio::test]
    async fn test_embedding_insertion() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_insert.db");
        
        let storage = VectorStorage::new(db_path).await.unwrap();
        storage.init_schema().await.unwrap();
        
        let chunk = Chunk {
            content: "fn test() {}".to_string(),
            start_line: 1,
            end_line: 1,
        };
        
        let embedding = vec![0.1f32; 768];
        let result = storage.insert_embedding("test.rs", 0, &chunk, embedding).await;
        assert!(result.is_ok());
        
        let count = storage.count().await.unwrap();
        assert_eq!(count, 1);
    }
    
    #[tokio::test]
    async fn test_similarity_search() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_search.db");
        
        let storage = VectorStorage::new(db_path).await.unwrap();
        storage.init_schema().await.unwrap();
        
        // Insert test embeddings
        let chunk1 = Chunk { content: "fn test1() {}".to_string(), start_line: 1, end_line: 1 };
        let chunk2 = Chunk { content: "fn test2() {}".to_string(), start_line: 3, end_line: 3 };
        
        let embedding1 = vec![1.0f32; 768];
        let embedding2 = vec![0.5f32; 768];
        
        storage.insert_embedding("test.rs", 0, &chunk1, embedding1.clone()).await.unwrap();
        storage.insert_embedding("test.rs", 1, &chunk2, embedding2).await.unwrap();
        
        // Search with query similar to embedding1
        let query = vec![1.0f32; 768];
        let results = storage.search_similar(query, 2).await.unwrap();
        
        assert_eq!(results.len(), 2);
        // First result should be more similar (embedding1)
        assert_eq!(results[0].chunk_index, 0);
    }
}