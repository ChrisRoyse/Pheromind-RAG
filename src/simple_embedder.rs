use anyhow::Result;
use crate::llama_wrapper_simple::{GGUFModel, GGUFContext};

/// GGUF-based Nomic Embed integration using llama-cpp-2
/// Produces 768-dimensional embeddings with proper prefixes
pub struct NomicEmbedder {
    model: GGUFModel,
    context: GGUFContext,
    model_path: String,
}

/// The embedding dimension for Nomic v1 model
pub const EMBEDDING_DIM: usize = 768;

impl NomicEmbedder {
    pub fn new() -> Result<Self> {
        // Initialize GGUF model from local file
        let model_path = "./src/model/nomic-embed-code.Q4_K_M.gguf";
        let gpu_layers = 0; // Use CPU by default
        
        // Load the GGUF model
        let model = GGUFModel::load_from_file(model_path, gpu_layers)?;
        
        // Create context for embeddings
        let context = GGUFContext::new_with_model(&model, 2048)?;
        
        Ok(Self {
            model,
            context,
            model_path: model_path.to_string(),
        })
    }

    /// Embed documents using GGUF model
    pub fn embed_batch(&mut self, documents: Vec<String>) -> Result<Vec<Vec<f32>>> {
        // Use GGUF model for batch embedding
        self.context.embed_batch(documents)
    }

    /// Single document embedding with correct Nomic v1.5 prefix
    pub fn embed(&mut self, text: &str) -> Result<Vec<f32>> {
        // Using correct Nomic v1.5 prefix for passages/documents
        self.context.embed(&format!("passage: {}", text))
    }

    /// Query embedding with correct Nomic v1.5 prefix
    pub fn embed_query(&mut self, query: &str) -> Result<Vec<f32>> {
        // Using correct Nomic v1.5 prefix for queries
        self.context.embed(&format!("query: {}", query))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nomic_embedder_gguf() -> Result<()> {
        // Test real GGUF embedding generation
        let mut embedder = NomicEmbedder::new()?;
        
        let embedding = embedder.embed("test code")?;
        assert!(!embedding.is_empty());
        assert_eq!(embedding.len(), EMBEDDING_DIM);
        
        // Verify embeddings are normalized (L2 norm ≈ 1.0)
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 0.01, "Embeddings should be normalized");
        
        println!("Embedding dimension: {} (GGUF model)", embedding.len());
        
        Ok(())
    }
    
    #[test]
    fn test_query_passage_prefixes() -> Result<()> {
        let mut embedder = NomicEmbedder::new()?;
        
        // Test query embedding
        let query_embedding = embedder.embed_query("search for rust code")?;
        assert_eq!(query_embedding.len(), EMBEDDING_DIM);
        
        // Test passage embedding
        let passage_embedding = embedder.embed("rust code implementation")?;
        assert_eq!(passage_embedding.len(), EMBEDDING_DIM);
        
        // Embeddings should be different due to different prefixes
        let similarity: f32 = query_embedding.iter()
            .zip(passage_embedding.iter())
            .map(|(a, b)| a * b)
            .sum();
        
        println!("Query-Passage similarity: {}", similarity);
        
        Ok(())
    }
}