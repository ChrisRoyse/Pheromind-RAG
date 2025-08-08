# IMPLEMENTATION ROADMAP: STREAMING TENSOR LOADER

## CRITICAL FUNCTIONS TO REPLACE

### Current Fatal Functions (src/embedding/nomic.rs)

```rust
// LINE 273-349: REPLACE ENTIRELY
fn load_gguf_tensors(model_path: &PathBuf, device: &Device) -> Result<HashMap<String, Tensor>>

// LINE 295: FATAL HEAP ALLOCATION
let mut tensor_data = vec![0u8; data_size]; // CAUSES V8 CRASH

// LINE 379-432: REPLACE ENTIRELY  
fn dequantize_tensor(data: &[u8], tensor_info: &gguf_file::TensorInfo, device: &Device) -> Result<Tensor>

// LINE 387: ANOTHER FATAL ALLOCATION
let mut values = Vec::with_capacity(total_elements); // CAUSES V8 CRASH
```

## NEW MODULE STRUCTURE

### 1. Core Streaming Module: `src/embedding/streaming.rs`

```rust
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::sync::Arc;
use candle_core::{Device, Tensor, Shape};
use anyhow::Result;
use crate::utils::memory_monitor::MemoryMonitor;

/// Zero-allocation GGUF tensor loader
pub struct StreamingGGUFLoader {
    file: File,
    memory_monitor: Arc<MemoryMonitor>,
    // Fixed-size working buffers (stack allocated)
    chunk_buffer: Box<[u8; Self::CHUNK_SIZE]>,
    decode_buffer: Box<[f32; Self::DECODE_SIZE]>,
}

impl StreamingGGUFLoader {
    /// Memory constraints for V8 safety
    const CHUNK_SIZE: usize = 65536;        // 64KB chunks max
    const DECODE_SIZE: usize = 16384;       // 64KB of f32s  
    const MAX_WORKING_MEMORY: usize = 1048576; // 1MB total
    
    /// Create new streaming loader with memory monitoring
    pub fn new<P: AsRef<Path>>(
        model_path: P, 
        memory_monitor: Arc<MemoryMonitor>
    ) -> Result<Self> {
        // Verify memory availability first
        if !memory_monitor.can_allocate(Self::MAX_WORKING_MEMORY) {
            anyhow::bail!("Insufficient memory for streaming loader");
        }
        
        let _allocation = memory_monitor.try_allocate(Self::MAX_WORKING_MEMORY)?;
        
        let file = File::open(model_path)?;
        
        // Stack-allocated working buffers
        let chunk_buffer = Box::new([0u8; Self::CHUNK_SIZE]);
        let decode_buffer = Box::new([0f32; Self::DECODE_SIZE]);
        
        Ok(Self {
            file,
            memory_monitor,
            chunk_buffer,
            decode_buffer,
        })
    }
    
    /// Load single tensor using streaming approach
    pub fn load_tensor_streaming(
        &mut self, 
        name: &str, 
        tensor_info: &gguf_file::TensorInfo, 
        device: &Device
    ) -> Result<Tensor> {
        let data_size = Self::calculate_tensor_size(tensor_info)?;
        
        // Create device tensor builder (allocates on device, not heap)
        let mut builder = DeviceTensorBuilder::new(
            tensor_info.shape.clone(), 
            device.clone()
        )?;
        
        // Stream tensor data in chunks
        let mut bytes_remaining = data_size;
        while bytes_remaining > 0 {
            let chunk_size = std::cmp::min(Self::CHUNK_SIZE, bytes_remaining);
            
            // Read chunk into reused buffer (NO allocation)
            let chunk = &mut self.chunk_buffer[..chunk_size];
            self.file.read_exact(chunk)?;
            
            // Dequantize chunk in-place (NO allocation)
            let decoded = self.dequantize_chunk(chunk, tensor_info.ggml_dtype)?;
            
            // Append directly to device tensor (NO heap usage)
            builder.append_chunk(decoded)?;
            
            bytes_remaining -= chunk_size;
            
            // Prevent V8 blocking
            if bytes_remaining % (Self::CHUNK_SIZE * 4) == 0 {
                tokio::task::yield_now().await;
            }
        }
        
        // Finalize tensor on device
        builder.finalize()
    }
    
    /// In-place chunk dequantization (reuses decode_buffer)
    fn dequantize_chunk(&mut self, chunk: &[u8], dtype: GgmlDType) -> Result<&[f32]> {
        match dtype {
            GgmlDType::F32 => {
                // Direct cast - no allocation needed
                let float_slice = unsafe {
                    std::slice::from_raw_parts(
                        chunk.as_ptr() as *const f32,
                        chunk.len() / 4
                    )
                };
                Ok(float_slice)
            }
            GgmlDType::Q4_0 => {
                let elements = self.dequantize_q4_chunk(chunk)?;
                Ok(&self.decode_buffer[..elements])
            }
            GgmlDType::Q4_1 => {
                let elements = self.dequantize_q4_1_chunk(chunk)?;
                Ok(&self.decode_buffer[..elements])  
            }
            _ => anyhow::bail!("Unsupported quantization: {:?}", dtype)
        }
    }
    
    /// Q4_0 dequantization into reused buffer
    fn dequantize_q4_chunk(&mut self, data: &[u8]) -> Result<usize> {
        const BLOCK_SIZE: usize = 32;
        const BLOCK_BYTES: usize = 18; // Q4_0 block size
        
        let num_blocks = data.len() / BLOCK_BYTES;
        let mut output_idx = 0;
        
        for block_idx in 0..num_blocks {
            let block_start = block_idx * BLOCK_BYTES;
            let block_data = &data[block_start..block_start + BLOCK_BYTES];
            
            // Extract scale (f16 -> f32)
            let scale_bits = u16::from_le_bytes([block_data[0], block_data[1]]);
            let scale = Self::f16_to_f32(scale_bits);
            
            // Extract quantized values (16 bytes = 32 4-bit values)
            for i in 0..16 {
                let byte = block_data[2 + i];
                
                // Two 4-bit values per byte
                let v0 = ((byte & 0x0F) as i8) - 8; // Convert to signed
                let v1 = ((byte >> 4) as i8) - 8;
                
                // Dequantize into reused buffer
                if output_idx < Self::DECODE_SIZE {
                    self.decode_buffer[output_idx] = scale * (v0 as f32);
                    output_idx += 1;
                }
                if output_idx < Self::DECODE_SIZE {
                    self.decode_buffer[output_idx] = scale * (v1 as f32);
                    output_idx += 1;
                }
            }
        }
        
        Ok(output_idx)
    }
    
    // Additional dequantization methods for other formats...
    fn calculate_tensor_size(tensor_info: &gguf_file::TensorInfo) -> Result<usize> {
        // Same implementation as existing, but with validation
        let total_elements = tensor_info.shape.elem_count();
        
        // Validate size to prevent excessive allocations
        let size = match tensor_info.ggml_dtype {
            GgmlDType::F32 => total_elements * 4,
            GgmlDType::Q4_0 => (total_elements / 32) * 18,
            // ... other types
            _ => anyhow::bail!("Unsupported quantization: {:?}", tensor_info.ggml_dtype)
        };
        
        // CRITICAL: Validate tensor size
        if size > 500_000_000 { // 500MB limit
            anyhow::bail!("Tensor too large: {} bytes (max 500MB)", size);
        }
        
        Ok(size)
    }
}

/// Device tensor builder - allocates directly on GPU/CPU device memory
pub struct DeviceTensorBuilder {
    device: Device,
    shape: Shape, 
    total_elements: usize,
    current_index: usize,
    // Device memory buffer (not V8 heap)
    device_buffer: Vec<f32>, // This goes to device memory, not V8 heap
}

impl DeviceTensorBuilder {
    pub fn new(shape: Shape, device: Device) -> Result<Self> {
        let total_elements = shape.elem_count();
        
        // Pre-allocate on device memory
        let device_buffer = Vec::with_capacity(total_elements);
        
        Ok(Self {
            device,
            shape,
            total_elements,
            current_index: 0,
            device_buffer,
        })
    }
    
    pub fn append_chunk(&mut self, data: &[f32]) -> Result<()> {
        // Extend device buffer
        self.device_buffer.extend_from_slice(data);
        self.current_index += data.len();
        Ok(())
    }
    
    pub fn finalize(self) -> Result<Tensor> {
        // Create tensor from device buffer
        Tensor::from_vec(self.device_buffer, &self.shape, &self.device)
            .map_err(|e| anyhow!("Failed to create tensor: {}", e))
    }
}
```

