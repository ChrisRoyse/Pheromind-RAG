# GGUF Model Implementation Plan for Pheromind-RAG

## Executive Summary

This document provides a comprehensive implementation plan to integrate the `nomic-embed-code.Q4_K_M.gguf` model (located in `./src/model/`) into the existing Pheromind-RAG system using llama.cpp FFI bindings. This approach leverages battle-tested C++ implementations through safe Rust wrappers, providing optimal performance with GPU acceleration support.

## Current State Analysis

### Current Status (Post FastEmbed Removal)
- **Current Embedder**: Placeholder implementation (FastEmbed removed)
- **Target Embedder**: GGUF with Nomic v1 model (768-dimensional embeddings)
- **Model File**: `./src/model/nomic-embed-code.Q4_K_M.gguf` (available)
- **Storage**: LanceDB for vector storage with Arrow format
- **Search**: Hybrid search combining vector (LanceDB) + text (Tantivy)
- **Model Location**: `./src/model/nomic-embed-code.Q4_K_M.gguf` (4.3GB)

### Key Components
1. `simple_embedder.rs` - Placeholder implementation (FastEmbed removed, ready for GGUF)
2. `simple_storage.rs` - LanceDB vector storage (unchanged)
3. `simple_search.rs` - Hybrid search orchestration (minimal changes)
4. `./src/model/nomic-embed-code.Q4_K_M.gguf` - GGUF model file (ready for integration)
4. `main.rs` - CLI interface (unchanged)

## Implementation Strategy: llama.cpp FFI Bindings

### Why llama.cpp FFI
- **Native GGUF Support**: Direct support for all quantization formats including Q4_K_M
- **Production Ready**: Battle-tested in thousands of production deployments
- **GPU Acceleration**: Full support for CUDA, Metal, and ROCm
- **Minimal Overhead**: Direct FFI calls with negligible performance impact
- **Active Development**: Regular updates and improvements from the community

### Selected Binding: llama-cpp-2
We'll use the `llama-cpp-2` crate which:
- Stays tightly coupled with llama.cpp API
- Uses bindgen for automatic FFI generation
- Provides safe Rust wrappers
- Maintained by utilityai for production use

## Detailed Implementation Plan

### Phase 1: System Setup & Dependencies

#### 1.1 Build System Configuration
```bash
# System dependencies installation
# Ubuntu/Debian
sudo apt-get update
sudo apt-get install -y build-essential cmake pkg-config libssl-dev clang libclang-dev

# CUDA support (optional)
wget https://developer.download.nvidia.com/compute/cuda/repos/ubuntu2204/x86_64/cuda-keyring_1.0-1_all.deb
sudo dpkg -i cuda-keyring_1.0-1_all.deb
sudo apt-get update
sudo apt-get install cuda-toolkit-11-8

export CUDA_PATH=/usr/local/cuda
export LD_LIBRARY_PATH=$CUDA_PATH/lib64:$LD_LIBRARY_PATH
export LLAMA_CUDA=1
```

#### 1.2 Cargo Dependencies
```toml
[dependencies]
# llama.cpp FFI bindings
llama-cpp-2 = { version = "0.1", features = ["cublas"] }
llama-cpp-sys-2 = "0.1"  # Low-level bindings

# FFI utilities
libc = "0.2"
once_cell = "1.19"
parking_lot = "0.12"
num_cpus = "1.16"

# Existing dependencies remain
lancedb = "0.8"
tantivy = "0.22"
arrow = "52"
arrow-array = "52"

[build-dependencies]
bindgen = "0.69"
cmake = "0.1"
cc = "1.0"

[features]
default = ["cuda"]
cuda = []
metal = []
hipblas = []
```

#### 1.3 Build Script (build.rs)
```rust
use std::env;

fn main() {
    // Detect and configure GPU acceleration
    if env::var("CUDA_PATH").is_ok() {
        println!("cargo:rustc-cfg=feature=\"cuda\"");
        println!("cargo:rustc-env=LLAMA_CUDA=1");
    }
    
    if cfg!(target_os = "macos") {
        println!("cargo:rustc-cfg=feature=\"metal\"");
        println!("cargo:rustc-env=LLAMA_METAL=1");
    }
    
    // Link to C++ standard library
    println!("cargo:rustc-link-lib=dylib=stdc++");
}
```

### Phase 2: FFI Wrapper Implementation

