# TDD Implementation Guide for Phase 4

## Red-Green-Refactor Strategy

### 1. RED: Write a Failing Test First

**Example: Task 2 - MCP Server Initialization Test**

```rust
// tests/mcp_server_test.rs
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_mcp_server_initialization_succeeds() {
        // This test will fail because MCPServer doesn't exist yet
        let project_path = PathBuf::from("/test/project");
        let server = MCPServer::new(project_path.clone());
        
        assert!(server.is_ok(), "Server should initialize successfully");
        
        let server = server.unwrap();
        assert_eq!(server.project_path, project_path);
    }

    #[test]
    fn test_mcp_server_validates_path() {
        // Test that non-existent paths are handled
        let invalid_path = PathBuf::from("/definitely/not/a/real/path");
        let server = MCPServer::new(invalid_path);
        
        assert!(server.is_err(), "Server should fail with invalid path");
    }
}
```

### 2. GREEN: Write Minimal Code to Pass

**Example: Task 3 - Minimal MCPServer Implementation**

```rust
// src/mcp_server.rs
use std::path::PathBuf;
use anyhow::{Result, Context};

pub struct MCPServer {
    pub project_path: PathBuf,
    // Add only what's needed to pass the test
}

impl MCPServer {
    pub fn new(project_path: PathBuf) -> Result<Self> {
        // Minimal validation to pass tests
        if !project_path.exists() {
            return Err(anyhow::anyhow!("Project path does not exist"));
        }
        
        Ok(Self {
            project_path,
        })
    }
}
```

### 3. REFACTOR: Improve Code Quality

**Example: Refactored MCPServer**

```rust
// src/mcp_server.rs
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::{Result, Context};

/// MCP Server for handling code search requests
pub struct MCPServer {
    project_path: PathBuf,
    // Prepare for future components
    searcher: Option<Arc<RwLock<UnifiedSearcher>>>,
    storage: Option<Arc<RwLock<VectorStorage>>>,
    git_watch: Option<Arc<RwLock<Phase3GitWatch>>>,
}

impl MCPServer {
    /// Creates a new MCP server instance
    /// 
    /// # Arguments
    /// * `project_path` - The root directory of the project to index
    /// 
    /// # Returns
    /// * `Result<Self>` - The initialized server or an error
    pub fn new(project_path: PathBuf) -> Result<Self> {
        // Validate and canonicalize path
        let canonical_path = project_path
            .canonicalize()
            .context("Failed to resolve project path")?;
        
        if !canonical_path.is_dir() {
            return Err(anyhow::anyhow!("Project path must be a directory"));
        }
        
        Ok(Self {
            project_path: canonical_path,
            searcher: None,
            storage: None,
            git_watch: None,
        })
    }
}
```

## TDD Patterns for Each Component

### Pattern 1: Tool Definition Tests

```rust
// RED: Test tool schema
#[test]
fn test_search_tool_schema() {
    let server = create_test_server();
    let tool = server.search_tool();
    
    assert_eq!(tool.name, "search_code");
    assert!(tool.description.contains("3-chunk context"));
    
    // Validate parameters schema
    let params = tool.parameters.as_object().unwrap();
    assert_eq!(params["type"], "object");
    
    let properties = params["properties"].as_object().unwrap();
    assert!(properties.contains_key("query"));
}

// GREEN: Implement tool definition
fn search_tool(&self) -> Tool {
    Tool {
        name: "search_code".to_string(),
        description: "Search code with 3-chunk context".to_string(),
        parameters: json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Search query"
                }
            },
            "required": ["query"]
        }),
    }
}
```

### Pattern 2: Handler Method Tests

```rust
// RED: Test async handler
#[tokio::test]
async fn test_handle_search_basic_query() {
    let server = create_test_server_with_mocks();
    let params = SearchParams {
        query: "test_function".to_string(),
    };
    
    let result = server.handle_search(params).await;
    
    assert!(result.is_ok());
    let response: SearchResponse = serde_json::from_value(
        result.unwrap().into_success().unwrap()
    ).unwrap();
    
    assert!(response.results.len() > 0);
    assert!(response.search_time_ms < 500);
}

// GREEN: Implement handler with mocks
async fn handle_search(&self, params: SearchParams) -> ToolResult {
    let start = Instant::now();
    
    // Use mock searcher in tests
    let results = vec![/* mock results */];
    
    let response = SearchResponse {
        results,
        total_found: 1,
        search_time_ms: start.elapsed().as_millis() as u64,
    };
    
    ToolResult::Success(serde_json::to_value(response)?)
}
```

