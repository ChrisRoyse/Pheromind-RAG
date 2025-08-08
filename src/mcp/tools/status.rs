//! Status tool implementation for MCP server
//!
//! Provides comprehensive system status including indexing statistics,
//! search performance metrics, and server health information.

use std::sync::Arc;
use tokio::sync::RwLock;
use serde::Deserialize;

use crate::mcp::{McpError, McpResult};
use crate::mcp::protocol::JsonRpcResponse;
use crate::mcp::types::{StatsResponse, IndexStats, CacheStats, PerformanceStats, ServerStats};
use crate::search::unified::UnifiedSearcher;

/// Parameters for get_status tool
#[derive(Debug, Deserialize)]
struct StatusParams {
    /// Include cache statistics (optional)
    include_cache: Option<bool>,
    /// Include performance metrics (optional)
    include_performance: Option<bool>,
    /// Include index statistics (optional)
    include_index: Option<bool>,
    /// Include detailed breakdown (optional)
    detailed: Option<bool>,
}

/// Execute get_status tool
/// Returns comprehensive system status and statistics
pub async fn execute_get_status(
    searcher: &Arc<RwLock<UnifiedSearcher>>,
    params: &serde_json::Value,
    id: Option<serde_json::Value>
) -> McpResult<JsonRpcResponse> {
    // Parse parameters (all optional)
    let status_params: StatusParams = if params.is_null() {
        StatusParams {
            include_cache: Some(true),
            include_performance: Some(true),
            include_index: Some(true),
            detailed: Some(false),
        }
    } else {
        serde_json::from_value(params.clone())
            .map_err(|e| McpError::InvalidParams {
                message: format!("Invalid status parameters: {}. All parameters are optional: include_cache (bool), include_performance (bool), include_index (bool), detailed (bool)", e)
            })?
    };
    
    println!("📊 MCP Tool: Getting system status (cache: {:?}, performance: {:?}, index: {:?})",
             status_params.include_cache, status_params.include_performance, status_params.include_index);
    
    let start_time = std::time::Instant::now();
    
    // Collect statistics from UnifiedSearcher
    let searcher_guard = searcher.read().await;
    
    // Get searcher statistics if ML feature is enabled
    #[cfg(feature = "ml")]
    let searcher_stats = searcher_guard.get_stats().await
        .map_err(|e| McpError::InternalError {
            message: format!("Failed to get searcher statistics: {}", e)
        })?;
    
    #[cfg(not(feature = "ml"))]
    let searcher_stats: Option<()> = None;
    
    drop(searcher_guard);
    
    // Build index statistics
    let index_stats = if status_params.include_index.unwrap_or(true) {
        #[cfg(feature = "ml")]
        {
            Some(IndexStats {
                total_files: 0, // Would need to track this in UnifiedSearcher
                total_chunks: searcher_stats.total_embeddings as u32,
                total_symbols: 0, // Would need symbol count from tree-sitter
                index_size_bytes: 0, // Would need to calculate from all indexes
                last_updated: chrono::Utc::now().to_rfc3339(),
            })
        }
        #[cfg(not(feature = "ml"))]
        {
            Some(IndexStats {
                total_files: 0,
                total_chunks: 0,
                total_symbols: 0,
                index_size_bytes: 0,
                last_updated: chrono::Utc::now().to_rfc3339(),
            })
        }
    } else {
        None
    };
    
    // Build cache statistics
    let cache_stats = if status_params.include_cache.unwrap_or(true) {
        #[cfg(feature = "ml")]
        {
            Some(CacheStats {
                search_cache_entries: searcher_stats.cache_entries as u32,
                search_cache_hit_rate: 0.0, // Would need to track hit rate
                embedding_cache_entries: searcher_stats.embedding_cache_entries as u32,
                embedding_cache_hit_rate: 0.0, // Would need to track hit rate
                total_cache_size_bytes: 0, // Would need to calculate cache memory usage
            })
        }
        #[cfg(not(feature = "ml"))]
        {
            Some(CacheStats {
                search_cache_entries: 0,
                search_cache_hit_rate: 0.0,
                embedding_cache_entries: 0,
                embedding_cache_hit_rate: 0.0,
                total_cache_size_bytes: 0,
            })
        }
    } else {
        None
    };
    
    // Build performance statistics
    let performance_stats = if status_params.include_performance.unwrap_or(true) {
        Some(PerformanceStats {
            avg_search_time_ms: 0.0, // Would need to track search timing
            avg_index_time_ms: 0.0,  // Would need to track indexing timing
            total_searches: 0,       // Would need to track search count
            total_indexes: 0,        // Would need to track indexing count
            uptime_seconds: 0,       // Would need to track server uptime
        })
    } else {
        None
    };
    
    // Build server statistics
    let server_stats = ServerStats {
        active_connections: 1, // Simplified for single-connection MCP
        total_requests: 0,     // Would need to track request count
        error_count: 0,        // Would need to track error count
        memory_usage_mb: get_memory_usage_mb(),
        cpu_usage_percent: get_cpu_usage_percent(),
    };
    
    let response = StatsResponse {
        index_stats,
        cache_stats,
        performance_stats,
        server_stats,
    };
    
    let status_time = start_time.elapsed();
    
    println!("✅ MCP Tool: Status collected in {:?}", status_time);
    
    JsonRpcResponse::success(response, id)
}

/// Get current memory usage in MB
/// This is a simplified implementation - would use system metrics in production
fn get_memory_usage_mb() -> f64 {
    // Placeholder implementation - would use system metrics
    0.0
}

/// Get current CPU usage percentage
/// This is a simplified implementation - would use system metrics in production
fn get_cpu_usage_percent() -> f64 {
    // Placeholder implementation - would use system metrics
    0.0
}
