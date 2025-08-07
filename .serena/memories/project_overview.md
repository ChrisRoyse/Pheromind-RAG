# Embed Search System - Project Overview

## Purpose
A high-performance semantic code search system written in Rust, designed for searching code repositories using multiple search strategies including BM25, Tantivy full-text search, and ML embeddings with vector databases.

## Tech Stack
- **Primary Language**: Rust (edition 2021)
- **Build System**: Cargo
- **Runtime**: Tokio async runtime
- **Node.js Integration**: 
  - better-sqlite3, sqlite3 for database operations
  - claude-flow for SPARC methodology

## Core Architecture
The system uses a modular architecture with feature flags to enable/disable components:
- **Core**: Always-enabled BM25 search, text processing, configuration
- **ML Feature**: Machine learning embeddings using Candle and GGUF models
- **VectorDB Feature**: LanceDB vector storage with Arrow integration
- **Tantivy Feature**: Full-text search with fuzzy matching
- **Tree-Sitter Feature**: Code symbol indexing for multiple languages

## Key Features (Verified)
- **Multi-Strategy Search**: Unified search combining BM25, Tantivy, and semantic embeddings
- **3-Chunk Context**: Returns code with surrounding context (above + target + below)
- **Multiple Storage Backends**: LanceDB, SimpleVectorDB, SafeVectorDB, LightweightStorage
- **Git Integration**: File change monitoring (src/git/)
- **Caching**: LRU caches for embeddings and search results
- **Symbol Indexing**: Tree-sitter based code symbol extraction
- **Observability**: Comprehensive metrics and tracing

## Search Backends
1. **BM25** (src/search/bm25.rs) - Term frequency-based ranking
2. **Tantivy** (src/search/tantivy_search.rs) - Full-text with fuzzy matching
3. **Native Search** (src/search/native_search.rs) - File-based search
4. **Unified Search** (src/search/unified.rs) - Combines all strategies
5. **Symbol Search** (src/search/symbol_enhanced_searcher.rs) - Code-aware search

## Performance Characteristics
- **Build Status**: Compiles successfully with warnings
- **Async Operations**: Full async/await with Tokio
- **Parallel Processing**: Rayon for CPU-bound tasks
- **Memory Management**: LRU caching, memory-mapped files (memmap2)
- **Error Handling**: Comprehensive error types with thiserror

## Development Status
**ACTIVE DEVELOPMENT** - Project is functional but incomplete:
- ✅ Core search functionality implemented
- ✅ Multiple storage backends working
- ✅ Error handling framework in place
- ⚠️ MCP integration NOT implemented (despite documentation)
- ⚠️ Some unused code warnings present
- ✅ Comprehensive test suite in /tests

## Key Technologies
- **Serialization**: serde, serde_json, bincode
- **Configuration**: config, toml, serde_yaml
- **CLI**: clap v4 with derive
- **Logging**: tracing, tracing-subscriber
- **Text Processing**: regex, unicode-normalization, rust-stemmers
- **Retry Logic**: backoff with exponential backoff
- **System Monitoring**: sysinfo for memory management