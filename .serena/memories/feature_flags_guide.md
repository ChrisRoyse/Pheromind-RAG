# Feature Flags Guide

## Understanding Feature Dependencies

### Core Feature (Always Enabled)
- Basic BM25 search
- Text processing
- Configuration management
- No external ML dependencies

### Individual Features

#### `tree-sitter` - Symbol Indexing
- **Enables**: Code parsing and symbol extraction
- **Adds**: Support for extracting functions, classes, methods from code
- **Languages**: Rust, Python, JavaScript, TypeScript, Go, Java, C, C++, HTML, CSS, JSON, Bash
- **Key files**: src/search/symbol_index.rs, src/search/symbol_enhanced_searcher.rs
- **Required for**: Symbol-based search, code structure analysis

#### `ml` - Machine Learning
- **Enables**: Semantic embeddings using Candle and GGUF models
- **Adds**: all-MiniLM-L6-v2 embedding model support
- **Key files**: src/embedding/nomic.rs
- **Dependencies**: Heavy (~500MB models), increases compile time
- **Required for**: Semantic similarity search

#### `vectordb` - Vector Database
- **Enables**: LanceDB for vector storage
- **Adds**: Persistent vector storage and retrieval
- **Key files**: src/storage/lancedb.rs
- **Must use with**: 'ml' feature for embeddings
- **Required for**: Storing and searching embedding vectors

#### `tantivy` - Full-Text Search
- **Enables**: High-performance text indexing
- **Adds**: Fuzzy matching, complex queries, ranking
- **Key files**: src/search/tantivy_search.rs
- **Binaries**: tantivy_migrator, test_persistence
- **Required for**: Advanced text search capabilities

### Feature Combinations

#### Minimal Build
```bash
cargo build  # Just core features
```

#### Text Search Only
```bash
cargo build --features "core,tantivy"
# or
cargo build --features "search-basic"
```

#### Code Intelligence
```bash
cargo build --features "core,tree-sitter,tantivy"
# or
cargo build --features "search-advanced"
```

#### Full ML System
```bash
cargo build --features "tree-sitter,ml,vectordb,tantivy"
# or
cargo build --features "full-system"
```

## How to Check Active Features

### In Code
Look for:
```rust
#[cfg(feature = "ml")]
#[cfg(feature = "tantivy")]
#[cfg(all(feature = "ml", feature = "vectordb"))]
```

### Finding Feature-Gated Code
Search patterns:
- `#[cfg(feature` - Find feature-gated code
- `#[cfg(not(feature` - Find code when feature is disabled
- `cfg_if!` - Conditional compilation blocks

## Important Notes
1. Some binaries require specific features (check src/bin/)
2. Tests may require certain features to run
3. Performance characteristics change with features
4. Memory usage increases significantly with ML features
5. Compilation time increases with more features