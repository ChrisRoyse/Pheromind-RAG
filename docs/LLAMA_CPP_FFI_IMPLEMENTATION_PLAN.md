# llama-cpp-2 Implementation Plan for GGUF Embeddings

## Executive Summary

This document provides a comprehensive implementation plan for integrating the `nomic-embed-code.Q4_K_M.gguf` model using the **llama-cpp-2** and **llama-cpp-sys-2** Rust bindings. This approach leverages utilityai's battle-tested bindings that stay tightly coupled with llama.cpp, providing a safe and performant interface to llama.cpp's embedding capabilities.

## Why llama-cpp-2?

**Maintainer**: utilityai  
**Philosophy**: Stay tightly coupled with llama.cpp, mimicking its API closely while being safe  
**Build System**: bindgen for automatic FFI generation  
**Key Advantages**:
- Direct mapping to llama.cpp C API
- Automatic bindings generation via bindgen
- Active maintenance and updates
- Proven production stability
- Safe Rust wrappers over unsafe FFI

## Core Dependencies

```toml
[dependencies]
llama-cpp-2 = { version = "0.1", features = ["cublas"] }
llama-cpp-sys-2 = "0.1"  # Low-level bindings
```  

## Build System Requirements

### System Dependencies

```bash
# Ubuntu/Debian
sudo apt-get update
sudo apt-get install -y \
    build-essential \
    cmake \
    pkg-config \
    libssl-dev \
    clang \
    libclang-dev

# macOS
brew install cmake llvm

# Windows (with MSVC)
# Install Visual Studio Build Tools
# Install LLVM from https://releases.llvm.org/
```

### GPU Support Dependencies

#### CUDA (NVIDIA)
```bash
# CUDA Toolkit 11.7+ required
wget https://developer.download.nvidia.com/compute/cuda/repos/ubuntu2204/x86_64/cuda-keyring_1.0-1_all.deb
sudo dpkg -i cuda-keyring_1.0-1_all.deb
sudo apt-get update
sudo apt-get install cuda-toolkit-11-8

# Set environment
export CUDA_PATH=/usr/local/cuda
export LD_LIBRARY_PATH=$CUDA_PATH/lib64:$LD_LIBRARY_PATH
export LLAMA_CUDA=1
```

#### Metal (macOS)
```bash
# Automatically available on macOS
export LLAMA_METAL=1
```

#### ROCm (AMD)
```bash
# Install ROCm 5.4+
export LLAMA_HIPBLAS=1
export ROCM_PATH=/opt/rocm
```

## Detailed Implementation Plan

### Phase 1: Project Setup & Dependencies

#### 1.1 Create Build Script (build.rs)

```rust
// build.rs
use std::env;
use std::path::PathBuf;

fn main() {
    // Detect available acceleration
    let cuda_available = env::var("CUDA_PATH").is_ok();
    let metal_available = cfg!(target_os = "macos");
    let rocm_available = env::var("ROCM_PATH").is_ok();
    
    // Set build features
    if cuda_available {
        println!("cargo:rustc-cfg=feature=\"cuda\"");
        println!("cargo:rustc-env=LLAMA_CUDA=1");
    }
    
    if metal_available {
        println!("cargo:rustc-cfg=feature=\"metal\"");
        println!("cargo:rustc-env=LLAMA_METAL=1");
    }
    
    if rocm_available {
        println!("cargo:rustc-cfg=feature=\"hipblas\"");
        println!("cargo:rustc-env=LLAMA_HIPBLAS=1");
    }
    
    // Link to system libraries
    println!("cargo:rustc-link-lib=dylib=stdc++");
    
    // Optional: Build llama.cpp from source
    if env::var("BUILD_LLAMA_FROM_SOURCE").is_ok() {
        build_llama_cpp();
    }
}

fn build_llama_cpp() {
    let dst = cmake::Config::new("llama.cpp")
        .define("LLAMA_BUILD_TESTS", "OFF")
        .define("LLAMA_BUILD_EXAMPLES", "OFF")
        .define("BUILD_SHARED_LIBS", "ON")
        .build();
    
    println!("cargo:rustc-link-search=native={}/lib", dst.display());
    println!("cargo:rustc-link-lib=llama");
}
```

