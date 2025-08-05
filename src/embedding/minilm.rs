use std::collections::HashMap;
use std::hash::{Hash, Hasher};

#[derive(Debug)]
pub enum EmbeddingError {
    ModelLoadError(String),
    InferenceError(String),
    InvalidInput(String),
}

impl std::fmt::Display for EmbeddingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EmbeddingError::ModelLoadError(msg) => write!(f, "Model load error: {}", msg),
            EmbeddingError::InferenceError(msg) => write!(f, "Inference error: {}", msg),
            EmbeddingError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
        }
    }
}

impl std::error::Error for EmbeddingError {}

pub struct MiniLMEmbedder {
    // Real model fields would go here when implemented
    cache: HashMap<String, Vec<f32>>,
}

impl MiniLMEmbedder {
    /// Default batch size for processing multiple texts efficiently
    pub const DEFAULT_BATCH_SIZE: usize = 32;

    /// Try to load real model
    pub fn new() -> Result<Self, EmbeddingError> {
        // Real model implementation would go here
        // For now, return error since we haven't implemented the actual model yet
        Err(EmbeddingError::ModelLoadError("Real all-MiniLM-L6-v2 model not yet implemented. Use alternative approach for now.".to_string()))
    }

    /// Single text -> 384-dim vector
    /// NOTE: This would use the real model when available
    pub fn embed(&self, text: &str) -> Result<Vec<f32>, EmbeddingError> {
        Err(EmbeddingError::ModelLoadError("No model loaded. Real embedding implementation needed.".to_string()))
    }

    /// Batch processing for efficiency
    pub fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        // For now, just map over embed() - real batch inference will be added later
        texts.iter().map(|text| self.embed(text)).collect()
    }

    /// Normalize vector to unit length (L2 normalization)
    fn normalize(&self, mut embedding: Vec<f32>) -> Vec<f32> {
        let magnitude: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        
        if magnitude > 0.0 {
            for value in &mut embedding {
                *value /= magnitude;
            }
        }
        
        embedding
    }

    /// Process in batches for memory efficiency
    pub fn embed_chunked(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        let mut results = Vec::with_capacity(texts.len());
        
        for chunk in texts.chunks(Self::DEFAULT_BATCH_SIZE) {
            let batch_results = self.embed_batch(chunk)?;
            results.extend(batch_results);
        }
        
        Ok(results)
    }

    /// Get embedding dimensions (always 384 for all-MiniLM-L6-v2)
    pub fn embedding_dim(&self) -> usize {
        384
    }

    /// Check if model is loaded (would return true when real model is implemented)
    pub fn is_loaded(&self) -> bool {
        false // No model loaded yet
    }
}

// Integration helpers for working with the chunking system
impl MiniLMEmbedder {
    pub fn embed_chunk(&self, chunk: &crate::chunking::Chunk) -> Result<Vec<f32>, EmbeddingError> {
        self.embed(&chunk.content)
    }

    pub fn embed_context(&self, context: &crate::chunking::ChunkContext) -> Result<Vec<f32>, EmbeddingError> {
        let full_content = context.get_full_content();
        self.embed(&full_content)
    }
}

// Thread-safe implementation
unsafe impl Send for MiniLMEmbedder {}
unsafe impl Sync for MiniLMEmbedder {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_embedder_basic() {
        let embedder = MiniLMEmbedder::mock();
        let embedding = embedder.embed("test").unwrap();
        assert_eq!(embedding.len(), 384);
    }

    #[test]
    fn test_normalization() {
        let embedder = MiniLMEmbedder::mock();
        let embedding = embedder.embed("test normalization").unwrap();
        let magnitude: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((magnitude - 1.0).abs() < 1e-6, "Expected normalized vector, got magnitude: {}", magnitude);
    }

    #[test]
    fn test_deterministic_behavior() {
        let embedder = MiniLMEmbedder::mock();
        let text = "deterministic test";
        
        let embed1 = embedder.embed(text).unwrap();
        let embed2 = embedder.embed(text).unwrap();
        
        assert_eq!(embed1, embed2);
    }

    #[test]
    fn test_different_inputs_different_outputs() {
        let embedder = MiniLMEmbedder::mock();
        
        let embed1 = embedder.embed("text one").unwrap();
        let embed2 = embedder.embed("text two").unwrap();
        
        assert_ne!(embed1, embed2);
    }

    #[test]
    fn test_empty_input() {
        let embedder = MiniLMEmbedder::mock();
        let embedding = embedder.embed("").unwrap();
        assert_eq!(embedding.len(), 384);
    }

    #[test]
    fn test_batch_embedding() {
        let embedder = MiniLMEmbedder::mock();
        let texts = vec!["text1", "text2", "text3"];
        let embeddings = embedder.embed_batch(&texts).unwrap();
        
        assert_eq!(embeddings.len(), 3);
        for embedding in embeddings {
            assert_eq!(embedding.len(), 384);
        }
    }
}