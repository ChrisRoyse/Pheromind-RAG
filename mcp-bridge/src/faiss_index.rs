use anyhow::{Result, Context};
use std::collections::HashMap;

/// Statistics about the FAISS index
#[derive(Debug, Clone)]
pub struct FaissStats {
    pub index_size: usize,
    pub total_vectors: usize,
    pub dimension: usize,
    pub index_type: String,
}

/// FAISS index manager for similarity search
pub struct FaissIndexManager {
    // Note: In a real implementation, this would use actual FAISS bindings
    // For now, we'll create a simplified in-memory implementation
    vectors: Vec<Vec<f32>>,
    ids: Vec<i64>,
    dimension: Option<usize>,
    initialized: bool,
}

impl FaissIndexManager {
    pub fn new() -> Self {
        Self {
            vectors: Vec::new(),
            ids: Vec::new(),
            dimension: None,
            initialized: false,
        }
    }

    /// Initialize the FAISS index
    pub fn initialize(&mut self) -> Result<()> {
        // In a real implementation, this would initialize FAISS library
        self.initialized = true;
        log::info!("FAISS index manager initialized");
        Ok(())
    }

    /// Add vectors to the index
    pub fn add_vectors(&mut self, embeddings: &[Vec<f32>], ids: &[i64]) -> Result<()> {
        if !self.initialized {
            return Err(anyhow::anyhow!("FAISS manager not initialized"));
        }

        if embeddings.len() != ids.len() {
            return Err(anyhow::anyhow!("Embeddings and IDs must have the same length"));
        }

        if embeddings.is_empty() {
            return Ok(());
        }

        // Set dimension from first vector
        let first_dim = embeddings[0].len();
        match self.dimension {
            None => self.dimension = Some(first_dim),
            Some(dim) if dim != first_dim => {
                return Err(anyhow::anyhow!(
                    "Dimension mismatch: expected {}, got {}", 
                    dim, first_dim
                ));
            }
            _ => {}
        }

        // Validate all vectors have same dimension
        for (i, embedding) in embeddings.iter().enumerate() {
            if embedding.len() != first_dim {
                return Err(anyhow::anyhow!(
                    "Vector {} has dimension {}, expected {}", 
                    i, embedding.len(), first_dim
                ));
            }
        }

        // Add vectors and IDs
        for (embedding, &id) in embeddings.iter().zip(ids.iter()) {
            self.vectors.push(embedding.clone());
            self.ids.push(id);
        }

        log::debug!("Added {} vectors to index", embeddings.len());
        Ok(())
    }

    /// Search for similar vectors
    pub fn search(&self, query_vector: &[f32], k: usize, threshold: f32) -> Result<Vec<(i64, f32)>> {
        if !self.initialized {
            return Err(anyhow::anyhow!("FAISS manager not initialized"));
        }

        if self.vectors.is_empty() {
            return Ok(Vec::new());
        }

        let dimension = self.dimension.context("No vectors in index")?;
        if query_vector.len() != dimension {
            return Err(anyhow::anyhow!(
                "Query vector dimension {} doesn't match index dimension {}", 
                query_vector.len(), dimension
            ));
        }

        // Compute similarities with all vectors
        let mut similarities: Vec<(i64, f32)> = self.vectors
            .iter()
            .zip(self.ids.iter())
            .map(|(vector, &id)| {
                let similarity = cosine_similarity(query_vector, vector);
                (id, similarity)
            })
            .filter(|(_, similarity)| *similarity >= threshold)
            .collect();

        // Sort by similarity (descending)
        similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Take top k results
        similarities.truncate(k);

        Ok(similarities)
    }

    /// Remove vectors by IDs
    pub fn remove_vectors(&mut self, ids_to_remove: &[i64]) -> Result<usize> {
        if !self.initialized {
            return Err(anyhow::anyhow!("FAISS manager not initialized"));
        }

        let initial_count = self.vectors.len();
        let ids_set: std::collections::HashSet<_> = ids_to_remove.iter().collect();

        // Remove vectors and IDs
        let mut i = 0;
        while i < self.vectors.len() {
            if ids_set.contains(&self.ids[i]) {
                self.vectors.remove(i);
                self.ids.remove(i);
            } else {
                i += 1;
            }
        }

        let removed_count = initial_count - self.vectors.len();
        log::debug!("Removed {} vectors from index", removed_count);

        Ok(removed_count)
    }

    /// Get index statistics
    pub fn get_stats(&self) -> Result<FaissStats> {
        Ok(FaissStats {
            index_size: self.vectors.len(),
            total_vectors: self.vectors.len(),
            dimension: self.dimension.unwrap_or(0),
            index_type: "InMemoryFlat".to_string(),
        })
    }

    /// Clear the index
    pub fn clear(&mut self) -> Result<()> {
        self.vectors.clear();
        self.ids.clear();
        self.dimension = None;
        log::debug!("Cleared FAISS index");
        Ok(())
    }

    /// Save index to file (placeholder implementation)
    pub fn save_index(&self, path: &str) -> Result<()> {
        // In a real implementation, this would save the FAISS index to disk
        log::info!("Index save requested to: {} (not implemented)", path);
        Ok(())
    }

    /// Load index from file (placeholder implementation)
    pub fn load_index(&mut self, path: &str) -> Result<()> {
        // In a real implementation, this would load a FAISS index from disk
        log::info!("Index load requested from: {} (not implemented)", path);
        Ok(())
    }

    /// Get current dimension
    pub fn get_dimension(&self) -> Option<usize> {
        self.dimension
    }

    /// Check if initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Cleanup resources
    pub fn cleanup(&mut self) -> Result<()> {
        self.clear()?;
        self.initialized = false;
        log::info!("FAISS index manager cleaned up");
        Ok(())
    }
}

/// Compute cosine similarity between two vectors
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }

    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot_product / (norm_a * norm_b)
    }
}

/// Compute Euclidean distance between two vectors
#[allow(dead_code)]
fn euclidean_distance(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return f32::INFINITY;
    }

    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x - y) * (x - y))
        .sum::<f32>()
        .sqrt()
}

impl Default for FaissIndexManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![0.0, 1.0, 0.0];
        assert_eq!(cosine_similarity(&a, &b), 0.0);

        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert_eq!(cosine_similarity(&a, &b), 1.0);
    }

    #[test]
    fn test_faiss_manager() {
        let mut manager = FaissIndexManager::new();
        manager.initialize().unwrap();

        let embeddings = vec![
            vec![1.0, 0.0, 0.0],
            vec![0.0, 1.0, 0.0],
            vec![0.0, 0.0, 1.0],
        ];
        let ids = vec![1, 2, 3];

        manager.add_vectors(&embeddings, &ids).unwrap();

        let query = vec![1.0, 0.0, 0.0];
        let results = manager.search(&query, 2, 0.0).unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].0, 1); // Should match the first vector exactly
        assert_eq!(results[0].1, 1.0);
    }
}