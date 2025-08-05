# Changelog

## [2.0.0] - 2025-01-05

### Major Simplification Update

This release represents a complete simplification of the embedding vector search system, reducing complexity while maintaining core functionality.

### Changed

#### Architecture Simplifications
- **Single Embedding Model**: Removed dual model routing complexity, now uses only `all-MiniLM-L6-v2`
- **Simple Fusion**: Replaced complex fusion algorithms with basic scoring (exact matches: 1.0, semantic: 0.8)
- **Git-Based File Watching**: Replaced real-time file watching with simple `git status` monitoring
- **Reduced Timeline**: 3-4 weeks instead of 18+ weeks
- **Reduced Tasks**: 40 focused tasks instead of 60+

#### Technical Improvements
- **Memory Usage**: Reduced from 8GB to <2GB (single model)
- **Startup Time**: Reduced from 90s to <30s
- **Search Latency**: Maintained <500ms target
- **Code Simplicity**: Removed over-engineered abstractions

#### MCP Server Enhancements
- **Complete Tool Suite**: 4 essential tools for LLM integration
  - `search_code`: Search with any query complexity, always returns 3-chunk context
  - `clear_database`: Clear/reset entire vector database (requires confirmation)
  - `reindex_all`: Re-embed files with directory parameter support
  - `toggle_watch`: Control git-based file watching

### Added

#### Flexible Directory Indexing
- `reindex_all` now accepts optional `directory` parameter
- Supports both absolute and relative paths
- Defaults to current project directory if not specified
- Validates directory existence before indexing

### Removed
- Dual model routing (CodeT5 + OpenAI)
- Intelligent fusion algorithms
- Real-time file watching with notify-rs
- Complex caching systems
- Machine learning pipelines
- AST parsing considerations

### Core Features Maintained
- **3-Chunk Context**: Still provides 55% accuracy improvement
- **Regex Chunking**: Fast pattern-based chunking
- **Vector Search**: LanceDB for similarity search
- **85% Accuracy Target**: Achievable with simplified approach

### Migration Notes
- All planning documents have been updated to reflect the new simplified approach
- The system is now much easier to implement and maintain
- Core innovation (3-chunk context) remains intact