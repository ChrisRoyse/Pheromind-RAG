# Project Structure (Verified)

## Root Directory Layout
```
embed/
├── src/                  # Rust source code
├── tests/               # Integration tests (19 test files)
├── benches/             # Performance benchmarks
├── docs/                # Documentation (includes phantom type analysis)
├── scripts/             # Utility scripts
├── .serena/             # Serena MCP configuration and cache
├── .embed/              # Embed search system data
├── .tantivy_index/      # Tantivy search index
├── .test_bm25_db/       # Test database
├── test_accuracy_db/    # Accuracy testing database
├── Cargo.toml           # Rust dependencies and features
├── Cargo.lock           # Dependency lock file
├── package.json         # Node.js dependencies
├── CLAUDE.md           # Claude Code configuration
├── README.md           # Project documentation
├── test-sqlite.js       # SQLite test script
├── test-sqlite3.js      # SQLite3 test script
└── *.pdb files          # Debug files (debug_bm25.pdb, etc.)
```

## Source Code Organization (`/src`)
```
src/
├── main.rs              # CLI entry point with commands
├── lib.rs               # Library exports
├── error.rs             # Error types (EmbedError, StorageError, etc.)
├── bin/                 # Binary executables (5 files)
│   ├── tantivy_migrator.rs
│   ├── verify_symbols.rs
│   ├── test_persistence.rs
│   ├── test_project_scoping.rs
│   └── test_unified_project_scope.rs
├── cache/               # Caching implementations
├── chunking/            # Code chunking logic
├── config/              # Configuration management
│   └── mod.rs          # Config struct with all settings
├── embedding/           # ML embedding functionality
│   ├── mod.rs
│   ├── nomic.rs        # Nomic embedding implementation
│   └── cache.rs        # Embedding cache
├── git/                 # Git integration for file monitoring
├── observability/       # Metrics and logging
│   └── metrics.rs      # MetricsCollector, SearchMetrics
├── search/              # Search implementations (13 files)
│   ├── mod.rs          # Module exports
│   ├── bm25.rs         # BM25 ranking algorithm
│   ├── tantivy_search.rs # Full-text search
│   ├── native_search.rs  # File-based search
│   ├── unified.rs      # UnifiedSearcher combining all
│   ├── fusion.rs       # SimpleFusion for score combination
│   ├── symbol_index.rs # Symbol extraction
│   ├── symbol_enhanced_searcher.rs
│   ├── inverted_index.rs
│   ├── preprocessing.rs
│   ├── text_processor.rs
│   ├── cache.rs        # Search result caching
│   └── search_adapter.rs
├── storage/             # Storage backends (6 files)
│   ├── mod.rs
│   ├── lancedb.rs      # Legacy LanceDB (has StorageError enum)
│   ├── lancedb_storage.rs # Current LanceDB (LanceStorageError)
│   ├── simple_vectordb.rs # SimpleVectorDB (StorageError)
│   ├── safe_vectordb.rs   # Thread-safe wrapper
│   └── lightweight_storage.rs # Lightweight alternative
└── utils/              # Utility functions
```

## CLI Commands (from main.rs)
- `index` - Index files into search system
- `search` - Execute searches
- `watch` - Monitor file changes
- `update` - Update indices
- `clear` - Clear databases
- `stats` - Show statistics
- `test` - Run tests
- `config` - Manage configuration
- `validate-config` - Validate configuration

## Test Structure (`/tests`)
```
tests/
├── search_validation/   # Search validation tests
├── bm25_integration_tests.rs
├── chunker_integration_tests.rs
├── compile_time_feature_tests.rs
├── comprehensive_search_test.py
├── config_search_backend_tests.rs
├── core_tests.rs
├── embedding_performance_benchmark.rs
├── fallback_prevention_test.rs
├── line_tracking_tests.rs
├── nomic_embedding_tests.rs
├── production_embedding_verification.rs
├── production_q4km_verification.rs
├── real_embedding_system_tests.rs
├── safety_audit.rs
├── search_accuracy_test.rs
├── symbol_indexing_tests.rs
├── test-better-sqlite.js

└── tree_sitter_feature_tests.rs
```

## Documentation (`/docs`)
Recent analysis documents:
- PHANTOM_VS_REAL_ANALYSIS.md
- PHASE1_REAL_ISSUES_ONLY.md
- TRUTHFUL_IMPLEMENTATION_PLAN.md
- TRUTHFUL_PHASE1_COMPILATION_FIXES.md

## Feature Flags (Cargo.toml)
- **Default**: Core functionality only
- **ml**: Machine learning embeddings (Candle, GGUF)
- **vectordb**: Vector database support (LanceDB, Arrow)
- **tantivy**: Full-text search engine
- **tree-sitter**: Symbol indexing for 12+ languages

## Build Artifacts
- Target directory: `target/` (gitignored)
- Debug symbols: `.pdb` files
- Test databases: `.test_*_db/` directories

## Configuration Files
- `Cargo.toml` - Rust project configuration
- `package.json` - Node.js dependencies
- `CLAUDE.md` - Claude Code specific settings
- `.serena/` - Serena MCP tool configuration