#### 2.1 Low-Level FFI Bindings
```rust
// src/ffi/mod.rs
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_float, c_int, c_void};

#[repr(C)]
pub struct llama_model {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

#[repr(C)]
pub struct llama_context {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

#[repr(C)]
pub struct llama_context_params {
    pub seed: u32,
    pub n_ctx: u32,
    pub n_batch: u32,
    pub n_threads: u32,
    pub n_gpu_layers: c_int,
    pub embeddings: bool,  // CRITICAL: Must be true
    pub rope_freq_scale: f32,
    // ... other fields
}

extern "C" {
    pub fn llama_backend_init();
    pub fn llama_load_model_from_file(
        path_model: *const c_char,
        params: llama_model_params,
    ) -> *mut llama_model;
    pub fn llama_new_context_with_model(
        model: *mut llama_model,
        params: llama_context_params,
    ) -> *mut llama_context;
    pub fn llama_get_embeddings(ctx: *mut llama_context) -> *const c_float;
    pub fn llama_n_embd(model: *const llama_model) -> c_int;
}
```

#### 2.2 Safe Rust Wrapper
```rust
// src/llama_wrapper.rs
use crate::ffi::*;
use anyhow::{Result, Context, bail};
use std::sync::Arc;
use parking_lot::Mutex;

pub struct GGUFModel {
    model: Arc<Mutex<*mut llama_model>>,
    embedding_dim: usize,
    model_path: String,
}

unsafe impl Send for GGUFModel {}
unsafe impl Sync for GGUFModel {}

impl GGUFModel {
    pub fn load_from_file<P: AsRef<Path>>(path: P, gpu_layers: i32) -> Result<Self> {
        let path_str = path.as_ref().to_str().context("Invalid path")?;
        let c_path = CString::new(path_str)?;
        
        // Initialize backend once
        static INIT: std::sync::Once = std::sync::Once::new();
        INIT.call_once(|| unsafe { llama_backend_init(); });
        
        let model_params = llama_model_params {
            n_gpu_layers: gpu_layers,
            use_mmap: true,
            use_mlock: false,
            ..Default::default()
        };
        
        let model = unsafe {
            llama_load_model_from_file(c_path.as_ptr(), model_params)
        };
        
        if model.is_null() {
            bail!("Failed to load model from: {}", path_str);
        }
        
        let embedding_dim = unsafe { llama_n_embd(model) } as usize;
        
        Ok(Self {
            model: Arc::new(Mutex::new(model)),
            embedding_dim,
            model_path: path_str.to_string(),
        })
    }
}
```

### Phase 3: High-Level Embedder Interface

#### 3.1 Task-Specific Prefixes for nomic-embed-code
```rust
// src/embedding_prefixes.rs
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EmbeddingTask {
    SearchQuery,     // User search queries
    SearchDocument,  // Documents for indexing
    CodeDefinition,  // Code definitions
    CodeUsage,       // Code usage examples
    Classification,  // Classification tasks
    Clustering,      // Clustering tasks
}

impl EmbeddingTask {
    pub fn prefix(&self) -> &'static str {
        match self {
            Self::SearchQuery => "search_query: ",
            Self::SearchDocument => "search_document: ",
            Self::CodeDefinition => "def: ",
            Self::CodeUsage => "usage: ",
            Self::Classification => "classification: ",
            Self::Clustering => "clustering: ",
        }
    }
    
    pub fn apply_prefix(&self, text: &str) -> String {
        format!("{}{}", self.prefix(), text)
    }
}
```

#### 3.2 GGUF Embedder with Caching
```rust
// src/gguf_embedder.rs
use lru::LruCache;
use std::num::NonZeroUsize;

pub struct GGUFEmbedderConfig {
    pub model_path: String,  // "./src/model/nomic-embed-code.Q4_K_M.gguf"
    pub context_size: u32,   // 8192
    pub gpu_layers: i32,     // -1 for all layers
    pub batch_size: usize,   // 32
    pub cache_size: usize,   // 1000
    pub normalize: bool,     // true
}

pub struct GGUFEmbedder {
    model: Arc<GGUFModel>,
    context: Arc<Mutex<GGUFContext>>,
    cache: Arc<Mutex<LruCache<String, Vec<f32>>>>,
    config: GGUFEmbedderConfig,
}

impl GGUFEmbedder {
    pub fn new(config: GGUFEmbedderConfig) -> Result<Self> {
        let model = Arc::new(GGUFModel::load_from_file(
            &config.model_path,
            config.gpu_layers,
        )?);
        
        let context = Arc::new(Mutex::new(
            GGUFContext::new_with_model(model.clone(), config.context_size)?
        ));
        
        let cache_size = NonZeroUsize::new(config.cache_size).unwrap();
        let cache = Arc::new(Mutex::new(LruCache::new(cache_size)));
        
        Ok(Self { model, context, cache, config })
    }
    
    pub fn embed(&self, text: &str, task: EmbeddingTask) -> Result<Vec<f32>> {
        let prefixed_text = task.apply_prefix(text);
        
        // Check cache
        {
            let mut cache = self.cache.lock();
            if let Some(cached) = cache.get(&prefixed_text) {
                return Ok(cached.clone());
            }
        }
        
        // Generate embedding
        let embedding = {
            let mut ctx = self.context.lock();
            ctx.embed(&prefixed_text)?
        };
        
        // Cache and return
        {
            let mut cache = self.cache.lock();
            cache.put(prefixed_text.clone(), embedding.clone());
        }
        
        Ok(embedding)
    }
}
```

