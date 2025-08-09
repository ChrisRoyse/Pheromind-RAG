use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub struct MinimalEmbedder {
    dimension: usize,
}

impl MinimalEmbedder {
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
            // Create more varied seeds using different operations
            let seed1 = base_hash.wrapping_mul(i as u64 + 1);
            let seed2 = seed1.rotate_left(i as u32 % 64);
            let seed = seed1 ^ seed2;
            
            // Convert to float with better range distribution
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