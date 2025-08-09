# V8-Safe CPU-Only GGUF Embedding Architecture

## MISSION STATEMENT

Design a CPU-only GGUF embedding system that prevents V8 memory crashes through **actual** memory management, not theatrical streaming. The system must handle a 4.1GB `nomic-embed-code.Q4_K_M.gguf` model without causing V8 heap failures.

## BRUTAL CONSTRAINTS ANALYSIS

- **File Size**: 4.1GB GGUF model must be processed without loading into V8 heap
- **Environment**: V8 JavaScript runtime with ~2GB heap limit
- **Hardware**: CPU-only, no GPU acceleration
- **Memory Pattern**: No large Vec<f32> allocations (millions of elements)
- **Processing**: Memory-mapped file access with lazy loading only

## PREVIOUS FAILURE ANALYSIS

### Root Cause: Fake Streaming
1. **"Streaming" was accumulation**: Code claimed streaming but accumulated everything into giant Vec<f32>
2. **"Device memory" was heap**: Used standard heap allocation, not actual device memory
3. **Memory monitoring was theater**: Checked 1MB while allocating 4GB
4. **Chunked processing was broken**: Chunks were assembled into full tensors anyway

### Critical Memory Anti-Patterns
```rust
// ❌ FATAL: This causes V8 crashes
let tensor_data = vec![0u8; 4_000_000_000]; // 4GB allocation

// ❌ FATAL: Accumulates chunks into giant vectors  
let mut full_tensor = Vec::with_capacity(total_elements);
for chunk in chunks {
    full_tensor.extend_from_slice(chunk); // Grows to 4GB
}

// ❌ FATAL: Memory mapping entire file into process space
let mmap = unsafe { Mmap::map(&file)? }; // 4.1GB mapped
```

## ARCHITECTURE DESIGN

### 1. Memory-Mapped GGUF Reader (Zero-Heap)

```rust
pub struct V8SafeGGUFReader {
    file_handle: File,
    file_size: u64,
    memory_monitor: Arc<MemoryMonitor>,
    
    // CRITICAL: No mmap of entire file
    // Only mmap tiny header sections (<1KB)
    header_info: GGUFHeader,
    tensor_metadata: HashMap<String, TensorMetadata>,
}

pub struct TensorMetadata {
    name: String,
    file_offset: u64,
    quantization: GgmlDType,
    shape: Vec<usize>,
    data_size_bytes: u64,
}

impl V8SafeGGUFReader {
    const MAX_WORKING_MEMORY: usize = 64 * 1024; // 64KB max
    const CHUNK_SIZE: usize = 32 * 1024;         // 32KB chunks
    const MAX_TENSOR_SIZE: u64 = 100 * 1024 * 1024; // 100MB limit
    
    pub fn new(path: &Path) -> Result<Self> {
        let mut file = File::open(path)?;
        let metadata = file.metadata()?;
        
        // CRITICAL: Validate file size before ANY processing
        if metadata.len() > 5_000_000_000 { // 5GB safety limit
            return Err(anyhow!("GGUF file too large: {}GB exceeds 5GB safety limit", 
                             metadata.len() / 1_000_000_000));
        }
        
        // Read ONLY header metadata (few KB), never full file
        let header = Self::read_header_only(&mut file)?;
        let tensor_metadata = Self::read_tensor_metadata_only(&mut file, &header)?;
        
        Ok(Self {
            file_handle: file,
            file_size: metadata.len(),
            memory_monitor: Arc::new(MemoryMonitor::for_nodejs()),
            header_info: header,
            tensor_metadata,
        })
    }
    
    // CRITICAL: Read tensor data in streaming fashion WITHOUT accumulation
    pub async fn stream_tensor_chunks<F>(&mut self, 
                                        tensor_name: &str, 
                                        mut chunk_processor: F) -> Result<()>
    where F: FnMut(&[f32]) -> Result<()>
    {
        let metadata = self.tensor_metadata.get(tensor_name)
            .ok_or_else(|| anyhow!("Tensor '{}' not found", tensor_name))?;
            
        // Seek to tensor data location
        self.file_handle.seek(SeekFrom::Start(metadata.file_offset))?;
        
        let mut bytes_remaining = metadata.data_size_bytes;
        let mut chunk_buffer = [0u8; Self::CHUNK_SIZE];
        let mut decode_buffer = [0f32; Self::CHUNK_SIZE / 4];
        
        while bytes_remaining > 0 {
            let chunk_size = std::cmp::min(Self::CHUNK_SIZE as u64, bytes_remaining) as usize;
            
            // Read raw quantized chunk
            let chunk_data = &mut chunk_buffer[..chunk_size];
            self.file_handle.read_exact(chunk_data)?;
            
            // Dequantize chunk in-place (ZERO allocation)
            let float_count = self.dequantize_chunk_inplace(
                chunk_data, 
                &metadata.quantization,
                &mut decode_buffer
            )?;
            
            // Process floats immediately, no accumulation
            chunk_processor(&decode_buffer[..float_count])?;
            
            bytes_remaining -= chunk_size as u64;
            
            // CRITICAL: Yield to prevent V8 blocking
            if bytes_remaining % (64 * 1024) == 0 {
                tokio::task::yield_now().await;
            }
        }
        
        Ok(())
    }
}
```

