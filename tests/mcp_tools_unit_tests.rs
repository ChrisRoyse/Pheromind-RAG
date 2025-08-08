//! Unit tests for MCP tools implementation
//!
//! Tests individual MCP tool functionality in isolation

use std::sync::Arc;
use tokio::sync::RwLock;
use tempfile::TempDir;
use serde_json::json;

use embed_search::mcp::tools::ToolRegistry;
use embed_search::search::unified::UnifiedSearcher;
use embed_search::config::Config;

async fn setup_test_searcher() -> Arc<RwLock<UnifiedSearcher>> {
    // Initialize config
    std::env::set_var("EMBED_LOG_LEVEL", "error");
    if let Err(_) = Config::init() {
        // Config already initialized, that's ok
    }
    
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path().to_path_buf();
    let db_path = project_path.join(".embed-search");
    
    let searcher = UnifiedSearcher::new(project_path, db_path).await.unwrap();
    Arc::new(RwLock::new(searcher))
}

#[tokio::test]
async fn test_search_tool_execution() {
    let searcher = setup_test_searcher().await;
    let registry = ToolRegistry::new(searcher);
    
    let params = json!({
        "query": "test function",
        "max_results": 10
    });
    
    let response = registry.execute_search(&params, Some(json!(1))).await;
    assert!(response.is_ok(), "Search tool should execute without error");
    
    let json_response = response.unwrap();
    let result = json_response.to_json();
    assert_eq!(result["jsonrpc"], "2.0");
    assert_eq!(result["id"], 1);
    assert!(result["result"].is_object());
}

#[tokio::test]
async fn test_status_tool_execution() {
    let searcher = setup_test_searcher().await;
    let registry = ToolRegistry::new(searcher);
    
    let params = json!({
        "include_cache": true,
        "include_performance": true,
        "include_index": true
    });
    
    let response = registry.execute_get_status(&params, Some(json!(2))).await;
    assert!(response.is_ok(), "Status tool should execute without error");
    
    let json_response = response.unwrap();
    let result = json_response.to_json();
    assert_eq!(result["jsonrpc"], "2.0");
    assert_eq!(result["id"], 2);
    assert!(result["result"]["server_stats"].is_object());
}

#[tokio::test]
async fn test_clear_tool_execution() {
    let searcher = setup_test_searcher().await;
    let registry = ToolRegistry::new(searcher);
    
    let params = json!({
        "confirm": true,
        "clear_type": "all"
    });
    
    let response = registry.execute_clear_index(&params, Some(json!(3))).await;
    assert!(response.is_ok(), "Clear tool should execute without error");
    
    let json_response = response.unwrap();
    let result = json_response.to_json();
    assert_eq!(result["jsonrpc"], "2.0");
    assert_eq!(result["id"], 3);
    assert_eq!(result["result"]["cleared"], true);
}

#[tokio::test]
async fn test_index_tool_execution() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path().to_path_buf();
    
    // Create a test file
    let test_file = project_path.join("test.rs");
    tokio::fs::write(&test_file, "fn test_function() { println!(\"Hello\"); }").await.unwrap();
    
    let searcher = setup_test_searcher().await;
    let registry = ToolRegistry::new(searcher);
    
    let params = json!({
        "directory_path": project_path.to_string_lossy(),
        "include_test_files": false
    });
    
    let response = registry.execute_index_directory(&params, Some(json!(4))).await;
    assert!(response.is_ok(), "Index tool should execute without error");
    
    let json_response = response.unwrap();
    let result = json_response.to_json();
    assert_eq!(result["jsonrpc"], "2.0");
    assert_eq!(result["id"], 4);
    assert!(result["result"]["files_indexed"].as_u64().unwrap_or(0) >= 0);
}

#[tokio::test]
async fn test_search_tool_parameter_validation() {
    let searcher = setup_test_searcher().await;
    let registry = ToolRegistry::new(searcher);
    
    // Test empty query
    let invalid_params = json!({
        "query": "",
        "max_results": 10
    });
    
    let response = registry.execute_search(&invalid_params, Some(json!(5))).await;
    assert!(response.is_err(), "Empty query should be rejected");
    
    // Test missing query
    let missing_query = json!({
        "max_results": 10
    });
    
    let response = registry.execute_search(&missing_query, Some(json!(6))).await;
    assert!(response.is_err(), "Missing query should be rejected");
}

#[tokio::test]
async fn test_clear_tool_confirmation_requirement() {
    let searcher = setup_test_searcher().await;
    let registry = ToolRegistry::new(searcher);
    
    // Test without confirmation
    let no_confirm = json!({
        "confirm": false
    });
    
    let response = registry.execute_clear_index(&no_confirm, Some(json!(7))).await;
    assert!(response.is_err(), "Clear without confirmation should be rejected");
    
    // Test missing confirmation
    let missing_confirm = json!({
        "clear_type": "all"
    });
    
    let response = registry.execute_clear_index(&missing_confirm, Some(json!(8))).await;
    assert!(response.is_err(), "Clear without explicit confirmation should be rejected");
}