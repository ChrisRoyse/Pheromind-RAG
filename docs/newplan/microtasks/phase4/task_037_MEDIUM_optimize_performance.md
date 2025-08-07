# Task 037 - MEDIUM: Optimize Embedding Generation Performance

## Priority: MEDIUM
## Estimated Time: 10 minutes
## Dependencies: Task 036

## Objective
Implement performance optimizations for the embedding generation pipeline to achieve production-ready speeds.

## Current Issue
- Embedding generation may be slow
- Memory usage not optimized
- GPU utilization could be improved

## Tasks
1. **Implement performance profiling** (3 min)
   ```rust
   // In src/ml/performance.rs
   use std::time::{Duration, Instant};
   use std::collections::HashMap;
   use serde::{Serialize, Deserialize};
   
   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct PerformanceMetrics {
       pub tokenization_ms: f64,
       pub tensor_conversion_ms: f64,
       pub model_inference_ms: f64,
       pub pooling_ms: f64,
       pub normalization_ms: f64,
       pub total_ms: f64,
       pub tokens_per_second: f64,
       pub memory_peak_mb: f64,
       pub gpu_utilization_percent: f64,
   }
   
   impl Default for PerformanceMetrics {
       fn default() -> Self {
           Self {
               tokenization_ms: 0.0,
               tensor_conversion_ms: 0.0,
               model_inference_ms: 0.0,
               pooling_ms: 0.0,
               normalization_ms: 0.0,
               total_ms: 0.0,
               tokens_per_second: 0.0,
               memory_peak_mb: 0.0,
               gpu_utilization_percent: 0.0,
           }
       }
   }
   
   pub struct PerformanceProfiler {
       stage_timers: HashMap<String, Instant>,
       metrics: PerformanceMetrics,
       start_time: Instant,
       total_tokens: usize,
   }
   
   impl PerformanceProfiler {
       pub fn new() -> Self {
           Self {
               stage_timers: HashMap::new(),
               metrics: PerformanceMetrics::default(),
               start_time: Instant::now(),
               total_tokens: 0,
           }
       }
       
       pub fn start_stage(&mut self, stage: &str) {
           self.stage_timers.insert(stage.to_string(), Instant::now());
       }
       
       pub fn end_stage(&mut self, stage: &str) -> Duration {
           if let Some(start_time) = self.stage_timers.remove(stage) {
               let duration = start_time.elapsed();
               let duration_ms = duration.as_secs_f64() * 1000.0;
               
               match stage {
                   "tokenization" => self.metrics.tokenization_ms += duration_ms,
                   "tensor_conversion" => self.metrics.tensor_conversion_ms += duration_ms,
                   "model_inference" => self.metrics.model_inference_ms += duration_ms,
                   "pooling" => self.metrics.pooling_ms += duration_ms,
                   "normalization" => self.metrics.normalization_ms += duration_ms,
                   _ => {}, // Unknown stage
               }
               
               duration
           } else {
               Duration::ZERO
           }
       }
       
       pub fn add_tokens(&mut self, token_count: usize) {
           self.total_tokens += token_count;
       }
       
       pub fn finalize(mut self) -> PerformanceMetrics {
           self.metrics.total_ms = self.start_time.elapsed().as_secs_f64() * 1000.0;
           
           if self.metrics.total_ms > 0.0 {
               self.metrics.tokens_per_second = 
                   (self.total_tokens as f64) / (self.metrics.total_ms / 1000.0);
           }
           
           // TODO: Add actual memory and GPU monitoring
           self.metrics.memory_peak_mb = self.estimate_memory_usage();
           self.metrics.gpu_utilization_percent = self.estimate_gpu_usage();
           
           self.metrics
       }
       
       fn estimate_memory_usage(&self) -> f64 {
           // Placeholder - would integrate with actual memory monitoring
           0.0
       }
       
       fn estimate_gpu_usage(&self) -> f64 {
           // Placeholder - would integrate with CUDA/Metal monitoring
           0.0
       }
   }
   ```

