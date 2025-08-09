#[cfg(test)]
mod tests {
    use embed_search::embedding::MinimalEmbedder;

    #[test]
    fn test_minimal_embedder_basic() {
        let embedder = MinimalEmbedder::new();
        
        // Test basic functionality
        let text = "hello world";
        let embedding = embedder.embed(text);
        
        assert_eq!(embedding.len(), 768);
        assert_eq!(embedder.dimension(), 768);
    }

    #[test]
    fn test_deterministic_outputs() {
        let embedder = MinimalEmbedder::new();
        
        let text = "test text";
        let embedding1 = embedder.embed(text);
        let embedding2 = embedder.embed(text);
        
        // Should be exactly the same
        assert_eq!(embedding1, embedding2);
    }

    #[test]
    fn test_different_inputs_different_outputs() {
        let embedder = MinimalEmbedder::new();
        
        let text1 = "hello";
        let text2 = "world";
        
        let embedding1 = embedder.embed(text1);
        let embedding2 = embedder.embed(text2);
        
        // Should be different
        assert_ne!(embedding1, embedding2);
    }

    #[test]
    fn test_normalized_vectors() {
        let embedder = MinimalEmbedder::new();
        
        let text = "some text for normalization test";
        let embedding = embedder.embed(text);
        
        // Calculate L2 norm (should be close to 1.0)
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        
        // Allow small floating point error
        assert!((norm - 1.0).abs() < 0.001, "Norm was {}, expected ~1.0", norm);
    }

    #[test]
    fn test_empty_string() {
        let embedder = MinimalEmbedder::new();
        
        let embedding = embedder.embed("");
        
        assert_eq!(embedding.len(), 768);
        
        // Empty string should still produce normalized vector
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 0.001, "Empty string norm was {}", norm);
    }

    #[test]
    fn test_various_inputs() {
        let embedder = MinimalEmbedder::new();
        
        let inputs = vec![
            "short",
            "A much longer string with various characters and symbols!@#$%",
            "123456789",
            "Mixed case STRING with Numbers 123",
            "Unicode: 中文 français 🚀 emoji",
        ];
        
        let mut embeddings = Vec::new();
        
        for input in &inputs {
            let embedding = embedder.embed(input);
            
            // Each should be proper length and normalized
            assert_eq!(embedding.len(), 768);
            
            let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
            assert!((norm - 1.0).abs() < 0.001, "Input '{}' norm was {}", input, norm);
            
            embeddings.push(embedding);
        }
        
        // All embeddings should be different from each other
        for i in 0..embeddings.len() {
            for j in (i+1)..embeddings.len() {
                assert_ne!(embeddings[i], embeddings[j], 
                    "Inputs '{}' and '{}' produced identical embeddings", 
                    inputs[i], inputs[j]);
            }
        }
    }
}