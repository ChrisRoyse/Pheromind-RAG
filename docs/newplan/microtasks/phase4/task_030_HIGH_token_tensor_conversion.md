# Task 030 - HIGH: Convert Tokens to Tensor Format for Model Input

## Priority: HIGH
## Estimated Time: 10 minutes
## Dependencies: Task 029

## Objective
Convert tokenized input to Candle tensor format compatible with the transformer model.

## Current Issue
- Tokenizer output needs tensor conversion
- Candle tensor creation from token IDs
- Proper device handling for tensors

## Tasks
1. **Implement tensor conversion** (6 min)
   ```rust
   // In src/ml/tensor_conversion.rs
   use candle_core::{Tensor, Device, DType};
   use candle_nn::VarBuilder;
   use anyhow::Result;
   use crate::ml::tokenizer::TokenizedInput;
   
   pub struct TensorConverter {
       device: Device,
       max_length: usize,
   }
   
   impl TensorConverter {
       pub fn new(device: Device, max_length: usize) -> Self {
           Self { device, max_length }
       }
       
       pub fn tokenized_to_tensors(&self, input: &TokenizedInput) -> Result<ModelInput> {
           // Convert token IDs to tensor
           let input_ids_tensor = self.create_input_ids_tensor(&input.input_ids)?;
           
           // Convert attention mask to tensor
           let attention_mask_tensor = self.create_attention_mask_tensor(&input.attention_mask)?;
           
           // Create position IDs (0, 1, 2, ... sequence_length)
           let position_ids = self.create_position_ids_tensor(input.input_ids.len())?;
           
           Ok(ModelInput {
               input_ids: input_ids_tensor,
               attention_mask: attention_mask_tensor,
               position_ids,
               sequence_length: input.token_count,
           })
       }
       
       fn create_input_ids_tensor(&self, input_ids: &[u32]) -> Result<Tensor> {
           let ids_i64: Vec<i64> = input_ids.iter().map(|&x| x as i64).collect();
           
           Tensor::from_vec(
               ids_i64,
               &[1, input_ids.len()], // [batch_size, sequence_length]
               &self.device,
           )
       }
       
       fn create_attention_mask_tensor(&self, attention_mask: &[u32]) -> Result<Tensor> {
           let mask_f32: Vec<f32> = attention_mask.iter().map(|&x| x as f32).collect();
           
           Tensor::from_vec(
               mask_f32,
               &[1, attention_mask.len()], // [batch_size, sequence_length]
               &self.device,
           )
       }
       
       fn create_position_ids_tensor(&self, sequence_length: usize) -> Result<Tensor> {
           let position_ids: Vec<i64> = (0..sequence_length as i64).collect();
           
           Tensor::from_vec(
               position_ids,
               &[1, sequence_length], // [batch_size, sequence_length]
               &self.device,
           )
       }
       
       pub fn batch_tokenized_to_tensors(&self, inputs: &[TokenizedInput]) -> Result<ModelInput> {
           if inputs.is_empty() {
               return Err(anyhow::anyhow!("Cannot convert empty batch"));
           }
           
           let batch_size = inputs.len();
           let seq_len = inputs[0].input_ids.len();
           
           // Verify all inputs have same length
           for input in inputs {
               if input.input_ids.len() != seq_len {
                   return Err(anyhow::anyhow!(
                       "Inconsistent sequence lengths in batch: {} vs {}",
                       input.input_ids.len(),
                       seq_len
                   ));
               }
           }
           
           // Flatten input IDs for batch
           let mut all_input_ids = Vec::with_capacity(batch_size * seq_len);
           let mut all_attention_mask = Vec::with_capacity(batch_size * seq_len);
           
           for input in inputs {
               all_input_ids.extend(input.input_ids.iter().map(|&x| x as i64));
               all_attention_mask.extend(input.attention_mask.iter().map(|&x| x as f32));
           }
           
           // Create batch tensors
           let input_ids_tensor = Tensor::from_vec(
               all_input_ids,
               &[batch_size, seq_len],
               &self.device,
           )?;
           
           let attention_mask_tensor = Tensor::from_vec(
               all_attention_mask,
               &[batch_size, seq_len],
               &self.device,
           )?;
           
           // Create position IDs for batch
           let position_ids: Vec<i64> = (0..seq_len as i64).cycle().take(batch_size * seq_len).collect();
           let position_ids_tensor = Tensor::from_vec(
               position_ids,
               &[batch_size, seq_len],
               &self.device,
           )?;
           
           Ok(ModelInput {
               input_ids: input_ids_tensor,
               attention_mask: attention_mask_tensor,
               position_ids: position_ids_tensor,
               sequence_length: seq_len,
           })
       }
   }
   
   #[derive(Debug)]
   pub struct ModelInput {
       pub input_ids: Tensor,
       pub attention_mask: Tensor,
       pub position_ids: Tensor,
       pub sequence_length: usize,
   }
   
   impl ModelInput {
       pub fn batch_size(&self) -> Result<usize> {
           Ok(self.input_ids.shape().dims()[0])
       }
       
       pub fn sequence_length(&self) -> Result<usize> {
           Ok(self.input_ids.shape().dims()[1])
       }
       
       pub fn device(&self) -> &Device {
           self.input_ids.device()
       }
   }
   ```