### Phase 4: Integration with Existing System

#### 4.1 Replace FastEmbed in simple_embedder.rs
```rust
// src/simple_embedder.rs
use crate::gguf_embedder::GGUFEmbedder;

pub struct NomicEmbedder {
    embedder: GGUFEmbedder,
}

impl NomicEmbedder {
    pub fn new() -> Result<Self> {
        let model_path = "./src/model/nomic-embed-code.Q4_K_M.gguf";
        let embedder = GGUFEmbedder::new(model_path)?;
        Ok(Self { embedder })
    }
    
    pub fn embed(&mut self, text: &str) -> Result<Vec<f32>> {
        self.embedder.embed_with_prefix(text, TaskType::Document)
    }
    
    pub fn embed_query(&mut self, query: &str) -> Result<Vec<f32>> {
        self.embedder.embed_with_prefix(query, TaskType::Search)
    }
}
```

#### 4.2 Batch Processing with Memory Management
```rust
pub fn embed_batch(&mut self, documents: Vec<String>) -> Result<Vec<Vec<f32>>> {
    // Process in chunks to manage memory
    let chunk_size = 32;
    let mut all_embeddings = Vec::new();
    
    for chunk in documents.chunks(chunk_size) {
        let batch_embeddings = self.embedder.embed_batch(chunk.to_vec())?;
        all_embeddings.extend(batch_embeddings);
    }
    
    Ok(all_embeddings)
}
```

### Phase 5: Performance Optimization

#### 5.1 Memory Pool Implementation
```rust
// src/utils/memory_pool.rs
pub struct EmbeddingMemoryPool {
    pool: Vec<Vec<f32>>,
    dimension: usize,
}

impl EmbeddingMemoryPool {
    pub fn acquire(&mut self) -> Vec<f32> {
        self.pool.pop().unwrap_or_else(|| vec![0.0; self.dimension])
    }
    
    pub fn release(&mut self, mut buffer: Vec<f32>) {
        buffer.clear();
        self.pool.push(buffer);
    }
}
```

#### 5.2 GPU Management
```rust
// src/gpu_manager.rs
pub struct GPUManager {
    device_count: usize,
    selected_device: i32,
}

impl GPUManager {
    pub fn new() -> Self {
        let device_count = Self::detect_gpu_count();
        let selected_device = env::var("CUDA_VISIBLE_DEVICES")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        
        Self { device_count, selected_device }
    }
    
    fn detect_gpu_count() -> usize {
        // CUDA detection
        if let Ok(output) = std::process::Command::new("nvidia-smi")
            .arg("--query-gpu=count")
            .arg("--format=csv,noheader")
            .output()
        {
            if let Ok(count_str) = String::from_utf8(output.stdout) {
                return count_str.trim().parse().unwrap_or(0);
            }
        }
        
        #[cfg(target_os = "macos")]
        return 1; // Metal always available on macOS
        
        0
    }
    
    pub fn optimal_layers(&self, model_size: usize) -> i32 {
        if self.device_count == 0 {
            return 0; // CPU only
        }
        // Full offload for 4.3GB model
        -1
    }
}
```

#### 5.3 Error Handling & Safety
```rust
// src/errors.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GGUFError {
    #[error("Model loading failed: {0}")]
    ModelLoadError(String),
    
    #[error("Context creation failed: {0}")]
    ContextError(String),
    
    #[error("Embedding generation failed: {0}")]
    EmbeddingError(String),
    
    #[error("GPU initialization failed: {0}")]
    GPUError(String),
    
    #[error("FFI error: {0}")]
    FFIError(String),
}

pub type GGUFResult<T> = Result<T, GGUFError>;

// Safety wrapper for batch operations
pub struct SafeBatch {
    batch: llama_batch,
    allocated: bool,
}

impl Drop for SafeBatch {
    fn drop(&mut self) {
        if self.allocated {
            unsafe { llama_batch_free(self.batch); }
            self.allocated = false;
        }
    }
}
```

