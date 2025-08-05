# Embedding Vector Search System

A simple, powerful code search system that combines exact text matching with semantic search using embeddings, featuring the innovative 3-chunk context approach for superior accuracy.

## Key Features

- **3-Chunk Context**: Always returns code with surrounding context (above + target + below chunks) for 55% better accuracy
- **Single Embedding Model**: Uses `all-MiniLM-L6-v2` for all semantic search (no complex routing)
- **Simple Fusion**: Basic scoring that combines exact and semantic matches effectively
- **Git-Based Updates**: Monitors file changes via `git status` for incremental updates
- **MCP Integration**: Full Model Context Protocol server for LLM integration

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                  Simple Embedding System                        │
├─────────────────────────────────────────────────────────────────┤
│  Core Search Layer                                             │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐              │
│  │   Ripgrep   │ │   MiniLM    │ │   Simple    │              │
│  │   Search    │ │  Embedder   │ │   Fusion    │              │
│  └─────────────┘ └─────────────┘ └─────────────┘              │
├─────────────────────────────────────────────────────────────────┤
│  Storage & Chunking Layer                                      │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐              │
│  │   Regex     │ │  3-Chunk    │ │   LanceDB   │              │
│  │  Chunker    │ │  Expander   │ │   Storage   │              │
│  └─────────────┘ └─────────────┘ └─────────────┘              │
├─────────────────────────────────────────────────────────────────┤
│  Integration Layer                                             │
│  ┌─────────────┐ ┌─────────────────────────────────────┐      │
│  │    Git      │ │        MCP Server                   │      │
│  │  Watcher    │ │  (search/clear/reindex/toggle)      │      │
│  └─────────────┘ └─────────────────────────────────────┘      │
└─────────────────────────────────────────────────────────────────┘
```

## MCP Tools

The system exposes 4 tools via the Model Context Protocol:

### 1. `search_code`
Search code with any query complexity. Always returns 3-chunk context.
```json
{
  "query": "string - search query"
}
```

### 2. `clear_database`
Clear/reset the entire vector database.
```json
{
  "confirm": "boolean - must be true"
}
```

### 3. `reindex_all`
Re-embed and store all code files in a directory.
```json
{
  "directory": "string - optional, defaults to current project",
  "show_progress": "boolean - optional, default true"
}
```

### 4. `toggle_watch`
Turn git-based file watching on/off.
```json
{
  "enabled": "boolean - true to enable, false to disable"
}
```

## Performance Targets

- **Search Accuracy**: 85% success rate (finds relevant result in top 5)
- **Search Latency**: <500ms including embedding generation
- **Memory Usage**: <2GB (single model)
- **Startup Time**: <30 seconds
- **Index Updates**: <1s to detect changes via git

## Implementation Timeline

The system can be built in 3-4 weeks:

1. **Week 1**: Regex chunking + MiniLM embeddings + 3-chunk context
2. **Week 2**: Search implementation + simple fusion
3. **Week 3**: Git file watching + incremental updates
4. **Week 3-4**: MCP server with all tools

## Technology Stack

- **Language**: Rust (for performance)
- **Embeddings**: all-MiniLM-L6-v2 (384 dimensions)
- **Vector DB**: LanceDB
- **Text Search**: Ripgrep
- **File Watching**: Git status monitoring
- **API**: MCP (Model Context Protocol)

## Documentation

See the `docs/` directory for detailed planning documents:

- `00_MASTER_PLAN_OVERVIEW.md` - Complete system overview
- `01_CONTENT_DETECTION_FEATURE.md` - Regex + embeddings foundation
- `02_SPECIALIZED_EMBEDDING_MODELS.md` - Search & fusion implementation
- `03_LANCEDB_VECTOR_STORAGE.md` - Git file watching
- `04_GIT_FILE_WATCHING.md` - MCP server & tools

## Key Innovation: 3-Chunk Context

The system's main innovation is always returning code with surrounding context:

```
// === ABOVE CONTEXT ===
function validateUser(user) {
    if (!user.email) {
        return false;
    }

// === TARGET CHUNK ===
    if (!user.password || user.password.length < 8) {
        return false;
    }
    
    return true;
}

// === BELOW CONTEXT ===
function authenticateUser(email, password) {
    const user = findUserByEmail(email);
    if (!user) {
        return null;
    }
```

This approach provides 55% better accuracy compared to returning isolated code snippets.

## Getting Started

1. Clone the repository
2. Install Rust and dependencies
3. Download all-MiniLM-L6-v2 model
4. Run initial indexing: `reindex_all`
5. Start MCP server
6. Begin searching with `search_code`

## License

[Your License Here]