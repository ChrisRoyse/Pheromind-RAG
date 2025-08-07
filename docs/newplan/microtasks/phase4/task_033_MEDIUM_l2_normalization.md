# Task 033 - MEDIUM: Implement L2 Normalization for Embedding Vectors

## Priority: MEDIUM
## Estimated Time: 10 minutes
## Dependencies: Task 032

## Objective
Refine and optimize L2 normalization for embedding vectors to ensure proper similarity computation.

## Current Issue
- L2 normalization needs optimization
- Batch processing efficiency required
- Numerical stability improvements needed

## Tasks
1. **Optimize L2 normalization** (5 min)
   ```rust
   // In src/ml/normalization.rs
   use candle_core::{Tensor, Result as CandleResult};
   use anyhow::Result;
   
   pub struct L2Normalizer {
       epsilon: f64,
   }
   
   impl L2Normalizer {
       pub fn new() -> Self {
           Self {
               epsilon: 1e-12, // Small value to prevent division by zero
           }
       }
       
       pub fn with_epsilon(epsilon: f64) -> Self {
           Self { epsilon }
       }
       
       /// Normalize vectors along the last dimension
       /// Input: [batch_size, embedding_dim] or [embedding_dim]
       /// Output: Same shape with L2 norm = 1.0
       pub fn normalize(&self, embeddings: &Tensor) -> Result<Tensor> {
           let shape = embeddings.shape();
           let dims = shape.dims();
           
           if dims.is_empty() {
               return Err(anyhow::anyhow!("Cannot normalize scalar tensor"));
           }
           
           // Calculate L2 norm along the last dimension
           let norms = self.compute_l2_norms(embeddings)?;
           
           // Normalize by dividing by norms
           let normalized = embeddings.broadcast_div(&norms)
               .map_err(|e| anyhow::anyhow!("Failed to normalize embeddings: {}", e))?;
           
           Ok(normalized)
       }
       
       /// Compute L2 norms for each vector
       fn compute_l2_norms(&self, embeddings: &Tensor) -> Result<Tensor> {
           let shape = embeddings.shape();
           let dims = shape.dims();
           let last_dim = dims.len() - 1;
           
           // Compute squared values
           let squared = embeddings.sqr()
               .map_err(|e| anyhow::anyhow!("Failed to square embeddings: {}", e))?;
           
           // Sum along last dimension
           let sum_squared = squared.sum_keepdim(last_dim)
               .map_err(|e| anyhow::anyhow!("Failed to sum squares: {}", e))?;
           
           // Take square root and add epsilon for stability
           let norms = (sum_squared + self.epsilon).sqrt()
               .map_err(|e| anyhow::anyhow!("Failed to compute square root: {}", e))?;
           
           // Ensure minimum norm value
           let stable_norms = norms.clamp(self.epsilon, f64::INFINITY)
               .map_err(|e| anyhow::anyhow!("Failed to clamp norms: {}", e))?;
           
           Ok(stable_norms)
       }
       
       /// Batch normalize multiple embedding vectors efficiently
       pub fn normalize_batch(&self, embeddings: &Tensor) -> Result<Tensor> {
           // For batch processing, this is the same as regular normalize
           // but we can add batch-specific optimizations here
           self.normalize(embeddings)
       }
       
       /// Check if vectors are already normalized (within tolerance)
       pub fn is_normalized(&self, embeddings: &Tensor, tolerance: f64) -> Result<bool> {
           let norms = self.compute_l2_norms(embeddings)?;
           
           // Check if all norms are close to 1.0
           let ones = Tensor::ones_like(&norms)
               .map_err(|e| anyhow::anyhow!("Failed to create ones tensor: {}", e))?;
           
           let diff = (norms - ones).abs()
               .map_err(|e| anyhow::anyhow!("Failed to compute norm difference: {}", e))?;
           
           let max_diff = diff.max_all()
               .map_err(|e| anyhow::anyhow!("Failed to find max difference: {}", e))?;
           
           let max_diff_value: f64 = max_diff.to_scalar()
               .map_err(|e| anyhow::anyhow!("Failed to extract max diff value: {}", e))?;
           
           Ok(max_diff_value <= tolerance)
       }
   }
   
   impl Default for L2Normalizer {
       fn default() -> Self {
           Self::new()
       }
   }
   ```

