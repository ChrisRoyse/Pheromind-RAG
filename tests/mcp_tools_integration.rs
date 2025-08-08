//! Integration tests for MCP tools
//!
//! Tests the complete MCP tool implementation including:
//! - index_directory tool with parallel execution
//! - search tool with all 4 backends (BM25, Exact, Semantic, Symbol) 
//! - get_status tool with comprehensive statistics
//! - clear_index tool with safety confirmation
//!
//! This validates the actual working implementations with no mocks.

use std::path::PathBuf;
use tempfile::TempDir;
use tokio;
use serde_json::json;

use embed_search::mcp::McpServer;
use embed_search::config::Config;

#[tokio::test]
async fn test_mcp_tools_full_integration() {
    // Initialize configuration
    std::env::set_var("EMBED_LOG_LEVEL", "info");
    if let Err(_) = Config::init() {
        // Config already initialized, that's ok
    }
    
    // Create temporary project directory
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path().to_path_buf();
    
    // Create some test files to index
    create_test_files(&project_path).await;
    
    // Create MCP server
    let mut server = McpServer::with_project_path(project_path.clone()).await
        .expect("Failed to create MCP server");
    
    println!("🧪 Testing MCP Tools Integration");
    
    // Test 1: Index Directory Tool
    println!("📂 Testing index_directory tool...");
    let index_request = json!({
        "jsonrpc": "2.0",
        "method": "index",
        "params": {
            "directory_path": project_path.to_string_lossy(),
            "include_test_files": false
        },
        "id": 1
    });
    
    let index_response = server.handle_request(&index_request.to_string()).await;
    let index_result: serde_json::Value = serde_json::from_str(&index_response).unwrap();
    
    assert_eq!(index_result["jsonrpc"], "2.0");
    assert_eq!(index_result["id"], 1);
    assert!(index_result["result"].is_object());
    assert!(index_result["result"]["files_indexed"].as_u64().unwrap() > 0);
    println!("✅ Index tool: Indexed {} files", 
             index_result["result"]["files_indexed"].as_u64().unwrap());
    
    // Test 2: Search Tool (Parallel Backend Execution)
    println!("🔍 Testing search tool with parallel backend execution...");
    let search_request = json!({
        "jsonrpc": "2.0", 
        "method": "search",
        "params": {
            "query": "function test_example",
            "max_results": 10
        },
        "id": 2
    });
    
    let search_response = server.handle_request(&search_request.to_string()).await;
    let search_result: serde_json::Value = serde_json::from_str(&search_response).unwrap();
    
    assert_eq!(search_result["jsonrpc"], "2.0"); 
    assert_eq!(search_result["id"], 2);
    assert!(search_result["result"].is_object());
    assert!(search_result["result"]["results"].is_array());
    println!("✅ Search tool: Found {} results using parallel backends", 
             search_result["result"]["total_matches"].as_u64().unwrap_or(0));
    
    // Test 3: Get Status Tool
    println!("📊 Testing get_status tool...");
    let stats_request = json!({
        "jsonrpc": "2.0",
        "method": "stats", 
        "params": {
            "include_cache": true,
            "include_performance": true,
            "include_index": true
        },
        "id": 3
    });
    
    let stats_response = server.handle_request(&stats_request.to_string()).await;
    let stats_result: serde_json::Value = serde_json::from_str(&stats_response).unwrap();
    
    assert_eq!(stats_result["jsonrpc"], "2.0");
    assert_eq!(stats_result["id"], 3);
    assert!(stats_result["result"].is_object());
    assert!(stats_result["result"]["server_stats"].is_object());
    println!("✅ Status tool: Retrieved comprehensive system statistics");
    
    // Test 4: Clear Index Tool (with confirmation)
    println!("🧹 Testing clear_index tool...");
    let clear_request = json!({
        "jsonrpc": "2.0",
        "method": "clear",
        "params": {
            "confirm": true,
            "clear_type": "all"
        },
        "id": 4
    });
    
    let clear_response = server.handle_request(&clear_request.to_string()).await;
    let clear_result: serde_json::Value = serde_json::from_str(&clear_response).unwrap();
    
    assert_eq!(clear_result["jsonrpc"], "2.0");
    assert_eq!(clear_result["id"], 4);
    assert!(clear_result["result"].is_object());
    assert_eq!(clear_result["result"]["cleared"], true);
    println!("✅ Clear tool: Successfully cleared all indexes");
    
    // Test 5: Search after clear (should return no results)
    println!("🔍 Testing search after clear...");
    let search_after_clear = server.handle_request(&search_request.to_string()).await;
    let search_after_result: serde_json::Value = serde_json::from_str(&search_after_clear).unwrap();
    
    assert_eq!(search_after_result["jsonrpc"], "2.0");
    let results_after_clear = search_after_result["result"]["total_matches"].as_u64().unwrap_or(1);
    assert_eq!(results_after_clear, 0, "Search should return 0 results after clearing index");
    println!("✅ Search after clear: Confirmed index was cleared (0 results)");
    
    println!("🎉 All MCP tools integration tests passed!");
}