### Phase 6: Testing & Validation

#### 6.1 Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_gguf_model_loading() {
        let embedder = GGUFEmbedder::new("./src/model/nomic-embed-code.Q4_K_M.gguf")
            .expect("Failed to load model");
        assert_eq!(embedder.embedding_dim, 768);
    }
    
    #[test]
    fn test_embedding_generation() {
        let mut embedder = GGUFEmbedder::new("./src/model/nomic-embed-code.Q4_K_M.gguf")
            .expect("Failed to load model");
        
        let embedding = embedder.embed("fn main() { println!(\"Hello\"); }")
            .expect("Failed to generate embedding");
        
        assert_eq!(embedding.len(), 768);
        assert!(embedding.iter().any(|&x| x != 0.0));
    }
    
    #[test]
    fn test_batch_embeddings() {
        let mut embedder = GGUFEmbedder::new("./src/model/nomic-embed-code.Q4_K_M.gguf")
            .expect("Failed to load model");
        
        let texts = vec![
            "struct User { name: String }".to_string(),
            "fn calculate(x: i32) -> i32 { x * 2 }".to_string(),
        ];
        
        let embeddings = embedder.embed_batch(texts)
            .expect("Failed to generate batch embeddings");
        
        assert_eq!(embeddings.len(), 2);
        assert_eq!(embeddings[0].len(), 768);
    }
}
```

#### 6.2 Integration Tests
```rust
#[tokio::test]
async fn test_end_to_end_search() {
    let db_path = "./test_gguf.db";
    let mut search = HybridSearch::new(db_path).await.unwrap();
    
    // Index test documents
    let contents = vec![
        "fn process_data(input: Vec<u8>) -> Result<String>".to_string(),
        "class DataProcessor: def process(self, data):".to_string(),
    ];
    let paths = vec!["test.rs".to_string(), "test.py".to_string()];
    
    search.index(contents, paths).await.unwrap();
    
    // Search with GGUF embeddings
    let results = search.search("data processing function", 5).await.unwrap();
    
    assert!(!results.is_empty());
    assert!(results[0].score > 0.5);
}
```

#### 6.3 Performance Benchmarks
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_gguf_embedding(c: &mut Criterion) {
    let mut embedder = GGUFEmbedder::new("./src/model/nomic-embed-code.Q4_K_M.gguf")
        .expect("Failed to load model");
    
    c.bench_function("single_embedding", |b| {
        b.iter(|| {
            embedder.embed(black_box("fn test() { return 42; }"))
        })
    });
    
    c.bench_function("batch_embedding_32", |b| {
        let texts: Vec<String> = (0..32)
            .map(|i| format!("fn test_{}() {{ return {}; }}", i, i))
            .collect();
        
        b.iter(|| {
            embedder.embed_batch(black_box(texts.clone()))
        })
    });
}

criterion_group!(benches, benchmark_gguf_embedding);
criterion_main!(benches);
```

### Phase 7: Migration from FastEmbed

#### 7.1 Compatibility Layer
```rust
pub trait EmbeddingProvider {
    fn embed(&mut self, text: &str) -> Result<Vec<f32>>;
    fn embed_batch(&mut self, texts: Vec<String>) -> Result<Vec<Vec<f32>>>;
    fn dimension(&self) -> usize;
}

// Both FastEmbed and GGUF implement this trait
impl EmbeddingProvider for FastEmbedProvider { ... }
impl EmbeddingProvider for GGUFProvider { ... }

// Runtime selection
pub fn create_embedder(config: &Config) -> Box<dyn EmbeddingProvider> {
    match config.embedding_backend {
        Backend::FastEmbed => Box::new(FastEmbedProvider::new()),
        Backend::GGUF => Box::new(GGUFProvider::new(&config.model_path)),
    }
}
```

