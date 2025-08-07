# Dependency Relationships (Verified from Cargo.toml)

## Core Dependencies (Always Enabled)
```toml
# Fundamental
regex = "1.11"                    # Pattern matching
serde = "1.0"                     # Serialization
serde_json = "1.0"                # JSON support
anyhow = "1.0"                    # Error handling
thiserror = "1.0"                 # Error definitions
tokio = "1.43"                    # Async runtime (full features)
futures = "0.3"                   # Async primitives
async-trait = "0.1"               # Async traits

# Configuration
toml = "0.8"                      # TOML parsing
config = "0.13"                   # Config management
serde_yaml = "0.9"                # YAML support
clap = "4.4"                      # CLI parsing (derive feature)

# Data Structures
once_cell = "1.19"                # Lazy statics
lru = "0.12"                      # LRU cache
parking_lot = "0.12"              # Fast mutexes
chrono = "0.4"                    # Date/time (serde feature)

# Text Processing
unicode-normalization = "0.1"     # Unicode handling
unicode-segmentation = "1.10"     # Text segmentation
rust-stemmers = "1.2"             # Word stemming

# File System
walkdir = "2.4"                   # Directory traversal
tempfile = "3.12"                 # Temp file management

# Parallel Processing
rayon = "1.7"                     # Data parallelism

# Observability
log = "0.4"                       # Logging facade
tracing = "0.1"                   # Structured logging
tracing-subscriber = "0.3"        # Logging impl (env-filter, json, chrono)

# System
sysinfo = "0.30"                  # System monitoring
backoff = "0.4"                   # Retry logic

# Serialization
bincode = "1.3"                   # Binary encoding
sha2 = "0.10"                     # SHA hashing
```

## ML Feature Dependencies (`ml` feature flag)
```toml
# Model Loading
reqwest = "0.11"                  # HTTP client (stream feature)
dirs = "5.0"                      # System directories
memmap2 = "0.9"                   # Memory-mapped files

# Candle ML Framework
candle-core = "0.9"               # Core tensor ops
candle-nn = "0.9"                 # Neural network layers
candle-transformers = "0.9"       # Transformer models

# Tokenization
tokenizers = "0.21"               # Tokenization (onig feature)
hf-hub = "0.3"                    # HuggingFace hub (tokio feature)

# Utils
byteorder = "1.5"                 # Byte order conversion
rand = "0.8"                      # Random numbers
```

## Vector Database Feature (`vectordb` feature flag)
```toml
# LanceDB
lancedb = "0.21.2"                # Vector database
arrow = "55.0"                    # Arrow format
arrow-array = "55.0"              # Arrow arrays
arrow-schema = "55.0"             # Arrow schemas

# Legacy
sled = "0.34"                     # Embedded database (for migration)
```

## Tantivy Feature (`tantivy` feature flag)
```toml
tantivy = "0.24"                  # Full-text search
tantivy-jieba = "0.16"            # Chinese tokenization
```

## Tree-Sitter Feature (`tree-sitter` feature flag)
```toml
tree-sitter = "0.23"              # Parsing framework
tree-sitter-rust = "0.23"         # Rust parser
tree-sitter-python = "0.23"       # Python parser
tree-sitter-javascript = "0.23"   # JavaScript parser
tree-sitter-typescript = "0.23"   # TypeScript parser
tree-sitter-go = "0.23"           # Go parser
tree-sitter-java = "0.23"         # Java parser
tree-sitter-c = "0.23"            # C parser
tree-sitter-cpp = "0.23"          # C++ parser
tree-sitter-html = "0.23"         # HTML parser
tree-sitter-css = "0.23"          # CSS parser
tree-sitter-json = "0.23"         # JSON parser
tree-sitter-bash = "0.23"         # Bash parser
```

## Node.js Dependencies (package.json)
```json
{
  "dependencies": {
    "better-sqlite3": "^11.6.0",
    "claude-flow": "alpha",
    "sqlite3": "^5.1.7"
  }
}
```

## Module Dependencies Within Project

### Core Module Graph
```
main.rs
  ├── lib.rs (exports)
  ├── config/mod.rs (Config struct)
  ├── error.rs (error types)
  └── Commands implementations
      ├── search/unified.rs
      ├── storage/* (based on features)
      └── embedding/* (if ml feature)

search/unified.rs
  ├── search/bm25.rs
  ├── search/tantivy_search.rs (if tantivy)
  ├── search/fusion.rs
  └── storage/* (for persistence)

storage/lancedb_storage.rs
  ├── lancedb (if vectordb feature)
  ├── arrow* (if vectordb feature)
  └── error.rs

embedding/nomic.rs
  ├── candle-* (if ml feature)
  ├── tokenizers (if ml feature)
  └── embedding/cache.rs
```

### Feature Flag Dependencies
- **Default**: Only core dependencies
- **ml**: Adds Candle, tokenizers, model loading
- **vectordb**: Adds LanceDB, Arrow ecosystem
- **tantivy**: Adds Tantivy search engine
- **tree-sitter**: Adds all language parsers

### Import Patterns
```rust
// Core imports (always available)
use anyhow::{Result, Context};
use serde::{Serialize, Deserialize};
use tokio;
use clap::{Parser, Subcommand};

// Feature-gated imports
#[cfg(feature = "ml")]
use candle_core::{Tensor, Device};

#[cfg(feature = "vectordb")]
use lancedb::Connection;

#[cfg(feature = "tantivy")]
use tantivy::{Index, Document};

#[cfg(feature = "tree-sitter")]
use tree_sitter::{Parser, Query};
```

## Version Compatibility Notes
- Arrow versions (55.0) must match across arrow, arrow-array, arrow-schema
- Tree-sitter versions (0.23) must match across all parsers
- Candle versions (0.9) must match across core, nn, transformers
- Tokio 1.43 with full features for complete async support