#### 1.2 Cargo.toml Configuration

```toml
[package]
name = "embed-search"
version = "0.3.0"
edition = "2021"
build = "build.rs"

[dependencies]
# Core dependencies
anyhow = "1.0"
thiserror = "2.0"
tokio = { version = "1.0", features = ["full"] }

# llama-cpp-2 bindings
llama-cpp-2 = { version = "0.1", features = ["cublas"] }
llama-cpp-sys-2 = "0.1"

# Existing dependencies
lancedb = "0.8"
tantivy = "0.22"
arrow = "52"
arrow-array = "52"
arrow-schema = "52"

# FFI utilities
libc = "0.2"
once_cell = "1.19"
num_cpus = "1.16"
parking_lot = "0.12"
lru = "0.12"

[build-dependencies]
cc = "1.0"
cmake = "0.1"

[features]
default = ["cuda"]
cuda = []
metal = []
hipblas = []
```

### Phase 2: llama-cpp-2 Integration

#### 2.1 Using llama-cpp-2 API

The llama-cpp-2 library provides both low-level (llama-cpp-sys-2) and high-level safe wrappers. We leverage the library's provided types and functions:

```rust
// src/llama_bindings.rs
use llama_cpp_2::{
    context::LlamaContext,
    context::params::LlamaContextParams,
    llama::LlamaModel,
    model::params::LlamaModelParams,
    model::params::LlamaModelParamsBuilder,
    model::AddBos,
    token::data_array::LlamaTokenDataArray,
};
use llama_cpp_sys_2::{
    llama_model,
    llama_context,
    llama_batch,
    llama_token,
    llama_pooling_type,
};

use std::path::Path;

```

#### 2.2 Safe Rust Wrapper Using llama-cpp-2

