# Embed Search Project State - January 8, 2025

## Project Overview
**Name**: embed-search  
**Version**: 0.1.0  
**Language**: Rust (2021 edition)  
**Description**: Rust-based semantic search system with ML embeddings, vector storage, and symbol indexing

## Recent Commits (Last 5)
1. **feat**: Add comprehensive watcher system with MCP server integration (ebf979f)
2. **chore**: Add deployment scripts and update gitignore (299176d)  
3. **docs**: Add remaining test reports and demo files (0c05233)
4. **feat**: Comprehensive testing framework and validation suite (d3de302)
5. **feat**: Enhanced embedding system with Q6K quantization and IVF-PQ indexing (6d13636)

## Core Architecture

### Search System (4-Way Parallel)
- **Native/Ripgrep**: Basic text search with line-level results
- **BM25**: Statistical ranking with IDF improvements
- **Tantivy**: Full-text search with fuzzy matching
- **Vector/Embedding**: Semantic search with Nomic embeddings
- **Unified Fusion**: Score normalization and result merging

### Embedding System
- **Model**: Nomic Embed Text v1.5 (Q4_K_M quantization, 84MB)
- **Dimensions**: 768-dimensional vectors
- **Quantization**: Q4_0, Q4_1, Q4K, Q5, Q6K, Q8 support
- **Caching**: LRU with SHA-256 hashing and disk persistence

### Storage Layer
- **LanceDB**: Vector database with IVF-PQ indexing
- **Features**: Atomic operations, corruption detection, checksums
- **Indexing**: 256 partitions, 16 sub-vectors for PQ

### Watcher System (NEW)
- **File Watching**: Real-time monitoring with debouncing
- **Git Integration**: Dual approach (file system + git status)
- **Edge Cases**: Comprehensive handling (100MB limit, symlinks, locks)
- **MCP Server**: TypeScript interface for IDE integration

## Current Status

### ✅ Working Components
- Core compilation and build system
- BM25 text processing and ranking
- File system operations and watching
- Configuration initialization
- Test framework infrastructure
- Edge case handling

### ⚠️ Issues Identified
1. **ML Embeddings**: Q4_K_M model file corrupted (NaN values)
2. **Integration Tests**: Feature flag misconfiguration (0 tests run)
3. **Windows ML**: Compilation failures with candle/datafusion
4. **MCP Bridge**: Missing native binding between Rust and TypeScript
5. **Code Complexity**: UnifiedSearcher exceeds 500-line limit (1063 lines)

### 📊 Metrics
- **Files Changed**: 263 files in recent commits
- **Lines Added**: ~75,532 lines
- **Lines Removed**: ~10,830 lines
- **Test Files**: 115 total, 80 Rust test files
- **Languages Supported**: 10+ (Rust, Python, JS, TS, Go, Java, C++, etc.)

## Feature Flags
- `ml`: Machine learning embeddings
- `vectordb`: LanceDB vector storage
- `tree-sitter`: AST parsing for symbols
- `tantivy`: Full-text search
- `full-system`: All features enabled

## Critical Paths
- `/src` - Core source code
- `/tests` - Comprehensive test suite
- `/mcp-server` - MCP protocol server
- `/mcp-bridge` - Rust-Node.js bridge
- `/docs` - Documentation and reports

## Integration Score: 25/100
Major functionality gaps exist between components, particularly in ML features and test execution.