2. **Add dimension validation and utilities** (3 min)
   ```rust
   impl L2Normalizer {
       /// Normalize and validate embedding dimensions
       pub fn normalize_with_validation(
           &self,
           embeddings: &Tensor,
           expected_dim: usize,
       ) -> Result<Tensor> {
           let shape = embeddings.shape();
           let dims = shape.dims();
           
           // Validate dimensions
           if dims.is_empty() {
               return Err(anyhow::anyhow!("Empty tensor cannot be normalized"));
           }
           
           let embedding_dim = dims[dims.len() - 1];
           if embedding_dim != expected_dim {
               return Err(anyhow::anyhow!(
                   "Embedding dimension mismatch: expected {}, got {}",
                   expected_dim,
                   embedding_dim
               ));
           }
           
           // Check for invalid values
           self.validate_tensor_values(embeddings)?;
           
           // Normalize
           self.normalize(embeddings)
       }
       
       /// Validate tensor values (check for NaN, Inf)
       fn validate_tensor_values(&self, tensor: &Tensor) -> Result<()> {
           // Check for NaN values
           let has_nan = tensor.isnan()
               .map_err(|e| anyhow::anyhow!("Failed to check for NaN: {}", e))?
               .sum_all()
               .map_err(|e| anyhow::anyhow!("Failed to sum NaN check: {}", e))?;
           
           let nan_count: f32 = has_nan.to_scalar()
               .map_err(|e| anyhow::anyhow!("Failed to extract NaN count: {}", e))?;
           
           if nan_count > 0.0 {
               return Err(anyhow::anyhow!("Tensor contains {} NaN values", nan_count));
           }
           
           // Check for infinite values
           let has_inf = tensor.isinf()
               .map_err(|e| anyhow::anyhow!("Failed to check for Inf: {}", e))?
               .sum_all()
               .map_err(|e| anyhow::anyhow!("Failed to sum Inf check: {}", e))?;
           
           let inf_count: f32 = has_inf.to_scalar()
               .map_err(|e| anyhow::anyhow!("Failed to extract Inf count: {}", e))?;
           
           if inf_count > 0.0 {
               return Err(anyhow::anyhow!("Tensor contains {} infinite values", inf_count));
           }
           
           Ok(())
       }
       
       /// Compute cosine similarity between normalized vectors
       pub fn cosine_similarity(
           &self,
           embeddings1: &Tensor,
           embeddings2: &Tensor,
       ) -> Result<Tensor> {
           // Ensure both tensors are normalized
           let norm1 = self.normalize(embeddings1)?;
           let norm2 = self.normalize(embeddings2)?;
           
           // For normalized vectors, cosine similarity is just dot product
           let similarity = norm1.matmul(&norm2.t()
               .map_err(|e| anyhow::anyhow!("Failed to transpose embeddings2: {}", e))?)
               .map_err(|e| anyhow::anyhow!("Failed to compute dot product: {}", e))?;
           
           Ok(similarity)
       }
       
       /// Compute pairwise distances between normalized vectors
       pub fn pairwise_distances(
           &self,
           embeddings1: &Tensor,
           embeddings2: &Tensor,
       ) -> Result<Tensor> {
           let similarities = self.cosine_similarity(embeddings1, embeddings2)?;
           
           // Convert cosine similarity to distance: distance = 1 - similarity
           let distances = (Tensor::ones_like(&similarities)
               .map_err(|e| anyhow::anyhow!("Failed to create ones tensor: {}", e))? - similarities)
               .map_err(|e| anyhow::anyhow!("Failed to compute distances: {}", e))?;
           
           Ok(distances)
       }
   }
   ```