```rust
// src/llama_wrapper.rs
use llama_cpp_2::{
    context::{LlamaContext, params::LlamaContextParams},
    llama::LlamaModel,
    model::{LlamaModelParams, AddBos},
    token::LlamaToken,
};
use anyhow::{Result, Context, bail};
use std::ffi::CString;
use std::path::Path;
use std::sync::Arc;
use parking_lot::Mutex;

/// Thread-safe GGUF model wrapper using llama-cpp-2
pub struct GGUFModel {
    model: Arc<LlamaModel>,
    embedding_dim: usize,
    model_path: String,
}

impl GGUFModel {
    /// Load GGUF model from file
    pub fn load_from_file<P: AsRef<Path>>(path: P, gpu_layers: i32) -> Result<Self> {
        let path = path.as_ref();
        let path_str = path.to_str()
            .context("Invalid path encoding")?;
        
        // Initialize llama backend
        llama_cpp_2::llama_backend_init();
        
        // Set model parameters using llama-cpp-2 builder
        let model_params = LlamaModelParams::default()
            .with_n_gpu_layers(gpu_layers as u32)
            .with_use_mmap(true)
            .with_use_mlock(false);
        
        // Load model using llama-cpp-2
        let model = LlamaModel::load_from_file(path, model_params)
            .context(format!("Failed to load model from: {}", path.display()))?;
        
        // Get embedding dimension
        let embedding_dim = model.n_embd() as usize;
        
        Ok(Self {
            model: Arc::new(model),
            embedding_dim,
            model_path: path_str.to_string(),
        })
    }
    
    pub fn embedding_dim(&self) -> usize {
        self.embedding_dim
    }
    
    pub fn model(&self) -> &Arc<LlamaModel> {
        &self.model
    }
}

/// GGUF embedding context using llama-cpp-2
pub struct GGUFContext {
    context: LlamaContext,
    model: Arc<GGUFModel>,
    embedding_dim: usize,
}

impl GGUFContext {
    /// Create new context for embeddings
    pub fn new_with_model(model: Arc<GGUFModel>, context_size: u32) -> Result<Self> {
        // Context parameters for embeddings using llama-cpp-2
        let ctx_params = LlamaContextParams::default()
            .with_n_ctx(context_size)
            .with_n_batch(2048)
            .with_n_threads(num_cpus::get() as u32)
            .with_n_threads_batch(num_cpus::get() as u32)
            .with_embeddings(true)  // CRITICAL for embeddings
            .with_logits_all(false)
            .with_offload_kqv(true);
        
        // Create context using llama-cpp-2
        let context = model.model()
            .new_context(ctx_params)
            .context("Failed to create context")?;
        
        Ok(Self {
            context,
            model: model.clone(),
            embedding_dim: model.embedding_dim(),
        })
    }
    
    /// Generate embeddings for text
    pub fn embed(&mut self, text: &str) -> Result<Vec<f32>> {
        // Tokenize using llama-cpp-2
        let tokens = self.model.model()
            .tokenize(text, true)?;  // add_special = true
        
        // Create batch and decode
        let n_past = 0;
        self.context.decode(&tokens, n_past)
            .context("Failed to decode tokens")?;
        
        // Extract embeddings using llama-cpp-2
        let embeddings = self.context.embeddings()
            .context("Failed to get embeddings")?;
        
        // Convert to Vec<f32> and normalize
        let mut embedding_vec: Vec<f32> = embeddings.to_vec();
        
        // L2 normalization
        let norm = embedding_vec.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for emb in &mut embedding_vec {
                *emb /= norm;
            }
        }
        
        Ok(embedding_vec)
    }
    
    /// Batch embedding generation
    pub fn embed_batch(&mut self, texts: Vec<String>) -> Result<Vec<Vec<f32>>> {
        let mut all_embeddings = Vec::new();
        
        for text in texts {
            let embedding = self.embed(&text)?;
            all_embeddings.push(embedding);
        }
        
        Ok(all_embeddings)
    }
}

// Note: llama-cpp-2 automatically handles cleanup through its Drop implementations
```

### Phase 3: Integration Layer

#### 3.1 Task-Specific Prefixes

```rust
// src/embedding_prefixes.rs
use anyhow::Result;

/// Task types for nomic-embed-code model
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
    /// Get the appropriate prefix for nomic-embed-code
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
    
    /// Apply prefix to text
    pub fn apply_prefix(&self, text: &str) -> String {
        format!("{}{}", self.prefix(), text)
    }
}

/// Language-specific code formatting
pub struct CodeFormatter;

impl CodeFormatter {
    /// Format code with language context
    pub fn format_code(code: &str, language: &str) -> String {
        match language {
            "rust" => format!("// Rust\n{}", code),
            "python" => format!("# Python\n{}", code),
            "javascript" | "typescript" => format!("// JavaScript\n{}", code),
            "go" => format!("// Go\n{}", code),
            "java" => format!("// Java\n{}", code),
            "cpp" | "c" => format!("// C++\n{}", code),
            _ => code.to_string(),
        }
    }
}
```

#### 3.2 High-Level Embedder Interface

