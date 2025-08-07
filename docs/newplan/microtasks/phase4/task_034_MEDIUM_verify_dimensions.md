# Task 034 - MEDIUM: Verify Embedding Dimensions Match Expected Output

## Priority: MEDIUM
## Estimated Time: 10 minutes
## Dependencies: Task 033

## Objective
Implement comprehensive dimension validation throughout the embedding pipeline to catch errors early.

## Current Issue
- Dimension mismatches cause runtime errors
- Need validation at each pipeline stage
- Better error messages for debugging

## Tasks
1. **Create dimension validator** (4 min)
   ```rust
   // In src/ml/dimension_validator.rs
   use candle_core::Tensor;
   use anyhow::{Result, anyhow};
   
   #[derive(Debug, Clone)]
   pub struct DimensionSpec {
       pub expected_dims: Vec<Option<usize>>, // None means any size allowed
       pub min_dims: Option<Vec<usize>>,
       pub max_dims: Option<Vec<usize>>,
       pub description: String,
   }
   
   impl DimensionSpec {
       pub fn new(expected_dims: Vec<Option<usize>>, description: &str) -> Self {
           Self {
               expected_dims,
               min_dims: None,
               max_dims: None,
               description: description.to_string(),
           }
       }
       
       pub fn with_bounds(
           expected_dims: Vec<Option<usize>>,
           min_dims: Option<Vec<usize>>,
           max_dims: Option<Vec<usize>>,
           description: &str,
       ) -> Self {
           Self {
               expected_dims,
               min_dims,
               max_dims,
               description: description.to_string(),
           }
       }
       
       // Common dimension specs for the embedding pipeline
       pub fn token_ids_2d() -> Self {
           Self::new(
               vec![None, None], // [batch_size, seq_len]
               "Token IDs tensor (batch_size, sequence_length)"
           )
       }
       
       pub fn attention_mask_2d() -> Self {
           Self::new(
               vec![None, None], // [batch_size, seq_len]
               "Attention mask tensor (batch_size, sequence_length)"
           )
       }
       
       pub fn hidden_states_3d(embedding_dim: usize) -> Self {
           Self::new(
               vec![None, None, Some(embedding_dim)], // [batch_size, seq_len, embedding_dim]
               &format!("Hidden states tensor (batch_size, sequence_length, {})", embedding_dim)
           )
       }
       
       pub fn embeddings_2d(embedding_dim: usize) -> Self {
           Self::new(
               vec![None, Some(embedding_dim)], // [batch_size, embedding_dim]
               &format!("Final embeddings tensor (batch_size, {})", embedding_dim)
           )
       }
       
       pub fn single_embedding_1d(embedding_dim: usize) -> Self {
           Self::new(
               vec![Some(embedding_dim)], // [embedding_dim]
               &format!("Single embedding vector ({})", embedding_dim)
           )
       }
   }
   
   pub struct DimensionValidator;
   
   impl DimensionValidator {
       pub fn validate(tensor: &Tensor, spec: &DimensionSpec) -> Result<()> {
           let shape = tensor.shape();
           let actual_dims = shape.dims();
           
           // Check number of dimensions
           if actual_dims.len() != spec.expected_dims.len() {
               return Err(anyhow!(
                   "Dimension count mismatch for {}: expected {} dimensions, got {} (shape: {:?})",
                   spec.description,
                   spec.expected_dims.len(),
                   actual_dims.len(),
                   actual_dims
               ));
           }
           
           // Check each dimension
           for (i, (actual, expected)) in actual_dims.iter().zip(spec.expected_dims.iter()).enumerate() {
               if let Some(expected_size) = expected {
                   if actual != expected_size {
                       return Err(anyhow!(
                           "Dimension {} mismatch for {}: expected {}, got {} (full shape: {:?})",
                           i,
                           spec.description,
                           expected_size,
                           actual,
                           actual_dims
                       ));
                   }
               }
           }
           
           // Check minimum bounds
           if let Some(min_dims) = &spec.min_dims {
               for (i, (actual, min_size)) in actual_dims.iter().zip(min_dims.iter()).enumerate() {
                   if actual < min_size {
                       return Err(anyhow!(
                           "Dimension {} too small for {}: minimum {}, got {} (full shape: {:?})",
                           i,
                           spec.description,
                           min_size,
                           actual,
                           actual_dims
                       ));
                   }
               }
           }
           
           // Check maximum bounds
           if let Some(max_dims) = &spec.max_dims {
               for (i, (actual, max_size)) in actual_dims.iter().zip(max_dims.iter()).enumerate() {
                   if actual > max_size {
                       return Err(anyhow!(
                           "Dimension {} too large for {}: maximum {}, got {} (full shape: {:?})",
                           i,
                           spec.description,
                           max_size,
                           actual,
                           actual_dims
                       ));
                   }
               }
           }
           
           Ok(())
       }
       
       pub fn validate_matching_dims(
           tensor1: &Tensor,
           tensor2: &Tensor,
           dims_to_match: &[usize],
           description: &str,
       ) -> Result<()> {
           let shape1 = tensor1.shape().dims();
           let shape2 = tensor2.shape().dims();
           
           for &dim_idx in dims_to_match {
               if dim_idx >= shape1.len() || dim_idx >= shape2.len() {
                   return Err(anyhow!(
                       "Dimension index {} out of bounds for {} validation (shapes: {:?} vs {:?})",
                       dim_idx,
                       description,
                       shape1,
                       shape2
                   ));
               }
               
               if shape1[dim_idx] != shape2[dim_idx] {
                   return Err(anyhow!(
                       "Dimension {} mismatch for {}: {} vs {} (full shapes: {:?} vs {:?})",
                       dim_idx,
                       description,
                       shape1[dim_idx],
                       shape2[dim_idx],
                       shape1,
                       shape2
                   ));
               }
           }
           
           Ok(())
       }
   }
   ```