3. **Add performance optimizations** (2 min)
   ```rust
   /// Optimized normalizer for large batches
   pub struct BatchL2Normalizer {
       base_normalizer: L2Normalizer,
       chunk_size: usize,
   }
   
   impl BatchL2Normalizer {
       pub fn new(chunk_size: usize) -> Self {
           Self {
               base_normalizer: L2Normalizer::new(),
               chunk_size,
           }
       }
       
       /// Process large batches in chunks to manage memory
       pub fn normalize_large_batch(&self, embeddings: &Tensor) -> Result<Tensor> {
           let shape = embeddings.shape();
           let dims = shape.dims();
           
           if dims.len() < 2 {
               // Small tensor, process normally
               return self.base_normalizer.normalize(embeddings);
           }
           
           let batch_size = dims[0];
           let embedding_dim = dims[1];
           
           if batch_size <= self.chunk_size {
               // Batch is small enough, process normally
               return self.base_normalizer.normalize(embeddings);
           }
           
           // Process in chunks
           let mut normalized_chunks = Vec::new();
           
           for start in (0..batch_size).step_by(self.chunk_size) {
               let end = (start + self.chunk_size).min(batch_size);
               
               let chunk = embeddings.narrow(0, start, end - start)
                   .map_err(|e| anyhow::anyhow!("Failed to extract chunk: {}", e))?;
               
               let normalized_chunk = self.base_normalizer.normalize(&chunk)?;
               normalized_chunks.push(normalized_chunk);
           }
           
           // Concatenate chunks
           let normalized_batch = Tensor::cat(&normalized_chunks, 0)
               .map_err(|e| anyhow::anyhow!("Failed to concatenate chunks: {}", e))?;
           
           Ok(normalized_batch)
       }
   }
   ```

## Success Criteria
- [ ] L2 normalization produces unit vectors
- [ ] Numerical stability maintained
- [ ] Batch processing works efficiently
- [ ] NaN and Inf values detected and handled
- [ ] Cosine similarity computation correct
- [ ] Large batch processing optimized

## Files to Create
- `src/ml/normalization.rs`

## Files to Modify
- `src/ml/mod.rs`
- `src/ml/pooling.rs` (integrate normalization)

## Validation
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use candle_core::{Tensor, Device};
    
    #[test]
    fn test_l2_normalization_correctness() {
        let device = Device::Cpu;
        let normalizer = L2Normalizer::new();
        
        // Test vector with known norm
        let embedding = Tensor::from_vec(
            vec![3.0, 4.0, 0.0], // L2 norm = 5.0
            &[1, 3],
            &device
        ).unwrap();
        
        let normalized = normalizer.normalize(&embedding).unwrap();
        
        // Check L2 norm is 1.0
        let norm = normalizer.compute_l2_norms(&normalized).unwrap();
        let norm_value: f64 = norm.to_scalar().unwrap();
        
        assert!((norm_value - 1.0).abs() < 1e-6);
        
        // Check values are correct
        let expected = Tensor::from_vec(
            vec![0.6, 0.8, 0.0], // [3/5, 4/5, 0/5]
            &[1, 3],
            &device
        ).unwrap();
        
        // Compare with small tolerance
        let diff = (normalized - expected).unwrap().abs().unwrap();
        let max_diff: f64 = diff.max_all().unwrap().to_scalar().unwrap();
        assert!(max_diff < 1e-6);
    }
    
    #[test]
    fn test_batch_normalization() {
        let device = Device::Cpu;
        let normalizer = L2Normalizer::new();
        
        // Batch of 3 vectors
        let embeddings = Tensor::from_vec(
            vec![
                1.0, 0.0, 0.0,  // First vector
                0.0, 2.0, 0.0,  // Second vector  
                1.0, 1.0, 1.0,  // Third vector
            ],
            &[3, 3],
            &device
        ).unwrap();
        
        let normalized = normalizer.normalize(&embeddings).unwrap();
        
        // Each vector should have unit norm
        for i in 0..3 {
            let vector = normalized.i((i, ..)).unwrap();
            let norm = normalizer.compute_l2_norms(&vector.unsqueeze(0).unwrap()).unwrap();
            let norm_value: f64 = norm.to_scalar().unwrap();
            assert!((norm_value - 1.0).abs() < 1e-6);
        }
    }
}
```

## Next Task
→ Task 034: Verify embedding dimensions match expected output