#### 7.2 Data Migration Script
```rust
// Script to re-embed existing data with new model
pub async fn migrate_embeddings(old_db: &str, new_db: &str) -> Result<()> {
    let mut old_storage = VectorStorage::new(old_db).await?;
    let mut new_storage = VectorStorage::new(new_db).await?;
    let mut embedder = GGUFEmbedder::new("./src/model/nomic-embed-code.Q4_K_M.gguf")?;
    
    // Read all documents
    let documents = old_storage.get_all_documents().await?;
    
    // Re-embed with new model
    for batch in documents.chunks(100) {
        let contents: Vec<String> = batch.iter().map(|d| d.content.clone()).collect();
        let paths: Vec<String> = batch.iter().map(|d| d.path.clone()).collect();
        
        let new_embeddings = embedder.embed_batch(contents.clone())?;
        new_storage.store(contents, new_embeddings, paths).await?;
    }
    
    Ok(())
}
```

## Deployment & Operations

### System Requirements

#### Hardware
- **Memory**: 8GB minimum (4.3GB model + overhead)
- **GPU** (Optional but recommended):
  - NVIDIA: CUDA 11.7+ compatible GPU with 6GB+ VRAM
  - Apple: M1/M2/M3 with Metal support
  - AMD: ROCm 5.4+ compatible GPU
- **Storage**: 5GB for model + indexes
- **CPU**: 4+ cores recommended for parallel tokenization

#### Software
- **Rust**: 1.70+ with cargo
- **C++ Toolchain**: clang 14+ or gcc 11+
- **Build Tools**: cmake 3.14+, pkg-config
- **CUDA Toolkit**: 11.7-12.x (for NVIDIA GPUs)
- **Python**: 3.8+ (for build scripts only)

### Environment Variables

```bash
# GPU Configuration
export CUDA_VISIBLE_DEVICES=0
export CUDA_PATH=/usr/local/cuda
export LD_LIBRARY_PATH=$CUDA_PATH/lib64:$LD_LIBRARY_PATH

# llama.cpp specific
export LLAMA_CUDA=1          # Enable CUDA
export LLAMA_METAL=1         # Enable Metal (macOS)
export LLAMA_HIPBLAS=1       # Enable ROCm (AMD)
export LLAMA_N_THREADS=8     # Thread count

# Model configuration
export GGUF_MODEL_PATH="./src/model/nomic-embed-code.Q4_K_M.gguf"
export GGUF_GPU_LAYERS=-1    # Use all GPU layers
export GGUF_CONTEXT_SIZE=8192
```

### Build Commands

```bash
# Development build
cargo build

# Release build with optimizations
cargo build --release

# Build with CUDA support
cargo build --release --features cuda

# Build with Metal support (macOS)
cargo build --release --features metal

# Run tests
cargo test --all-features
cargo test --release

# Run benchmarks
cargo bench
```

## Risk Mitigation

### Risk 1: Model Loading Performance
**Mitigation**: Implement model preloading and memory mapping

### Risk 2: Memory Consumption
**Mitigation**: Use streaming for batch processing, implement memory pools

### Risk 3: Compatibility Issues
**Mitigation**: Maintain FastEmbed fallback, comprehensive testing

### Risk 4: Performance Regression
**Mitigation**: Benchmark before/after, GPU acceleration, caching

### Troubleshooting Guide

#### Common Issues and Solutions

1. **Model Loading Fails**
   - Verify file exists: `ls -la ./src/model/nomic-embed-code.Q4_K_M.gguf`
   - Check file permissions: `chmod 644 ./src/model/*.gguf`
   - Verify GGUF format: `file ./src/model/nomic-embed-code.Q4_K_M.gguf`
   - Check available memory: `free -h`

2. **GPU Not Detected**
   - NVIDIA: Run `nvidia-smi` to verify driver
   - Check CUDA installation: `nvcc --version`
   - Verify environment: `echo $CUDA_PATH`
   - Run with debug: `RUST_LOG=debug cargo run`

3. **Segmentation Faults**
   - Usually indicates FFI struct mismatch
   - Verify llama.cpp version compatibility
   - Check with valgrind: `valgrind ./target/release/embed-search`
   - Ensure release build: `cargo build --release`

4. **Poor Performance**
   - Verify GPU offloading: Check logs for "offloaded N/N layers"
   - Monitor GPU usage: `nvidia-smi dmon -s um`
   - Check batch sizes in config
   - Profile with: `cargo build --release && perf record ./target/release/embed-search`

5. **Embedding Dimension Mismatch**
   - Verify model outputs 768 dimensions
   - Check normalization is applied
   - Ensure correct task prefix is used

## Success Metrics

1. **Functional**
   - Successfully loads nomic-embed-code.Q4_K_M.gguf
   - Generates 768-dimensional embeddings
   - Maintains search quality

