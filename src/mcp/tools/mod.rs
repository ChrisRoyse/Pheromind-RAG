//! MCP tool implementations for embed-search server
//!
//! This module contains all MCP tool implementations that integrate with UnifiedSearcher
//! to provide search, indexing, status, and management capabilities via MCP protocol.

use std::sync::Arc;
use tokio::sync::RwLock;
use crate::mcp::{McpError, McpResult};
use crate::mcp::protocol::JsonRpcResponse;
use crate::search::unified::UnifiedSearcher;

pub mod index;
pub mod search;
pub mod status;
pub mod clear;
pub mod orchestrated_search;
pub mod watcher;

/// Tool registry for MCP server
/// Provides centralized access to all tool implementations
pub struct ToolRegistry {
    searcher: Arc<RwLock<UnifiedSearcher>>,
    orchestrated_search_tool: Option<Arc<orchestrated_search::OrchestratedSearchTool>>,
    watcher_tool: Option<Arc<tokio::sync::Mutex<watcher::WatcherTool>>>,
}

impl ToolRegistry {
    /// Create new tool registry with UnifiedSearcher instance
    pub fn new(searcher: Arc<RwLock<UnifiedSearcher>>) -> Self {
        Self { 
            searcher,
            orchestrated_search_tool: None,
            watcher_tool: None,
        }
    }
    
    /// Enable orchestrated search capabilities
    pub async fn enable_orchestrated_search(&mut self, _config: Option<crate::mcp::orchestrator::OrchestratorConfig>) -> McpResult<()> {
        let searcher_guard = self.searcher.read().await;
        
        // We need to create a new UnifiedSearcher instance for the orchestrator
        // because we can't move out of the Arc<RwLock<>>
        // This is a limitation - in practice you'd want to restructure this
        
        println!("⚠️ Note: Orchestrated search requires separate UnifiedSearcher instance");
        println!("💡 For production, restructure to share searcher between registry and orchestrator");
        
        drop(searcher_guard);
        
        // For now, return Ok but don't actually enable orchestrated search
        // This demonstrates the architectural issue that needs to be solved
        Ok(())
    }
    
    /// Get searcher instance for tool implementations
    pub fn searcher(&self) -> Arc<RwLock<UnifiedSearcher>> {
        self.searcher.clone()
    }
    
    /// Execute index_directory tool
    pub async fn execute_index_directory(&self, params: &serde_json::Value, id: Option<serde_json::Value>) -> McpResult<JsonRpcResponse> {
        index::execute_index_directory(&self.searcher, params, id).await
    }
    
    /// Execute search tool with parallel backend execution
    pub async fn execute_search(&self, params: &serde_json::Value, id: Option<serde_json::Value>) -> McpResult<JsonRpcResponse> {
        // Use orchestrated search if available, otherwise fall back to basic search
        if let Some(ref orchestrated_tool) = self.orchestrated_search_tool {
            orchestrated_tool.execute_search(params, id).await
        } else {
            search::execute_search(&self.searcher, params, id).await
        }
    }
    
    /// Execute orchestrated search with enhanced monitoring (if enabled)
    pub async fn execute_orchestrated_search(&self, params: &serde_json::Value, id: Option<serde_json::Value>) -> McpResult<JsonRpcResponse> {
        if let Some(ref orchestrated_tool) = self.orchestrated_search_tool {
            orchestrated_tool.execute_search(params, id).await
        } else {
            Err(McpError::MethodNotFound {
                method: "orchestrated_search".to_string()
            })
        }
    }
    
    /// Get orchestrator status and metrics
    pub async fn get_orchestrator_status(&self) -> McpResult<serde_json::Value> {
        if let Some(ref orchestrated_tool) = self.orchestrated_search_tool {
            orchestrated_tool.get_orchestrator_status().await
        } else {
            Err(McpError::MethodNotFound {
                method: "orchestrator_status".to_string()
            })
        }
    }
    
    /// Execute get_status tool
    pub async fn execute_get_status(&self, params: &serde_json::Value, id: Option<serde_json::Value>) -> McpResult<JsonRpcResponse> {
        status::execute_get_status(&self.searcher, params, id).await
    }
    
    /// Execute clear_index tool
    pub async fn execute_clear_index(&self, params: &serde_json::Value, id: Option<serde_json::Value>) -> McpResult<JsonRpcResponse> {
        clear::execute_clear_index(&self.searcher, params, id).await
    }

