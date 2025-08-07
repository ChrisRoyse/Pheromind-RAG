# Project Structure

## Root Directory
```
embed/
├── src/                  # Main Rust source code
├── tests/               # Integration tests
├── benches/             # Performance benchmarks
├── docs/                # Documentation
├── scripts/             # Utility scripts
├── .serena/             # Serena MCP tool configuration
├── .embed/              # Embed search system data
├── .tantivy_index/      # Tantivy search index
├── Cargo.toml           # Rust project configuration
├── Cargo.lock           # Dependency lock file
├── package.json         # Node.js dependencies
├── CLAUDE.md           # Claude Code configuration
└── README.md           # Project documentation
```

## Source Code Organization (`/src`)
```
src/
├── main.rs              # Application entry point
├── lib.rs               # Library root
├── error.rs             # Error types and handling
├── bin/                 # Binary executables
│   ├── tantivy_migrator.rs
│   ├── verify_symbols.rs
│   └── test_*.rs        # Test binaries
├── cache/               # Caching implementations
├── chunking/            # Code chunking logic
├── config/              # Configuration management
├── embedding/           # ML embedding functionality
├── git/                 # Git integration
├── observability/       # Logging and metrics
├── search/              # Search implementations
│   ├── bm25.rs         # BM25 text search
│   ├── tantivy.rs      # Tantivy full-text search
│   └── hybrid.rs       # Hybrid search fusion
├── storage/             # Data storage backends
│   ├── lancedb.rs      # Vector database
│   └── migrations.rs   # Database migrations
└── utils/              # Utility functions
```

## Key Files

### Configuration
- `Cargo.toml` - Rust dependencies and features
- `package.json` - Node.js dependencies
- `.embed/config.toml` - Embed system configuration

### Entry Points
- `src/main.rs` - Main application
- `src/lib.rs` - Library interface
- Various binaries in `src/bin/`

### Testing
- `/tests/*.rs` - Integration tests
- `/benches/*.rs` - Performance benchmarks
- Unit tests within source files

## Feature Modules

### Core Features (always enabled)
- BM25 text search
- Basic text processing
- Configuration management

### Optional Features
- `tree-sitter` - Symbol indexing for code
- `ml` - Machine learning embeddings
- `vectordb` - LanceDB vector storage
- `tantivy` - Full-text search engine

## Data Directories
- `.embed/` - Application data and cache
- `.tantivy_index/` - Search indices
- `.test_*_db/` - Test databases
- `.serena/` - MCP tool state

## Important Patterns
- Async/await throughout with Tokio
- Feature flags for optional capabilities
- Modular architecture with clear separation
- Error propagation with Result types
- Comprehensive test coverage