```rust
// src/gguf_embedder.rs
use crate::llama_wrapper::{GGUFModel, GGUFContext};
use crate::embedding_prefixes::{EmbeddingTask, CodeFormatter};
use anyhow::Result;
use std::sync::Arc;
use parking_lot::Mutex;
use lru::LruCache;
use std::num::NonZeroUsize;

/// Configuration for GGUF embedder
#[derive(Debug, Clone)]
pub struct GGUFEmbedderConfig {
    pub model_path: String,
    pub context_size: u32,
    pub gpu_layers: i32,
    pub batch_size: usize,
    pub cache_size: usize,
    pub normalize: bool,
}

impl Default for GGUFEmbedderConfig {
    fn default() -> Self {
        Self {
            model_path: "./src/model/nomic-embed-code.Q4_K_M.gguf".to_string(),
            context_size: 8192,
            gpu_layers: -1, // Use all GPU layers
            batch_size: 32,
            cache_size: 1000,
            normalize: true,
        }
    }
}

/// Thread-safe GGUF embedder with caching
pub struct GGUFEmbedder {
    model: Arc<GGUFModel>,
    context: Arc<Mutex<GGUFContext>>,
    cache: Arc<Mutex<LruCache<String, Vec<f32>>>>,
    config: GGUFEmbedderConfig,
}

impl GGUFEmbedder {
    /// Create new embedder
    pub fn new(config: GGUFEmbedderConfig) -> Result<Self> {
        // Load model
        let model = Arc::new(GGUFModel::load_from_file(
            &config.model_path,
            config.gpu_layers,
        )?);
        
        // Create context
        let context = Arc::new(Mutex::new(
            GGUFContext::new_with_model(model.clone(), config.context_size)?
        ));
        
        // Initialize cache
        let cache_size = NonZeroUsize::new(config.cache_size).unwrap();
        let cache = Arc::new(Mutex::new(LruCache::new(cache_size)));
        
        Ok(Self {
            model,
            context,
            cache,
            config,
        })
    }
    
    /// Generate embedding with caching
    pub fn embed(&self, text: &str, task: EmbeddingTask) -> Result<Vec<f32>> {
        // Apply task prefix
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
        
        // Cache result
        {
            let mut cache = self.cache.lock();
            cache.put(prefixed_text.clone(), embedding.clone());
        }
        
        Ok(embedding)
    }
    
    /// Batch embedding generation
    pub fn embed_batch(&self, texts: Vec<String>, task: EmbeddingTask) -> Result<Vec<Vec<f32>>> {
        let mut results = Vec::new();
        let mut uncached_indices = Vec::new();
        let mut uncached_texts = Vec::new();
        
        // Check cache first
        for (i, text) in texts.iter().enumerate() {
            let prefixed = task.apply_prefix(text);
            let mut cache = self.cache.lock();
            if let Some(cached) = cache.get(&prefixed) {
                results.push(Some(cached.clone()));
            } else {
                results.push(None);
                uncached_indices.push(i);
                uncached_texts.push(prefixed);
            }
        }
        
        // Process uncached in batches
        for chunk in uncached_texts.chunks(self.config.batch_size) {
            let mut ctx = self.context.lock();
            let embeddings = ctx.embed_batch(chunk.to_vec())?;
            
            // Update results and cache
            for (chunk_idx, embedding) in embeddings.into_iter().enumerate() {
                let result_idx = uncached_indices[chunk_idx];
                results[result_idx] = Some(embedding.clone());
                
                // Cache the result
                let mut cache = self.cache.lock();
                cache.put(chunk[chunk_idx].clone(), embedding);
            }
        }
        
        // Collect results
        Ok(results.into_iter().map(|r| r.unwrap()).collect())
    }
    
    /// Embed code with language awareness
    pub fn embed_code(&self, code: &str, language: &str) -> Result<Vec<f32>> {
        let formatted = CodeFormatter::format_code(code, language);
        self.embed(&formatted, EmbeddingTask::CodeDefinition)
    }
    
    /// Get embedding dimension
    pub fn dimension(&self) -> usize {
        self.model.embedding_dim()
    }
}
```

### Phase 4: Error Handling & Safety

#### 4.1 Error Types

```rust
// src/errors.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GGUFError {
    #[error("Model loading failed: {0}")]
    ModelLoadError(String),
    
    #[error("Context creation failed: {0}")]
    ContextError(String),
    
    #[error("Tokenization failed: {0}")]
    TokenizationError(String),
    
    #[error("Embedding generation failed: {0}")]
    EmbeddingError(String),
    
    #[error("GPU initialization failed: {0}")]
    GPUError(String),
    
    #[error("Memory allocation failed: {0}")]
    MemoryError(String),
    
    #[error("FFI error: {0}")]
    FFIError(String),
    
    #[error("Invalid configuration: {0}")]
    ConfigError(String),
}

pub type GGUFResult<T> = Result<T, GGUFError>;
```

