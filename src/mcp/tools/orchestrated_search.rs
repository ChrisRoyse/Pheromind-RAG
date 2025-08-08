//! Orchestrated Search Tool - Enhanced coordination with performance monitoring
//!
//! This tool uses SearchOrchestrator to provide enhanced search capabilities
//! with performance monitoring, graceful failure handling, and resource management.
//! 
//! Truth: This builds on UnifiedSearcher's existing tokio::join! parallel execution
//! and adds production-ready orchestration features.

use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

use crate::mcp::{McpError, McpResult};
use crate::mcp::protocol::JsonRpcResponse;
use crate::mcp::types::{SearchMatch, SearchType};
use crate::mcp::orchestrator::{SearchOrchestrator, OrchestratorConfig};
use crate::search::unified::UnifiedSearcher;

/// Enhanced search parameters with orchestration features
#[derive(Debug, Deserialize)]
struct OrchestratedSearchParams {
    /// Search query string
    query: String,
    /// Maximum number of results to return
    max_results: Option<u32>,
    /// Specific search types to prioritize
    search_types: Option<Vec<SearchType>>,
    /// File pattern filters
    file_filters: Option<Vec<String>>,
    /// Enable detailed performance metrics in response
    include_metrics: Option<bool>,
    /// Timeout override in seconds
    timeout_seconds: Option<u32>,
    /// Enable partial failure tolerance
    allow_partial_failures: Option<bool>,
}

