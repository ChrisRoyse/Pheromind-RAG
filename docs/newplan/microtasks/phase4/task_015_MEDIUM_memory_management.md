# Task 015 - MEDIUM: Optimize Memory Management for Large Models

## Priority: MEDIUM
## Estimated Time: 10 minutes
## Dependencies: Task 014

## Objective
Implement efficient memory management to handle the 267MB model file without excessive memory usage.

## Current Issue
- Model loading may use excessive memory
- No memory pooling or efficient allocation
- Risk of OOM with large models

## Tasks
1. **Implement memory pool** (5 min)
   ```rust
   // In src/ml/memory_pool.rs
   use std::sync::{Arc, Mutex};
   use std::collections::VecDeque;
   use anyhow::Result;
   
   pub struct MemoryPool {
       pools: Arc<Mutex<std::collections::HashMap<usize, VecDeque<Vec<u8>>>>>,
       max_pool_size: usize,
   }
   
   impl MemoryPool {
       pub fn new(max_pool_size: usize) -> Self {
           Self {
               pools: Arc::new(Mutex::new(std::collections::HashMap::new())),
               max_pool_size,
           }
       }
       
       pub fn get_buffer(&self, size: usize) -> Vec<u8> {
           let mut pools = self.pools.lock().unwrap();
           
           if let Some(pool) = pools.get_mut(&size) {
               if let Some(mut buffer) = pool.pop_front() {
                   buffer.clear();
                   buffer.resize(size, 0);
                   return buffer;
               }
           }
           
           // Create new buffer if none available
           vec![0u8; size]
       }
       
       pub fn return_buffer(&self, buffer: Vec<u8>) {
           let size = buffer.capacity();
           let mut pools = self.pools.lock().unwrap();
           
           let pool = pools.entry(size).or_insert_with(VecDeque::new);
           if pool.len() < self.max_pool_size {
               pool.push_back(buffer);
           }
           // Buffer will be dropped if pool is full
       }
   }
   
   // Global memory pool instance
   lazy_static::lazy_static! {
       pub static ref MEMORY_POOL: MemoryPool = MemoryPool::new(10);
   }
   ```

2. **Add streaming tensor loader** (4 min)
   ```rust
   // Update tensor_loader.rs for streaming
   pub fn load_tensor_streaming<R: Read + Seek>(
       reader: &mut R,
       tensor_info: &TensorInfo,
       device: &Device,
   ) -> Result<Tensor> {
       // Use memory pool for buffer allocation
       let buffer = MEMORY_POOL.get_buffer(tensor_info.size as usize);
       
       reader.seek(SeekFrom::Start(tensor_info.offset))?;
       
       match tensor_info.data_type {
           GGUFDataType::Q4_K_M => {
               // Process in chunks to reduce memory pressure
               load_quantized_streaming(reader, tensor_info, device, buffer)
           },
           GGUFDataType::F32 => {
               load_f32_streaming(reader, tensor_info, device, buffer)
           },
           _ => load_tensor(reader, tensor_info, device), // Fallback
       }
   }
   
   fn load_quantized_streaming<R: Read>(
       reader: &mut R,
       tensor_info: &TensorInfo,
       device: &Device,
       mut buffer: Vec<u8>,
   ) -> Result<Tensor> {
       let element_count = tensor_info.dimensions.iter().product::<u64>() as usize;
       let mut result = Vec::with_capacity(element_count);
       
       // Process in 64KB chunks
       const CHUNK_SIZE: usize = 64 * 1024;
       let mut remaining = tensor_info.size as usize;
       
       while remaining > 0 {
           let chunk_size = remaining.min(CHUNK_SIZE);
           buffer.resize(chunk_size, 0);
           reader.read_exact(&buffer[..chunk_size])?;
           
           // Process chunk
           let chunk_result = crate::ml::quantization::dequantize_q4_k_m_chunk(
               &buffer[..chunk_size],
               chunk_size
           )?;
           result.extend(chunk_result);
           
           remaining -= chunk_size;
       }
       
       // Return buffer to pool
       MEMORY_POOL.return_buffer(buffer);
       
       result.truncate(element_count);
       let shape: Vec<usize> = tensor_info.dimensions.iter().map(|&d| d as usize).collect();
       Tensor::from_vec(result, &shape, device)
   }
   ```

3. **Add memory monitoring** (1 min)
   ```rust
   pub fn get_memory_usage() -> MemoryUsage {
       let current_rss = get_current_rss();
       let peak_rss = get_peak_rss();
       
       MemoryUsage {
           current_mb: current_rss / 1024 / 1024,
           peak_mb: peak_rss / 1024 / 1024,
       }
   }
   
   #[derive(Debug)]
   pub struct MemoryUsage {
       pub current_mb: u64,
       pub peak_mb: u64,
   }
   
   #[cfg(target_os = "linux")]
   fn get_current_rss() -> u64 {
       let status = std::fs::read_to_string("/proc/self/status").unwrap_or_default();
       // Parse VmRSS line
       0 // Placeholder
   }
   
   #[cfg(not(target_os = "linux"))]
   fn get_current_rss() -> u64 {
       0 // Placeholder for other platforms
   }
   ```

## Success Criteria
- [ ] Memory pool implementation works
- [ ] Streaming tensor loading reduces memory usage
- [ ] Memory monitoring provides useful info
- [ ] Model loading uses <1GB RAM
- [ ] No memory leaks detected

## Files to Create
- `src/ml/memory_pool.rs`

## Files to Modify
- `src/ml/tensor_loader.rs`
- `src/ml/quantization.rs` (add chunk processing)
- `Cargo.toml` (add lazy_static dependency)

## Next Task
→ Task 016: Remove Sled database code completely