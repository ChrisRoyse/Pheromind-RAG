# Task 035 - MEDIUM: Handle Batch Processing for Multiple Texts

## Priority: MEDIUM
## Estimated Time: 10 minutes
## Dependencies: Task 034

## Objective
Implement efficient batch processing to handle multiple text inputs simultaneously for better throughput.

## Current Issue
- Single text processing is inefficient
- Need batching for production workloads
- Memory and performance optimization required

## Tasks
1. **Create batch processor** (5 min)
   ```rust
   // In src/ml/batch_processor.rs
   use crate::ml::{
       tokenizer::{EmbeddingTokenizer, TokenizedInput},
       tensor_conversion::{TensorConverter, ModelInput},
       transformer::NomicTransformer,
       pooling::{MeanPooler, AdaptivePooler, PoolingStrategy},
       normalization::L2Normalizer,
       dimension_validator::EmbeddingPipelineValidator,
   };
   use candle_core::{Tensor, Device};
   use anyhow::{Result, anyhow};
   use std::time::Instant;
   
   pub struct BatchEmbeddingProcessor {
       tokenizer: EmbeddingTokenizer,
       tensor_converter: TensorConverter,
       transformer: NomicTransformer,
       pooler: AdaptivePooler,
       normalizer: L2Normalizer,
       validator: EmbeddingPipelineValidator,
       max_batch_size: usize,
       embedding_dim: usize,
   }
   
   impl BatchEmbeddingProcessor {
       pub async fn new(
           device: Device,
           max_batch_size: usize,
           embedding_dim: usize,
       ) -> Result<Self> {
           let tokenizer = EmbeddingTokenizer::new().await?;
           let tensor_converter = TensorConverter::new(device.clone(), tokenizer.max_length());
           
           // Load transformer model (simplified - would need actual model loading)
           let transformer = NomicTransformer::new(/* model_structure */, device)?;
           
           let pooler = AdaptivePooler::new(embedding_dim, PoolingStrategy::Mean);
           let normalizer = L2Normalizer::new();
           let validator = EmbeddingPipelineValidator::new(embedding_dim, tokenizer.max_length())
               .with_max_batch_size(max_batch_size);
           
           Ok(Self {
               tokenizer,
               tensor_converter,
               transformer,
               pooler,
               normalizer,
               validator,
               max_batch_size,
               embedding_dim,
           })
       }
       
       pub async fn process_batch(&self, texts: &[&str]) -> Result<BatchEmbeddingResult> {
           if texts.is_empty() {
               return Ok(BatchEmbeddingResult::empty());
           }
           
           let start_time = Instant::now();
           
           // Split into manageable chunks if too large
           if texts.len() > self.max_batch_size {
               return self.process_large_batch(texts).await;
           }
           
           let batch_result = self.process_single_batch(texts).await?;
           
           Ok(BatchEmbeddingResult {
               embeddings: batch_result.embeddings,
               processing_time: start_time.elapsed(),
               batch_size: texts.len(),
               tokens_processed: batch_result.tokens_processed,
               memory_used_mb: batch_result.memory_used_mb,
           })
       }
       
       async fn process_single_batch(&self, texts: &[&str]) -> Result<SingleBatchResult> {
           // Step 1: Tokenization
           let tokenized_inputs = self.tokenizer.tokenize_batch(texts)?;
           let total_tokens: usize = tokenized_inputs.iter().map(|t| t.token_count).sum();
           
           // Step 2: Convert to tensors
           let model_input = self.tensor_converter.batch_tokenized_to_tensors(&tokenized_inputs)?;
           
           // Step 3: Validate dimensions
           self.validator.validate_tokenizer_output(
               &model_input.input_ids,
               &model_input.attention_mask,
           )?;
           
           // Step 4: Transformer forward pass
           let hidden_states = self.transformer.forward(&model_input)?;
           self.validator.validate_transformer_output(&hidden_states)?;
           
           // Step 5: Pooling
           let pooled_embeddings = self.pooler.forward(&hidden_states, &model_input.attention_mask)?;
           self.validator.validate_pooled_output(&pooled_embeddings)?;
           
           // Step 6: Normalization
           let normalized_embeddings = self.normalizer.normalize(&pooled_embeddings)?;
           self.validator.validate_final_embeddings(&normalized_embeddings)?;
           
           // Step 7: Convert to Vec<Vec<f32>> for easier handling
           let embeddings = self.tensor_to_embeddings(normalized_embeddings)?;
           
           // Calculate memory usage
           let memory_used_mb = self.estimate_memory_usage(&model_input, &hidden_states, &pooled_embeddings);
           
           Ok(SingleBatchResult {
               embeddings,
               tokens_processed: total_tokens,
               memory_used_mb,
           })
       }
       
       async fn process_large_batch(&self, texts: &[&str]) -> Result<BatchEmbeddingResult> {
           let start_time = Instant::now();
           let mut all_embeddings = Vec::new();
           let mut total_tokens = 0;
           let mut max_memory_mb = 0.0;
           
           // Process in chunks
           for chunk in texts.chunks(self.max_batch_size) {
               let chunk_result = self.process_single_batch(chunk).await?;
               
               all_embeddings.extend(chunk_result.embeddings);
               total_tokens += chunk_result.tokens_processed;
               max_memory_mb = max_memory_mb.max(chunk_result.memory_used_mb);
           }
           
           Ok(BatchEmbeddingResult {
               embeddings: all_embeddings,
               processing_time: start_time.elapsed(),
               batch_size: texts.len(),
               tokens_processed: total_tokens,
               memory_used_mb: max_memory_mb,
           })
       }
   }
   
   #[derive(Debug)]
   struct SingleBatchResult {
       embeddings: Vec<Vec<f32>>,
       tokens_processed: usize,
       memory_used_mb: f64,
   }
   
   #[derive(Debug)]
   pub struct BatchEmbeddingResult {
       pub embeddings: Vec<Vec<f32>>,
       pub processing_time: std::time::Duration,
       pub batch_size: usize,
       pub tokens_processed: usize,
       pub memory_used_mb: f64,
   }
   ```