2. **Implement caching optimizations** (4 min)
   ```rust
   // Performance optimization strategies
   pub struct PerformanceOptimizer {
       enable_tensor_caching: bool,
       enable_attention_caching: bool,
       use_mixed_precision: bool,
       optimize_memory_layout: bool,
   }
   
   impl PerformanceOptimizer {
       pub fn new() -> Self {
           Self {
               enable_tensor_caching: true,
               enable_attention_caching: false, // Disabled by default due to memory usage
               use_mixed_precision: false,      // Disabled by default for accuracy
               optimize_memory_layout: true,
           }
       }
       
       pub fn optimize_for_throughput(mut self) -> Self {
           self.enable_tensor_caching = true;
           self.enable_attention_caching = true;
           self.optimize_memory_layout = true;
           self
       }
       
       pub fn optimize_for_latency(mut self) -> Self {
           self.enable_tensor_caching = true;
           self.enable_attention_caching = false;
           self.use_mixed_precision = false;
           self.optimize_memory_layout = true;
           self
       }
       
       pub fn optimize_for_memory(mut self) -> Self {
           self.enable_tensor_caching = false;
           self.enable_attention_caching = false;
           self.use_mixed_precision = true;
           self.optimize_memory_layout = true;
           self
       }
   }
   
   // Tensor operation optimizations
   pub struct TensorOptimizer;
   
   impl TensorOptimizer {
       pub fn optimize_matmul(a: &candle_core::Tensor, b: &candle_core::Tensor) -> candle_core::Result<candle_core::Tensor> {
           // Check if we can use more efficient operations
           let a_shape = a.shape();
           let b_shape = b.shape();
           
           // Use specialized kernels for common shapes
           if a_shape.dims().len() == 2 && b_shape.dims().len() == 2 {
               // Use optimized GEMM
               return a.matmul(b);
           }
           
           // Fall back to standard matmul
           a.matmul(b)
       }
       
       pub fn optimize_attention(
           queries: &candle_core::Tensor,
           keys: &candle_core::Tensor,
           values: &candle_core::Tensor,
           mask: Option<&candle_core::Tensor>,
       ) -> candle_core::Result<candle_core::Tensor> {
           // Implement flash attention or other optimized attention
           let attention_scores = queries.matmul(&keys.transpose(-2, -1)?)?;
           
           let attention_scores = if let Some(mask) = mask {
               // Apply mask efficiently
               mask.where_cond(
                   &attention_scores,
                   &candle_core::Tensor::full(
                       f32::NEG_INFINITY,
                       attention_scores.shape(),
                       attention_scores.device(),
                   )?,
               )?
           } else {
               attention_scores
           };
           
           let attention_probs = candle_nn::ops::softmax(&attention_scores, -1)?;
           attention_probs.matmul(values)
       }
   }
   ```

