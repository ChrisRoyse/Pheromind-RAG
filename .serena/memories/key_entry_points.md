# Key Entry Points and Navigation Guide

## Main Application Entry
- **src/main.rs** - CLI entry point with all commands
  - `index_command` - Index files into search system
  - `search_command` - Execute searches
  - `watch_command` - Monitor file changes
  - `update_command` - Update indices
  - `clear_command` - Clear databases
  - `stats_command` - Show statistics
  - `test_command` - Run tests
  - `config_command` - Manage configuration

## Core Library Interface
- **src/lib.rs** - Public API exports and feature flag management

## Search Implementations (src/search/)
Each search backend has its own module:
- **bm25.rs** - BM25 text ranking algorithm
- **tantivy_search.rs** - Full-text search with fuzzy matching
- **native_search.rs** - Basic file-based search
- **unified.rs** - Unified search interface combining all backends
- **fusion.rs** - Score fusion for hybrid search
- **symbol_index.rs** - Code symbol indexing
- **symbol_enhanced_searcher.rs** - Symbol-aware search

## Embedding System (src/embedding/)
- **mod.rs** - Module exports
- **nomic.rs** - Nomic embedding model implementation
- **cache.rs** - Embedding cache management

## Storage Backends (src/storage/)
- **lancedb.rs** - Vector database storage (requires 'vectordb' feature)
- **migrations.rs** - Database migration logic

## Configuration (src/config/)
- Main configuration management for the system

## Git Integration (src/git/)
- File change monitoring and incremental updates

## Quick Navigation Tips for AI:
1. To find main logic: Start at src/main.rs, follow Commands enum
2. To find search logic: Check src/search/unified.rs first
3. To find ML features: Look in src/embedding/ (requires 'ml' feature)
4. To find storage: Check src/storage/ modules
5. To find tests: Look in tests/ dir and #[cfg(test)] modules
6. To find binaries: Check src/bin/ directory