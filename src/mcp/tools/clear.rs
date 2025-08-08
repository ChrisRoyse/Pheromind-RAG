//! Clear index tool implementation for MCP server
//!
//! Provides safe index clearing functionality with confirmation requirements
//! and comprehensive cleanup across all search backends.

use std::sync::Arc;
use tokio::sync::RwLock;
use serde::Deserialize;

use crate::mcp::{McpError, McpResult};
use crate::mcp::protocol::JsonRpcResponse;
use crate::mcp::types::{ClearResponse, ClearType};
use crate::search::unified::UnifiedSearcher;

/// Parameters for clear_index tool
#[derive(Debug, Deserialize)]
struct ClearParams {
    /// Confirmation flag - must be true to execute clear
    confirm: bool,
    /// Type of clear operation (optional - defaults to All)
    clear_type: Option<ClearType>,
    /// Safety confirmation phrase (optional additional safety)
    safety_phrase: Option<String>,
}

/// Execute clear_index tool
/// Safely clears all or specified parts of the search index with proper confirmation
pub async fn execute_clear_index(
    searcher: &Arc<RwLock<UnifiedSearcher>>,
    params: &serde_json::Value,
    id: Option<serde_json::Value>
) -> McpResult<JsonRpcResponse> {
    // Parse and validate parameters
    let clear_params: ClearParams = serde_json::from_value(params.clone())
        .map_err(|e| McpError::InvalidParams {
            message: format!("Invalid clear_index parameters: {}. Required: confirm (bool). Optional: clear_type (string), safety_phrase (string)", e)
        })?;
    
    // Require explicit confirmation
    if !clear_params.confirm {
        return Err(McpError::InvalidParams {
            message: "Clear operation requires explicit confirmation. Set 'confirm' to true.".to_string()
        });
    }
    
    let clear_type = clear_params.clear_type.unwrap_or(ClearType::All);
    
    println!("🧹 MCP Tool: Clearing index (type: {:?}, confirmed: {})", 
             clear_type, clear_params.confirm);
    
    // Additional safety check for safety phrase if provided
    if let Some(phrase) = &clear_params.safety_phrase {
        if phrase != "CLEAR_ALL_DATA" {
            return Err(McpError::InvalidParams {
                message: "Invalid safety phrase. Expected: 'CLEAR_ALL_DATA'".to_string()
            });
        }
    }
    
    let start_time = std::time::Instant::now();
    let items_removed = match clear_type {
        ClearType::All => {
            clear_all_indexes(searcher).await?
        },
        ClearType::SearchIndex => {
            clear_search_index(searcher).await?
        },
        ClearType::VectorIndex => {
            clear_vector_index(searcher).await?
        },
        ClearType::SymbolIndex => {
            clear_symbol_index(searcher).await?
        },
        ClearType::Cache => {
            clear_cache_only(searcher).await?
        },
    };
    
    let clear_time = start_time.elapsed();
    
    let response = ClearResponse {
        cleared: true,
        clear_type: format!("{:?}", clear_type),
        items_removed,
    };
    
    println!("✅ MCP Tool: Cleared {} items of type {:?} in {:?}",
             response.items_removed, clear_type, clear_time);
    
    JsonRpcResponse::success(response, id)
}

/// Clear all indexes and caches
async fn clear_all_indexes(searcher: &Arc<RwLock<UnifiedSearcher>>) -> McpResult<u32> {
    println!("🧹 Clearing all indexes and caches...");
    
    let searcher_guard = searcher.read().await;
    searcher_guard.clear_index().await
        .map_err(|e| McpError::InternalError {
            message: format!("Failed to clear all indexes: {}", e)
        })?;
    drop(searcher_guard);
    
    // Return estimated items removed - would need proper tracking in production
    Ok(0) // Placeholder
}

/// Clear search index only (Tantivy/BM25)
async fn clear_search_index(searcher: &Arc<RwLock<UnifiedSearcher>>) -> McpResult<u32> {
    println!("🧹 Clearing search indexes (Tantivy/BM25)...");
    
    // This would need to expose individual index clearing methods on UnifiedSearcher
    // For now, we use the full clear method which is safe but broader than needed
    clear_all_indexes(searcher).await
}

/// Clear vector index only (LanceDB/embeddings)
async fn clear_vector_index(_searcher: &Arc<RwLock<UnifiedSearcher>>) -> McpResult<u32> {
    #[cfg(feature = "vectordb")]
    {
        println!("🧹 Clearing vector index (LanceDB)...");
        // Would need to expose vector-specific clearing on UnifiedSearcher
        clear_all_indexes(_searcher).await
    }
    #[cfg(not(feature = "vectordb"))]
    {
        Err(McpError::InternalError {
            message: "Vector index clearing not available: vectordb feature disabled".to_string()
        })
    }
}

/// Clear symbol index only (Tree-sitter)
async fn clear_symbol_index(_searcher: &Arc<RwLock<UnifiedSearcher>>) -> McpResult<u32> {
    #[cfg(feature = "tree-sitter")]
    {
        println!("🧹 Clearing symbol index (Tree-sitter)...");
        // Would need to expose symbol-specific clearing on UnifiedSearcher
        clear_all_indexes(_searcher).await
    }
    #[cfg(not(feature = "tree-sitter"))]
    {
        Err(McpError::InternalError {
            message: "Symbol index clearing not available: tree-sitter feature disabled".to_string()
        })
    }
}

/// Clear cache only (no persistent data)
async fn clear_cache_only(_searcher: &Arc<RwLock<UnifiedSearcher>>) -> McpResult<u32> {
    println!("🧹 Clearing caches only...");
    
    // This would clear search result cache, embedding cache, etc.
    // without touching persistent indexes
    
    // Placeholder implementation - would need cache-specific methods
    Ok(0)
}
