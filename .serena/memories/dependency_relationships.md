# Dependency Relationships Map

## Core Module Dependencies

### Layer Architecture
```
┌─────────────────────────────────────┐
│         CLI (main.rs)               │
├─────────────────────────────────────┤
│     Unified Search (unified.rs)     │
├─────────────────────────────────────┤
│   Search Backends │ Storage │ ML    │
├─────────────────────────────────────┤
│        Core Utilities               │
└─────────────────────────────────────┘
```

## Module Dependency Graph

### src/main.rs (Entry Point)
**Depends on:**
- `src/search/unified.rs` - Main search interface
- `src/config/` - Configuration loading
- `src/git/` - File watching
- `clap` - CLI parsing

**Used by:** Nothing (top-level)

### src/search/unified.rs (Orchestrator)
**Depends on:**
- `src/search/bm25.rs` - Text search
- `src/search/tantivy_search.rs` - Full-text search
- `src/search/fusion.rs` - Score combination
- `src/embedding/` - ML embeddings (if feature enabled)
- `src/storage/` - Vector storage (if feature enabled)
- `src/chunking/` - File chunking

**Used by:** 
- `src/main.rs` - CLI commands
- Tests in `/tests`

### src/search/bm25.rs (Core Search)
**Depends on:**
- `src/search/text_processor.rs` - Text preprocessing
- `src/search/inverted_index.rs` - Index structure
- No external features required

**Used by:**
- `src/search/unified.rs`
- `src/search/symbol_enhanced_searcher.rs`

### src/embedding/ (ML Module)
**Depends on:**
- `candle-*` crates (if ML feature)
- `src/cache/` - Embedding caching
- `tokenizers` - Text tokenization
- Model files (~500MB)

**Used by:**
- `src/search/unified.rs`
- `src/storage/lancedb.rs`

### src/storage/lancedb.rs (Vector DB)
**Depends on:**
- `lancedb` crate
- `arrow-*` crates
- `src/embedding/` - For vector dimensions
- Requires both 'ml' and 'vectordb' features

**Used by:**
- `src/search/unified.rs`

### src/chunking/ (Text Processing)
**Depends on:**
- `regex` - Pattern matching
- `tree-sitter-*` (if tree-sitter feature)
- No heavy dependencies

**Used by:**
- `src/search/unified.rs`
- `src/search/bm25.rs`
- `src/embedding/`

### src/config/ (Configuration)
**Depends on:**
- `config` crate
- `serde` - Serialization
- `toml` - Config format

**Used by:** Almost everything

### src/git/ (File Monitoring)
**Depends on:**
- System git command
- `tokio` - Async runtime
- File system access

**Used by:**
- `src/main.rs` - Watch command

## Feature Flag Dependencies

### No Features (Core Only)
```
Available:
- BM25 search
- Basic text processing
- Configuration
- File operations

Not Available:
- ML embeddings
- Vector storage
- Symbol indexing
- Tantivy search
```

### With 'ml' Feature
```
Adds:
- src/embedding/nomic.rs
- Candle framework
- Model downloading
- Tokenizers

Enables:
- Semantic search
- Embedding generation
```

### With 'vectordb' Feature
```
Adds:
- src/storage/lancedb.rs
- Arrow integration
- Vector persistence

Requires:
- 'ml' feature for embeddings
```

### With 'tree-sitter' Feature
```
Adds:
- src/search/symbol_index.rs
- Language parsers
- AST analysis

Enables:
- Symbol extraction
- Code structure search
```

### With 'tantivy' Feature  
```
Adds:
- src/search/tantivy_search.rs
- Full-text indexing
- Fuzzy matching

Enables:
- Advanced text search
- Query syntax
```

## Critical Dependency Paths

### Search Request Flow
```
1. main.rs (CLI)
   ↓
2. unified.rs (orchestrate)
   ↓
3. Parallel:
   - bm25.rs (text scores)
   - tantivy_search.rs (full-text)
   - embedding + storage (semantic)
   ↓
4. fusion.rs (combine scores)
   ↓
5. Return results
```

### Indexing Flow
```
1. main.rs (index command)
   ↓
2. File discovery (walkdir)
   ↓
3. Parallel processing:
   - chunking/ (split files)
   - symbol_index.rs (extract symbols)
   - embedding/ (generate vectors)
   ↓
4. Storage:
   - tantivy index
   - lancedb vectors
   - BM25 index
```

## Circular Dependency Prevention

### No Circular Deps Between:
- `search/*` modules are independent
- `storage/` doesn't depend on `search/`
- `embedding/` doesn't depend on `search/`
- `config/` has no dependencies on app logic

### Shared Dependencies Only:
- `error.rs` - Used everywhere
- `utils/` - Utility functions
- `observability/` - Logging/metrics

## Testing Dependencies

### Unit Tests
- Located in same file
- Minimal dependencies
- Use `#[cfg(test)]`

### Integration Tests  
**Depend on:**
- Full feature set
- Temp directories
- Mock data

**Located in:** `/tests`

## Build Order (Important for Compilation)

1. **First tier** (no internal deps):
   - error.rs
   - config/
   - utils/
   
2. **Second tier** (basic deps):
   - chunking/
   - text_processor.rs
   - cache/
   
3. **Third tier** (feature-gated):
   - embedding/ (if ml)
   - tree-sitter symbols (if tree-sitter)
   - tantivy_search.rs (if tantivy)
   
4. **Fourth tier** (needs features):
   - storage/lancedb.rs (needs ml)
   - symbol_enhanced_searcher.rs (needs tree-sitter)
   
5. **Fifth tier** (orchestration):
   - unified.rs
   - fusion.rs
   
6. **Top tier**:
   - main.rs

## Finding Dependencies

### Check Module Imports
```
# Find what a module depends on
search_for_pattern "^use " relative_path="src/module.rs"

# Find what depends on a module  
search_for_pattern "use.*module_name"
```

### Check Cargo Dependencies
```
# External crates used
search_for_pattern "extern crate"
search_for_pattern 'use [a-z_]+::'  # External crate usage
```

### Check Feature Dependencies
```
# Find feature-gated imports
search_for_pattern '#\[cfg\(feature.*\)\].*use'
```