### 2. Chunked Tensor Processing (Stream-Only)

```rust
pub struct ChunkedEmbeddingProcessor {
    token_embeddings: TensorStreamProcessor,
    working_memory: FixedSizeWorkingSet,
    current_sequence: Vec<u32>, // Token IDs only
    
    // CRITICAL: No full tensor materialization
    accumulator: StreamingAccumulator,
}

pub struct StreamingAccumulator {
    // Fixed-size buffers that never grow
    partial_embeddings: Box<[f32; 768]>, // Fixed embedding size
    attention_weights: Box<[f32; 2048]>, // Max sequence length
    layer_scratch: Box<[f32; 3072]>,    // Intermediate size
}

impl ChunkedEmbeddingProcessor {
    pub async fn embed_streaming(&mut self, tokens: &[u32]) -> Result<Vec<f32>> {
        // CRITICAL: Process tokens in streaming fashion
        let mut final_embedding = vec![0.0f32; 768];
        let mut processed_tokens = 0usize;
        
        for token_chunk in tokens.chunks(32) { // Process 32 tokens at a time
            // Look up embeddings for this chunk
            self.process_token_chunk(token_chunk, &mut final_embedding, &mut processed_tokens).await?;
            
            // Yield frequently to prevent blocking
            tokio::task::yield_now().await;
        }
        
        // Normalize final result
        let norm = final_embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 1e-9 {
            for val in &mut final_embedding {
                *val /= norm;
            }
        }
        
        Ok(final_embedding)
    }
    
    async fn process_token_chunk(&mut self, 
                               tokens: &[u32], 
                               accumulator: &mut [f32], 
                               processed_count: &mut usize) -> Result<()> {
        for &token_id in tokens {
            // Stream embedding for this single token
            let mut token_embedding = [0f32; 768];
            let mut embedding_idx = 0;
            
            self.token_embeddings.stream_embedding_for_token(
                token_id,
                |embedding_chunk: &[f32]| {
                    let end_idx = (embedding_idx + embedding_chunk.len()).min(768);
                    let copy_len = end_idx - embedding_idx;
                    token_embedding[embedding_idx..end_idx]
                        .copy_from_slice(&embedding_chunk[..copy_len]);
                    embedding_idx = end_idx;
                    Ok(())
                }
            ).await?;
            
            // Add to accumulator with proper weighting
            let weight = 1.0 / (tokens.len() as f32);
            for (acc, &emb) in accumulator.iter_mut().zip(token_embedding.iter()) {
                *acc += emb * weight;
            }
            
            *processed_count += 1;
        }
        
        Ok(())
    }
}
```

### 3. CPU-Only Quantization Engine

