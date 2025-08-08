# Technology Stack and Features

## Feature Flags Architecture
The system uses conditional compilation with Cargo features:

- `core` - Basic BM25 and text processing (default)
- `ml` - **Required for embeddings** - Enables Nomic embedder with GGUF support
- `vectordb` - LanceDB vector storage
- `tantivy` - Full-text search with fuzzy matching  
- `tree-sitter` - Symbol indexing for code analysis

### Feature Combinations
- `search-basic` = core + tantivy (text search only)
- `search-advanced` = core + tree-sitter + tantivy (text + symbol search)
- `full-system` = all features enabled
- `test-integration` = full-system (for tests)

## Key Dependencies
- **Core**: tokio, serde, anyhow, thiserror, clap
- **ML**: candle-core, candle-nn, candle-transformers, tokenizers, hf-hub
- **Vector DB**: lancedb, arrow, arrow-array, arrow-schema
- **Text Search**: tantivy, tantivy-jieba
- **Symbol Parsing**: tree-sitter + language parsers (rust, python, js, ts, go, java, c, cpp, html, css, json, bash)
- **Processing**: rayon, walkdir, regex, unicode-normalization
- **Monitoring**: tracing, tracing-subscriber, sysinfo

## 4-Method Search Integration
1. **Exact Search** (Tantivy) - Perfect string matches with fuzzy support
2. **Semantic Search** (ML embeddings) - Vector similarity using all-MiniLM-L6-v2
3. **Symbol Search** (Tree-sitter) - Code structure analysis (functions, classes, etc.)
4. **Statistical Search** (BM25) - TF-IDF statistical relevance scoring

## Concurrency Model
- All 4 search methods run in parallel using `tokio::join!`
- Thread-safe data structures with `Arc<RwLock<T>>`
- Async/await throughout the system
- No blocking operations in async contexts