// Safe VectorStorage Implementation - Phase 1: Foundation & Safety
// This module provides thread-safe vector storage without unsafe code

use std::sync::Arc;
use parking_lot::RwLock;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use crate::error::{EmbedError, Result};

/// Thread-safe vector storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub max_vectors: usize,
    pub dimension: usize,
    pub cache_size: usize,
    pub enable_compression: bool,
}

// PRINCIPLE 0 ENFORCEMENT: No Default implementation for StorageConfig
// All storage configuration MUST be explicit - no fallback values allowed

impl StorageConfig {
    /// TEST-ONLY: Create a test storage configuration with explicit values
    /// This should NEVER be used in production code
    #[cfg(test)]
    pub fn new_test_config() -> Self {
        Self {
            max_vectors: 1_000_000,
            dimension: 768,  // Nomic embedding dimension
            cache_size: 10_000,
            enable_compression: false,
        }
    }
}

/// Metadata associated with each vector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorMetadata {
    pub id: String,
    pub source: Option<String>,
    pub timestamp: u64,
    pub tags: Vec<String>,
    pub properties: HashMap<String, String>,
}

/// Thread-safe vector storage implementation
/// Uses Arc<RwLock> for safe concurrent access - no unsafe code needed
#[derive(Clone)]
pub struct VectorStorage {
    /// Vector data storage with concurrent read/write access
    vectors: Arc<RwLock<Vec<Vec<f32>>>>,
    
    /// Metadata storage with concurrent access
    metadata: Arc<RwLock<HashMap<String, VectorMetadata>>>,
    
    /// Index mapping IDs to vector positions
    id_index: Arc<RwLock<HashMap<String, usize>>>,
    
    /// Immutable configuration
    config: Arc<StorageConfig>,
    
    /// Statistics tracking
    stats: Arc<RwLock<StorageStats>>,
}

/// Storage statistics for monitoring
#[derive(Debug, Clone)]
pub struct StorageStats {
    total_vectors: usize,
    total_searches: usize,
    #[allow(dead_code)]
    cache_hits: usize,
    #[allow(dead_code)]
    cache_misses: usize,
    #[allow(dead_code)]
    last_cleanup: u64,
}

impl StorageStats {
    pub fn new() -> Self {
        Self {
            total_vectors: 0,
            total_searches: 0,
            cache_hits: 0,
            cache_misses: 0,
            last_cleanup: 0,
        }
    }
}

impl VectorStorage {
    /// Create a new vector storage instance
    pub fn new(config: StorageConfig) -> Result<Self> {
        // Validate configuration
        if config.dimension == 0 {
            return Err(EmbedError::Validation {
                field: "dimension".to_string(),
                reason: "Dimension must be greater than 0".to_string(),
                value: Some(config.dimension.to_string()),
            });
        }
        
        if config.max_vectors == 0 {
            return Err(EmbedError::Validation {
                field: "max_vectors".to_string(),
                reason: "Maximum vectors must be greater than 0".to_string(),
                value: Some(config.max_vectors.to_string()),
            });
        }
        
        Ok(Self {
            vectors: Arc::new(RwLock::new(Vec::with_capacity(
                config.max_vectors.min(10_000)  // Reasonable initial capacity
            ))),
            metadata: Arc::new(RwLock::new(HashMap::new())),
            id_index: Arc::new(RwLock::new(HashMap::new())),
            config: Arc::new(config),
            stats: Arc::new(RwLock::new(StorageStats::new())),
        })
    }
    