#### 4.2 Safety Guarantees with llama-cpp-2

The llama-cpp-2 library provides comprehensive safety wrappers:

- **Automatic resource management**: All resources are properly cleaned up through Rust's Drop trait
- **Thread safety**: Models and contexts are Send + Sync where appropriate
- **Memory safety**: No manual memory management required
- **Error handling**: All operations return proper Result types
- **Type safety**: Strong typing prevents misuse of the API

This eliminates the need for custom safety wrappers that would be required with raw FFI bindings.

### Phase 5: Testing Strategy

#### 5.1 Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_model_loading() {
        let config = GGUFEmbedderConfig::default();
        let embedder = GGUFEmbedder::new(config);
        assert!(embedder.is_ok());
        
        if let Ok(emb) = embedder {
            assert_eq!(emb.dimension(), 768);
        }
    }
    
    #[test]
    fn test_embedding_generation() {
        let config = GGUFEmbedderConfig::default();
        let embedder = GGUFEmbedder::new(config).unwrap();
        
        let text = "fn main() { println!(\"Hello, world!\"); }";
        let embedding = embedder.embed(text, EmbeddingTask::CodeDefinition).unwrap();
        
        assert_eq!(embedding.len(), 768);
        assert!(embedding.iter().any(|&x| x != 0.0));
        
        // Check normalization
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 0.001);
    }
    
    #[test]
    fn test_batch_processing() {
        let config = GGUFEmbedderConfig {
            batch_size: 2,
            ..Default::default()
        };
        let embedder = GGUFEmbedder::new(config).unwrap();
        
        let texts = vec![
            "struct User { name: String }".to_string(),
            "fn process(data: Vec<u8>) -> Result<()>".to_string(),
            "impl Display for User {}".to_string(),
        ];
        
        let embeddings = embedder.embed_batch(texts, EmbeddingTask::CodeDefinition).unwrap();
        
        assert_eq!(embeddings.len(), 3);
        for emb in &embeddings {
            assert_eq!(emb.len(), 768);
        }
    }
    
    #[test]
    fn test_caching() {
        let config = GGUFEmbedderConfig {
            cache_size: 10,
            ..Default::default()
        };
        let embedder = GGUFEmbedder::new(config).unwrap();
        
        let text = "cached text";
        
        // First call - generates embedding
        let emb1 = embedder.embed(text, EmbeddingTask::SearchQuery).unwrap();
        
        // Second call - should use cache
        let emb2 = embedder.embed(text, EmbeddingTask::SearchQuery).unwrap();
        
        assert_eq!(emb1, emb2);
    }
    
    #[test]
    fn test_prefix_application() {
        let task = EmbeddingTask::SearchQuery;
        let text = "rust async runtime";
        let prefixed = task.apply_prefix(text);
        
        assert_eq!(prefixed, "search_query: rust async runtime");
    }
}
```

#### 5.2 Integration Tests

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_with_vector_storage() {
        // Initialize embedder
        let embedder = GGUFEmbedder::new(Default::default()).unwrap();
        
        // Generate embeddings
        let code_samples = vec![
            "async fn fetch_data(url: &str) -> Result<String>".to_string(),
            "struct Database { connection: Pool<Postgres> }".to_string(),
        ];
        
        let embeddings = embedder.embed_batch(
            code_samples.clone(),
            EmbeddingTask::CodeDefinition
        ).unwrap();
        
        // Store in LanceDB
        let storage = VectorStorage::new("./test_db").await.unwrap();
        storage.store(code_samples, embeddings, vec![
            "fetch.rs".to_string(),
            "db.rs".to_string(),
        ]).await.unwrap();
        
        // Search
        let query_embedding = embedder.embed(
            "async database connection",
            EmbeddingTask::SearchQuery
        ).unwrap();
        
        let results = storage.search(query_embedding, 5).await.unwrap();
        assert!(!results.is_empty());
    }
}
```

