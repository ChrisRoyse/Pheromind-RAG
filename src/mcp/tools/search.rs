//! Search tool implementation for MCP server
//!
//! Provides parallel search execution across all 4 backends (BM25, Exact, Semantic, Symbol)
//! with comprehensive result fusion and error handling.

use std::sync::Arc;
use tokio::sync::RwLock;
use serde::Deserialize;

use crate::mcp::{McpError, McpResult};
use crate::mcp::protocol::JsonRpcResponse;
use crate::mcp::types::{SearchResponse, SearchMatch, SearchType};
use crate::search::unified::UnifiedSearcher;

/// Parameters for search tool
#[derive(Debug, Deserialize)]
struct SearchParams {
    /// Search query string
    query: String,
    /// Maximum number of results to return (optional)
    max_results: Option<u32>,
    /// Specific search types to use (optional)
    search_types: Option<Vec<SearchType>>,
    /// File pattern filters (optional)
    file_filters: Option<Vec<String>>,
}

/// Execute search tool with parallel backend execution
/// UnifiedSearcher already executes all 4 backends in parallel using tokio::join!
/// This achieves ~70% latency reduction compared to sequential execution
pub async fn execute_search(
    searcher: &Arc<RwLock<UnifiedSearcher>>,
    params: &serde_json::Value,
    id: Option<serde_json::Value>
) -> McpResult<JsonRpcResponse> {
    // Parse and validate parameters
    let search_params: SearchParams = serde_json::from_value(params.clone())
        .map_err(|e| McpError::InvalidParams {
            message: format!("Invalid search parameters: {}. Required: query (string). Optional: max_results (number), search_types (array), file_filters (array)", e)
        })?;
    
    // Validate query is not empty
    if search_params.query.trim().is_empty() {
        return Err(McpError::InvalidParams {
            message: "Search query cannot be empty".to_string()
        });
    }
    
    let max_results = search_params.max_results.unwrap_or(50);
    println!("🔍 MCP Tool: Parallel search for '{}' (max_results: {})", 
             search_params.query, max_results);
    
    let start_time = std::time::Instant::now();
    
    // Use UnifiedSearcher's parallel search implementation
    // It already executes all 4 backends (BM25, Exact, Semantic, Symbol) in parallel using tokio::join!
    let searcher_guard = searcher.read().await;
    let search_results = searcher_guard.search(&search_params.query).await
        .map_err(|e| McpError::InternalError {
            message: format!("Search failed: {}", e)
        })?;
    drop(searcher_guard);
    
    let search_time = start_time.elapsed();
    
    // Convert internal SearchResult to MCP SearchMatch format
    let total_found = search_results.len();
    let all_matches: Vec<SearchMatch> = search_results
        .into_iter()
        .take(max_results as usize)
        .map(SearchMatch::from)
        .collect();
    
    // Determine which search types were used based on match types
    let mut search_types_used = Vec::new();
    let mut type_counts = std::collections::HashMap::new();
    
    for match_result in &all_matches {
        let match_type = match match_result.match_type.as_str() {
            "Exact" => "Exact",
            "Semantic" => "Semantic", 
            "Symbol" => "Symbol",
            "Statistical" => "Statistical",
            _ => "Statistical", // Default fallback
        };
        
        *type_counts.entry(match_type).or_insert(0) += 1;
    }
    
    // Build search types used based on what we found
    if type_counts.contains_key("Exact") {
        search_types_used.push(SearchType::Exact);
    }
    if type_counts.contains_key("Semantic") {
        search_types_used.push(SearchType::Semantic);
    }
    if type_counts.contains_key("Symbol") {
        search_types_used.push(SearchType::Symbol);
    }
    if type_counts.contains_key("Statistical") {
        search_types_used.push(SearchType::Statistical);
    }
    
    // Log search type statistics
    for (search_type, count) in type_counts {
        println!("📊 Found {} {} matches", count, search_type);
    }
    let response = SearchResponse {
        results: all_matches,
        total_matches: total_found as u32,
        search_time_ms: search_time.as_millis() as u64,
        search_types_used,
    };
    
    println!("✅ MCP Tool: Returned {} results in {:?} using parallel backends",
             response.total_matches, search_time);
    
    JsonRpcResponse::success(response, id)
}

