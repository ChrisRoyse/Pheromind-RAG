# Task 032 - HIGH: Implement Mean Pooling for Embeddings

## Priority: HIGH
## Estimated Time: 10 minutes
## Dependencies: Task 031

## Objective
Implement mean pooling to convert transformer output sequences to fixed-size embedding vectors.

## Current Issue
- Transformer outputs variable-length sequences
- Need fixed-size embeddings for similarity search
- Must handle attention masks for proper pooling

## Tasks
1. **Implement mean pooling** (5 min)
   ```rust
   // In src/ml/pooling.rs
   use candle_core::{Tensor, Result as CandleResult};
   use anyhow::Result;
   
   pub struct MeanPooler {
       embedding_dim: usize,
   }
   
   impl MeanPooler {
       pub fn new(embedding_dim: usize) -> Self {
           Self { embedding_dim }
       }
       
       /// Apply mean pooling to transformer output
       /// 
       /// Args:
       ///   - hidden_states: [batch_size, seq_len, hidden_dim]
       ///   - attention_mask: [batch_size, seq_len] - 1 for real tokens, 0 for padding
       /// 
       /// Returns:
       ///   - pooled_output: [batch_size, hidden_dim]
       pub fn forward(
           &self,
           hidden_states: &Tensor,
           attention_mask: &Tensor,
       ) -> Result<Tensor> {
           let (batch_size, seq_len, hidden_dim) = hidden_states.shape().dims3()
               .map_err(|e| anyhow::anyhow!("Invalid hidden_states shape: {}", e))?;
           
           if hidden_dim != self.embedding_dim {
               return Err(anyhow::anyhow!(
                   "Hidden dimension mismatch: expected {}, got {}",
                   self.embedding_dim,
                   hidden_dim
               ));
           }
           
           // Expand attention mask to match hidden states dimensions
           let expanded_mask = attention_mask
               .unsqueeze(2) // Add hidden dimension: [batch, seq_len, 1]
               .map_err(|e| anyhow::anyhow!("Failed to expand attention mask: {}", e))?
               .expand(&[batch_size, seq_len, hidden_dim]) // [batch, seq_len, hidden_dim]
               .map_err(|e| anyhow::anyhow!("Failed to expand mask to hidden dims: {}", e))?;
           
           // Apply mask to hidden states (zero out padding tokens)
           let masked_hidden_states = (hidden_states * &expanded_mask)
               .map_err(|e| anyhow::anyhow!("Failed to apply mask to hidden states: {}", e))?;
           
           // Sum across sequence dimension
           let sum_hidden_states = masked_hidden_states
               .sum(1) // Sum along seq_len dimension
               .map_err(|e| anyhow::anyhow!("Failed to sum hidden states: {}", e))?;
           
           // Count non-padding tokens for proper averaging
           let token_counts = attention_mask
               .sum(1) // Sum along seq_len dimension: [batch_size]
               .map_err(|e| anyhow::anyhow!("Failed to count tokens: {}", e))?
               .unsqueeze(1) // [batch_size, 1]
               .map_err(|e| anyhow::anyhow!("Failed to reshape token counts: {}", e))?
               .clamp(1.0, f32::INFINITY) // Avoid division by zero
               .map_err(|e| anyhow::anyhow!("Failed to clamp token counts: {}", e))?;
           
           // Compute mean by dividing sum by count
           let mean_pooled = (sum_hidden_states / token_counts)
               .map_err(|e| anyhow::anyhow!("Failed to compute mean: {}", e))?;
           
           Ok(mean_pooled)
       }
       
       /// Apply mean pooling without attention mask (assumes all tokens are valid)
       pub fn forward_simple(&self, hidden_states: &Tensor) -> Result<Tensor> {
           let pooled = hidden_states
               .mean(1) // Mean along sequence dimension
               .map_err(|e| anyhow::anyhow!("Failed to compute simple mean pooling: {}", e))?;
           
           Ok(pooled)
       }
   }
   ```

2. **Add alternative pooling strategies** (3 min)
   ```rust
   pub enum PoolingStrategy {
       Mean,
       Max,
       Cls,        // Use [CLS] token (first token)
       WeightedMean, // Weighted by attention scores
   }
   
   pub struct AdaptivePooler {
       embedding_dim: usize,
       strategy: PoolingStrategy,
   }
   
   impl AdaptivePooler {
       pub fn new(embedding_dim: usize, strategy: PoolingStrategy) -> Self {
           Self { embedding_dim, strategy }
       }
       
       pub fn forward(
           &self,
           hidden_states: &Tensor,
           attention_mask: &Tensor,
       ) -> Result<Tensor> {
           match self.strategy {
               PoolingStrategy::Mean => {
                   let pooler = MeanPooler::new(self.embedding_dim);
                   pooler.forward(hidden_states, attention_mask)
               },
               PoolingStrategy::Max => {
                   self.max_pooling(hidden_states, attention_mask)
               },
               PoolingStrategy::Cls => {
                   self.cls_pooling(hidden_states)
               },
               PoolingStrategy::WeightedMean => {
                   // For now, fall back to regular mean pooling
                   let pooler = MeanPooler::new(self.embedding_dim);
                   pooler.forward(hidden_states, attention_mask)
               },
           }
       }
       
       fn max_pooling(
           &self,
           hidden_states: &Tensor,
           attention_mask: &Tensor,
       ) -> Result<Tensor> {
           let (batch_size, seq_len, hidden_dim) = hidden_states.shape().dims3()
               .map_err(|e| anyhow::anyhow!("Invalid shape for max pooling: {}", e))?;
           
           // Expand mask
           let expanded_mask = attention_mask
               .unsqueeze(2)?
               .expand(&[batch_size, seq_len, hidden_dim])?;
           
           // Set padding positions to large negative value
           let mask_value = Tensor::full(
               f32::NEG_INFINITY,
               &[batch_size, seq_len, hidden_dim],
               hidden_states.device()
           ).map_err(|e| anyhow::anyhow!("Failed to create mask value tensor: {}", e))?;
           
           let masked_hidden = expanded_mask.where_cond(
               hidden_states,
               &mask_value,
           ).map_err(|e| anyhow::anyhow!("Failed to apply mask for max pooling: {}", e))?;
           
           // Take max along sequence dimension
           let max_pooled = masked_hidden
               .max(1) // Max along seq_len dimension
               .map_err(|e| anyhow::anyhow!("Failed to compute max pooling: {}", e))?;
           
           Ok(max_pooled)
       }
       
       fn cls_pooling(&self, hidden_states: &Tensor) -> Result<Tensor> {
           // Extract first token ([CLS] token) for each sequence in batch
           let cls_tokens = hidden_states
               .i((.., 0, ..)) // [batch_size, hidden_dim]
               .map_err(|e| anyhow::anyhow!("Failed to extract CLS tokens: {}", e))?;
           
           Ok(cls_tokens)
       }
   }
   ```