#### 5.3 Performance Benchmarks

```rust
#[cfg(test)]
mod benches {
    use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
    
    fn benchmark_single_embedding(c: &mut Criterion) {
        let embedder = GGUFEmbedder::new(Default::default()).unwrap();
        
        c.bench_function("single_embedding", |b| {
            b.iter(|| {
                embedder.embed(
                    black_box("fn test() { return 42; }"),
                    EmbeddingTask::CodeDefinition
                )
            })
        });
    }
    
    fn benchmark_batch_sizes(c: &mut Criterion) {
        let mut group = c.benchmark_group("batch_embedding");
        
        for size in [1, 8, 16, 32, 64].iter() {
            let embedder = GGUFEmbedder::new(GGUFEmbedderConfig {
                batch_size: *size,
                ..Default::default()
            }).unwrap();
            
            let texts: Vec<String> = (0..*size)
                .map(|i| format!("fn test_{}() {{ return {}; }}", i, i))
                .collect();
            
            group.bench_with_input(
                BenchmarkId::from_parameter(size),
                size,
                |b, _| {
                    b.iter(|| {
                        embedder.embed_batch(
                            black_box(texts.clone()),
                            EmbeddingTask::CodeDefinition
                        )
                    })
                }
            );
        }
        group.finish();
    }
    
    fn benchmark_gpu_vs_cpu(c: &mut Criterion) {
        let mut group = c.benchmark_group("gpu_vs_cpu");
        
        // CPU benchmark
        let cpu_embedder = GGUFEmbedder::new(GGUFEmbedderConfig {
            gpu_layers: 0,
            ..Default::default()
        }).unwrap();
        
        group.bench_function("cpu", |b| {
            b.iter(|| {
                cpu_embedder.embed(
                    black_box("test code"),
                    EmbeddingTask::CodeDefinition
                )
            })
        });
        
        // GPU benchmark
        let gpu_embedder = GGUFEmbedder::new(GGUFEmbedderConfig {
            gpu_layers: -1,
            ..Default::default()
        }).unwrap();
        
        group.bench_function("gpu", |b| {
            b.iter(|| {
                gpu_embedder.embed(
                    black_box("test code"),
                    EmbeddingTask::CodeDefinition
                )
            })
        });
        
        group.finish();
    }
}
```

### Phase 6: Complete GGUF Integration (FastEmbed Already Removed)

#### 6.1 Compatibility Layer

```rust
// src/compat.rs
use anyhow::Result;

/// Trait for embedding providers
pub trait EmbeddingProvider: Send + Sync {
    fn embed(&self, text: &str) -> Result<Vec<f32>>;
    fn embed_batch(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>>;
    fn dimension(&self) -> usize;
}

/// Direct GGUF Embedder using llama-cpp-2
pub struct NomicEmbedder {
    embedder: GGUFEmbedder,
}

impl NomicEmbedder {
    pub fn new() -> Result<Self> {
        let config = GGUFEmbedderConfig::default();
        Ok(Self {
            embedder: GGUFEmbedder::new(config)?,
        })
    }
    
    pub fn embed(&self, text: &str) -> Result<Vec<f32>> {
        self.embedder.embed(text, EmbeddingTask::SearchDocument)
    }
    
    pub fn embed_query(&self, query: &str) -> Result<Vec<f32>> {
        self.embedder.embed(query, EmbeddingTask::SearchQuery)
    }
    
    pub fn embed_batch(&self, documents: Vec<String>) -> Result<Vec<Vec<f32>>> {
        self.embedder.embed_batch(documents, EmbeddingTask::SearchDocument)
    }
}
```

## Performance Optimization

### GPU Memory Management