    /// Add a vector to storage
    pub async fn add_vector(
        &self,
        id: String,
        vector: Vec<f32>,
        metadata: VectorMetadata,
    ) -> Result<()> {
        // Validate vector dimension
        if vector.len() != self.config.dimension {
            return Err(EmbedError::Validation {
                field: "vector".to_string(),
                reason: format!(
                    "Vector dimension {} does not match configured dimension {}",
                    vector.len(),
                    self.config.dimension
                ),
                value: Some(vector.len().to_string()),
            });
        }
        
        // Check if ID already exists
        {
            let id_index = self.id_index.read();
            if id_index.contains_key(&id) {
                return Err(EmbedError::AlreadyExists {
                    resource: "vector".to_string(),
                    id: Some(id),
                });
            }
        }
        
        // Check capacity
        {
            let vectors = self.vectors.read();
            if vectors.len() >= self.config.max_vectors {
                return Err(EmbedError::ResourceExhausted {
                    resource: "vector storage".to_string(),
                    limit: Some(self.config.max_vectors),
                    current: Some(vectors.len()),
                });
            }
        }
        
        // Add vector and metadata
        {
            let mut vectors = self.vectors.write();
            let mut metadata_store = self.metadata.write();
            let mut id_index = self.id_index.write();
            let mut stats = self.stats.write();
            
            let index = vectors.len();
            vectors.push(vector);
            metadata_store.insert(id.clone(), metadata);
            id_index.insert(id, index);
            stats.total_vectors += 1;
        }
        
        Ok(())
    }
    
    /// Get a vector by ID
    pub async fn get_vector(&self, id: &str) -> Result<(Vec<f32>, VectorMetadata)> {
        let id_index = self.id_index.read();
        let index = id_index.get(id)
            .ok_or_else(|| EmbedError::NotFound {
                resource: "vector".to_string(),
                id: Some(id.to_string()),
            })?;
        
        let vectors = self.vectors.read();
        let vector = vectors.get(*index)
            .ok_or_else(|| EmbedError::Internal {
                message: format!("Index {} out of bounds", index),
                backtrace: None,
            })?
            .clone();
        
        let metadata_store = self.metadata.read();
        let metadata = metadata_store.get(id)
            .ok_or_else(|| EmbedError::Internal {
                message: format!("Metadata missing for ID {}", id),
                backtrace: None,
            })?
            .clone();
        
        Ok((vector, metadata))
    }
    
    /// Delete a vector by ID
    pub async fn delete_vector(&self, id: &str) -> Result<()> {
        let mut id_index = self.id_index.write();
        let index = id_index.remove(id)
            .ok_or_else(|| EmbedError::NotFound {
                resource: "vector".to_string(),
                id: Some(id.to_string()),
            })?;
        
        let mut vectors = self.vectors.write();
        let mut metadata_store = self.metadata.write();
        let mut stats = self.stats.write();
        
        // Mark as deleted (don't actually remove to preserve indices)
        vectors[index].clear();
        metadata_store.remove(id);
        stats.total_vectors = stats.total_vectors.saturating_sub(1);
        
        Ok(())
    }
    
    /// Search for similar vectors (basic linear search for now)
    pub async fn search(
        &self,
        query: &[f32],
        top_k: usize,
    ) -> Result<Vec<(String, f32)>> {
        if query.len() != self.config.dimension {
            return Err(EmbedError::Validation {
                field: "query".to_string(),
                reason: format!(
                    "Query dimension {} does not match configured dimension {}",
                    query.len(),
                    self.config.dimension
                ),
                value: Some(query.len().to_string()),
            });
        }
        
        let vectors = self.vectors.read();
        let id_index = self.id_index.read();
        let mut stats = self.stats.write();
        
        stats.total_searches += 1;
        
        // Compute similarities
        let mut similarities: Vec<(String, f32)> = Vec::new();
        
        for (id, &index) in id_index.iter() {
            if let Some(vector) = vectors.get(index) {
                if !vector.is_empty() {  // Skip deleted vectors
                    let similarity = cosine_similarity(query, vector);
                    similarities.push((id.clone(), similarity));
                }
            }
        }
        
        // Validate all similarity scores are finite before sorting - PRINCIPLE 0: No NaN fallbacks
        for (idx, (_, similarity)) in similarities.iter().enumerate() {
            if !similarity.is_finite() {
                return Err(EmbedError::Internal {
                    message: format!(
                        "Similarity calculation produced invalid result (NaN or infinite) at index {}. Score: {}. This indicates corrupted similarity computation and cannot be recovered from.", 
                        idx, similarity
                    ),
                    backtrace: None,
                });
            }
        }
        
        // Sort by similarity (descending) - safe after validation
        similarities.sort_by(|a, b| {
            b.1.partial_cmp(&a.1).unwrap() // Safe after validation
        });
        
        // Return top-k results
        Ok(similarities.into_iter().take(top_k).collect())
    }
    
