# MCP (Model Context Protocol) Integration

## Overview
This project implements an MCP server that exposes 4 tools for LLM integration, enabling semantic code search capabilities.

## MCP Tools Exposed

### 1. search_code
- **Purpose**: Search code with any query complexity
- **Returns**: Always returns 3-chunk context (above + target + below)
- **Parameters**: 
  - `query` (string) - The search query
- **Implementation**: Combines semantic and exact matching

### 2. clear_database
- **Purpose**: Clear/reset the entire vector database
- **Parameters**:
  - `confirm` (boolean) - Must be true for safety
- **Use case**: Fresh start or troubleshooting

### 3. reindex_all
- **Purpose**: Re-embed and store all code files
- **Parameters**:
  - `directory` (string, optional) - Target directory
  - `show_progress` (boolean, optional) - Display progress
- **Use case**: Initial setup or full refresh

### 4. toggle_watch
- **Purpose**: Control git-based file watching
- **Parameters**:
  - `enabled` (boolean) - true to enable, false to disable
- **Use case**: Automatic index updates on file changes

## Implementation Details

### Key Files for MCP
- **docs/05_MCP_SERVER_IMPLEMENTATION.md** - Full implementation guide
- **docs/04_GIT_FILE_WATCHING.md** - Watch functionality details
- Look for handler methods:
  - `handle_search_code`
  - `handle_clear_database`
  - `handle_reindex_all`
  - `handle_toggle_watch`

### MCP Server Registration Pattern
```rust
server.register_tool(self.search_code_tool());
server.register_tool(self.clear_database_tool());
server.register_tool(self.reindex_all_tool());
server.register_tool(self.toggle_watch_tool());
```

### Tool Handler Pattern
```rust
match request.method.as_str() {
    "search_code" => self.handle_search_code(request.params).await,
    "clear_database" => self.handle_clear_database(request.params).await,
    "reindex_all" => self.handle_reindex_all(request.params).await,
    "toggle_watch" => self.handle_toggle_watch(request.params).await,
    _ => // unknown method
}
```

## Usage Workflow
1. Initialize: Run `reindex_all` to build initial index
2. Enable watching: `toggle_watch(true)` for auto-updates
3. Search: Use `search_code` for queries
4. Reset if needed: `clear_database` with confirmation

## Integration Points to Check
- Tool registration logic
- Parameter validation
- Error handling and responses
- Progress reporting for long operations
- Git integration for file watching

## Performance Considerations
- 3-chunk context ensures good accuracy (55% improvement)
- Git-based watching minimizes overhead
- Incremental updates avoid full reindexing