2. **Add pipeline stage validation** (4 min)
   ```rust
   // Pipeline validation helpers
   pub struct EmbeddingPipelineValidator {
       embedding_dim: usize,
       max_seq_len: usize,
       max_batch_size: Option<usize>,
   }
   
   impl EmbeddingPipelineValidator {
       pub fn new(embedding_dim: usize, max_seq_len: usize) -> Self {
           Self {
               embedding_dim,
               max_seq_len,
               max_batch_size: None,
           }
       }
       
       pub fn with_max_batch_size(mut self, max_batch_size: usize) -> Self {
           self.max_batch_size = Some(max_batch_size);
           self
       }
       
       pub fn validate_tokenizer_output(
           &self,
           input_ids: &Tensor,
           attention_mask: &Tensor,
       ) -> Result<()> {
           // Validate input_ids
           let mut input_spec = DimensionSpec::token_ids_2d();
           if let Some(max_batch) = self.max_batch_size {
               input_spec.max_dims = Some(vec![max_batch, self.max_seq_len]);
           }
           DimensionValidator::validate(input_ids, &input_spec)?;
           
           // Validate attention_mask
           let mut mask_spec = DimensionSpec::attention_mask_2d();
           if let Some(max_batch) = self.max_batch_size {
               mask_spec.max_dims = Some(vec![max_batch, self.max_seq_len]);
           }
           DimensionValidator::validate(attention_mask, &mask_spec)?;
           
           // Validate matching dimensions
           DimensionValidator::validate_matching_dims(
               input_ids,
               attention_mask,
               &[0, 1], // Both batch and sequence dimensions must match
               "tokenizer output (input_ids vs attention_mask)"
           )?;
           
           Ok(())
       }
       
       pub fn validate_transformer_output(&self, hidden_states: &Tensor) -> Result<()> {
           let mut spec = DimensionSpec::hidden_states_3d(self.embedding_dim);
           if let Some(max_batch) = self.max_batch_size {
               spec.max_dims = Some(vec![max_batch, self.max_seq_len, self.embedding_dim]);
           }
           
           DimensionValidator::validate(hidden_states, &spec)?;
           Ok(())
       }
       
       pub fn validate_pooled_output(&self, embeddings: &Tensor) -> Result<()> {
           let mut spec = DimensionSpec::embeddings_2d(self.embedding_dim);
           if let Some(max_batch) = self.max_batch_size {
               spec.max_dims = Some(vec![max_batch, self.embedding_dim]);
           }
           
           DimensionValidator::validate(embeddings, &spec)?;
           Ok(())
       }
       
       pub fn validate_final_embeddings(&self, embeddings: &Tensor) -> Result<()> {
           // Can be either 1D (single embedding) or 2D (batch)
           let shape = embeddings.shape().dims();
           
           match shape.len() {
               1 => {
                   let spec = DimensionSpec::single_embedding_1d(self.embedding_dim);
                   DimensionValidator::validate(embeddings, &spec)?
               },
               2 => {
                   let mut spec = DimensionSpec::embeddings_2d(self.embedding_dim);
                   if let Some(max_batch) = self.max_batch_size {
                       spec.max_dims = Some(vec![max_batch, self.embedding_dim]);
                   }
                   DimensionValidator::validate(embeddings, &spec)?
               },
               _ => return Err(anyhow!(
                   "Final embeddings must be 1D or 2D, got shape: {:?}",
                   shape
               )),
           }
           
           Ok(())
       }
   }
   ```

