/// Mathematical utilities for embeddings and vector operations
use std::f32;

/// Calculate cosine similarity between two vectors
/// 
/// Returns a value between -1.0 and 1.0, where:
/// - 1.0 means vectors are identical
/// - 0.0 means vectors are orthogonal  
/// - -1.0 means vectors are opposite
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
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
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0];
        let b = vec![1.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 0.001);
        
        let c = vec![0.0, 1.0];
        assert!((cosine_similarity(&a, &c) - 0.0).abs() < 0.001);
        
        let d = vec![1.0, 1.0];
        let expected = 1.0 / f32::sqrt(2.0);
        assert!((cosine_similarity(&a, &d) - expected).abs() < 0.001);
    }
}