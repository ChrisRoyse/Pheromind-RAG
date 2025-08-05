# Starter Code Templates for Phase 4

## Task 1: Project Structure Setup

### Cargo.toml
```toml
[package]
name = "mcp_server"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1.35", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
async-trait = "0.1"
tracing = "0.2"
tracing-subscriber = "0.3"

# From previous phases
unified_searcher = { path = "../unified_searcher" }
vector_storage = { path = "../vector_storage" }
git_watcher = { path = "../git_watcher" }

[dev-dependencies]
tempfile = "3.8"
mockall = "0.11"
tokio-test = "0.4"
```

### Directory Structure
```
mcp_server/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── server.rs
│   ├── tools/
│   │   ├── mod.rs
│   │   ├── search.rs
│   │   ├── clear_database.rs
│   │   ├── reindex.rs
│   │   └── toggle_watch.rs
│   ├── transport/
│   │   ├── mod.rs
│   │   └── stdio.rs
│   └── types.rs
├── tests/
│   ├── unit/
│   │   └── mcp_server_test.rs
│   └── integration/
│       └── end_to_end_test.rs
└── examples/
    └── mcp_client.rs
```

## Task 2-3: MCP Server Initialization

### tests/unit/mcp_server_test.rs
```rust
use mcp_server::MCPServer;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_mcp_server_initialization_succeeds() {
    // Create a temporary directory for testing
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path().to_path_buf();
    
    // This will fail initially (RED phase)
    let server = MCPServer::new(project_path.clone());
    
    assert!(server.is_ok(), "Server should initialize successfully");
    
    let server = server.unwrap();
    // Verify the path was stored correctly
    assert_eq!(server.project_path(), &project_path);
}

#[test]
fn test_mcp_server_validates_path() {
    let invalid_path = PathBuf::from("/definitely/not/a/real/path/xyz123");
    let server = MCPServer::new(invalid_path);
    
    assert!(server.is_err(), "Server should fail with invalid path");
    
    let error = server.unwrap_err();
    assert!(error.to_string().contains("path"), 
            "Error message should mention path issue");
}

#[test]
fn test_mcp_server_requires_directory() {
    // Create a file, not a directory
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("file.txt");
    std::fs::write(&file_path, "test").unwrap();
    
    let server = MCPServer::new(file_path);
    
    assert!(server.is_err(), "Server should fail with file path");
    assert!(server.unwrap_err().to_string().contains("directory"));
}
```

### src/lib.rs
```rust
pub mod server;
pub mod tools;
pub mod transport;
pub mod types;

pub use server::MCPServer;
pub use types::*;

// Re-export commonly used items
pub use anyhow::{Result, Error};
```

### src/server.rs (GREEN phase implementation)
```rust
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::{Result, Context};

use unified_searcher::UnifiedSearcher;
use vector_storage::VectorStorage;
use git_watcher::Phase3GitWatch;

pub struct MCPServer {
    project_path: PathBuf,
    searcher: Option<Arc<RwLock<UnifiedSearcher>>>,
    storage: Option<Arc<RwLock<VectorStorage>>>,
    git_watch: Option<Arc<RwLock<Phase3GitWatch>>>,
}

impl MCPServer {
    pub fn new(project_path: PathBuf) -> Result<Self> {
        // Validate path exists
        if !project_path.exists() {
            return Err(anyhow::anyhow!("Project path does not exist: {}", 
                                      project_path.display()));
        }
        
        // Validate it's a directory
        if !project_path.is_dir() {
            return Err(anyhow::anyhow!("Project path must be a directory: {}", 
                                      project_path.display()));
        }
        
        // Canonicalize for consistent paths
        let canonical_path = project_path
            .canonicalize()
            .context("Failed to resolve project path")?;
        
        Ok(Self {
            project_path: canonical_path,
            searcher: None,
            storage: None,
            git_watch: None,
        })
    }
    
    pub fn project_path(&self) -> &PathBuf {
        &self.project_path
    }
}
```

## Task 4-5: Tool Registration

### Test for Tool Registration
```rust
#[test]
fn test_register_tools() {
    let temp_dir = TempDir::new().unwrap();
    let mut server = MCPServer::new(temp_dir.path().to_path_buf()).unwrap();
    
    // Create a mock MCP server instance
    let mut mcp = MockMCPProtocol::new();
    
    // Register tools should add all 4 tools
    server.register_tools(&mut mcp);
    
    let tools = mcp.get_registered_tools();
    assert_eq!(tools.len(), 4, "Should register exactly 4 tools");
    
    // Verify tool names
    let tool_names: Vec<&str> = tools.iter()
        .map(|t| t.name.as_str())
        .collect();
    
    assert!(tool_names.contains(&"search_code"));
    assert!(tool_names.contains(&"clear_database"));
    assert!(tool_names.contains(&"reindex_all"));
    assert!(tool_names.contains(&"toggle_watch"));
}
```

