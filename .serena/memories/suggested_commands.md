# Suggested Commands for Development

## Build Commands
```bash
# Build the project (default features)
cargo build

# Build with all features
cargo build --all-features

# Build optimized release version
cargo build --release

# Build specific feature combinations
cargo build --features "core,tantivy"
cargo build --features "search-advanced"
cargo build --features "full-system"
```

## Testing Commands
```bash
# Run all tests
cargo test

# Run tests with all features
cargo test --all-features

# Run specific test file
cargo test --test <test_name>

# Run tests with output
cargo test -- --nocapture

# Run benchmarks
cargo bench
```

## Linting and Formatting
```bash
# Format code
cargo fmt

# Check formatting without changes
cargo fmt -- --check

# Run clippy linter
cargo clippy

# Run clippy with all features
cargo clippy --all-features
```

## Running the Application
```bash
# Run main binary
cargo run

# Run with specific features
cargo run --features "full-system"

# Run specific binary
cargo run --bin tantivy_migrator --features tantivy
cargo run --bin verify_symbols --features tree-sitter
```

## Node.js Commands (for SPARC integration)
```bash
# SPARC methodology commands
npx claude-flow sparc modes
npx claude-flow sparc run <mode> "<task>"
npx claude-flow sparc tdd "<feature>"

# Build and test JavaScript components
npm run build
npm run test
npm run lint
npm run typecheck
```

## Git Commands (Windows)
```bash
# Check status
git status

# Stage changes
git add .

# Commit changes
git commit -m "message"

# View recent commits
git log --oneline -10
```

## Utility Commands
```bash
# Check dependencies
cargo tree

# Update dependencies
cargo update

# Clean build artifacts
cargo clean

# Generate documentation
cargo doc --open
```