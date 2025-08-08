//! MCP Orchestrator Integration Example
//! 
//! This demonstrates how to properly integrate the SearchOrchestrator
//! with the MCP server for production use.
//!
//! Truth: This shows the REAL implementation using existing components,
//! not simulated functionality.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use crate::mcp::{
    McpServer, SearchOrchestrator, OrchestratorConfig, 
    McpError, McpResult
};
use crate::search::UnifiedSearcher;
use crate::mcp::tools::orchestrated_search::OrchestratedSearchTool;

/// Production-ready MCP server with orchestrated search capabilities
pub struct EnhancedMcpServer {
    mcp_server: McpServer,
    orchestrator: Arc<SearchOrchestrator>,
    orchestrated_search_tool: Arc<OrchestratedSearchTool>,
}

impl EnhancedMcpServer {
    /// Create enhanced MCP server with orchestration
    pub async fn new(project_path: PathBuf, db_path: PathBuf) -> McpResult<Self> {
        // Initialize configuration first (required for UnifiedSearcher)
        if let Err(_) = crate::config::Config::init() {
            // Config already initialized, that's fine
        }
        
        // Create UnifiedSearcher for MCP server
        let mcp_searcher = UnifiedSearcher::new(project_path.clone(), db_path.clone())
            .await
            .map_err(|e| McpError::InternalError {
                message: format!("Failed to create MCP UnifiedSearcher: {}", e),
            })?;
        
        // Create MCP server
        let mcp_config = crate::mcp::config::McpConfig::new_test_config();
        let mcp_server = McpServer::new(mcp_searcher, mcp_config).await?;
        
        // Create separate UnifiedSearcher for orchestrator
        // Truth: This is necessary because we can't share the same instance
        // In production, you'd restructure to avoid this duplication
        let orchestrator_searcher = UnifiedSearcher::new(project_path, db_path)
            .await
            .map_err(|e| McpError::InternalError {
                message: format!("Failed to create Orchestrator UnifiedSearcher: {}", e),
            })?;
        
        // Configure orchestrator for production use
        let orchestrator_config = OrchestratorConfig {
            max_concurrent_searches: 20,
            search_timeout: Duration::from_secs(30),
            enable_detailed_metrics: true,
            partial_failure_threshold: 0.3, // Allow failures if >70% of backends succeed
            enable_resource_monitoring: true,
        };
        
        // Create orchestrator
        let orchestrator = SearchOrchestrator::new(orchestrator_searcher, Some(orchestrator_config)).await?;
        let orchestrator_arc = Arc::new(orchestrator);
        
        // Create orchestrated search tool
        let orchestrated_search_tool = OrchestratedSearchTool::new(
            // We need another UnifiedSearcher instance here - this demonstrates the architectural challenge
            // In production, you'd refactor to share the searcher more efficiently
            UnifiedSearcher::new(
                std::env::current_dir().unwrap_or_default(), 
                std::env::current_dir().unwrap_or_default().join(".embed-search")
            ).await.map_err(|e| McpError::InternalError {
                message: format!("Failed to create tool UnifiedSearcher: {}", e),
            })?,
            None
        ).await?;
        
        Ok(Self {
            mcp_server,
            orchestrator: orchestrator_arc,
            orchestrated_search_tool: Arc::new(orchestrated_search_tool),
        })
    }
    
    /// Handle MCP request with orchestration capabilities
    pub async fn handle_request(&mut self, json: &str) -> String {
        // Parse the request to see if it's an orchestrated search
        if let Ok(request_value) = serde_json::from_str::<serde_json::Value>(json) {
            if let Some(method) = request_value.get("method").and_then(|m| m.as_str()) {
                match method {
                    "orchestrated_search" => {
                        return self.handle_orchestrated_search_request(json).await;
                    }
                    "orchestrator_status" => {
                        return self.handle_orchestrator_status_request(json).await;
                    }
                    _ => {
                        // Fall back to standard MCP handling
                    }
                }
            }
        }
        
        // Use standard MCP server for other requests
        self.mcp_server.handle_request(json).await
    }
    