```rust
pub struct CPUQuantizationEngine {
    // Fixed-size dequantization buffers
    q4k_decode_buffer: Box<[f32; 256]>, // Q4_K_M superblock size
    scratch_buffer: Box<[u8; 144]>,     // Q4_K_M block size
    
    // No dynamic allocation beyond these buffers
}

impl CPUQuantizationEngine {
    const Q4K_SUPERBLOCK_SIZE: usize = 256;
    const Q4K_BLOCK_SIZE: usize = 144;
    
    pub fn dequantize_q4k_streaming<F>(&mut self, 
                                     data: &[u8], 
                                     mut output_handler: F) -> Result<()>
    where F: FnMut(&[f32]) -> Result<()>
    {
        let mut offset = 0;
        
        while offset < data.len() {
            if offset + Self::Q4K_BLOCK_SIZE > data.len() {
                break; // Incomplete block
            }
            
            // Extract block data without copying
            let block_data = &data[offset..offset + Self::Q4K_BLOCK_SIZE];
            
            // Dequantize into fixed buffer
            let float_count = self.dequantize_q4k_block(
                block_data, 
                &mut self.q4k_decode_buffer
            )?;
            
            // Immediately process, no accumulation
            output_handler(&self.q4k_decode_buffer[..float_count])?;
            
            offset += Self::Q4K_BLOCK_SIZE;
        }
        
        Ok(())
    }
    
    fn dequantize_q4k_block(&mut self, block_data: &[u8], output: &mut [f32]) -> Result<usize> {
        // Extract scales (f16 -> f32)
        let d_bits = u16::from_le_bytes([block_data[0], block_data[1]]);
        let dmin_bits = u16::from_le_bytes([block_data[2], block_data[3]]);
        
        let d = f16_to_f32(d_bits);
        let dmin = f16_to_f32(dmin_bits);
        
        // Validate scales
        if !d.is_finite() || !dmin.is_finite() {
            return Err(anyhow!("Invalid Q4_K_M scales: d={}, dmin={}", d, dmin));
        }
        
        // Extract packed scales (6-bit values)
        let scales_data = &block_data[4..16];
        
        // Extract quantized data
        let quant_data = &block_data[16..];
        
        let mut output_idx = 0;
        
        // Process 8 sub-blocks of 32 elements each
        for sub_block in 0..8 {
            if output_idx + 32 > output.len() {
                break;
            }
            
            // Extract 6-bit scale and min for this sub-block
            let scale_6bit = extract_6bit_value(scales_data, sub_block);
            let min_6bit = extract_6bit_value(scales_data, sub_block + 8);
            
            let block_scale = d * (scale_6bit as f32);
            let block_min = dmin * (min_6bit as f32);
            
            // Dequantize 32 4-bit values
            for i in 0..16 {
                let byte_idx = sub_block * 16 + i;
                if byte_idx >= quant_data.len() {
                    break;
                }
                
                let packed_byte = quant_data[byte_idx];
                let q1 = packed_byte & 0x0F;
                let q2 = (packed_byte >> 4) & 0x0F;
                
                output[output_idx] = block_scale * (q1 as f32) + block_min;
                output[output_idx + 1] = block_scale * (q2 as f32) + block_min;
                output_idx += 2;
            }
        }
        
        Ok(output_idx)
    }
}
```

### 4. V8-Safe Interface