2. **Add utility methods and optimizations** (3 min)
   ```rust
   impl BatchEmbeddingProcessor {
       fn tensor_to_embeddings(&self, tensor: Tensor) -> Result<Vec<Vec<f32>>> {
           let shape = tensor.shape();
           let dims = shape.dims();
           
           if dims.len() != 2 {
               return Err(anyhow!("Expected 2D tensor for embeddings, got shape: {:?}", dims));
           }
           
           let batch_size = dims[0];
           let embedding_dim = dims[1];
           
           // Convert tensor to flat Vec<f32>
           let flat_data: Vec<f32> = tensor.to_vec2()
               .map_err(|e| anyhow!("Failed to convert tensor to vec: {}", e))?
               .into_iter()
               .flatten()
               .collect();
           
           // Reshape into Vec<Vec<f32>>
           let mut embeddings = Vec::with_capacity(batch_size);
           for i in 0..batch_size {
               let start = i * embedding_dim;
               let end = start + embedding_dim;
               embeddings.push(flat_data[start..end].to_vec());
           }
           
           Ok(embeddings)
       }
       
       fn estimate_memory_usage(
           &self,
           model_input: &ModelInput,
           hidden_states: &Tensor,
           pooled_embeddings: &Tensor,
       ) -> f64 {
           use crate::ml::dimension_validator::MemoryEstimator;
           
           let input_memory = MemoryEstimator::estimate_tensor_memory(&model_input.input_ids);
           let mask_memory = MemoryEstimator::estimate_tensor_memory(&model_input.attention_mask);
           let hidden_memory = MemoryEstimator::estimate_tensor_memory(hidden_states);
           let pooled_memory = MemoryEstimator::estimate_tensor_memory(pooled_embeddings);
           
           input_memory + mask_memory + hidden_memory + pooled_memory
       }
       
       pub fn optimal_batch_size(&self, available_memory_mb: f64) -> usize {
           // Estimate memory usage per sequence
           let seq_len = 512; // Average sequence length
           let memory_per_seq = crate::ml::dimension_validator::MemoryEstimator::estimate_pipeline_memory(
               1,
               seq_len,
               self.embedding_dim,
               12, // Typical transformer layers
           );
           
           if memory_per_seq <= 0.0 {
               return self.max_batch_size;
           }
           
           let optimal_batch = ((available_memory_mb * 0.8) / memory_per_seq) as usize; // Use 80% of available memory
           optimal_batch.max(1).min(self.max_batch_size)
       }
       
       pub async fn process_with_auto_batching(
           &self,
           texts: &[&str],
           available_memory_mb: Option<f64>,
       ) -> Result<BatchEmbeddingResult> {
           let batch_size = if let Some(memory_mb) = available_memory_mb {
               self.optimal_batch_size(memory_mb)
           } else {
               self.max_batch_size
           };
           
           if texts.len() <= batch_size {
               self.process_batch(texts).await
           } else {
               self.process_with_dynamic_batching(texts, batch_size).await
           }
       }
       
       async fn process_with_dynamic_batching(
           &self,
           texts: &[&str],
           initial_batch_size: usize,
       ) -> Result<BatchEmbeddingResult> {
           let start_time = Instant::now();
           let mut all_embeddings = Vec::new();
           let mut total_tokens = 0;
           let mut max_memory_mb = 0.0;
           let mut current_batch_size = initial_batch_size;
           
           let mut remaining_texts = texts;
           
           while !remaining_texts.is_empty() {
               let chunk_size = current_batch_size.min(remaining_texts.len());
               let (chunk, rest) = remaining_texts.split_at(chunk_size);
               
               match self.process_single_batch(chunk).await {
                   Ok(result) => {
                       all_embeddings.extend(result.embeddings);
                       total_tokens += result.tokens_processed;
                       max_memory_mb = max_memory_mb.max(result.memory_used_mb);
                       remaining_texts = rest;
                   },
                   Err(e) if chunk_size > 1 => {
                       // If batch processing failed, try smaller batch size
                       current_batch_size = (current_batch_size / 2).max(1);
                       eprintln!("Batch processing failed, reducing batch size to {}: {}", current_batch_size, e);
                       continue;
                   },
                   Err(e) => {
                       // Even single item failed, propagate error
                       return Err(e);
                   },
               }
           }
           
           Ok(BatchEmbeddingResult {
               embeddings: all_embeddings,
               processing_time: start_time.elapsed(),
               batch_size: texts.len(),
               tokens_processed: total_tokens,
               memory_used_mb: max_memory_mb,
           })
       }
   }
   ```