### Implementation
```rust
// src/types.rs
use serde::{Serialize, Deserialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub parameters: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Request {
    pub id: String,
    pub method: String,
    pub params: Value,
}

#[derive(Debug)]
pub enum ToolResult {
    Success(Value),
    Error(String),
}

// src/server.rs (add to existing)
impl MCPServer {
    pub fn register_tools(&mut self, mcp: &mut dyn MCPProtocol) {
        mcp.register_tool(self.search_tool());
        mcp.register_tool(self.clear_database_tool());
        mcp.register_tool(self.reindex_all_tool());
        mcp.register_tool(self.toggle_watch_tool());
    }
    
    fn search_tool(&self) -> Tool {
        Tool {
            name: "search_code".to_string(),
            description: "Search code with 3-chunk context".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        }
    }
    
    fn clear_database_tool(&self) -> Tool {
        Tool {
            name: "clear_database".to_string(),
            description: "Clear the vector database".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        }
    }
    
    fn reindex_all_tool(&self) -> Tool {
        Tool {
            name: "reindex_all".to_string(),
            description: "Reindex all files".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        }
    }
    
    fn toggle_watch_tool(&self) -> Tool {
        Tool {
            name: "toggle_watch".to_string(),
            description: "Toggle file watching".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        }
    }
}
```

## Task 6-7: Search Tool Schema

### Test for Proper Schema
```rust
#[test]
fn test_search_tool_schema() {
    let temp_dir = TempDir::new().unwrap();
    let server = MCPServer::new(temp_dir.path().to_path_buf()).unwrap();
    
    let tool = server.search_tool();
    
    // Verify basic properties
    assert_eq!(tool.name, "search_code");
    assert!(tool.description.contains("3-chunk context"));
    
    // Verify parameters schema
    let params = tool.parameters.as_object()
        .expect("Parameters should be an object");
    
    assert_eq!(params.get("type").unwrap(), "object");
    
    let properties = params.get("properties").unwrap().as_object().unwrap();
    assert!(properties.contains_key("query"));
    
    let query_schema = properties.get("query").unwrap().as_object().unwrap();
    assert_eq!(query_schema.get("type").unwrap(), "string");
    assert!(query_schema.contains_key("description"));
    
    let required = params.get("required").unwrap().as_array().unwrap();
    assert!(required.contains(&json!("query")));
}
```

### Implementation with Full Schema
```rust
fn search_tool(&self) -> Tool {
    Tool {
        name: "search_code".to_string(),
        description: "Search code with 3-chunk context. Handles any query complexity.".to_string(),
        parameters: json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Search query (can be simple text or complex expression)"
                }
            },
            "required": ["query"]
        }),
    }
}
```

## Mock Helpers for Testing

### tests/unit/mocks.rs
```rust
use mockall::mock;
use mcp_server::{Tool, MCPProtocol};

mock! {
    pub MCPProtocol {}
    
    impl MCPProtocol for MCPProtocol {
        fn register_tool(&mut self, tool: Tool);
        fn get_registered_tools(&self) -> Vec<Tool>;
    }
}

// Helper to create server with mocked dependencies
pub fn create_test_server() -> MCPServer {
    let temp_dir = TempDir::new().unwrap();
    MCPServer::new(temp_dir.path().to_path_buf()).unwrap()
}

pub fn create_test_server_with_mocks() -> MCPServer {
    let mut server = create_test_server();
    
    // Initialize with mock components
    // This will be implemented as we progress through tasks
    
    server
}
```

## Running the Tests

```bash
# Run all tests for the first few tasks
cargo test --package mcp_server

# Run with output to see failures
cargo test --package mcp_server -- --nocapture

# Run specific test
cargo test test_mcp_server_initialization_succeeds

# Watch mode for TDD
cargo watch -x "test --package mcp_server"
```

## Next Steps

After completing these initial tasks:

1. Continue with Task 8-11 for SearchParams handling
2. Implement response structures (Tasks 12-17)
3. Move on to other tools following the same TDD pattern
4. Integrate all components in final stages

Remember: 
- Always write the test first (RED)
- Write minimal code to pass (GREEN)
- Refactor for quality (REFACTOR)
- Each task should take ~15 minutes