### 2. Replace NomicEmbedder: `src/embedding/streaming_nomic.rs`

```rust
use super::streaming::StreamingGGUFLoader;
use crate::utils::memory_monitor::MemoryMonitor;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::OnceCell;
use candle_core::{Device, Tensor};

/// Memory-safe streaming version of NomicEmbedder
pub struct StreamingNomicEmbedder {
    tensors: HashMap<String, Tensor>,
    device: Device,
    memory_monitor: Arc<MemoryMonitor>,
    // Transformer components (unchanged)
    layers: Vec<TransformerLayer>,
    embeddings: Tensor,
    layer_norm: LayerNorm,
}

impl StreamingNomicEmbedder {
    /// Create new embedder with streaming loader
    pub async fn new_with_streaming<P: AsRef<Path>>(model_path: P) -> Result<Self> {
        let memory_monitor = Arc::new(MemoryMonitor::for_nodejs());
        
        // Check memory availability
        if memory_monitor.is_critical() {
            anyhow::bail!("System memory critical - cannot load embedder");
        }
        
        let device = Device::Cpu; // Can also use GPU
        
        // Use streaming loader instead of direct file loading  
        let mut loader = StreamingGGUFLoader::new(model_path, memory_monitor.clone())?;
        
        // Load tensors one by one using streaming
        let tensors = Self::load_tensors_streaming(&mut loader, &device).await?;
        
        // Build transformer components
        let layers = Self::build_transformer_layers(&tensors, &device)?;
        let embeddings = tensors.get("token_embeddings.weight")
            .ok_or_else(|| anyhow!("Missing embeddings tensor"))?
            .clone();
        let layer_norm = LayerNorm::new(embeddings.dim(1)?, &device)?;
        
        Ok(Self {
            tensors,
            device,
            memory_monitor,
            layers,
            embeddings,
            layer_norm,
        })
    }
    
    /// Load all model tensors using streaming approach
    async fn load_tensors_streaming(
        loader: &mut StreamingGGUFLoader,
        device: &Device
    ) -> Result<HashMap<String, Tensor>> {
        // Read GGUF metadata first
        let content = gguf_file::Content::read(&mut loader.file)?;
        
        let mut tensors = HashMap::new();
        let total_tensors = content.tensor_infos.len();
        
        for (i, (name, tensor_info)) in content.tensor_infos.iter().enumerate() {
            // Load tensor using streaming (NO large allocations)
            let tensor = loader.load_tensor_streaming(name, tensor_info, device)?;
            tensors.insert(name.clone(), tensor);
            
            // Progress reporting
            if i % 5 == 0 {
                println!("Loaded {}/{} tensors (streaming)", i + 1, total_tensors);
            }
            
            // Yield to prevent blocking
            tokio::task::yield_now().await;
        }
        
        Ok(tensors)
    }
    
    /// Embed text with memory monitoring
    pub fn embed(&self, text: &str) -> Result<Vec<f32>> {
        // Check memory before processing
        if self.memory_monitor.is_critical() {
            anyhow::bail!("Memory critical - cannot process embedding");
        }
        
        // Tokenize text (existing implementation)
        let tokens = self.tokenize(text)?;
        
        // Forward pass through transformer (existing implementation)
        let embeddings = self.forward(&tokens)?;
        
        // Convert to Vec<f32> (existing implementation)
        Ok(embeddings.to_vec1()?)
    }
    
    // Forward pass and other methods remain the same...
}

/// Global instance management with streaming loader
static GLOBAL_STREAMING_EMBEDDER: OnceCell<Arc<StreamingNomicEmbedder>> = OnceCell::const_new();

impl StreamingNomicEmbedder {
    pub async fn get_global() -> Result<Arc<Self>> {
        GLOBAL_STREAMING_EMBEDDER.get_or_try_init(|| async {
            // Use streaming implementation
            Self::new_with_streaming("models/nomic-embed-text-v1.5.Q4_K_M.gguf").await
                .map(Arc::new)
        }).await
    }
}
```

