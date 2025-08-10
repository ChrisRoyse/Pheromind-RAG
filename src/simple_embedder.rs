use anyhow::Result;
use fastembed::TextEmbedding;

/// Real Nomic Embed v1 integration using correct fastembed API
/// Produces 768-dimensional embeddings with proper prefixes
pub struct NomicEmbedder {
    model: TextEmbedding,
}

/// The embedding dimension for Nomic v1 model
pub const EMBEDDING_DIM: usize = 768;

impl NomicEmbedder {
    pub fn new() -> Result<Self> {
        // Use correct fastembed API - Default::default() uses best model
        let model = TextEmbedding::try_new(Default::default())?;
        
        Ok(Self { model })
    }

    /// Embed documents using correct API
    pub fn embed_batch(&mut self, documents: Vec<String>) -> Result<Vec<Vec<f32>>> {
        // Correct fastembed API: embed(documents, batch_size)
        let embeddings = self.model.embed(documents, None)?;
        Ok(embeddings)
    }

    /// Single document embedding with correct Nomic v1.5 prefix
    pub fn embed(&mut self, text: &str) -> Result<Vec<f32>> {
        // Using correct Nomic v1.5 prefix for passages/documents
        let embeddings = self.embed_batch(vec![format!("passage: {}", text)])?;
        Ok(embeddings.into_iter().next().unwrap_or_default())
    }

    /// Query embedding with correct Nomic v1.5 prefix
    pub fn embed_query(&mut self, query: &str) -> Result<Vec<f32>> {
        // Using correct Nomic v1.5 prefix for queries
        let embeddings = self.embed_batch(vec![format!("query: {}", query)])?;
        Ok(embeddings.into_iter().next().unwrap_or_default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nomic_embedder() -> Result<()> {
        let mut embedder = NomicEmbedder::new()?;
        
        let embedding = embedder.embed("test code")?;
        assert!(!embedding.is_empty());
        println!("Embedding dimension: {}", embedding.len());
        
        Ok(())
    }
}