2. **Performance**
   - Single embedding: < 50ms
   - Batch (32 docs): < 500ms
   - Memory usage: < 6GB steady state

3. **Quality**
   - Code search precision: > 85%
   - Semantic similarity scores: > 0.8 for relevant matches

## Implementation Timeline

### Week 1: Foundation & FFI Setup
- **Day 1**: System dependencies, build environment setup
- **Day 2**: Implement low-level FFI bindings
- **Day 3**: Create safe Rust wrappers for llama.cpp
- **Day 4**: Implement GGUFModel and GGUFContext
- **Day 5**: Unit tests for FFI layer

### Week 2: Integration & Features
- **Day 1**: Implement embedding prefixes and task types
- **Day 2**: Create GGUFEmbedder with caching
- **Day 3**: Replace FastEmbed in simple_embedder.rs
- **Day 4**: Implement batch processing and memory pools
- **Day 5**: Integration testing with existing system

### Week 3: Optimization & Deployment
- **Day 1**: GPU optimization and benchmarking
- **Day 2**: Performance tuning and profiling
- **Day 3**: Migration scripts and compatibility layer
- **Day 4**: End-to-end testing and validation
- **Day 5**: Documentation and deployment preparation

## Key Implementation Patterns

### FFI Safety Patterns
- **RAII Wrappers**: All FFI resources wrapped in Drop implementations
- **Static Initialization**: Backend initialized once with std::sync::Once
- **Panic Recovery**: FFI calls wrapped in catch_unwind
- **Null Checks**: Defensive programming for all pointer operations

### Performance Patterns
- **Memory Pooling**: Reuse embedding buffers to reduce allocations
- **LRU Caching**: Cache frequent embeddings with bounded memory
- **Batch Processing**: Process multiple texts in single FFI call
- **GPU Offloading**: Automatic detection and optimal layer placement

### Code-Specific Optimizations
- **Task Prefixes**: Correct prefixes for nomic-embed-code model
- **Language Detection**: Apply language-specific formatting
- **Normalization**: L2 normalization for consistent similarity scores
- **Context Windows**: Respect 8192 token limit with chunking

## Conclusion

This implementation plan provides a production-ready approach to integrating the nomic-embed-code.Q4_K_M.gguf model using llama.cpp FFI bindings. The architecture leverages battle-tested C++ implementations while maintaining Rust safety guarantees through careful wrapper design. The phased approach enables incremental validation with clear rollback points if issues arise.

### Key Advantages of This Approach
- **Performance**: Native C++ performance with minimal FFI overhead
- **Safety**: Rust's type system ensures memory safety
- **Compatibility**: Direct GGUF support without conversion
- **Scalability**: GPU acceleration for production workloads
- **Maintainability**: Clear separation of concerns and error handling

## Appendix: Quick Reference

### Installation Checklist
- [ ] Install system dependencies (clang, cmake, pkg-config)
- [ ] Install CUDA toolkit (if using NVIDIA GPU)
- [ ] Clone repository and navigate to project
- [ ] Verify model file exists at `./src/model/nomic-embed-code.Q4_K_M.gguf`
- [ ] Run `cargo build --release`
- [ ] Run tests: `cargo test --release`
- [ ] Benchmark: `cargo bench`

### Command Examples

```bash
# Index a directory
./target/release/embed-search index ./src --extensions "rs,py,js,ts"

# Search with GPU acceleration
CUDA_VISIBLE_DEVICES=0 ./target/release/embed-search search "async trait implementation"

# Search with specific settings
./target/release/embed-search search "database connection pool" \
  --limit 20 \
  --index-path ./search_index

# Extract symbols from a file
./target/release/embed-search symbols ./src/main.rs

# Show system status
./target/release/embed-search status

# Batch index with progress
find . -name "*.rs" | xargs -I {} \
  ./target/release/embed-search index {} --verbose

# Test embedding generation
echo "fn test() { return 42; }" | \
  ./target/release/embed-search embed --stdin
```

### Performance Tuning

```bash
# Maximum GPU performance
export CUDA_VISIBLE_DEVICES=0
export GGUF_GPU_LAYERS=-1
export GGUF_BATCH_SIZE=64
export GGUF_CACHE_SIZE=2000

# CPU-only optimization
export GGUF_GPU_LAYERS=0
export GGUF_N_THREADS=$(nproc)
export GGUF_BATCH_SIZE=16

# Memory-constrained systems
export GGUF_GPU_LAYERS=20  # Partial offload
export GGUF_BATCH_SIZE=8
export GGUF_CACHE_SIZE=500
```