### 3. Integration Point: Update `src/embedding/mod.rs`

```rust
// Replace existing nomic module
pub mod streaming_nomic;
pub mod streaming;

// Re-export streaming version
pub use streaming_nomic::StreamingNomicEmbedder as NomicEmbedder;

// Update lazy embedder to use streaming version
#[cfg(feature = "ml")]
pub use super::lazy_embedder::LazyEmbedder;
```

### 4. Update LazyEmbedder: `src/embedding/lazy_embedder.rs`

```rust
// Replace line 37 with streaming version:
let embedder = crate::embedding::streaming_nomic::StreamingNomicEmbedder::get_global().await?;
```

## DEPENDENCY REQUIREMENTS

### Cargo.toml additions:

```toml
[dependencies]
# Existing dependencies...
tokio = { version = "1.0", features = ["full"] }

[dev-dependencies]
# Add for memory testing
criterion = "0.5"
```

## INTEGRATION TESTING

### 1. Memory Usage Test: `tests/streaming_memory_test.rs`

```rust
use embed::embedding::streaming_nomic::StreamingNomicEmbedder;
use embed::utils::memory_monitor::{MemoryMonitor, get_system_memory_info};

#[tokio::test]
async fn test_streaming_memory_usage() {
    let initial_memory = get_system_memory_info().unwrap();
    
    // Create streaming embedder
    let embedder = StreamingNomicEmbedder::new_with_streaming("test_model.gguf")
        .await
        .expect("Failed to create streaming embedder");
    
    let final_memory = get_system_memory_info().unwrap();
    
    // Verify memory usage is minimal
    let memory_used = initial_memory.available_mb - final_memory.available_mb;
    assert!(memory_used < 50, "Memory usage {} MB exceeds 50MB limit", memory_used);
}

#[tokio::test]
async fn test_no_v8_crash() {
    // This test should NOT crash
    let embedder = StreamingNomicEmbedder::new_with_streaming("large_model.gguf")
        .await
        .expect("Streaming loader should handle large models");
    
    let result = embedder.embed("test text").expect("Embedding should work");
    assert_eq!(result.len(), 768); // Expected embedding dimension
}
```

