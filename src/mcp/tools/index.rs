//! Index directory tool implementation for MCP server
//!
//! Provides directory indexing functionality with comprehensive error handling
//! and integration with UnifiedSearcher's indexing capabilities.

use std::sync::Arc;
use std::path::PathBuf;
use tokio::sync::RwLock;
use serde::Deserialize;

use crate::mcp::{McpError, McpResult};
use crate::mcp::protocol::JsonRpcResponse;
use crate::mcp::types::IndexResponse;
use crate::search::unified::UnifiedSearcher;

/// Parameters for index_directory tool
#[derive(Debug, Deserialize)]
struct IndexDirectoryParams {
    /// Directory path to index
    directory_path: String,
    /// Whether to include test files (optional)
    include_test_files: Option<bool>,
    /// File patterns to include (optional)
    include_patterns: Option<Vec<String>>,
    /// File patterns to exclude (optional)
    exclude_patterns: Option<Vec<String>>,
}

/// Execute index_directory tool
/// Indexes specified directory with all files matching criteria
pub async fn execute_index_directory(
    searcher: &Arc<RwLock<UnifiedSearcher>>,
    params: &serde_json::Value,
    id: Option<serde_json::Value>
) -> McpResult<JsonRpcResponse> {
    // Parse and validate parameters
    let index_params: IndexDirectoryParams = serde_json::from_value(params.clone())
        .map_err(|e| McpError::InvalidParams {
            message: format!("Invalid index_directory parameters: {}. Required: directory_path (string). Optional: include_test_files (bool), include_patterns (array), exclude_patterns (array)", e)
        })?;
    
    // Validate directory path exists
    let dir_path = PathBuf::from(&index_params.directory_path);
    if !dir_path.exists() {
        return Err(McpError::InvalidParams {
            message: format!("Directory does not exist: {}", index_params.directory_path)
        });
    }
    
    if !dir_path.is_dir() {
        return Err(McpError::InvalidParams {
            message: format!("Path is not a directory: {}", index_params.directory_path)
        });
    }
    
    println!("📂 MCP Tool: Indexing directory {:?} (include_test_files: {:?})", 
             dir_path, index_params.include_test_files);
    
    // Execute indexing operation
    let start_time = std::time::Instant::now();
    
    let searcher_guard = searcher.read().await;
    let stats = searcher_guard.index_directory(&dir_path).await
        .map_err(|e| McpError::InternalError {
            message: format!("Failed to index directory {:?}: {}", dir_path, e)
        })?;
    drop(searcher_guard);
    
    let index_time = start_time.elapsed();
    
    // Create response
    let response = IndexResponse {
        files_indexed: stats.files_indexed as u32,
        chunks_created: stats.chunks_created as u32,
        symbols_extracted: 0, // Not tracked in current implementation
        errors: stats.errors as u32,
        index_time_ms: index_time.as_millis() as u64,
        index_stats: None, // Could be extended if needed
    };
    
    println!("✅ MCP Tool: Successfully indexed {} files, created {} chunks in {:?}",
             response.files_indexed, response.chunks_created, index_time);
    
    JsonRpcResponse::success(response, id)
}
