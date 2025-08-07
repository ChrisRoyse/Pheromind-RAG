# Task 031 - CRITICAL: Implement Transformer Forward Pass

## Priority: CRITICAL
## Estimated Time: 10 minutes
## Dependencies: Task 030

## Objective
Implement the core transformer forward pass using loaded model weights and tensors.

## Current Issue
- No forward pass implementation
- Need attention mechanism and feed-forward layers
- Must integrate with loaded GGUF weights

## Tasks
1. **Implement attention mechanism** (4 min)
   ```rust
   // In src/ml/transformer.rs
   use candle_core::{Tensor, Device, Result as CandleResult};
   use candle_nn::{Linear, LayerNorm, Activation};
   use anyhow::Result;
   use crate::ml::model_layers::{TransformerLayer, ModelStructure};
   use crate::ml::tensor_conversion::ModelInput;
   
   pub struct NomicTransformer {
       token_embeddings: Tensor,
       layers: Vec<TransformerLayerImpl>,
       output_norm: LayerNorm,
       embedding_dim: usize,
       num_heads: usize,
       head_dim: usize,
       device: Device,
   }
   
   impl NomicTransformer {
       pub fn new(model_structure: ModelStructure, device: Device) -> Result<Self> {
           let embedding_dim = model_structure.embedding_dim;
           let num_heads = 12; // Nomic model has 12 attention heads
           let head_dim = embedding_dim / num_heads;
           
           // Convert model layers to implementations
           let mut layer_impls = Vec::new();
           for layer in model_structure.layers {
               let layer_impl = TransformerLayerImpl::new(layer, embedding_dim, num_heads, &device)?;
               layer_impls.push(layer_impl);
           }
           
           // Create output layer norm
           let output_norm = LayerNorm::new(
               model_structure.output_norm,
               1e-12, // Default epsilon for BERT-style models
           );
           
           Ok(Self {
               token_embeddings: model_structure.token_embeddings,
               layers: layer_impls,
               output_norm,
               embedding_dim,
               num_heads,
               head_dim,
               device,
           })
       }
       
       pub fn forward(&self, input: &ModelInput) -> Result<Tensor> {
           let batch_size = input.batch_size()?;
           let seq_len = input.sequence_length()?;
           
           // Get token embeddings
           let mut hidden_states = self.get_embeddings(&input.input_ids)?;
           
           // Apply transformer layers
           for (i, layer) in self.layers.iter().enumerate() {
               hidden_states = layer.forward(
                   &hidden_states,
                   &input.attention_mask,
               ).map_err(|e| anyhow::anyhow!("Layer {} forward failed: {}", i, e))?;
           }
           
           // Apply output normalization
           hidden_states = self.output_norm.forward(&hidden_states)
               .map_err(|e| anyhow::anyhow!("Output norm failed: {}", e))?;
           
           Ok(hidden_states)
       }
       
       fn get_embeddings(&self, input_ids: &Tensor) -> CandleResult<Tensor> {
           // Perform embedding lookup
           self.token_embeddings.embedding(input_ids)
       }
   }
   
   struct TransformerLayerImpl {
       attention: MultiHeadAttention,
       feed_forward: FeedForward,
       attention_norm: LayerNorm,
       ffn_norm: LayerNorm,
   }
   
   impl TransformerLayerImpl {
       fn new(
           layer: TransformerLayer,
           embedding_dim: usize,
           num_heads: usize,
           device: &Device,
       ) -> Result<Self> {
           let attention = MultiHeadAttention::new(
               layer.attention_q,
               layer.attention_k,
               layer.attention_v,
               layer.attention_o,
               embedding_dim,
               num_heads,
           )?;
           
           let feed_forward = FeedForward::new(
               layer.ffn_up,
               layer.ffn_down,
               embedding_dim,
           )?;
           
           let attention_norm = LayerNorm::new(layer.attention_norm, 1e-12);
           let ffn_norm = LayerNorm::new(layer.ffn_norm, 1e-12);
           
           Ok(Self {
               attention,
               feed_forward,
               attention_norm,
               ffn_norm,
           })
       }
       
       fn forward(
           &self,
           hidden_states: &Tensor,
           attention_mask: &Tensor,
       ) -> CandleResult<Tensor> {
           // Pre-norm architecture (like RoBERTa)
           
           // Attention block with residual connection
           let normed_states = self.attention_norm.forward(hidden_states)?;
           let attention_output = self.attention.forward(&normed_states, attention_mask)?;
           let hidden_states = (hidden_states + &attention_output)?;
           
           // Feed-forward block with residual connection
           let normed_states = self.ffn_norm.forward(&hidden_states)?;
           let ffn_output = self.feed_forward.forward(&normed_states)?;
           let hidden_states = (hidden_states + ffn_output)?;
           
           Ok(hidden_states)
       }
   }
   ```