3. **Add embedding post-processing** (2 min)
   ```rust
   pub struct EmbeddingPostProcessor {
       normalize: bool,
       target_dim: Option<usize>,
   }
   
   impl EmbeddingPostProcessor {
       pub fn new(normalize: bool, target_dim: Option<usize>) -> Self {
           Self { normalize, target_dim }
       }
       
       pub fn process(&self, embeddings: &Tensor) -> Result<Tensor> {
           let mut processed = embeddings.clone();
           
           // Apply L2 normalization if requested
           if self.normalize {
               processed = self.l2_normalize(&processed)?;
           }
           
           // Apply dimension reduction if requested
           if let Some(target_dim) = self.target_dim {
               processed = self.reduce_dimension(&processed, target_dim)?;
           }
           
           Ok(processed)
       }
       
       fn l2_normalize(&self, embeddings: &Tensor) -> Result<Tensor> {
           // Calculate L2 norm along the last dimension
           let norms = embeddings
               .sqr()
               .map_err(|e| anyhow::anyhow!("Failed to square embeddings: {}", e))?
               .sum_keepdim(-1) // Sum along last dimension, keep dims
               .map_err(|e| anyhow::anyhow!("Failed to sum squares: {}", e))?
               .sqrt()
               .map_err(|e| anyhow::anyhow!("Failed to compute sqrt: {}", e))?
               .clamp(1e-8, f32::INFINITY) // Avoid division by zero
               .map_err(|e| anyhow::anyhow!("Failed to clamp norms: {}", e))?;
           
           // Normalize
           let normalized = (embeddings / norms)
               .map_err(|e| anyhow::anyhow!("Failed to normalize embeddings: {}", e))?;
           
           Ok(normalized)
       }
       
       fn reduce_dimension(&self, embeddings: &Tensor, target_dim: usize) -> Result<Tensor> {
           let current_dim = embeddings.shape().dims().last().copied().unwrap_or(0);
           
           if current_dim <= target_dim {
               // No reduction needed
               return Ok(embeddings.clone());
           }
           
           // Simple truncation for now (could implement PCA or other methods)
           let reduced = embeddings
               .narrow(-1, 0, target_dim) // Take first target_dim dimensions
               .map_err(|e| anyhow::anyhow!("Failed to reduce dimension: {}", e))?;
           
           Ok(reduced)
       }
   }
   ```

## Success Criteria
- [ ] Mean pooling produces correct output shape
- [ ] Attention mask handled properly
- [ ] Alternative pooling strategies work
- [ ] L2 normalization functions correctly
- [ ] No division by zero errors
- [ ] Pooling is differentiable (if needed)

## Files to Create
- `src/ml/pooling.rs`

## Files to Modify
- `src/ml/mod.rs`

## Validation
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use candle_core::{Tensor, Device};
    
    #[test]
    fn test_mean_pooling() {
        let device = Device::Cpu;
        let pooler = MeanPooler::new(768);
        
        // Create test data: [batch_size=2, seq_len=5, hidden_dim=768]
        let hidden_states = Tensor::randn(
            0.0, 1.0,
            &[2, 5, 768],
            &device
        ).unwrap();
        
        // Attention mask: first sequence has 3 real tokens, second has 4
        let attention_mask = Tensor::from_vec(
            vec![1.0, 1.0, 1.0, 0.0, 0.0,  // First sequence
                 1.0, 1.0, 1.0, 1.0, 0.0],  // Second sequence
            &[2, 5],
            &device
        ).unwrap();
        
        let result = pooler.forward(&hidden_states, &attention_mask).unwrap();
        
        // Should output [batch_size=2, hidden_dim=768]
        assert_eq!(result.shape().dims(), &[2, 768]);
    }
    
    #[test]
    fn test_l2_normalization() {
        let device = Device::Cpu;
        let processor = EmbeddingPostProcessor::new(true, None);
        
        let embeddings = Tensor::from_vec(
            vec![3.0, 4.0, 0.0], // L2 norm should be 5.0
            &[1, 3],
            &device
        ).unwrap();
        
        let normalized = processor.process(&embeddings).unwrap();
        
        // Check that L2 norm is approximately 1.0
        let norm = normalized.sqr().unwrap().sum_all().unwrap().sqrt().unwrap();
        let norm_value: f32 = norm.to_scalar().unwrap();
        
        assert!((norm_value - 1.0).abs() < 1e-6);
    }
}
```

## Next Task
→ Task 033: Implement L2 normalization for embedding vectors