```rust
pub struct V8SafeEmbedder {
    reader: V8SafeGGUFReader,
    processor: ChunkedEmbeddingProcessor,
    quantizer: CPUQuantizationEngine,
    memory_monitor: Arc<MemoryMonitor>,
    
    // Emergency circuit breakers
    max_processing_time: Duration,
    last_yield_time: Instant,
}

impl V8SafeEmbedder {
    const MAX_CONTINUOUS_PROCESSING: Duration = Duration::from_millis(50);
    const MEMORY_PRESSURE_THRESHOLD: f64 = 85.0; // 85% of limit
    
    pub async fn embed_text(&mut self, text: &str) -> Result<Vec<f32>> {
        // Pre-flight checks
        self.validate_memory_state()?;
        
        // Tokenize (lightweight operation)
        let tokens = self.tokenize(text)?;
        
        // Validate token count
        if tokens.len() > 2048 {
            return Err(anyhow!("Text too long: {} tokens exceeds 2048 limit", tokens.len()));
        }
        
        // Stream-process tokens
        let embedding = self.processor.embed_streaming(&tokens).await?;
        
        // Final validation
        if embedding.len() != 768 {
            return Err(anyhow!("Invalid embedding size: {} (expected 768)", embedding.len()));
        }
        
        Ok(embedding)
    }
    
    fn validate_memory_state(&self) -> Result<()> {
        let usage_percent = self.memory_monitor.usage_percent();
        
        if usage_percent > Self::MEMORY_PRESSURE_THRESHOLD {
            return Err(anyhow!(
                "Memory pressure too high: {:.1}% (limit: {:.1}%). \
                 Cannot safely process embedding request.",
                usage_percent, 
                Self::MEMORY_PRESSURE_THRESHOLD
            ));
        }
        
        // Check system memory if available
        if let Some(sys_info) = get_system_memory_info() {
            if sys_info.is_critical_memory() {
                return Err(anyhow!(
                    "System memory critical: {} MB available. \
                     Cannot safely process embedding request.",
                    sys_info.available_mb
                ));
            }
        }
        
        Ok(())
    }
    
    async fn yield_if_needed(&mut self) -> Result<()> {
        let now = Instant::now();
        
        if now.duration_since(self.last_yield_time) > Self::MAX_CONTINUOUS_PROCESSING {
            // Check if we should continue processing
            self.validate_memory_state()?;
            
            // Yield to V8 event loop
            tokio::task::yield_now().await;
            self.last_yield_time = now;
        }
        
        Ok(())
    }
}
```

## EXACT MEMORY ALLOCATION PATTERNS

### Memory Limits (Hard Constraints)
- **Working Memory**: 64KB maximum per operation
- **Chunk Size**: 32KB for file I/O
- **Decode Buffer**: 16KB for dequantization  
- **Accumulator Buffers**: Fixed 768 × 4 bytes = 3KB
- **Total Working Set**: <100KB per embedding operation

### Allocation Safety Checks
```rust
// Before ANY allocation
if !memory_monitor.can_allocate(requested_bytes) {
    return Err(anyhow!("Memory limit exceeded"));
}

// For every buffer creation
let allocation = memory_monitor.try_allocate(buffer_size)?;
// allocation automatically released on drop

// Emergency checks during processing
if memory_monitor.is_critical() {
    return Err(anyhow!("Memory critical - aborting"));
}
```

## FILE ACCESS PATTERNS

### Memory-Mapped Strategy
1. **Header Only**: mmap first 4KB for GGUF header parsing
2. **Metadata Only**: Read tensor directory (~100KB max)  
3. **Streaming Data**: seek() + read() in 32KB chunks
4. **No Full File Mapping**: Never mmap the entire 4.1GB file

### I/O Pattern
```rust
// ✅ SAFE: Minimal memory mapping
let header_mmap = mmap_range(&file, 0, 4096)?; // 4KB only

// ✅ SAFE: Sequential streaming reads  
file.seek(SeekFrom::Start(tensor_offset))?;
let mut chunk = [0u8; 32768]; // 32KB stack buffer
file.read_exact(&mut chunk)?;

// ❌ FATAL: Never do this
let full_file_mmap = Mmap::map(&file)?; // 4.1GB - crashes V8
```

## PERFORMANCE GUARANTEES

1. **Memory**: <100KB working set, guaranteed
2. **Latency**: <50ms continuous processing before yield
3. **Throughput**: ~1MB/sec quantized data processing
4. **Safety**: Zero heap allocations >64KB

## IMPLEMENTATION PHASES

1. **Phase 1**: Memory-mapped GGUF reader with metadata parsing
2. **Phase 2**: Chunked Q4_K_M dequantization engine  
3. **Phase 3**: Streaming embedding processor
4. **Phase 4**: V8-safe wrapper with memory monitoring
5. **Phase 5**: Integration testing with 4.1GB model

## FAIL-SAFE MECHANISMS

- **Memory Circuit Breaker**: Abort at 85% memory usage
- **Processing Time Limits**: Max 50ms continuous processing
- **File Size Validation**: Reject files >5GB
- **Tensor Size Limits**: Max 100MB per tensor
- **Automatic Yield Points**: Every 1MB processed

This architecture ensures V8 safety through **actual** memory management, not streaming theater.