2. **Implement attention mechanism** (4 min)
   ```rust
   struct MultiHeadAttention {
       query_proj: Tensor,
       key_proj: Tensor,
       value_proj: Tensor,
       output_proj: Tensor,
       num_heads: usize,
       head_dim: usize,
       scale: f64,
   }
   
   impl MultiHeadAttention {
       fn new(
           query_weight: Tensor,
           key_weight: Tensor,
           value_weight: Tensor,
           output_weight: Tensor,
           embedding_dim: usize,
           num_heads: usize,
       ) -> Result<Self> {
           let head_dim = embedding_dim / num_heads;
           let scale = 1.0 / ((head_dim as f64).sqrt());
           
           Ok(Self {
               query_proj: query_weight,
               key_proj: key_weight,
               value_proj: value_weight,
               output_proj: output_weight,
               num_heads,
               head_dim,
               scale,
           })
       }
       
       fn forward(
           &self,
           hidden_states: &Tensor,
           attention_mask: &Tensor,
       ) -> CandleResult<Tensor> {
           let (batch_size, seq_len, embedding_dim) = hidden_states.shape().dims3()?;
           
           // Project to Q, K, V
           let queries = hidden_states.matmul(&self.query_proj.t()?)?;
           let keys = hidden_states.matmul(&self.key_proj.t()?)?;
           let values = hidden_states.matmul(&self.value_proj.t()?)?;
           
           // Reshape for multi-head attention
           let queries = queries.reshape(&[
               batch_size,
               seq_len,
               self.num_heads,
               self.head_dim,
           ])?.transpose(1, 2)?; // [batch, heads, seq_len, head_dim]
           
           let keys = keys.reshape(&[
               batch_size,
               seq_len,
               self.num_heads,
               self.head_dim,
           ])?.transpose(1, 2)?;
           
           let values = values.reshape(&[
               batch_size,
               seq_len,
               self.num_heads,
               self.head_dim,
           ])?.transpose(1, 2)?;
           
           // Compute attention scores
           let attention_scores = queries.matmul(&keys.transpose(2, 3)?)?;
           let attention_scores = (attention_scores * self.scale)?;
           
           // Apply attention mask
           let attention_scores = self.apply_attention_mask(
               &attention_scores,
               attention_mask,
           )?;
           
           // Apply softmax
           let attention_probs = candle_nn::ops::softmax(&attention_scores, 3)?;
           
           // Apply attention to values
           let attention_output = attention_probs.matmul(&values)?;
           
           // Reshape back
           let attention_output = attention_output
               .transpose(1, 2)?
               .reshape(&[batch_size, seq_len, embedding_dim])?;
           
           // Final projection
           attention_output.matmul(&self.output_proj.t()?)
       }
       
       fn apply_attention_mask(
           &self,
           attention_scores: &Tensor,
           attention_mask: &Tensor,
       ) -> CandleResult<Tensor> {
           // Expand mask to match attention scores shape
           let mask = attention_mask
               .unsqueeze(1)? // Add head dimension
               .unsqueeze(1)? // Add query dimension
               .expand(attention_scores.shape())?;
           
           // Apply mask (set masked positions to large negative value)
           let masked_scores = mask.where_cond(
               attention_scores,
               &Tensor::full(f32::NEG_INFINITY, attention_scores.shape(), attention_scores.device())?,
           )?;
           
           Ok(masked_scores)
       }
   }
   ```

3. **Implement feed-forward network** (2 min)
   ```rust
   struct FeedForward {
       up_proj: Tensor,
       down_proj: Tensor,
       activation: Activation,
   }
   
   impl FeedForward {
       fn new(
           up_weight: Tensor,
           down_weight: Tensor,
           embedding_dim: usize,
       ) -> Result<Self> {
           Ok(Self {
               up_proj: up_weight,
               down_proj: down_weight,
               activation: Activation::Gelu, // BERT uses GELU
           })
       }
       
       fn forward(&self, hidden_states: &Tensor) -> CandleResult<Tensor> {
           // Up projection
           let intermediate = hidden_states.matmul(&self.up_proj.t()?)?;
           
           // Apply activation
           let intermediate = match self.activation {
               Activation::Gelu => {
                   // GELU activation: x * 0.5 * (1 + tanh(sqrt(2/π) * (x + 0.044715 * x³)))
                   let x = &intermediate;
                   let x3 = x.powf(3.0)?;
                   let inner = (x + x3 * 0.044715)? * (2.0_f64 / std::f64::consts::PI).sqrt();
                   let tanh_inner = inner.tanh()?;
                   (x * 0.5)? * (tanh_inner + 1.0)?
               },
               _ => return Err(candle_core::Error::UnsupportedDType { dt: candle_core::DType::F32 }),
           };
           
           // Down projection
           intermediate.matmul(&self.down_proj.t()?)
       }
   }
   ```

## Success Criteria
- [ ] Forward pass compiles and runs
- [ ] Attention mechanism works correctly
- [ ] Feed-forward layers function
- [ ] Layer normalization applied
- [ ] Residual connections implemented
- [ ] Output tensors have correct shapes

## Files to Create
- `src/ml/transformer.rs`

## Files to Modify
- `src/ml/mod.rs`

## Validation
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use candle_core::Device;
    
    #[test]
    fn test_attention_shapes() {
        let device = Device::Cpu;
        
        // Create dummy weights for testing
        let embedding_dim = 768;
        let seq_len = 128;
        let batch_size = 1;
        
        let hidden_states = Tensor::randn(
            0.0, 1.0, 
            &[batch_size, seq_len, embedding_dim], 
            &device
        ).unwrap();
        
        let attention_mask = Tensor::ones(
            &[batch_size, seq_len], 
            candle_core::DType::F32, 
            &device
        ).unwrap();
        
        // Test that shapes work correctly
        assert_eq!(hidden_states.shape().dims(), &[1, 128, 768]);
        assert_eq!(attention_mask.shape().dims(), &[1, 128]);
    }
}
```

## Next Task
→ Task 032: Implement mean pooling for embeddings