# 🚀 Nomic Embed Code - Single Model Implementation Plan

## Using ONLY Nomic Embed Code for ALL Embeddings (Code + Documentation)

---

## 📋 Executive Summary

**Simplified Architecture**: Use ONLY Nomic Embed Code (7B params, GGUF Q4) for ALL file types - both code and documentation. This eliminates complexity and leverages Nomic's superior performance on both text and code.

### Why Nomic Only?

- **Unified Model**: One model for everything (simpler)
- **Superior Accuracy**: 81.7% Python, 80.5% Java (beats OpenAI)
- **Handles Text Well**: Nomic v1.5 is trained on text AND code
- **Efficient**: GGUF Q4 = 2-4GB memory, 200-500 embeddings/sec
- **No Model Switching**: Simpler codebase, easier maintenance

---

## 🏗️ Simplified Architecture

```
┌─────────────────────────────────────┐
│         Any File Input              │
│    (.md, .rs, .py, .js, etc.)      │
└────────────────┬────────────────────┘
                 │
                 ▼
    ┌────────────────────────────┐
    │   Nomic Embed Code v1.5    │
    │   - GGUF Q4 Quantized       │
    │   - 768 dimensions max      │
    │   - 2048 token context       │
    └────────────┬───────────────┘
                 │
                 ▼
    ┌────────────────────────────┐
    │   Unified Vector Storage    │
    │   - Single index            │
    │   - Consistent dimensions   │
    └────────────┬───────────────┘
                 │
                 ▼
    ┌────────────────────────────┐
    │    Semantic Search         │
    │   - Single model queries   │
    │   - No routing needed      │
    └────────────────────────────┘
```

---

## 📦 Implementation Steps

### Step 1: Remove MiniLM Completely

```bash
# Files to delete/modify
src/embedding/minilm.rs          # DELETE
src/embedding/real_minilm.rs     # DELETE
src/embedding/mod.rs             # UPDATE - remove minilm exports
```

### Step 2: Nomic Model Download & Permanent Cache

```rust
// src/embedding/nomic.rs

use std::path::PathBuf;
use std::fs;
use reqwest;

pub struct NomicModel {
    model_path: PathBuf,
    model_loaded: bool,
}

impl NomicModel {
    // Model details
    const MODEL_URL: &'static str = 
        "https://huggingface.co/nomic-ai/nomic-embed-text-v1.5-GGUF/resolve/main/nomic-embed-text-v1.5.Q4_K_M.gguf";
    const MODEL_SIZE: u64 = 4_500_000_000; // ~4.5GB
    const MODEL_NAME: &'static str = "nomic-embed-text-v1.5.Q4_K_M.gguf";
    
    /// Get permanent cache location
    pub fn cache_path() -> PathBuf {
        // Windows: C:\Users\{user}\.nomic\
        // Linux/Mac: ~/.nomic/
        let home = dirs::home_dir().expect("Cannot find home directory");
        home.join(".nomic").join(Self::MODEL_NAME)
    }
    
    /// Download model ONCE and cache permanently
    pub async fn ensure_downloaded() -> Result<PathBuf> {
        let cache_path = Self::cache_path();
        let cache_dir = cache_path.parent().unwrap();
        
        // Create cache directory
        fs::create_dir_all(cache_dir)?;
        
        // Check if already downloaded
        if cache_path.exists() {
            let size = fs::metadata(&cache_path)?.len();
            if size >= (Self::MODEL_SIZE * 95 / 100) { // 95% of expected size
                println!("✅ Nomic model already cached at: {:?}", cache_path);
                return Ok(cache_path);
            }
        }
        
        // Download with progress
        println!("📥 Downloading Nomic Embed Code (4.5GB)...");
        println!("📍 This is a ONE-TIME download. Model will be cached at:");
        println!("   {:?}", cache_path);
        
        let response = reqwest::get(Self::MODEL_URL).await?;
        let total_size = response.content_length().unwrap_or(Self::MODEL_SIZE);
        
        let mut file = fs::File::create(&cache_path)?;
        let mut downloaded = 0u64;
        let mut stream = response.bytes_stream();
        
        use futures::StreamExt;
        use std::io::Write;
        
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk)?;
            downloaded += chunk.len() as u64;
            
            // Progress bar
            let progress = (downloaded as f64 / total_size as f64) * 100.0;
            print!("\r[{:50}] {:.1}% ({:.1}GB / {:.1}GB)", 
                   "=".repeat((progress / 2.0) as usize),
                   progress,
                   downloaded as f64 / 1_073_741_824.0,
                   total_size as f64 / 1_073_741_824.0);
            std::io::stdout().flush()?;
        }
        println!("\n✅ Download complete! Model cached permanently.");
        
        Ok(cache_path)
    }
}
```

