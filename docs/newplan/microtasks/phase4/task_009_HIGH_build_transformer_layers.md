# Task 009 - HIGH: Build Transformer Layer Structure

## Priority: HIGH
## Estimated Time: 10 minutes
## Dependencies: Task 008

## Objective
Create transformer layer structure from parsed tensor information to prepare for model construction.

## Current Issue
- Need to organize tensors into transformer layers
- Must handle attention and feed-forward components
- Create proper model hierarchy

## Tasks
1. **Define layer structures** (4 min)
   ```rust
   // In src/ml/model_layers.rs
   use candle_core::{Tensor, Device};
   use std::collections::HashMap;
   
   #[derive(Debug)]
   pub struct TransformerLayer {
       pub layer_id: usize,
       pub attention_norm: Tensor,
       pub attention_q: Tensor,
       pub attention_k: Tensor,
       pub attention_v: Tensor,
       pub attention_o: Tensor,
       pub ffn_norm: Tensor,
       pub ffn_up: Tensor,
       pub ffn_down: Tensor,
   }
   
   #[derive(Debug)]
   pub struct ModelStructure {
       pub token_embeddings: Tensor,
       pub layers: Vec<TransformerLayer>,
       pub output_norm: Tensor,
       pub embedding_dim: usize,
       pub num_layers: usize,
       pub vocab_size: usize,
   }
   ```

2. **Implement layer builder** (5 min)
   ```rust
   pub fn build_model_structure(
       tensors: &[TensorInfo], 
       device: &Device
   ) -> Result<ModelStructure> {
       let mut tensor_map = HashMap::new();
       
       // Group tensors by name pattern
       for tensor in tensors {
           tensor_map.insert(tensor.name.clone(), tensor.clone());
       }
       
       // Extract token embeddings
       let token_embeddings = load_tensor(
           &tensor_map["token_embd.weight"], 
           device
       )?;
       
       // Build transformer layers
       let mut layers = Vec::new();
       let mut layer_id = 0;
       
       while tensor_map.contains_key(&format!("blk.{}.attn_norm.weight", layer_id)) {
           let layer = TransformerLayer {
               layer_id,
               attention_norm: load_tensor(
                   &tensor_map[&format!("blk.{}.attn_norm.weight", layer_id)],
                   device
               )?,
               attention_q: load_tensor(
                   &tensor_map[&format!("blk.{}.attn_q.weight", layer_id)],
                   device
               )?,
               attention_k: load_tensor(
                   &tensor_map[&format!("blk.{}.attn_k.weight", layer_id)],
                   device
               )?,
               attention_v: load_tensor(
                   &tensor_map[&format!("blk.{}.attn_v.weight", layer_id)],
                   device
               )?,
               attention_o: load_tensor(
                   &tensor_map[&format!("blk.{}.attn_o.weight", layer_id)],
                   device
               )?,
               ffn_norm: load_tensor(
                   &tensor_map[&format!("blk.{}.ffn_norm.weight", layer_id)],
                   device
               )?,
               ffn_up: load_tensor(
                   &tensor_map[&format!("blk.{}.ffn_up.weight", layer_id)],
                   device
               )?,
               ffn_down: load_tensor(
                   &tensor_map[&format!("blk.{}.ffn_down.weight", layer_id)],
                   device
               )?,
           };
           layers.push(layer);
           layer_id += 1;
       }
       
       let output_norm = load_tensor(
           &tensor_map["output_norm.weight"],
           device
       )?;
       
       Ok(ModelStructure {
           token_embeddings,
           layers,
           output_norm,
           embedding_dim: 768, // From model config
           num_layers: layer_id,
           vocab_size: 30522, // From model config
       })
   }
   ```

3. **Add helper function** (1 min)
   - Tensor loading from GGUF data
   - Shape validation

## Success Criteria
- [ ] Layer structure compiles
- [ ] All transformer layers identified
- [ ] Attention components mapped correctly
- [ ] Feed-forward layers organized
- [ ] Model hierarchy created

## Files to Create
- `src/ml/model_layers.rs`

## Next Task
→ Task 010: Load embedding layer tensors