3. **Add result handling and statistics** (2 min)
   ```rust
   impl BatchEmbeddingResult {
       pub fn empty() -> Self {
           Self {
               embeddings: Vec::new(),
               processing_time: std::time::Duration::from_secs(0),
               batch_size: 0,
               tokens_processed: 0,
               memory_used_mb: 0.0,
           }
       }
       
       pub fn throughput_per_second(&self) -> f64 {
           if self.processing_time.is_zero() {
               return 0.0;
           }
           
           self.batch_size as f64 / self.processing_time.as_secs_f64()
       }
       
       pub fn tokens_per_second(&self) -> f64 {
           if self.processing_time.is_zero() {
               return 0.0;
           }
           
           self.tokens_processed as f64 / self.processing_time.as_secs_f64()
       }
       
       pub fn average_processing_time_ms(&self) -> f64 {
           if self.batch_size == 0 {
               return 0.0;
           }
           
           self.processing_time.as_millis() as f64 / self.batch_size as f64
       }
       
       pub fn memory_per_item_mb(&self) -> f64 {
           if self.batch_size == 0 {
               return 0.0;
           }
           
           self.memory_used_mb / self.batch_size as f64
       }
   }
   ```

## Success Criteria
- [ ] Batch processing works for multiple texts
- [ ] Memory usage is optimized
- [ ] Large batches are handled efficiently
- [ ] Dynamic batch sizing works
- [ ] Performance metrics are accurate
- [ ] Error handling is robust

## Files to Create
- `src/ml/batch_processor.rs`

## Files to Modify
- `src/ml/mod.rs`
- `src/ml/embedding_service.rs` (integrate batch processing)

## Validation
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_batch_processing() {
        let processor = create_test_processor().await;
        
        let texts = vec![
            "Hello world",
            "This is a test",
            "Batch processing works",
        ];
        
        let result = processor.process_batch(&texts).await.unwrap();
        
        assert_eq!(result.embeddings.len(), 3);
        assert_eq!(result.batch_size, 3);
        assert!(result.processing_time.as_millis() > 0);
        
        // Each embedding should be normalized
        for embedding in &result.embeddings {
            let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
            assert!((norm - 1.0).abs() < 1e-6);
        }
    }
}
```

## Next Task
→ Task 036: Implement error handling throughout pipeline