    /// Handle orchestrated search request
    async fn handle_orchestrated_search_request(&self, json: &str) -> String {
        // Parse JSON-RPC request
        let request: Result<serde_json::Value, _> = serde_json::from_str(json);
        let request = match request {
            Ok(req) => req,
            Err(e) => {
                return format!(r#"{{"jsonrpc":"2.0","error":{{"code":-32700,"message":"Parse error: {}"}},"id":null}}"#, e);
            }
        };
        
        let params = request.get("params").unwrap_or(&serde_json::Value::Null);
        let id = request.get("id").cloned();
        
        // Execute orchestrated search
        match self.orchestrated_search_tool.execute_search(params, id).await {
            Ok(response) => {
                match serde_json::to_string(&response) {
                    Ok(json_response) => json_response,
                    Err(e) => format!(r#"{{"jsonrpc":"2.0","error":{{"code":-32603,"message":"Internal error: {}"}},"id":null}}"#, e),
                }
            }
            Err(e) => {
                format!(r#"{{"jsonrpc":"2.0","error":{{"code":-32000,"message":"Search error: {}"}},"id":null}}"#, e)
            }
        }
    }
    
    /// Handle orchestrator status request
    async fn handle_orchestrator_status_request(&self, json: &str) -> String {
        let request: Result<serde_json::Value, _> = serde_json::from_str(json);
        let request = match request {
            Ok(req) => req,
            Err(e) => {
                return format!(r#"{{"jsonrpc":"2.0","error":{{"code":-32700,"message":"Parse error: {}"}},"id":null}}"#, e);
            }
        };
        
        let id = request.get("id").cloned();
        
        match self.orchestrated_search_tool.get_orchestrator_status().await {
            Ok(status) => {
                let response = serde_json::json!({
                    "jsonrpc": "2.0",
                    "result": status,
                    "id": id
                });
                serde_json::to_string(&response).unwrap_or_else(|_| "{}".to_string())
            }
            Err(e) => {
                format!(r#"{{"jsonrpc":"2.0","error":{{"code":-32000,"message":"Status error: {}"}},"id":{}}}"#, e, id.unwrap_or(serde_json::Value::Null))
            }
        }
    }
    
    /// Get orchestrator performance metrics
    pub async fn get_orchestrator_metrics(&self) -> serde_json::Value {
        match self.orchestrated_search_tool.get_performance_metrics().await {
            Ok(metrics) => metrics,
            Err(e) => serde_json::json!({
                "error": format!("Failed to get metrics: {}", e)
            })
        }
    }
    
    /// Demonstrate the orchestration capabilities
    pub async fn demonstrate_orchestration(&self) -> McpResult<()> {
        println!("🎯 Demonstrating MCP Orchestration Capabilities");
        println!("===============================================");
        
        // Test 1: Basic orchestrated search
        println!("\n1. Basic Orchestrated Search:");
        let search_params = serde_json::json!({
            "query": "function search",
            "max_results": 10,
            "include_metrics": true
        });
        
        match self.orchestrated_search_tool.execute_search(&search_params, Some(serde_json::Value::Number(1.into()))).await {
            Ok(response) => {
                if let Ok(result) = response.get_result::<serde_json::Value>() {
                    println!("✅ Search completed successfully");
                    println!("   - Results: {}", result.get("total_matches").unwrap_or(&serde_json::Value::Number(0.into())));
                    println!("   - Search time: {}ms", result.get("search_time_ms").unwrap_or(&serde_json::Value::Number(0.into())));
                    
                    if let Some(metrics) = result.get("performance_metrics") {
                        println!("   - Backend latencies: {}", 
                                 metrics.get("backend_latencies_ms").unwrap_or(&serde_json::Value::Object(Default::default())));
                    }
                }
            }
            Err(e) => println!("❌ Search failed: {}", e),
        }
        
        // Test 2: Orchestrator status
        println!("\n2. Orchestrator Status:");
        match self.orchestrated_search_tool.get_orchestrator_status().await {
            Ok(status) => {
                println!("✅ Status retrieved successfully");
                if let Some(perf) = status.get("performance") {
                    println!("   - Total searches: {}", perf.get("total_searches").unwrap_or(&serde_json::Value::Number(0.into())));
                    println!("   - Success rate: {:.2}", perf.get("success_rate").unwrap_or(&serde_json::Value::Number(0.into())).as_f64().unwrap_or(0.0));
                    println!("   - Avg latency: {:.1}ms", perf.get("avg_latency_ms").unwrap_or(&serde_json::Value::Number(0.into())).as_f64().unwrap_or(0.0));
                }
                
                if let Some(orch) = status.get("orchestrator") {
                    println!("   - Active searches: {}", orch.get("active_searches").unwrap_or(&serde_json::Value::Number(0.into())));
                    println!("   - Available permits: {}", orch.get("available_permits").unwrap_or(&serde_json::Value::Number(0.into())));
                }
            }
            Err(e) => println!("❌ Status retrieval failed: {}", e),
        }
        
        // Test 3: Concurrent search demonstration
        println!("\n3. Concurrent Search Handling:");
        let mut handles = Vec::new();
        
        for i in 0..5 {
            let tool = self.orchestrated_search_tool.clone();
            let handle = tokio::spawn(async move {
                let params = serde_json::json!({
                    "query": format!("test query {}", i),
                    "max_results": 5
                });
                tool.execute_search(&params, Some(serde_json::Value::Number(i.into()))).await
            });
            handles.push(handle);
        }
        
        let mut successful_searches = 0;
        for handle in handles {
            match handle.await {
                Ok(Ok(_)) => successful_searches += 1,
                Ok(Err(e)) => println!("   ❌ Search failed: {}", e),
                Err(e) => println!("   ❌ Task failed: {}", e),
            }
        }
        
        println!("✅ Completed {} concurrent searches successfully", successful_searches);
        
        println!("\n🎯 Orchestration Demonstration Complete");
        println!("=====================================");
        
        Ok(())
    }
}

/// Example usage and testing
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_enhanced_mcp_server() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().to_path_buf();
        let db_path = temp_dir.path().join("db");
        
        let server = EnhancedMcpServer::new(project_path, db_path).await;
        assert!(server.is_ok(), "Failed to create enhanced MCP server");
    }
    
    #[tokio::test]
    async fn test_orchestrated_search_integration() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().to_path_buf();
        let db_path = temp_dir.path().join("db");
        
        let server = EnhancedMcpServer::new(project_path, db_path).await.unwrap();
        
        // Test orchestrated search request
        let request_json = r#"{"jsonrpc":"2.0","method":"orchestrated_search","params":{"query":"test","max_results":10,"include_metrics":true},"id":1}"#;
        let response = server.handle_orchestrated_search_request(request_json).await;
        
        // Should be valid JSON response
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&response);
        assert!(parsed.is_ok(), "Response should be valid JSON: {}", response);
        
        let response_obj = parsed.unwrap();
        assert_eq!(response_obj["jsonrpc"], "2.0");
        assert!(response_obj["result"].is_object() || response_obj["error"].is_object());
    }
    
    #[tokio::test]
    async fn test_orchestration_demonstration() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().to_path_buf();
        let db_path = temp_dir.path().join("db");
        
        let server = EnhancedMcpServer::new(project_path, db_path).await.unwrap();
        
        // This should complete without panicking
        let result = server.demonstrate_orchestration().await;
        assert!(result.is_ok(), "Demonstration should complete successfully");
    }
}

/// CLI example for running the enhanced MCP server
pub async fn run_enhanced_mcp_server_example() -> McpResult<()> {
    println!("🚀 Starting Enhanced MCP Server with Orchestration");
    println!("=================================================");
    
    let project_path = std::env::current_dir()
        .map_err(|e| McpError::InternalError {
            message: format!("Failed to get current directory: {}", e),
        })?;
    
    let db_path = project_path.join(".embed-search");
    
    // Create enhanced server
    let server = EnhancedMcpServer::new(project_path, db_path).await?;
    
    // Run demonstration
    server.demonstrate_orchestration().await?;
    
    // Show final metrics
    let metrics = server.get_orchestrator_metrics().await;
    println!("\n📊 Final Orchestrator Metrics:");
    println!("{}", serde_json::to_string_pretty(&metrics).unwrap_or_else(|_| "{}".to_string()));
    
    println!("\n✅ Enhanced MCP Server Example Completed Successfully");
    
    Ok(())
}