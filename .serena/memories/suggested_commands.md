# Suggested Commands for Embed Search System

## Build Commands
```bash
# With embeddings (required for semantic search)
cargo build --features ml,vectordb,tantivy

# All features enabled (recommended)
cargo build --features full-system

# Basic build without embeddings (text search only)
cargo build --features tantivy

# Release build for production
cargo build --release --features full-system
```

## Testing Commands
```bash
# Run all tests
cargo test --features full-system

# Run specific test files
cargo test --features full-system chunker_integration_tests
cargo test --features full-system line_tracking_tests
cargo test --features full-system search_accuracy_test

# Run benchmarks
cargo bench line_tracking_bench

# Performance tests
cargo test --features test-performance --release
```

## Development Commands
```bash
# Format code
cargo fmt

# Lint code
cargo clippy --features full-system

# Check compilation without building
cargo check --features full-system

# Generate documentation
cargo doc --features full-system --open
```

## Runtime Commands
```bash
# Index a directory
cargo run --features full-system -- index /path/to/code

# Search indexed code
cargo run --features full-system -- search "your query"

# Watch for file changes
cargo run --features full-system -- watch

# Clear index
cargo run --features full-system -- clear

# Show statistics
cargo run --features full-system -- stats

# Run comprehensive tests
cargo run --features full-system -- test

# Show configuration
cargo run --features full-system -- config
```

## Binary Utilities
```bash
# Migrate Tantivy indexes
cargo run --bin tantivy_migrator --features tantivy

# Verify symbols extraction
cargo run --bin verify_symbols --features tree-sitter

# Test persistence
cargo run --bin test_persistence --features tantivy
```

## System Commands (Windows)
- `dir` - List directory contents
- `cd` - Change directory  
- `type` - Display file contents
- `findstr` - Search in files
- `git status` - Check git status
- `git diff` - Show changes