3. **Add batch size optimization** (3 min)
   ```rust
   pub struct BatchSizeOptimizer {
       min_batch_size: usize,
       max_batch_size: usize,
       target_memory_mb: f64,
       target_latency_ms: f64,
   }
   
   impl BatchSizeOptimizer {
       pub fn new(min_batch: usize, max_batch: usize) -> Self {
           Self {
               min_batch_size: min_batch,
               max_batch_size: max_batch,
               target_memory_mb: 1024.0, // 1GB default
               target_latency_ms: 1000.0, // 1 second default
           }
       }
       
       pub fn find_optimal_batch_size(
           &self,
           sequence_length: usize,
           embedding_dim: usize,
           available_memory_mb: f64,
       ) -> usize {
           // Estimate memory per item
           let memory_per_item_mb = self.estimate_memory_per_item(
               sequence_length,
               embedding_dim,
           );
           
           if memory_per_item_mb <= 0.0 {
               return self.max_batch_size;
           }
           
           // Calculate max batch size based on memory
           let memory_limited_batch = ((available_memory_mb * 0.8) / memory_per_item_mb) as usize;
           
           // Clamp to our limits
           memory_limited_batch
               .max(self.min_batch_size)
               .min(self.max_batch_size)
       }
       
       fn estimate_memory_per_item(
           &self,
           sequence_length: usize,
           embedding_dim: usize,
       ) -> f64 {
           // Rough estimation of memory usage per item in MB
           let input_tokens_mb = (sequence_length * 8) as f64 / (1024.0 * 1024.0); // i64
           let attention_mask_mb = (sequence_length * 4) as f64 / (1024.0 * 1024.0); // f32
           let hidden_states_mb = (sequence_length * embedding_dim * 4) as f64 / (1024.0 * 1024.0); // f32
           let embeddings_mb = (embedding_dim * 4) as f64 / (1024.0 * 1024.0); // f32
           
           // Include intermediate activations (rough multiplier)
           (input_tokens_mb + attention_mask_mb + hidden_states_mb + embeddings_mb) * 3.0
       }
       
       pub async fn benchmark_batch_size(
           &self,
           processor: &crate::ml::batch_processor::BatchEmbeddingProcessor,
           test_texts: &[&str],
           target_batch_size: usize,
       ) -> Result<PerformanceMetrics, crate::ml::errors::EmbeddingError> {
           let batch = &test_texts[..target_batch_size.min(test_texts.len())];
           
           let mut profiler = PerformanceProfiler::new();
           
           profiler.start_stage("total");
           let result = processor.process_batch(batch).await?;
           profiler.end_stage("total");
           
           profiler.add_tokens(result.tokens_processed);
           
           let mut metrics = profiler.finalize();
           metrics.memory_peak_mb = result.memory_used_mb;
           
           Ok(metrics)
       }
   }
   
   // Adaptive performance tuning
   pub struct AdaptivePerformanceTuner {
       batch_optimizer: BatchSizeOptimizer,
       performance_history: Vec<PerformanceMetrics>,
       current_batch_size: usize,
   }
   
   impl AdaptivePerformanceTuner {
       pub fn new(initial_batch_size: usize) -> Self {
           Self {
               batch_optimizer: BatchSizeOptimizer::new(1, 128),
               performance_history: Vec::new(),
               current_batch_size: initial_batch_size,
           }
       }
       
       pub fn record_performance(&mut self, metrics: PerformanceMetrics) {
           self.performance_history.push(metrics);
           
           // Keep only recent history
           if self.performance_history.len() > 10 {
               self.performance_history.drain(0..1);
           }
           
           self.adjust_batch_size();
       }
       
       fn adjust_batch_size(&mut self) {
           if self.performance_history.len() < 3 {
               return; // Need more data
           }
           
           let recent_metrics: Vec<_> = self.performance_history.iter()
               .rev()
               .take(3)
               .collect();
           
           let avg_latency = recent_metrics.iter()
               .map(|m| m.total_ms)
               .sum::<f64>() / recent_metrics.len() as f64;
           
           let avg_throughput = recent_metrics.iter()
               .map(|m| m.tokens_per_second)
               .sum::<f64>() / recent_metrics.len() as f64;
           
           // Adjust based on performance targets
           if avg_latency > 2000.0 && self.current_batch_size > 1 {
               // Too slow, reduce batch size
               self.current_batch_size = (self.current_batch_size / 2).max(1);
           } else if avg_latency < 500.0 && avg_throughput > 100.0 {
               // Fast enough, try increasing batch size
               self.current_batch_size = (self.current_batch_size * 2).min(128);
           }
       }
       
       pub fn recommended_batch_size(&self) -> usize {
           self.current_batch_size
       }
   }
   ```

## Success Criteria
- [ ] Performance profiling works accurately
- [ ] Batch size optimization improves throughput
- [ ] Memory usage is optimized
- [ ] Tensor operations are efficient
- [ ] Adaptive tuning adjusts parameters
- [ ] Overall pipeline performance improved

## Files to Create
- `src/ml/performance.rs`

## Files to Modify
- `src/ml/mod.rs`
- `src/ml/batch_processor.rs` (integrate profiling)

## Performance Targets
- Tokenization: <10ms per 1000 tokens
- Model inference: <100ms per batch (batch size 8)
- Total pipeline: <200ms per batch
- Memory usage: <2GB for typical workloads
- Throughput: >50 embeddings/second

## Next Task
→ Task 038: Create comprehensive integration tests