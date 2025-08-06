# 🎯 Complete Context for Nomic Embed Implementation

## Your Mission
Transform the existing embedding search system to use ONLY Nomic Embed Code v1.5 (GGUF Q4 quantized) for ALL file types, removing MiniLM entirely.

## Current System State

### What Exists Now:
```
C:\code\embed\
├── src/
│   ├── embedding/
│   │   ├── minilm.rs         # DELETE THIS - old MiniLM wrapper
│   │   ├── real_minilm.rs    # DELETE THIS - actual MiniLM implementation
│   │   ├── cache.rs          # KEEP - LRU cache, works with any embedder
│   │   ├── nomic_code.rs     # MODIFY - partial Nomic implementation started
│   │   └── mod.rs            # MODIFY - remove MiniLM exports
│   ├── search/
│   │   ├── unified.rs        # MODIFY - currently uses MiniLM, needs Nomic
│   │   ├── fusion.rs         # KEEP - search result fusion logic
│   │   └── preprocessing.rs  # KEEP - query preprocessing
│   ├── storage/
│   │   ├── lancedb_storage.rs # KEEP or REPLACE - heavy dependencies
│   │   └── lightweight_storage.rs # OPTION - lighter alternative
│   └── main.rs               # MODIFY - update to use Nomic
├── Cargo.toml                # MODIFY - remove Candle deps, add GGUF support
└── docs/
    ├── nomic_only_plan.md    # The implementation plan
    └── codeembed.md          # Original dual-model plan (ignore)
```

### Current Dependencies to REMOVE from Cargo.toml:
```toml
# DELETE these lines:
candle-core = "0.9"
candle-nn = "0.9"
candle-transformers = "0.9"
tokenizers = "0.21"
hf-hub = { version = "0.3", features = ["tokio"] }
```

### Dependencies to ADD:
```toml
# For GGUF model support
candle-gguf = "0.7"  # If available, or:
llm = { version = "0.1", features = ["gguf"] }  # Alternative
# OR use llama-cpp bindings:
llama-cpp-rs = "0.1"

# For memory-mapped model loading
memmap2 = "0.9"
```

## Nomic Model Details

**Model**: Nomic Embed Text v1.5 GGUF
- **Download URL**: `https://huggingface.co/nomic-ai/nomic-embed-text-v1.5-GGUF/resolve/main/nomic-embed-text-v1.5.Q4_K_M.gguf`
- **Size**: ~4.5GB (Q4 quantization)
- **Dimensions**: 768 (supports Matryoshka: 64, 128, 256, 512, 768)
- **Context**: 2048 tokens max
- **Performance**: 200-500 embeddings/sec on CPU

## Implementation Requirements

### 1. Model Download & Caching
Create a one-time download system that:
- Downloads the 4.5GB GGUF model file
- Saves to `~/.nomic/nomic-embed-text-v1.5.Q4_K_M.gguf` (permanent cache)
- Shows progress bar during download
- Verifies file integrity (size check minimum)
- Never re-downloads if file exists and is valid

### 2. GGUF Model Loading
Implement model loading that:
- Memory-maps the GGUF file for efficiency
- Uses CPU inference (no GPU required)
- Loads as singleton (OnceCell pattern)
- Supports batch processing (optimal batch size: 64)

### 3. Embedding Generation
The embedder must:
- Accept any text (code or documentation)
- Return 768-dimensional float vectors (or configured dimension)
- Support Matryoshka dimension reduction
- Handle batches efficiently

### 4. Update Search System
Modify `src/search/unified.rs` to:
- Remove all MiniLM references
- Use NomicEmbedder for everything
- Remove file type routing (not needed)
- Keep the fusion and ranking logic

### 5. Storage Compatibility
Ensure storage works with:
- 768-dimensional vectors (not 384 like MiniLM)
- Single index for all file types
- Efficient similarity search

## Code Structure to Implement

### src/embedding/nomic.rs (Main Implementation)
```rust
use once_cell::sync::OnceCell;
use std::sync::Arc;
use std::path::PathBuf;

static GLOBAL_EMBEDDER: OnceCell<Arc<NomicEmbedder>> = OnceCell::new();

pub struct NomicEmbedder {
    model: /* GGUF model type */,
    dimensions: usize,
}

impl NomicEmbedder {
    pub async fn get_global() -> Result<Arc<Self>> {
        // Singleton pattern
    }
    
    pub async fn new() -> Result<Self> {
        // 1. Ensure model downloaded to ~/.nomic/
        // 2. Load GGUF model with memory mapping
        // 3. Initialize with 768 dimensions
    }
    
    pub fn embed(&self, text: &str) -> Result<Vec<f32>> {
        // Generate embedding for any text
    }
    
    pub fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        // Batch processing (optimal: 64 texts)
    }
}
```

### src/embedding/mod.rs (Updated Exports)
```rust
pub mod nomic;
pub mod cache;

pub use nomic::NomicEmbedder;
pub use cache::EmbeddingCache;
// DELETE: pub use minilm::*
// DELETE: pub use real_minilm::*
```

### Files to DELETE:
- `src/embedding/minilm.rs`
- `src/embedding/real_minilm.rs`

### Files to MODIFY:
- `src/search/unified.rs` - Replace MiniLM with Nomic
- `src/main.rs` - Update initialization
- `Cargo.toml` - Remove Candle, add GGUF support

## Testing Requirements

Create a test that:
1. Downloads the model (first run only)
2. Verifies it loads correctly
3. Generates embeddings for sample text
4. Confirms 768 dimensions
5. Tests batch processing
6. Measures performance (should be 200+ embeddings/sec)

## Expected Outcomes

After implementation:
- **Code search accuracy**: 75-85% (up from 50-65%)
- **Doc search accuracy**: 85-95% (maintained or improved)
- **Memory usage**: 2-4GB (acceptable for 7B param model)
- **Speed**: 200-500 embeddings/sec
- **Model stored permanently** at `~/.nomic/`

## Critical Notes

1. **Model MUST be cached permanently** - users should download only once
2. **Use memory mapping** for the 4.5GB model file (don't load entirely into RAM)
3. **Singleton pattern is essential** - only one model instance
4. **Remove ALL MiniLM code** - complete replacement
5. **Keep the caching system** - it works with any embedder

## Success Criteria

✅ Model downloads once and caches permanently
✅ All MiniLM code removed
✅ Single embedder handles all file types
✅ 768-dimensional embeddings generated
✅ Search accuracy improves to >75% on code
✅ System uses <4GB memory with Q4 model
✅ Performance: 200+ embeddings/second

## Tree-sitter Note
The Cargo.toml has tree-sitter dependencies added (lines 57-63). These can be used for enhanced symbol extraction but are optional for the basic implementation.

---

## Start Here:
1. First, implement the model download and caching in `src/embedding/nomic.rs`
2. Then implement GGUF model loading
3. Add embedding generation methods
4. Update the search system to use Nomic
5. Delete all MiniLM files
6. Test the complete system