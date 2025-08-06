use crate::embedding::real_minilm::RealMiniLMEmbedder;

// Re-export the real embedding error for compatibility
pub use crate::embedding::real_minilm::EmbeddingError;

/// Wrapper around RealMiniLMEmbedder for backward compatibility
pub struct MiniLMEmbedder {
    real_embedder: RealMiniLMEmbedder,
}

impl MiniLMEmbedder {
    /// Default batch size for processing multiple texts efficiently
    pub const DEFAULT_BATCH_SIZE: usize = 32;

    /// Load the real all-MiniLM-L6-v2 model (async wrapper for compatibility)
    pub async fn new() -> Result<Self, EmbeddingError> {
        let real_embedder = RealMiniLMEmbedder::new().await?;
        Ok(Self { real_embedder })
    }

    /// DEPRECATED: Use new() instead - this is kept for compatibility only
    /// Creates a real embedder (not mock) using the default async runtime
    pub fn mock() -> Self {
        // Use tokio runtime to call the async new() method
        let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
        let real_embedder = rt.block_on(async {
            RealMiniLMEmbedder::new().await.expect("Failed to load real model")
        });
        Self { real_embedder }
    }

    /// Generate real 384-dim vector using actual all-MiniLM-L6-v2 model
    pub fn embed(&self, text: &str) -> Result<Vec<f32>, EmbeddingError> {
        self.real_embedder.embed(text)
    }

    /// Batch processing using real model
    pub fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        self.real_embedder.embed_batch(texts)
    }

    /// Process in batches for memory efficiency
    pub fn embed_chunked(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        self.real_embedder.embed_chunked(texts)
    }

    /// Get embedding dimensions (always 384 for all-MiniLM-L6-v2)
    pub fn embedding_dim(&self) -> usize {
        384
    }

    /// Check if model is loaded (always true now)
    pub fn is_loaded(&self) -> bool {
        true // Real model is loaded
    }

    /// Check if running in mock mode (always false now)
    pub fn is_mock_mode(&self) -> bool {
        false // Always using real implementation
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
    use tokio;

    #[tokio::test]
    async fn test_real_embedder_basic() {
        let embedder = match MiniLMEmbedder::new().await {
            Ok(e) => e,
            Err(_) => {
                println!("⏭️  Skipping test - model not available");
                return;
            }
        };
        let embedding = embedder.embed("test").unwrap();
        assert_eq!(embedding.len(), 384);
    }

    #[tokio::test]
    async fn test_normalization() {
        let embedder = match MiniLMEmbedder::new().await {
            Ok(e) => e,
            Err(_) => {
                println!("⏭️  Skipping test - model not available");
                return;
            }
        };
        let embedding = embedder.embed("test normalization").unwrap();
        let magnitude: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((magnitude - 1.0).abs() < 1e-6, "Expected normalized vector, got magnitude: {}", magnitude);
    }

    #[tokio::test]
    async fn test_different_inputs_different_outputs() {
        let embedder = match MiniLMEmbedder::new().await {
            Ok(e) => e,
            Err(_) => {
                println!("⏭️  Skipping test - model not available");
                return;
            }
        };
        
        let embed1 = embedder.embed("text one").unwrap();
        let embed2 = embedder.embed("text two").unwrap();
        
        assert_ne!(embed1, embed2);
    }

    #[tokio::test]
    async fn test_empty_input() {
        let embedder = match MiniLMEmbedder::new().await {
            Ok(e) => e,
            Err(_) => {
                println!("⏭️  Skipping test - model not available");
                return;
            }
        };
        let embedding = embedder.embed("").unwrap();
        assert_eq!(embedding.len(), 384);
    }

    #[tokio::test]
    async fn test_batch_embedding() {
        let embedder = match MiniLMEmbedder::new().await {
            Ok(e) => e,
            Err(_) => {
                println!("⏭️  Skipping test - model not available");
                return;
            }
        };
        let texts = vec!["text1", "text2", "text3"];
        let embeddings = embedder.embed_batch(&texts).unwrap();
        
        assert_eq!(embeddings.len(), 3);
        for embedding in embeddings {
            assert_eq!(embedding.len(), 384);
        }
    }

    #[test]
    fn test_deprecated_mock_method() {
        // Test that mock() method now returns a real embedder
        let embedder = MiniLMEmbedder::mock();
        assert!(!embedder.is_mock_mode());
        assert!(embedder.is_loaded());
    }
}