3. **Add memory usage estimation** (2 min)
   ```rust
   pub struct MemoryEstimator;
   
   impl MemoryEstimator {
       pub fn estimate_tensor_memory(tensor: &Tensor) -> f64 {
           let shape = tensor.shape();
           let element_count: usize = shape.dims().iter().product();
           
           // Estimate based on dtype (assuming f32 for simplicity)
           let bytes_per_element = 4; // f32 = 4 bytes
           let total_bytes = element_count * bytes_per_element;
           
           total_bytes as f64 / (1024.0 * 1024.0) // Convert to MB
       }
       
       pub fn estimate_pipeline_memory(
           batch_size: usize,
           seq_len: usize,
           embedding_dim: usize,
           num_layers: usize,
       ) -> f64 {
           // Estimate memory usage for entire pipeline
           let input_ids_mb = (batch_size * seq_len * 8) as f64 / (1024.0 * 1024.0); // i64
           let attention_mask_mb = (batch_size * seq_len * 4) as f64 / (1024.0 * 1024.0); // f32
           let hidden_states_mb = (batch_size * seq_len * embedding_dim * 4) as f64 / (1024.0 * 1024.0); // f32
           let final_embeddings_mb = (batch_size * embedding_dim * 4) as f64 / (1024.0 * 1024.0); // f32
           
           // Rough estimate including intermediate activations
           let total_mb = input_ids_mb + attention_mask_mb + 
                         (hidden_states_mb * num_layers as f64 * 2.0) + // 2x for forward pass activations
                         final_embeddings_mb;
           
           total_mb
       }
   }
   ```

## Success Criteria
- [ ] Dimension validation catches mismatches
- [ ] Clear error messages for debugging
- [ ] Pipeline stage validation works
- [ ] Memory estimation is accurate
- [ ] Performance impact is minimal
- [ ] Edge cases handled properly

## Files to Create
- `src/ml/dimension_validator.rs`

## Files to Modify
- `src/ml/mod.rs`
- `src/ml/embedding_service.rs` (add validation calls)

## Validation
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use candle_core::{Tensor, Device};
    
    #[test]
    fn test_dimension_validation_success() {
        let device = Device::Cpu;
        let validator = EmbeddingPipelineValidator::new(768, 512);
        
        let input_ids = Tensor::zeros(&[2, 128], candle_core::DType::I64, &device).unwrap();
        let attention_mask = Tensor::ones(&[2, 128], candle_core::DType::F32, &device).unwrap();
        
        // Should pass validation
        assert!(validator.validate_tokenizer_output(&input_ids, &attention_mask).is_ok());
    }
    
    #[test]
    fn test_dimension_validation_failure() {
        let device = Device::Cpu;
        let validator = EmbeddingPipelineValidator::new(768, 512);
        
        let input_ids = Tensor::zeros(&[2, 128], candle_core::DType::I64, &device).unwrap();
        let attention_mask = Tensor::ones(&[2, 64], candle_core::DType::F32, &device).unwrap(); // Wrong seq_len
        
        // Should fail validation
        assert!(validator.validate_tokenizer_output(&input_ids, &attention_mask).is_err());
    }
    
    #[test]
    fn test_memory_estimation() {
        let memory_mb = MemoryEstimator::estimate_pipeline_memory(
            4,   // batch_size
            512, // seq_len
            768, // embedding_dim
            12,  // num_layers
        );
        
        // Should be reasonable estimate (around 150-200MB for this configuration)
        assert!(memory_mb > 50.0 && memory_mb < 500.0);
    }
}
```

## Next Task
→ Task 035: Handle batch processing for multiple texts