# Build Scripts Documentation

## Overview
This directory contains optimized build scripts for the embed-search project, designed to handle Windows compilation resource constraints.

## Available Scripts

### 1. `build_core.bat` - Minimal Dependencies
- **Features**: Core only (BM25, basic text processing)
- **Build Time**: 30-60 seconds
- **Resource Usage**: Low memory, minimal CPU
- **Use Case**: Basic text search without ML or advanced features

### 2. `build_search_basic.bat` - Text Search Only
- **Features**: core + tantivy
- **Build Time**: 45-90 seconds  
- **Resource Usage**: Medium memory, moderate CPU
- **Use Case**: Full-text search with fuzzy matching

### 3. `build_search_advanced.bat` - Text + Symbol Search
- **Features**: core + tree-sitter + tantivy
- **Build Time**: 60-120 seconds
- **Resource Usage**: Medium-High memory, moderate CPU
- **Use Case**: Code search with symbol indexing

### 4. `build_optimized.bat` - Resource-Conscious
- **Features**: tree-sitter + tantivy (proven working combination)
- **Strategy**: Single-threaded compilation with memory limits
- **Build Time**: 90-150 seconds
- **Use Case**: When standard builds timeout due to resource constraints

### 5. `build_release_minimal.bat` - Production Build
- **Features**: core + tantivy (production-safe)
- **Build Type**: Release (optimized)
- **Build Time**: 120-180 seconds
- **Use Case**: Production deployment with reliable feature set

## Resource Management

### Known Working Combinations (from verification testing):
- ✅ `core` - Always works (minimal dependencies)
- ✅ `tantivy` - Reliable text search
- ✅ `tree-sitter` - Symbol parsing works individually
- ✅ `tree-sitter,tantivy` - Verified working combination (14.1s clean)
- ✅ `search-advanced` - Feature alias for above combination

### Problematic Combinations (avoid):
- ❌ `--all-features` - Causes resource exhaustion and timeouts
- ❌ `ml,vectordb` - High memory usage, compilation timeouts
- ❌ `full-system` - All features together overwhelm Windows resources

### Build Optimization Strategies:
1. **Single-threaded compilation**: Set `CARGO_BUILD_JOBS=1`
2. **Incremental builds**: Use separate target directories
3. **Feature selection**: Only enable needed capabilities
4. **Clean builds**: Run `cargo clean` before major builds
5. **Memory monitoring**: Watch system resources during compilation

## Usage Examples

```batch
# Quick development build
scripts\build\build_core.bat

# Full-featured development
scripts\build\build_search_advanced.bat

# Production deployment
scripts\build\build_release_minimal.bat

# Resource-constrained environments
scripts\build\build_optimized.bat
```

## Troubleshooting

### Build Timeouts
- Use `build_optimized.bat` for single-threaded compilation
- Run `cargo clean` before building
- Close other applications to free memory

### Linker Errors (LNK1104)
- Indicates resource exhaustion or file locking
- Try building with fewer parallel jobs
- Restart terminal/IDE to release file handles

### Out of Memory
- Use minimal feature combinations
- Build in release mode for better memory usage
- Consider upgrading system RAM for full ML features

## Performance Benchmarks

Based on verification testing:
- Core build: ~30 seconds
- Search Basic: ~45 seconds  
- Search Advanced: ~60 seconds (14.1s incremental)
- Optimized build: ~90 seconds
- Release build: ~120 seconds

*Times vary based on system specifications and current resource usage*