2. **Add tensor validation** (3 min)
   ```rust
   impl TensorConverter {
       pub fn validate_tensor_shapes(&self, input: &ModelInput) -> Result<()> {
           let input_ids_shape = input.input_ids.shape();
           let attention_mask_shape = input.attention_mask.shape();
           let position_ids_shape = input.position_ids.shape();
           
           // Check dimensions match
           if input_ids_shape.dims() != attention_mask_shape.dims() {
               return Err(anyhow::anyhow!(
                   "Shape mismatch: input_ids {:?} vs attention_mask {:?}",
                   input_ids_shape.dims(),
                   attention_mask_shape.dims()
               ));
           }
           
           if input_ids_shape.dims() != position_ids_shape.dims() {
               return Err(anyhow::anyhow!(
                   "Shape mismatch: input_ids {:?} vs position_ids {:?}",
                   input_ids_shape.dims(),
                   position_ids_shape.dims()
               ));
           }
           
           // Check sequence length doesn't exceed max
           let seq_len = input_ids_shape.dims()[1];
           if seq_len > self.max_length {
               return Err(anyhow::anyhow!(
                   "Sequence length {} exceeds maximum {}",
                   seq_len,
                   self.max_length
               ));
           }
           
           // Check all tensors are on same device
           if input.input_ids.device() != input.attention_mask.device() {
               return Err(anyhow::anyhow!("Tensors on different devices"));
           }
           
           Ok(())
       }
       
       pub fn tensor_info(&self, input: &ModelInput) -> Result<TensorInfo> {
           let batch_size = input.batch_size()?;
           let seq_len = input.sequence_length()?;
           let device = input.device();
           
           let memory_usage = self.estimate_memory_usage(batch_size, seq_len);
           
           Ok(TensorInfo {
               batch_size,
               sequence_length: seq_len,
               device_type: format!("{:?}", device),
               memory_usage_mb: memory_usage,
           })
       }
       
       fn estimate_memory_usage(&self, batch_size: usize, seq_len: usize) -> f64 {
           let input_ids_bytes = batch_size * seq_len * 8; // i64
           let attention_mask_bytes = batch_size * seq_len * 4; // f32
           let position_ids_bytes = batch_size * seq_len * 8; // i64
           
           let total_bytes = input_ids_bytes + attention_mask_bytes + position_ids_bytes;
           total_bytes as f64 / (1024.0 * 1024.0) // Convert to MB
       }
   }
   
   #[derive(Debug)]
   pub struct TensorInfo {
       pub batch_size: usize,
       pub sequence_length: usize,
       pub device_type: String,
       pub memory_usage_mb: f64,
   }
   ```

3. **Add integration helpers** (1 min)
   ```rust
   // Helper functions for easy integration
   impl TensorConverter {
       pub fn text_to_model_input(
           &self,
           tokenizer: &crate::ml::tokenizer::EmbeddingTokenizer,
           text: &str,
       ) -> Result<ModelInput> {
           let tokenized = tokenizer.tokenize(text)?;
           let model_input = self.tokenized_to_tensors(&tokenized)?;
           self.validate_tensor_shapes(&model_input)?;
           Ok(model_input)
       }
       
       pub fn texts_to_model_input(
           &self,
           tokenizer: &crate::ml::tokenizer::EmbeddingTokenizer,
           texts: &[&str],
       ) -> Result<ModelInput> {
           let tokenized_inputs = tokenizer.tokenize_batch(texts)?;
           let model_input = self.batch_tokenized_to_tensors(&tokenized_inputs)?;
           self.validate_tensor_shapes(&model_input)?;
           Ok(model_input)
       }
   }
   ```

## Success Criteria
- [ ] Token to tensor conversion works
- [ ] Batch processing supported
- [ ] Shape validation passes
- [ ] Memory usage estimated correctly
- [ ] Device handling proper
- [ ] Integration helpers functional

## Files to Create
- `src/ml/tensor_conversion.rs`

## Files to Modify
- `src/ml/mod.rs`

## Validation
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::ml::tokenizer::*;
    use candle_core::Device;
    
    #[tokio::test]
    async fn test_tensor_conversion() {
        let device = Device::Cpu;
        let converter = TensorConverter::new(device, 512);
        
        let tokenized = TokenizedInput {
            input_ids: vec![101, 1045, 2293, 2023, 102], // Sample token IDs
            attention_mask: vec![1, 1, 1, 1, 1],
            token_count: 5,
        };
        
        let model_input = converter.tokenized_to_tensors(&tokenized).unwrap();
        
        assert_eq!(model_input.input_ids.shape().dims(), &[1, 5]);
        assert_eq!(model_input.attention_mask.shape().dims(), &[1, 5]);
        assert_eq!(model_input.position_ids.shape().dims(), &[1, 5]);
        
        // Validate shapes
        converter.validate_tensor_shapes(&model_input).unwrap();
    }
}
```

## Next Task
→ Task 031: Implement transformer forward pass