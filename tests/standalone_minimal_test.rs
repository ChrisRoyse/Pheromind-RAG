// Standalone test for minimal embedder without tempfile dependency
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub struct TestMinimalEmbedder {
    dimension: usize,
}

impl TestMinimalEmbedder {
    pub fn new() -> Self {
        Self { dimension: 768 }
    }
    
    pub fn dimension(&self) -> usize {
        self.dimension
    }
    
    pub fn embed(&self, text: &str) -> Vec<f32> {
        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        let base_hash = hasher.finish();
        
        let mut embedding = Vec::with_capacity(self.dimension);
        for i in 0..self.dimension {
            let seed1 = base_hash.wrapping_mul(i as u64 + 1);
            let seed2 = seed1.rotate_left(i as u32 % 64);
            let seed = seed1 ^ seed2;
            
            let normalized = (seed as f64) / (u64::MAX as f64);
            let value = (normalized * 2.0 - 1.0) as f32;
            
            embedding.push(value);
        }
        
        // Normalize to unit length
        let norm = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            embedding.iter_mut().for_each(|x| *x /= norm);
        }
        
        embedding
    }
}

#[cfg(test)]
mod tests {
    use super::TestMinimalEmbedder;

    #[test]
    fn test_deterministic_embeddings() {
        let embedder = TestMinimalEmbedder::new();
        let text = "hello world";
        
        let embedding1 = embedder.embed(text);
        let embedding2 = embedder.embed(text);
        
        assert_eq!(embedding1, embedding2, "Embeddings should be deterministic");
        assert_eq!(embedding1.len(), 768, "Should have 768 dimensions");
    }

    #[test]
    fn test_different_texts_different_embeddings() {
        let embedder = TestMinimalEmbedder::new();
        
        let embedding1 = embedder.embed("hello world");
        let embedding2 = embedder.embed("goodbye world");
        
        assert_ne!(embedding1, embedding2, "Different texts should produce different embeddings");
    }

    #[test]
    fn test_normalization() {
        let embedder = TestMinimalEmbedder::new();
        let embedding = embedder.embed("test text");
        
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 1e-6, "Embedding should be normalized to unit length, got norm: {}", norm);
    }

    #[test]
    fn test_empty_string() {
        let embedder = TestMinimalEmbedder::new();
        let embedding = embedder.embed("");
        
        assert_eq!(embedding.len(), 768);
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 1e-6, "Empty string embedding should also be normalized");
    }

    #[test]
    fn test_similarity_correlation() {
        let embedder = TestMinimalEmbedder::new();
        
        let similar1 = embedder.embed("the quick brown fox");
        let similar2 = embedder.embed("the quick brown fox");
        let different = embedder.embed("completely different text");
        
        // Calculate cosine similarity
        fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
            let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
            dot // Since vectors are normalized, dot product = cosine similarity
        }
        
        let self_similarity = cosine_similarity(&similar1, &similar2);
        let cross_similarity = cosine_similarity(&similar1, &different);
        
        assert_eq!(self_similarity, 1.0, "Identical text should have cosine similarity of 1.0");
        assert!(cross_similarity < 1.0, "Different texts should have cosine similarity less than 1.0");
    }

    #[test]
    fn test_no_crashes_various_inputs() {
        let embedder = TestMinimalEmbedder::new();
        
        let test_cases = vec![
            "",
            "a",
            "hello world",
            "The quick brown fox jumps over the lazy dog",
            "123456789",
            "!@#$%^&*()",
            "Unicode: 你好世界 🌍",
            "Very long text ".repeat(100),
        ];
        
        for input in test_cases {
            let embedding = embedder.embed(&input);
            assert_eq!(embedding.len(), 768);
            
            let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
            assert!((norm - 1.0).abs() < 1e-6, "Failed normalization for input: {}", input);
        }
    }
}