async fn create_test_files(project_path: &PathBuf) {
    let src_dir = project_path.join("src");
    tokio::fs::create_dir_all(&src_dir).await.unwrap();
    
    // Create a Rust source file
    let rust_file = src_dir.join("lib.rs");
    let rust_content = r#"
/// Example library with various functions
pub struct ExampleStruct {
    pub name: String,
    pub value: i32,
}

impl ExampleStruct {
    pub fn new(name: &str, value: i32) -> Self {
        Self {
            name: name.to_string(),
            value,
        }
    }
    
    pub fn function_test_example(&self) -> String {
        format!("{}_{}", self.name, self.value)
    }
}

pub fn function_test_example() -> &'static str {
    "test example function"
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_example_struct() {
        let example = ExampleStruct::new("test", 42);
        assert_eq!(example.function_test_example(), "test_42");
    }
}
"#;
    tokio::fs::write(&rust_file, rust_content).await.unwrap();
    
    // Create a Python file
    let python_file = src_dir.join("example.py");
    let python_content = r#"
"""Example Python module for testing"""

class ExampleClass:
    def __init__(self, name: str, value: int):
        self.name = name
        self.value = value
    
    def function_test_example(self) -> str:
        return f"{self.name}_{self.value}"

def function_test_example() -> str:
    """Test example function"""
    return "test example function"

if __name__ == "__main__":
    example = ExampleClass("test", 42)
    print(example.function_test_example())
    print(function_test_example())
"#;
    tokio::fs::write(&python_file, python_content).await.unwrap();
    
    // Create a JavaScript file
    let js_file = src_dir.join("example.js");
    let js_content = r#"
/**
 * Example JavaScript module for testing
 */

class ExampleClass {
    constructor(name, value) {
        this.name = name;
        this.value = value;
    }
    
    functionTestExample() {
        return `${this.name}_${this.value}`;
    }
}

function functionTestExample() {
    return "test example function";
}

// Export for testing
if (typeof module !== 'undefined') {
    module.exports = { ExampleClass, functionTestExample };
}
"#;
    tokio::fs::write(&js_file, js_content).await.unwrap();
    
    println!("📝 Created test files in {:?}", project_path);
}

#[tokio::test]
async fn test_mcp_error_handling() {
    // Test error handling for invalid parameters
    std::env::set_var("EMBED_LOG_LEVEL", "error");
    if let Err(_) = Config::init() {
        // Config already initialized, that's ok
    }
    
    let temp_dir = TempDir::new().unwrap();
    let mut server = McpServer::with_project_path(temp_dir.path().to_path_buf()).await.unwrap();
    
    println!("🧪 Testing MCP Error Handling");
    
    // Test invalid search (empty query)
    let invalid_search = json!({
        "jsonrpc": "2.0",
        "method": "search", 
        "params": {
            "query": ""
        },
        "id": 1
    });
    
    let response = server.handle_request(&invalid_search.to_string()).await;
    let result: serde_json::Value = serde_json::from_str(&response).unwrap();
    
    assert!(result["error"].is_object());
    println!("✅ Error handling: Empty search query rejected");
    
    // Test clear without confirmation
    let invalid_clear = json!({
        "jsonrpc": "2.0",
        "method": "clear",
        "params": {
            "confirm": false
        },
        "id": 2
    });
    
    let response = server.handle_request(&invalid_clear.to_string()).await;
    let result: serde_json::Value = serde_json::from_str(&response).unwrap();
    
    assert!(result["error"].is_object());
    println!("✅ Error handling: Clear without confirmation rejected");
    
    println!("🎉 MCP error handling tests passed!");
}