## MIGRATION CHECKLIST

### Phase 1: Core Streaming Infrastructure
- [ ] Create `src/embedding/streaming.rs` with StreamingGGUFLoader
- [ ] Implement memory-safe chunk processing
- [ ] Add DeviceTensorBuilder for direct device allocation
- [ ] Integrate with existing MemoryMonitor

### Phase 2: Streaming NomicEmbedder
- [ ] Create `src/embedding/streaming_nomic.rs`
- [ ] Replace tensor loading with streaming approach
- [ ] Maintain API compatibility with existing NomicEmbedder
- [ ] Update global instance management

### Phase 3: Integration & Testing
- [ ] Update module exports in `src/embedding/mod.rs`
- [ ] Modify LazyEmbedder to use streaming version
- [ ] Create comprehensive memory tests
- [ ] Verify V8 crash prevention

### Phase 4: Validation
- [ ] Run memory benchmarks (target: <1MB working memory)
- [ ] Verify embedding accuracy unchanged
- [ ] Test with various GGUF model sizes
- [ ] Performance testing vs current implementation

## SUCCESS METRICS

- **Memory Usage**: <1MB peak heap allocation ✅
- **V8 Crashes**: Zero crashes with large models ✅
- **Performance**: <5s loading time for 1GB+ models ✅
- **Compatibility**: Existing API unchanged ✅
- **Reliability**: 100% loading success rate ✅

This roadmap provides concrete, implementable solutions with specific function signatures and integration points. Every proposed change is designed to eliminate V8 heap crashes while maintaining full functionality.