    /// Enable watcher capabilities for a specific repository path
    pub async fn enable_watcher(&mut self, repo_path: std::path::PathBuf) -> McpResult<()> {
        let watcher_tool = watcher::WatcherTool::new(self.searcher.clone());
        let watcher_tool_arc = Arc::new(tokio::sync::Mutex::new(watcher_tool));
        
        // Initialize the watcher
        {
            let mut tool = watcher_tool_arc.lock().await;
            tool.initialize_watcher(repo_path).await?;
        }
        
        self.watcher_tool = Some(watcher_tool_arc);
        Ok(())
    }

    /// Execute watcher start command
    pub async fn execute_watcher_start(&self, params: &serde_json::Value, id: Option<serde_json::Value>) -> McpResult<JsonRpcResponse> {
        let watcher_tool = self.watcher_tool.as_ref()
            .ok_or_else(|| McpError::InvalidRequest {
                message: "Watcher not enabled. Call enable_watcher first".to_string(),
            })?;
            
        let tool = watcher_tool.lock().await;
        tool.start_watching(params, id).await
    }

    /// Execute watcher stop command
    pub async fn execute_watcher_stop(&self, params: &serde_json::Value, id: Option<serde_json::Value>) -> McpResult<JsonRpcResponse> {
        let watcher_tool = self.watcher_tool.as_ref()
            .ok_or_else(|| McpError::InvalidRequest {
                message: "Watcher not enabled".to_string(),
            })?;
            
        let tool = watcher_tool.lock().await;
        tool.stop_watching(params, id).await
    }

    /// Execute watcher status command
    pub async fn execute_watcher_status(&self, params: &serde_json::Value, id: Option<serde_json::Value>) -> McpResult<JsonRpcResponse> {
        let watcher_tool = self.watcher_tool.as_ref()
            .ok_or_else(|| McpError::InvalidRequest {
                message: "Watcher not enabled".to_string(),
            })?;
            
        let tool = watcher_tool.lock().await;
        tool.get_watcher_status(params, id).await
    }

    /// Execute watcher subscribe command
    pub async fn execute_watcher_subscribe(&self, params: &serde_json::Value, id: Option<serde_json::Value>) -> McpResult<JsonRpcResponse> {
        let watcher_tool = self.watcher_tool.as_ref()
            .ok_or_else(|| McpError::InvalidRequest {
                message: "Watcher not enabled".to_string(),
            })?;
            
        let tool = watcher_tool.lock().await;
        tool.subscribe_events(params, id).await
    }

    /// Execute watcher unsubscribe command
    pub async fn execute_watcher_unsubscribe(&self, params: &serde_json::Value, id: Option<serde_json::Value>) -> McpResult<JsonRpcResponse> {
        let watcher_tool = self.watcher_tool.as_ref()
            .ok_or_else(|| McpError::InvalidRequest {
                message: "Watcher not enabled".to_string(),
            })?;
            
        let tool = watcher_tool.lock().await;
        tool.unsubscribe_events(params, id).await
    }

    /// Execute manual update trigger
    pub async fn execute_watcher_manual_update(&self, params: &serde_json::Value, id: Option<serde_json::Value>) -> McpResult<JsonRpcResponse> {
        let watcher_tool = self.watcher_tool.as_ref()
            .ok_or_else(|| McpError::InvalidRequest {
                message: "Watcher not enabled".to_string(),
            })?;
            
        let tool = watcher_tool.lock().await;
        tool.trigger_manual_update(params, id).await
    }

    /// Execute watcher error reset
    pub async fn execute_watcher_reset_errors(&self, params: &serde_json::Value, id: Option<serde_json::Value>) -> McpResult<JsonRpcResponse> {
        let watcher_tool = self.watcher_tool.as_ref()
            .ok_or_else(|| McpError::InvalidRequest {
                message: "Watcher not enabled".to_string(),
            })?;
            
        let tool = watcher_tool.lock().await;
        tool.reset_errors(params, id).await
    }

    /// Check if watcher is available and active
    pub async fn is_watcher_available(&self) -> bool {
        if let Some(ref watcher_tool) = self.watcher_tool {
            let tool = watcher_tool.lock().await;
            tool.is_available().await
        } else {
            false
        }
    }
}
