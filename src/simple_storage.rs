use anyhow::Result;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Simple in-memory vector storage for CPU-only systems
/// Replaces LanceDB to avoid arrow dependency conflicts
#[derive(Clone)]
pub struct VectorStorage {
    documents: Vec<Document>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Document {
    id: usize,
    content: String,
    file_path: String,
    embedding: Vec<f32>,
}

impl VectorStorage {
    pub fn new(_db_path: &str) -> Result<Self> {
        Ok(Self {
            documents: Vec::new(),
        })
    }

    /// Store embeddings with metadata
    pub fn store(&mut self, 
                contents: Vec<String>, 
                embeddings: Vec<Vec<f32>>, 
                file_paths: Vec<String>) -> Result<()> {
        
        let start_id = self.documents.len();
        
        for (i, ((content, embedding), file_path)) in contents.into_iter()
            .zip(embeddings.into_iter())
            .zip(file_paths.into_iter())
            .enumerate() {
            
            let document = Document {
                id: start_id + i,
                content,
                file_path,
                embedding,
            };
            
            self.documents.push(document);
        }
        
        Ok(())
    }

    /// Search using simple cosine similarity
    pub fn search(&self, query_embedding: Vec<f32>, limit: usize) -> Result<Vec<SearchResult>> {
        let mut results: Vec<(usize, f32)> = Vec::new();
        
        for (idx, doc) in self.documents.iter().enumerate() {
            let similarity = cosine_similarity(&query_embedding, &doc.embedding);
            results.push((idx, similarity));
        }
        
        // Sort by similarity (descending)
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        // Take top results and convert to SearchResult
        let search_results = results.into_iter()
            .take(limit)
            .map(|(idx, similarity)| {
                let doc = &self.documents[idx];
                SearchResult {
                    content: doc.content.clone(),
                    file_path: doc.file_path.clone(),
                    score: similarity,
                }
            })
            .collect();
            
        Ok(search_results)
    }

    /// Clear all data
    pub fn clear(&mut self) -> Result<()> {
        self.documents.clear();
        Ok(())
    }
    
    /// Get number of stored documents
    pub fn len(&self) -> usize {
        self.documents.len()
    }
    
    /// Check if storage is empty
    pub fn is_empty(&self) -> bool {
        self.documents.is_empty()
    }
}

#[derive(Debug)]
pub struct SearchResult {
    pub content: String,
    pub file_path: String,
    pub score: f32,
}

/// Calculate cosine similarity between two vectors
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_storage() -> Result<()> {
        let mut storage = VectorStorage::new("test.db")?;
        
        // Test data
        let contents = vec!["Hello world".to_string()];
        let embeddings = vec![vec![0.1; 768]]; // 768-dim embedding
        let file_paths = vec!["test.rs".to_string()];
        
        storage.store(contents, embeddings, file_paths)?;
        
        let results = storage.search(vec![0.1; 768], 5)?;
        assert!(!results.is_empty());
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].content, "Hello world");
        
        Ok(())
    }
    
    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        let similarity = cosine_similarity(&a, &b);
        assert!((similarity - 1.0).abs() < 1e-6);
        
        let c = vec![0.0, 1.0, 0.0];
        let similarity2 = cosine_similarity(&a, &c);
        assert!(similarity2.abs() < 1e-6);
    }
}