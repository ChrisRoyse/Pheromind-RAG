# Troubleshooting Guide for Embed Search System

## Common Issues and Solutions

### Build Issues

#### Feature Compilation Errors
**Problem**: Build fails with missing dependencies
**Solution**: Check feature flags are correct
```bash
# Clean build
cargo clean

# Build with specific features
cargo build --features "full-system"

# Check feature dependencies
cargo tree --features "ml,vectordb"
```

#### Long Compilation Times
**Problem**: ML features take forever to compile
**Solution**: Build without ML first, add incrementally
```bash
cargo build --features "core,tantivy"  # Fast
cargo build --features "full-system"   # Slower
```

### Search Issues

#### No Results Found
**Check**:
1. Is database indexed? Run `reindex_all`
2. Are features enabled? Need 'ml' for semantic search
3. Check logs in `.embed/logs/`
4. Verify file extensions are supported

#### Poor Search Quality
**Check**:
1. Is 3-chunk context working? Check chunk boundaries
2. Are embeddings cached? Check `.embed/cache/`
3. Is scoring fusion configured? Check config.toml
4. Test with exact matches first

### Memory Issues

#### High Memory Usage
**Check**:
```rust
# Find memory leaks
search_for_pattern "Box::leak|mem::forget"

# Check cache sizes
search_for_pattern "LruCache::new"

# Monitor with
cargo run -- stats
```

#### Out of Memory with ML
**Solution**: Reduce batch sizes in config
```toml
[embedding]
batch_size = 16  # Reduce from 32
```

### Performance Issues

#### Slow Indexing
**Check**:
1. Parallel processing enabled? Check rayon usage
2. Disk I/O bottleneck? Use SSD
3. Too many small files? Batch operations

#### Slow Searches
**Profile**:
```bash
cargo build --release
cargo run --release -- test
```

### Database Issues

#### Corrupt Index
**Fix**:
```bash
# Clear and rebuild
cargo run -- clear --confirm
cargo run -- index --directory .
```

#### Migration Failures
**Check**: src/storage/migrations.rs
**Fix**: Clear `.tantivy_index/` and rebuild

### Git Integration Issues

#### File Watch Not Working
**Check**:
1. Is git initialized? `git status`
2. Watch enabled? `toggle_watch(true)`
3. Check git ignore patterns

### Testing Issues

#### Tests Failing
**Common Causes**:
1. Missing features: `cargo test --all-features`
2. Temp directory issues: Check /tmp permissions
3. Database locks: Kill other processes

#### Integration Test Setup
```bash
# Run specific test
cargo test --test search_accuracy_test --features "full-system"

# With output
cargo test -- --nocapture
```

## Debug Techniques

### Enable Logging
```bash
RUST_LOG=debug cargo run
RUST_LOG=embed_search=trace cargo run
```

### Check Configurations
```bash
cargo run -- config --validate
```

### Inspect Indices
```bash
cargo run --bin verify_symbols --features tree-sitter
```

### Profile Performance
```bash
cargo bench
cargo run --release -- test --benchmark
```

## Key Files to Check When Debugging

1. **Logs**: `.embed/logs/`
2. **Config**: `.embed/config.toml`
3. **Cache**: `.embed/cache/`
4. **Index**: `.tantivy_index/`
5. **Error types**: `src/error.rs`
6. **Main logic**: `src/search/unified.rs`

## Getting Help

### Diagnostic Commands
```bash
# System info
cargo run -- stats

# Config validation
cargo run -- config --validate

# Test search
cargo run -- search "test query"

# Check versions
cargo --version
rustc --version
```

### Where to Look for Errors
```
find_symbol "Error" substring_matching=true
search_for_pattern "anyhow::|thiserror::"
search_for_pattern "\.context\(.*error"
```