/// Enhanced search response with orchestration metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
struct OrchestratedSearchResponse {
    /// Search results
    results: Vec<SearchMatch>,
    /// Total matches found before truncation
    total_matches: u32,
    /// Search execution time in milliseconds
    search_time_ms: u64,
    /// Which search types were actually used
    search_types_used: Vec<SearchType>,
    /// Backend status and availability
    backend_status: Option<BackendStatusResponse>,
    /// Detailed performance metrics (optional)
    performance_metrics: Option<PerformanceMetricsResponse>,
    /// Resource usage information
    resource_usage: Option<ResourceUsageResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BackendStatusResponse {
    bm25_available: bool,
    exact_available: bool,
    semantic_available: bool,
    symbol_available: bool,
    failed_backends: Vec<String>,
    partial_failures: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PerformanceMetricsResponse {
    total_latency_ms: u64,
    backend_latencies_ms: std::collections::HashMap<String, u64>,
    fusion_time_ms: u64,
    results_before_fusion: u32,
    results_after_fusion: u32,
    deduplication_savings: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ResourceUsageResponse {
    active_searches: u32,
    available_permits: usize,
    max_concurrent: usize,
    memory_usage_estimate_mb: Option<f64>,
}

/// Orchestrated search tool registry
pub struct OrchestratedSearchTool {
    orchestrator: Arc<RwLock<SearchOrchestrator>>,
}

impl OrchestratedSearchTool {
    /// Create new orchestrated search tool
    pub async fn new(searcher: UnifiedSearcher, config: Option<OrchestratorConfig>) -> McpResult<Self> {
        let orchestrator = SearchOrchestrator::new(searcher, config).await?;
        
        Ok(Self {
            orchestrator: Arc::new(RwLock::new(orchestrator)),
        })
    }
    
    /// Execute orchestrated search with enhanced monitoring and coordination
    pub async fn execute_search(
        &self,
        params: &serde_json::Value,
        id: Option<serde_json::Value>
    ) -> McpResult<JsonRpcResponse> {
        // Parse and validate parameters
        let search_params: OrchestratedSearchParams = serde_json::from_value(params.clone())
            .map_err(|e| McpError::InvalidParams {
                message: format!(
                    "Invalid orchestrated search parameters: {}. Required: query (string). \
                     Optional: max_results (number), include_metrics (boolean), \
                     timeout_seconds (number), allow_partial_failures (boolean)", 
                    e
                )
            })?;
        
        // Validate query is not empty
        if search_params.query.trim().is_empty() {
            return Err(McpError::InvalidParams {
                message: "Search query cannot be empty".to_string()
            });
        }
        
        let max_results = search_params.max_results.unwrap_or(50);
        let include_metrics = search_params.include_metrics.unwrap_or(false);
        
        println!("🎯 Orchestrated Search: '{}' (max_results: {}, metrics: {})", 
                 search_params.query, max_results, include_metrics);
        
        // Execute orchestrated search with enhanced coordination
        let orchestrator = self.orchestrator.read().await;
        let orchestrated_result = orchestrator.search(&search_params.query).await?;
        drop(orchestrator);
        
        // Convert results to MCP format
        let results_count = orchestrated_result.results.len();
        let search_matches: Vec<SearchMatch> = orchestrated_result.results
            .into_iter()
            .take(max_results as usize)
            .map(SearchMatch::from)
            .collect();
        
        // Determine which search types were used based on match types
        let mut search_types_used = Vec::new();
        let mut type_counts = std::collections::HashMap::new();
        
        for match_result in &search_matches {
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
        
        // Build backend status response
        let backend_status = BackendStatusResponse {
            bm25_available: orchestrated_result.backend_status.bm25_available,
            exact_available: orchestrated_result.backend_status.exact_available,
            semantic_available: orchestrated_result.backend_status.semantic_available,
            symbol_available: orchestrated_result.backend_status.symbol_available,
            failed_backends: orchestrated_result.backend_status.failed_backends,
            partial_failures: orchestrated_result.metrics.partial_failures.clone(),
        };
        
        // Build performance metrics if requested
        let performance_metrics = if include_metrics {
            Some(PerformanceMetricsResponse {
                total_latency_ms: orchestrated_result.metrics.total_latency_ms,
                backend_latencies_ms: orchestrated_result.metrics.backend_latencies_ms
                    .into_iter()
                    .map(|(k, v)| (k, v))
                    .collect(),
                fusion_time_ms: orchestrated_result.metrics.fusion_time_ms,
                results_before_fusion: orchestrated_result.metrics.results_found,
                results_after_fusion: orchestrated_result.metrics.results_after_fusion,
                deduplication_savings: if orchestrated_result.metrics.results_found > 0 {
                    Some(1.0 - (orchestrated_result.metrics.results_after_fusion as f64 / 
                                orchestrated_result.metrics.results_found as f64))
                } else {
                    None
                },
            })
        } else {
            None
        };
        
        // Build resource usage information
        let orchestrator_guard = self.orchestrator.read().await;
        let status = orchestrator_guard.get_status().await;
        let resource_usage = Some(ResourceUsageResponse {
            active_searches: status["orchestrator"]["active_searches"].as_u64().unwrap_or(0) as u32,
            available_permits: status["orchestrator"]["available_permits"].as_u64().unwrap_or(0) as usize,
            max_concurrent: status["orchestrator"]["max_concurrent"].as_u64().unwrap_or(0) as usize,
            memory_usage_estimate_mb: None, // Could be implemented with system metrics
        });
        drop(orchestrator_guard);
        
        let response = OrchestratedSearchResponse {
            results: search_matches,
            total_matches: results_count as u32,
            search_time_ms: orchestrated_result.metrics.total_latency_ms,
            search_types_used,
            backend_status: Some(backend_status),
            performance_metrics,
            resource_usage,
        };
        
        // Log search type statistics
        for (search_type, count) in type_counts {
            println!("📊 Orchestrated: {} {} matches", count, search_type);
        }
        
        println!("✅ Orchestrated Search: {} results in {}ms with enhanced coordination",
                 response.total_matches, response.search_time_ms);
        
        JsonRpcResponse::success(response, id)
    }
    
    /// Get orchestrator status and metrics
    pub async fn get_orchestrator_status(&self) -> McpResult<serde_json::Value> {
        let orchestrator = self.orchestrator.read().await;
        Ok(orchestrator.get_status().await)
    }
    
    /// Get detailed performance metrics
    pub async fn get_performance_metrics(&self) -> McpResult<serde_json::Value> {
        let orchestrator = self.orchestrator.read().await;
        let metrics = orchestrator.get_metrics().await;
        Ok(serde_json::to_value(metrics).unwrap_or(serde_json::json!({})))
    }
    
    /// Reset metrics (useful for testing and monitoring)
    pub async fn reset_metrics(&self) -> McpResult<()> {
        let orchestrator = self.orchestrator.read().await;
        orchestrator.reset_metrics().await;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    async fn create_test_tool() -> OrchestratedSearchTool {
        let temp_dir = TempDir::new().unwrap();
        
        // Initialize config first
        if let Err(_) = crate::config::Config::init() {
            // Already initialized, that's ok
        }
        
        let searcher = UnifiedSearcher::new(
            temp_dir.path().to_path_buf(),
            temp_dir.path().join("db")
        ).await.unwrap();
        
        OrchestratedSearchTool::new(searcher, None).await.unwrap()
    }
    
    #[tokio::test]
    async fn test_orchestrated_search_tool() {
        let tool = create_test_tool().await;
        
        let params = serde_json::json!({
            "query": "test function",
            "max_results": 20,
            "include_metrics": true
        });
        
        let result = tool.execute_search(&params, Some(serde_json::Value::Number(1.into()))).await;
        assert!(result.is_ok());
        
        let response = result.unwrap();
        if let Ok(parsed) = response.get_result::<OrchestratedSearchResponse>() {
            assert!(parsed.performance_metrics.is_some());
            assert!(parsed.resource_usage.is_some());
        }
    }
    
    #[tokio::test]
    async fn test_orchestrator_status() {
        let tool = create_test_tool().await;
        
        let status = tool.get_orchestrator_status().await;
        assert!(status.is_ok());
        
        let status_value = status.unwrap();
        assert!(status_value["orchestrator"].is_object());
        assert!(status_value["performance"].is_object());
    }
    
    #[tokio::test]
    async fn test_performance_metrics() {
        let tool = create_test_tool().await;
        
        // Execute a search first to generate metrics
        let params = serde_json::json!({
            "query": "test",
            "max_results": 10
        });
        let _ = tool.execute_search(&params, None).await;
        
        let metrics = tool.get_performance_metrics().await;
        assert!(metrics.is_ok());
        
        let metrics_value = metrics.unwrap();
        assert!(metrics_value["total_searches"].as_u64().unwrap() > 0);
    }
}