# Key Entry Points (Verified)

## Main Application Entry
**src/main.rs** - CLI application with clap v4
- Struct `Cli` - Command-line interface definition
- Enum `Commands` - All available commands:
  - `index_command` - Index files into search system
  - `search_command` - Execute searches
  - `watch_command` - Monitor file changes (duplicated in code)
  - `update_command` - Update indices (duplicated in code)
  - `clear_command` - Clear databases
  - `stats_command` - Show statistics
  - `test_command` - Run tests
  - `config_command` - Manage configuration
  - `validate_config_command` - Validate configuration

## Library Interface
**src/lib.rs** - Public API exports
- Feature flag management
- Module re-exports

## Binary Executables (`src/bin/`)
1. **tantivy_migrator.rs** - Migrate to Tantivy search
2. **verify_symbols.rs** - Verify symbol extraction
3. **test_persistence.rs** - Test data persistence
4. **test_project_scoping.rs** - Test project scoping
5. **test_unified_project_scope.rs** - Test unified scoping

## Core Modules

### Error Handling (`src/error.rs`)
- `EmbedError` - Main error enum
- `StorageError` - Storage-specific errors
- `EmbeddingError` - Embedding errors
- `SearchError` - Search errors
- `LoggingError` - Logging errors
- `ErrorContext` trait - Add context to errors
- `SafeUnwrap` trait - Safe unwrapping
- `RetryConfig` - Retry configuration
- `retry_with_backoff` - Retry logic

### Configuration (`src/config/mod.rs`)
- `Config` struct with 30+ fields:
  - project_path, chunk_size, batch_size
  - vector_db_path, cache_dir
  - search_backend, model_name
  - BM25 parameters (k1, b, stop_words)
  - Fusion weights (exact, bm25, semantic, symbol)
  - Feature flags (stemming, ngrams)

### Search Implementations (`src/search/`)
1. **unified.rs** - `UnifiedSearcher` combining all strategies
2. **bm25.rs** - BM25 text ranking
3. **tantivy_search.rs** - Full-text with fuzzy matching
4. **native_search.rs** - Basic file search
5. **fusion.rs** - `SimpleFusion` for score combination
6. **symbol_index.rs** - Code symbol extraction
7. **symbol_enhanced_searcher.rs** - Symbol-aware search
8. **search_adapter.rs** - Search adapter pattern
9. **inverted_index.rs** - Inverted index implementation
10. **preprocessing.rs** - Text preprocessing
11. **text_processor.rs** - Text processing utilities
12. **cache.rs** - Search result caching

### Storage Backends (`src/storage/`)
1. **lancedb_storage.rs** - Main LanceDB implementation (`LanceStorageError`)
2. **simple_vectordb.rs** - Simple vector database (`StorageError`)
3. **safe_vectordb.rs** - Thread-safe wrapper
4. **lightweight_storage.rs** - Lightweight alternative
5. **lancedb.rs** - Legacy LanceDB (being phased out)

### Embedding System (`src/embedding/`)
1. **nomic.rs** - Nomic embedding model
2. **cache.rs** - Embedding cache management
3. **mod.rs** - Module exports

### Observability (`src/observability/`)
- **metrics.rs** - `MetricsCollector`, `SearchMetrics`, `EmbeddingMetrics`

## Quick Navigation Guide

### To find main logic:
```rust
// Start here
src/main.rs -> Commands enum -> specific command functions
```

### To find search implementation:
```rust
// Unified search combines all
src/search/unified.rs -> UnifiedSearcher

// Individual search strategies
src/search/bm25.rs -> BM25 algorithm
src/search/tantivy_search.rs -> Full-text search
```

### To find storage:
```rust
// Main storage
src/storage/lancedb_storage.rs -> LanceDB vector store

// Alternatives
src/storage/simple_vectordb.rs -> Simple implementation
```

### To find ML/embeddings:
```rust
// Requires 'ml' feature flag
src/embedding/nomic.rs -> Embedding generation
```

### To find configuration:
```rust
src/config/mod.rs -> Config struct with all settings
```

### To find tests:
```rust
// Integration tests
tests/*.rs

// Unit tests
Look for #[cfg(test)] modules in source files
```

## Important Notes
- **No MCP implementation found** - Despite documentation, no MCP server code exists
- **Duplicate functions** - watch_command and update_command appear twice
- **Unused code warnings** - Several fields and methods are never used
- **Feature-gated code** - Many modules require specific feature flags