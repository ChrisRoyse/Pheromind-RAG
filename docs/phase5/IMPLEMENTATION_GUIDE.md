# MCP Server Implementation Guide

## Overview
This guide explains how to implement the MCP server using the micro tasks defined in MCP_SERVER_MICRO_TASKS.md. Each task follows TDD principles with red-green-refactor cycles.

## Implementation Order and Dependencies

### Week 3-4 Timeline

#### Day 1-2: Foundation (Tasks 031.1-031.16)
Start with the MCP server foundation. These tasks establish the core structure and component integration.

**Key Dependencies:**
- Storage system from Phase 3
- Embedder from Phase 1  
- GitWatcher from Phase 3
- Searcher components from Phase 2

#### Day 3-4: Core Tools (Tasks 032.1-035.8)
Implement the four essential MCP tools:
1. **search_code**: The primary search interface
2. **clear_database**: Database management
3. **reindex_all**: Bulk indexing capability
4. **toggle_watch**: File monitoring control

#### Day 5: Integration (Tasks 036.1-037.8)
Connect the tools to the MCP protocol:
- Request routing
- Transport layer (stdio)
- Message handling

#### Day 6: Polish (Tasks 038.1-040.8)
- Error handling
- Documentation
- Integration testing
- Performance validation

## TDD Approach for Each Task

### Red Phase
1. Write a failing test that describes the desired behavior
2. Run the test to confirm it fails
3. The test should be minimal but meaningful

### Green Phase
1. Write the minimum code to make the test pass
2. Don't worry about elegance yet
3. Focus only on passing the test

### Refactor Phase
1. Improve the code without changing behavior
2. Remove duplication
3. Improve naming and structure
4. Ensure tests still pass

## Example Implementation Pattern

```rust
// Task 031.2: Define MCPServer Struct

// RED: Write failing test
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mcp_server_creation() {
        let server = MCPServer::new(PathBuf::from("/test"));
        assert!(server.is_ok());
    }
}

// GREEN: Minimal implementation
pub struct MCPServer {
    project_path: PathBuf,
}

impl MCPServer {
    pub fn new(project_path: PathBuf) -> Result<Self> {
        Ok(Self { project_path })
    }
}

// REFACTOR: Add proper fields and error handling
pub struct MCPServer {
    searcher: Arc<RwLock<UnifiedSearcher>>,
    storage: Arc<RwLock<VectorStorage>>,
    git_watch: Arc<RwLock<Phase3GitWatch>>,
    project_path: PathBuf,
}
```

## Critical Integration Points

### 1. Storage Layer
- Must connect to existing LanceDB storage
- Preserve existing vector data
- Support concurrent access

### 2. Search Components
- Integrate UnifiedSearcher from Phase 2
- Ensure 3-chunk context expansion works
- Maintain <500ms latency

### 3. Git Watching
- Connect to Phase 3 git monitoring
- Support enable/disable functionality
- Handle file change events

### 4. MCP Protocol
- Follow MCP specification exactly
- Support JSON-RPC over stdio
- Handle all required tool methods

## Testing Strategy

### Unit Tests (Each Micro Task)
- Test individual components in isolation
- Mock dependencies where needed
- Focus on single responsibility

### Integration Tests (Task Groups)
- Test tool workflows end-to-end
- Verify component interactions
- Check performance metrics

### System Tests (Task 040)
- Full MCP server operation
- LLM integration verification
- Performance benchmarks

## Common Patterns

### Arc<RwLock> Usage
```rust
// Shared mutable state pattern
let storage = Arc::new(RwLock::new(VectorStorage::new()?));
let storage_clone = Arc::clone(&storage);

// Reading
let reader = storage.read().await;
// Writing
let mut writer = storage.write().await;
```

### Error Handling
```rust
// Consistent error propagation
pub async fn handle_search(&self, params: SearchParams) -> ToolResult {
    let searcher = self.searcher.read().await;
    let results = searcher.search(&params.query)
        .await
        .map_err(|e| ToolError::SearchFailed(e))?;
    
    ToolResult::Success(serde_json::to_value(results)?)
}
```

### Tool Definition Pattern
```rust
fn search_tool(&self) -> Tool {
    Tool {
        name: "search_code".to_string(),
        description: "Search with context".to_string(),
        parameters: self.search_params_schema(),
    }
}
```

## Performance Considerations

### Memory Usage
- Single embedding model (~500MB)
- Vector storage (~1GB for large codebases)
- Keep total under 2GB target

### Latency Targets
- Search: <500ms average
- Clear: <100ms
- Reindex: <10ms per file
- Toggle: <50ms

### Concurrency
- All tools must be thread-safe
- Use async/await throughout
- Minimize lock contention

## Debugging Tips

1. **Enable verbose logging** for MCP messages
2. **Test with simple queries** first
3. **Monitor memory usage** during reindexing
4. **Check git status** when file watching issues occur
5. **Validate JSON-RPC** format for all responses

## Success Criteria

When implementation is complete, you should have:

1. **Functional MCP server** responding to all 4 tools
2. **85% search accuracy** on test queries
3. **<500ms search latency** consistently
4. **Reliable file watching** with git integration
5. **Complete documentation** for LLM users
6. **Passing integration tests** with real LLMs

## Next Steps

After completing all micro tasks:

1. Run full integration test suite
2. Benchmark performance metrics
3. Test with Claude/GPT integration
4. Document any limitations
5. Create usage examples
6. Deploy to production environment