    /// Get storage statistics
    pub async fn get_stats(&self) -> StorageStats {
        self.stats.read().clone()
    }
    
    /// Clear all vectors from storage
    pub async fn clear(&self) -> Result<()> {
        let mut vectors = self.vectors.write();
        let mut metadata = self.metadata.write();
        let mut id_index = self.id_index.write();
        let mut stats = self.stats.write();
        
        vectors.clear();
        metadata.clear();
        id_index.clear();
        *stats = StorageStats::new();
        
        Ok(())
    }
    
    /// Get the number of stored vectors
    pub async fn len(&self) -> usize {
        self.stats.read().total_vectors
    }
    
    /// Check if storage is empty
    pub async fn is_empty(&self) -> bool {
        self.stats.read().total_vectors == 0
    }
}

/// Compute cosine similarity between two vectors
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }
    
    let mut dot_product = 0.0;
    let mut norm_a = 0.0;
    let mut norm_b = 0.0;
    
    for i in 0..a.len() {
        dot_product += a[i] * b[i];
        norm_a += a[i] * a[i];
        norm_b += b[i] * b[i];
    }
    
    let denominator = (norm_a * norm_b).sqrt();
    if denominator == 0.0 {
        0.0
    } else {
        dot_product / denominator
    }
}

// VectorStorage automatically implements Send + Sync because:
// - Arc<RwLock<T>> implements Send + Sync when T: Send + Sync
// - Vec<f32>, HashMap, and our other types all implement Send + Sync
// No unsafe implementation needed!

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_vector_storage_thread_safety() {
        let storage = Arc::new(
            VectorStorage::new(StorageConfig::new_test_config())
                .expect("Failed to create storage")
        );
        
        let mut handles = vec![];
        
        // Spawn 100 concurrent operations
        for i in 0..100 {
            let storage_clone = storage.clone();
            handles.push(tokio::spawn(async move {
                let vector = vec![i as f32; 768];
                let metadata = VectorMetadata {
                    id: format!("vec_{}", i),
                    source: Some("test".to_string()),
                    timestamp: i as u64,
                    tags: vec![],
                    properties: HashMap::new(),
                };
                storage_clone.add_vector(
                    format!("vec_{}", i),
                    vector,
                    metadata
                ).await
            }));
        }
        
        // Wait for all operations to complete
        for handle in handles {
            assert!(handle.await.unwrap().is_ok());
        }
        
        // Verify final state
        assert_eq!(storage.len().await, 100);
    }
    
    #[tokio::test]
    async fn test_vector_operations() {
        let storage = VectorStorage::new(StorageConfig::new_test_config())
            .expect("Failed to create storage");
        
        // Add a vector
        let id = "test_vec".to_string();
        let vector = vec![1.0; 768];
        let metadata = VectorMetadata {
            id: id.clone(),
            source: Some("test".to_string()),
            timestamp: 0,
            tags: vec!["test".to_string()],
            properties: HashMap::new(),
        };
        
        storage.add_vector(id.clone(), vector.clone(), metadata.clone())
            .await
            .expect("Failed to add vector");
        
        // Get the vector
        let (retrieved_vec, retrieved_meta) = storage.get_vector(&id)
            .await
            .expect("Failed to get vector");
        
        assert_eq!(retrieved_vec, vector);
        assert_eq!(retrieved_meta.id, id);
        
        // Search for similar vectors
        let results = storage.search(&vector, 10)
            .await
            .expect("Failed to search");
        
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, id);
        assert!((results[0].1 - 1.0).abs() < 0.001);  // Should be perfect similarity
        
        // Delete the vector
        storage.delete_vector(&id)
            .await
            .expect("Failed to delete vector");
        
        // Verify it's gone
        assert!(storage.get_vector(&id).await.is_err());
    }
    
    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 0.001);
        
        let c = vec![0.0, 1.0, 0.0];
        assert!((cosine_similarity(&a, &c) - 0.0).abs() < 0.001);
        
        let d = vec![1.0, 1.0, 0.0];
        let expected = 1.0 / 2.0_f32.sqrt();
        assert!((cosine_similarity(&a, &d) - expected).abs() < 0.001);
    }
}