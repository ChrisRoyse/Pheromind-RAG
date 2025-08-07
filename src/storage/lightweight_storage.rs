/// Lightweight in-memory vector storage for fast testing
/// This replaces LanceDB for testing purposes to avoid heavy compilation

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::Result;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightweightRecord {
    pub id: String,
    pub file_path: String,
    pub chunk_index: u64,
    pub content: String,
    pub embedding: Vec<f32>,
    pub start_line: u64,
    pub end_line: u64,
}

/// Lightweight in-memory storage that mimics LanceDB interface
pub struct LightweightStorage {
    records: Arc<RwLock<Vec<LightweightRecord>>>,
    index: Arc<RwLock<HashMap<String, Vec<usize>>>>, // file_path -> record indices
}

impl LightweightStorage {
    pub fn new() -> Self {
        Self {
            records: Arc::new(RwLock::new(Vec::new())),
            index: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Insert a batch of records
    pub async fn insert_batch(&self, new_records: Vec<LightweightRecord>) -> Result<()> {
        let mut records = self.records.write().await;
        let mut index = self.index.write().await;
        
        for record in new_records {
            let idx = records.len();
            let file_path = record.file_path.clone();
            records.push(record);
            
            index.entry(file_path)
                .or_insert_with(Vec::new)
                .push(idx);
        }
        
        Ok(())
    }
    
    /// Search for similar embeddings using cosine similarity
    pub async fn search_similar(&self, query_embedding: Vec<f32>, limit: usize) -> Result<Vec<LightweightRecord>> {
        let records = self.records.read().await;
        
        if records.is_empty() {
            log::info!("No records in lightweight storage to search");
            return Ok(Vec::new());
        }
        
        // Normalize query embedding
        let query_norm = normalize_embedding(&query_embedding);
        
        // Calculate similarities
        let mut similarities: Vec<(f32, &LightweightRecord)> = records
            .iter()
            .map(|record| {
                let score = cosine_similarity(&query_norm, &record.embedding);
                (score, record)
            })
            .collect();
        
        // Validate all similarity scores are finite before sorting - PRINCIPLE 0: No NaN fallbacks
        for (idx, (similarity, _)) in similarities.iter().enumerate() {
            if !similarity.is_finite() {
                return Err(anyhow::anyhow!(
                    "Similarity calculation produced invalid result (NaN or infinite) at index {}. Score: {}. This indicates corrupted similarity computation and cannot be recovered from.", 
                    idx, similarity
                ));
            }
        }
        
        // Sort by similarity (descending) - safe after validation
        similarities.sort_by(|a, b| {
            b.0.partial_cmp(&a.0).unwrap() // Safe after validation
        });
        
        // Return top results
        Ok(similarities
            .into_iter()
            .take(limit)
            .map(|(_, record)| record.clone())
            .collect())
    }
    
    /// Delete records by file path
    pub async fn delete_by_file(&self, file_path: &str) -> Result<()> {
        let mut records = self.records.write().await;
        let mut index = self.index.write().await;
        
        // Remove from index
        if let Some(indices) = index.remove(file_path) {
            // Mark records for deletion (set id to empty)
            for idx in indices {
                if idx < records.len() {
                    records[idx].id = String::new(); // Mark as deleted
                }
            }
        }
        
        // Compact records (remove deleted)
        records.retain(|r| !r.id.is_empty());
        
        // Rebuild index
        index.clear();
        for (idx, record) in records.iter().enumerate() {
            index.entry(record.file_path.clone())
                .or_insert_with(Vec::new)
                .push(idx);
        }
        
        Ok(())
    }
    
    /// Clear all records
    pub async fn clear_all(&self) -> Result<()> {
        let mut records = self.records.write().await;
        let mut index = self.index.write().await;
        
        records.clear();
        index.clear();
        
        Ok(())
    }
    
    /// Get total count of records
    pub async fn count(&self) -> Result<usize> {
        let records = self.records.read().await;
        Ok(records.len())
    }
}

/// Normalize embedding vector (L2 normalization)
fn normalize_embedding(embedding: &[f32]) -> Vec<f32> {
    let norm = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        embedding.iter().map(|x| x / norm).collect()
    } else {
        embedding.to_vec()
    }
}

/// Calculate cosine similarity between two normalized embeddings
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }
    
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_lightweight_storage() {
        let storage = LightweightStorage::new();
        
        // Create test records
        let records = vec![
            LightweightRecord {
                id: "1".to_string(),
                file_path: "test.rs".to_string(),
                chunk_index: 0,
                content: "test content".to_string(),
                embedding: vec![1.0, 0.0, 0.0],
                start_line: 1,
                end_line: 10,
            },
            LightweightRecord {
                id: "2".to_string(),
                file_path: "test.rs".to_string(),
                chunk_index: 1,
                content: "more content".to_string(),
                embedding: vec![0.0, 1.0, 0.0],
                start_line: 11,
                end_line: 20,
            },
        ];
        
        // Insert records
        storage.insert_batch(records).await.unwrap();
        
        // Test count
        assert_eq!(storage.count().await.unwrap(), 2);
        
        // Test search
        let query = vec![0.9, 0.1, 0.0];
        let results = storage.search_similar(query, 1).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "1");
        
        // Test delete
        storage.delete_by_file("test.rs").await.unwrap();
        assert_eq!(storage.count().await.unwrap(), 0);
    }
    
    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert_eq!(cosine_similarity(&a, &b), 1.0);
        
        let c = vec![0.0, 1.0, 0.0];
        assert_eq!(cosine_similarity(&a, &c), 0.0);
    }
}