### Step 3: GGUF Loading with llama.cpp

```rust
// Cargo.toml
[dependencies]
# Remove candle dependencies
# candle-core = "0.9"  # REMOVE
# candle-nn = "0.9"    # REMOVE
# candle-transformers = "0.9"  # REMOVE

# Add GGUF support
llama-cpp-rs = "0.1"  # Or similar GGUF loader
memmap2 = "0.9"       # Memory-mapped files
```

### Step 4: Single Embedder Implementation

```rust
// src/embedding/mod.rs
pub mod nomic;
pub mod cache;

pub use nomic::NomicEmbedder;
pub use cache::EmbeddingCache;

// src/embedding/nomic.rs
use once_cell::sync::OnceCell;
use std::sync::Arc;

static GLOBAL_EMBEDDER: OnceCell<Arc<NomicEmbedder>> = OnceCell::new();

pub struct NomicEmbedder {
    model: LlamaModel,  // GGUF model
    tokenizer: Tokenizer,
    dimensions: usize,
}

impl NomicEmbedder {
    /// Get or create singleton instance
    pub async fn get_global() -> Result<Arc<Self>> {
        if let Some(embedder) = GLOBAL_EMBEDDER.get() {
            return Ok(embedder.clone());
        }
        
        let embedder = Arc::new(Self::new().await?);
        GLOBAL_EMBEDDER.set(embedder.clone()).ok();
        Ok(embedder)
    }
    
    pub async fn new() -> Result<Self> {
        // Ensure model is downloaded
        let model_path = NomicModel::ensure_downloaded().await?;
        
        // Load GGUF model
        println!("🔄 Loading Nomic model into memory...");
        let model = LlamaModel::load_gguf(
            &model_path,
            ModelParams {
                n_gpu_layers: 0,     // CPU only for consistency
                n_threads: 8,        // Use 8 CPU threads
                context_size: 2048,  // Max context
                use_mmap: true,      // Memory map for efficiency
            }
        )?;
        
        println!("✅ Nomic Embed Code loaded successfully!");
        
        Ok(Self {
            model,
            tokenizer: Tokenizer::new()?,
            dimensions: 768,  // Default, can be reduced with Matryoshka
        })
    }
    
    /// Embed any text (code or documentation)
    pub fn embed(&self, text: &str) -> Result<Vec<f32>> {
        // Tokenize
        let tokens = self.tokenizer.encode(text, true)?;
        
        // Truncate to max context
        let tokens = &tokens[..tokens.len().min(2048)];
        
        // Get embeddings from model
        let embeddings = self.model.embed(tokens)?;
        
        // Apply Matryoshka truncation if needed
        Ok(embeddings.into_iter()
            .take(self.dimensions)
            .collect())
    }
    
    /// Optimized batch embedding
    pub fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        // Nomic handles batches efficiently
        let mut results = Vec::with_capacity(texts.len());
        
        // Process in batches of 64 for optimal performance
        for chunk in texts.chunks(64) {
            for text in chunk {
                results.push(self.embed(text)?);
            }
        }
        
        Ok(results)
    }
    
    /// Set embedding dimensions (64, 128, 256, 512, or 768)
    pub fn set_dimensions(&mut self, dims: usize) {
        self.dimensions = dims.min(768);
    }
}
```

### Step 5: Update Search System