```rust
// src/gpu_manager.rs
use std::env;

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
        
        Self {
            device_count,
            selected_device,
        }
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
        
        // Metal detection (macOS)
        #[cfg(target_os = "macos")]
        {
            return 1; // Metal is always available on macOS
        }
        
        0
    }
    
    pub fn optimal_layers(&self, model_size: usize) -> i32 {
        if self.device_count == 0 {
            return 0; // CPU only
        }
        
        // Estimate based on model size
        // nomic-embed-code.Q4_K_M.gguf is ~4.3GB
        if model_size < 6_000_000_000 {
            -1 // Full offload for models < 6GB
        } else {
            28 // Partial offload for larger models
        }
    }
}
```

### Memory Pool

```rust
// src/memory_pool.rs
use std::sync::Arc;
use parking_lot::Mutex;

pub struct EmbeddingMemoryPool {
    pool: Arc<Mutex<Vec<Vec<f32>>>>,
    dimension: usize,
}

impl EmbeddingMemoryPool {
    pub fn new(dimension: usize, initial_capacity: usize) -> Self {
        let mut pool = Vec::with_capacity(initial_capacity);
        for _ in 0..initial_capacity {
            pool.push(vec![0.0; dimension]);
        }
        
        Self {
            pool: Arc::new(Mutex::new(pool)),
            dimension,
        }
    }
    
    pub fn acquire(&self) -> Vec<f32> {
        let mut pool = self.pool.lock();
        pool.pop().unwrap_or_else(|| vec![0.0; self.dimension])
    }
    
    pub fn release(&self, mut buffer: Vec<f32>) {
        buffer.clear();
        buffer.resize(self.dimension, 0.0);
        
        let mut pool = self.pool.lock();
        if pool.len() < pool.capacity() {
            pool.push(buffer);
        }
    }
}
```

## Deployment Checklist

### Pre-deployment
- [ ] Verify GGUF model file exists at `./src/model/nomic-embed-code.Q4_K_M.gguf`
- [ ] Test model loading and embedding generation
- [ ] Benchmark performance vs placeholder implementation
- [ ] Compare with expected FastEmbed performance metrics
- [ ] Verify embedding dimensions (768)
- [ ] Test GPU acceleration if available
- [ ] Run full test suite

### Build Commands

```bash
# Development build
cargo build

# Release build with optimizations
cargo build --release

# Build with specific GPU acceleration
cargo build --release --features cuda
cargo build --release --features metal
cargo build --release --features hipblas

# Run tests
cargo test --all-features
cargo test --release

# Run benchmarks
cargo bench

# Check for unsafe code
cargo miri test
```

### Environment Variables

```bash
# GPU configuration
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

## Troubleshooting

### Common Issues

1. **Model Loading Fails**
   - Check file path and permissions
   - Verify GGUF format compatibility
   - Check available memory (needs ~5GB)

2. **GPU Not Detected**
   - Verify CUDA/Metal/ROCm installation
   - Check environment variables
   - Run with `RUST_LOG=debug` for diagnostics

3. **Segmentation Faults**
   - Usually indicates FFI issues
   - Check llama.cpp version compatibility
   - Verify struct layouts match C headers

4. **Poor Performance**
   - Ensure release build (`--release`)
   - Check GPU offloading
   - Verify batch sizes
   - Monitor memory usage

## Conclusion

This implementation plan provides a production-ready approach to integrating GGUF models using the llama-cpp-2 and llama-cpp-sys-2 Rust bindings. The architecture maintains safety through Rust's type system while leveraging the performance of native C++ implementations through utilityai's well-maintained bindings. The phased approach allows for incremental testing and validation at each step, with llama-cpp-2 providing the optimal balance of safety, performance, and maintainability.

## References

- [llama-cpp-2 Crate Documentation](https://docs.rs/llama-cpp-2)
- [llama-cpp-sys-2 Crate Documentation](https://docs.rs/llama-cpp-sys-2)  
- [llama.cpp Documentation](https://github.com/ggml-org/llama.cpp)
- [GGUF Format Specification](https://github.com/ggml-org/ggml/blob/master/docs/gguf.md)
- [Nomic Embed Documentation](https://docs.nomic.ai/reference/endpoints/nomic-embed-text)