### Pattern 3: Integration Tests

```rust
// RED: Test full MCP request flow
#[tokio::test]
async fn test_mcp_request_routing() {
    let server = create_full_test_server().await;
    
    let request = Request {
        id: "test-1".to_string(),
        method: "search_code".to_string(),
        params: json!({
            "query": "impl Debug"
        }),
    };
    
    let result = server.handle_request(request).await;
    
    assert!(matches!(result, ToolResult::Success(_)));
}

// GREEN: Implement request router
async fn handle_request(&self, request: Request) -> ToolResult {
    match request.method.as_str() {
        "search_code" => {
            let params: SearchParams = serde_json::from_value(request.params)?;
            self.handle_search(params).await
        }
        _ => ToolResult::Error(format!("Unknown method: {}", request.method))
    }
}
```

## Testing Best Practices

### 1. Test File Organization

```
tests/
├── unit/
│   ├── mcp_server_test.rs
│   ├── search_tool_test.rs
│   ├── clear_database_test.rs
│   ├── reindex_tool_test.rs
│   └── toggle_watch_test.rs
├── integration/
│   ├── mcp_protocol_test.rs
│   ├── tool_integration_test.rs
│   └── end_to_end_test.rs
└── fixtures/
    ├── test_project/
    └── mock_responses.json
```

### 2. Mock Objects

```rust
// Create test doubles for external dependencies
pub struct MockSearcher {
    responses: HashMap<String, Vec<SearchResult>>,
}

impl MockSearcher {
    pub fn with_responses(responses: HashMap<String, Vec<SearchResult>>) -> Self {
        Self { responses }
    }
}

#[async_trait]
impl Searcher for MockSearcher {
    async fn search(&self, query: &str, _path: &Path) -> Result<Vec<SearchResult>> {
        Ok(self.responses.get(query).cloned().unwrap_or_default())
    }
}
```

### 3. Test Data Builders

```rust
// Builder pattern for test data
pub struct TestDataBuilder {
    chunks: Vec<Chunk>,
    search_results: Vec<SearchResult>,
}

impl TestDataBuilder {
    pub fn new() -> Self {
        Self {
            chunks: vec![],
            search_results: vec![],
        }
    }
    
    pub fn with_chunk(mut self, content: &str, start: usize, end: usize) -> Self {
        self.chunks.push(Chunk {
            content: content.to_string(),
            start_line: start,
            end_line: end,
        });
        self
    }
    
    pub fn build_three_chunk_context(self) -> ThreeChunkContext {
        ThreeChunkContext {
            above: self.chunks.get(0).cloned(),
            target: self.chunks.get(1).cloned().unwrap(),
            below: self.chunks.get(2).cloned(),
        }
    }
}
```

## Performance Testing

```rust
#[tokio::test]
async fn test_search_performance() {
    let server = create_production_like_server().await;
    let iterations = 100;
    let mut durations = Vec::new();
    
    for _ in 0..iterations {
        let start = Instant::now();
        let _ = server.handle_search(SearchParams {
            query: "test query".to_string(),
        }).await;
        durations.push(start.elapsed());
    }
    
    let avg_duration = durations.iter().sum::<Duration>() / iterations;
    assert!(avg_duration < Duration::from_millis(500), 
            "Average search time {} ms exceeds 500ms limit", 
            avg_duration.as_millis());
}
```

## Error Testing

```rust
#[tokio::test]
async fn test_graceful_error_handling() {
    let server = create_server_with_failing_storage();
    
    let result = server.handle_clear_database(json!({
        "confirm": true
    })).await;
    
    assert!(matches!(result, ToolResult::Error(_)));
    
    if let ToolResult::Error(msg) = result {
        assert!(msg.contains("storage"), "Error should mention storage issue");
        assert!(!msg.contains("panic"), "Should not expose internal panics");
    }
}
```

## Continuous Testing

### Run Tests Continuously During Development

```bash
# Watch for changes and run tests
cargo watch -x test

# Run specific test file
cargo test --test mcp_server_test

# Run with verbose output
cargo test -- --nocapture

# Run only unit tests
cargo test --lib

# Run only integration tests  
cargo test --test '*'
```

### Test Coverage

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html --output-dir coverage

# Aim for >80% coverage on critical paths
```