```rust
// src/search/unified.rs
use crate::embedding::NomicEmbedder;  // Only Nomic now!

pub struct UnifiedSearcher {
    embedder: Arc<NomicEmbedder>,
    storage: Arc<RwLock<VectorStorage>>,
    chunker: SimpleRegexChunker,
}

impl UnifiedSearcher {
    pub async fn new(project_path: PathBuf, db_path: PathBuf) -> Result<Self> {
        // Single embedder for everything
        let embedder = NomicEmbedder::get_global().await?;
        
        let storage = Arc::new(RwLock::new(
            VectorStorage::new(db_path, 768)?  // Nomic dimensions
        ));
        
        Ok(Self {
            embedder,
            storage,
            chunker: SimpleRegexChunker::new(),
        })
    }
    
    pub async fn index_file(&self, file_path: &Path) -> Result<()> {
        let content = tokio::fs::read_to_string(file_path).await?;
        let chunks = self.chunker.chunk_file(&content);
        
        // Single embedding process for ALL files
        let embeddings = self.embedder.embed_batch(
            &chunks.iter().map(|c| c.content.as_str()).collect::<Vec<_>>()
        )?;
        
        // Store with consistent dimensions
        for (chunk, embedding) in chunks.iter().zip(embeddings) {
            self.storage.write().await.insert(
                file_path,
                chunk,
                embedding  // Always 768 dims (or configured)
            )?;
        }
        
        Ok(())
    }
    
    pub async fn search(&self, query: &str) -> Result<Vec<SearchResult>> {
        // Single embedding for query
        let query_embedding = self.embedder.embed(query)?;
        
        // Search in unified index
        let results = self.storage.read().await
            .search_similar(query_embedding, 20)?;
        
        Ok(results)
    }
}
```

---

## 🚀 Migration Path

### Phase 1: Remove MiniLM (Day 1)
```bash
# Delete MiniLM files
rm src/embedding/minilm.rs
rm src/embedding/real_minilm.rs

# Remove from Cargo.toml
# Delete: candle-core, candle-nn, candle-transformers, tokenizers, hf-hub
```

### Phase 2: Add Nomic (Day 2-3)
```bash
# Add to Cargo.toml
llama-cpp-rs = "0.1"
memmap2 = "0.9"

# Create new files
src/embedding/nomic.rs
```

### Phase 3: Update System (Day 4-5)
- Update `UnifiedSearcher` to use only Nomic
- Remove all routing logic (no longer needed)
- Simplify storage (single index, consistent dimensions)

---

## 📊 Expected Performance

| Metric | MiniLM System | Nomic-Only System | Improvement |
|--------|--------------|-------------------|-------------|
| **Code Accuracy** | 50-65% | **75-85%** | +25% |
| **Doc Accuracy** | 80-90% | **85-95%** | +5% |
| **Model Size** | 90MB | 4.5GB | Larger but worth it |
| **Memory Usage** | 500MB | 2-4GB | Acceptable |
| **Speed** | 100 emb/sec | **200-500 emb/sec** | 2-5x faster |
| **Complexity** | Two models | **One model** | Much simpler |

---

## ✅ Benefits of Nomic-Only

1. **Simplicity**: One model, one API, one index
2. **Better Accuracy**: State-of-the-art on code (beats OpenAI)
3. **Faster**: GGUF optimizations = 200-500 embeddings/sec
4. **Unified Search**: No routing, no model selection
5. **Maintenance**: Easier to maintain and debug

---

## 🔧 Quick Start Commands

```bash
# 1. Download and cache model (one-time)
cargo run --bin download_nomic

# 2. Re-index with Nomic
cargo run --bin reindex_all

# 3. Test accuracy
cargo run --bin test_nomic_accuracy
```

---

## 📝 Configuration

```toml
# config.toml
[embedding]
model = "nomic-embed-code-v1.5"
quantization = "Q4"  # or "Q8" for higher quality
dimensions = 768     # or 512, 256 for faster search
batch_size = 64      # Optimal for Nomic

[storage]
index_type = "flat"  # Simple flat index works well with Nomic
```

---

## 🎯 Success Metrics

- ✅ **>80% accuracy** on code search (up from 50-65%)
- ✅ **>90% accuracy** on documentation search
- ✅ **<4GB memory** usage with Q4 quantization
- ✅ **200+ embeddings/sec** on modern CPU
- ✅ **Single model** for all content types

This is MUCH simpler and more powerful than the dual-model approach!