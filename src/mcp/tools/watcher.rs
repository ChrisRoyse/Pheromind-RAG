use std::sync::Arc;
use tokio::sync::RwLock;
use serde_json::{Value, json};

use crate::mcp::{
    error::{McpError, McpResult},
    protocol::JsonRpcResponse,
    watcher::{McpWatcher, EventFilter},
};
use crate::search::unified::UnifiedSearcher;

/// MCP tool handler for watcher-related operations
pub struct WatcherTool {
    watcher: Option<Arc<McpWatcher>>,
    searcher: Arc<RwLock<UnifiedSearcher>>,
}

impl WatcherTool {
    pub fn new(searcher: Arc<RwLock<UnifiedSearcher>>) -> Self {
        Self {
            watcher: None,
            searcher,
        }
    }

    /// Initialize watcher for a specific path
    pub async fn initialize_watcher(&mut self, repo_path: std::path::PathBuf) -> McpResult<()> {
        if self.watcher.is_some() {
            return Err(McpError::InvalidRequest {
                message: "Watcher is already initialized".to_string(),
            });
        }

        let watcher = McpWatcher::new(repo_path, self.searcher.clone()).await?;
        self.watcher = Some(Arc::new(watcher));
        Ok(())
    }

    /// Start file watching
    pub async fn start_watching(&self, params: &Value, request_id: Option<Value>) -> McpResult<JsonRpcResponse> {
        let watcher = self.watcher.as_ref()
            .ok_or_else(|| McpError::InvalidRequest {
                message: "Watcher not initialized. Call initialize_watcher first".to_string(),
            })?;

        watcher.start().await?;

        let result = json!({
            "status": "started",
            "message": "File watching started successfully",
            "watcher_active": true
        });

        JsonRpcResponse::success(result, request_id.clone())
    }

    /// Stop file watching
    pub async fn stop_watching(&self, params: &Value, request_id: Option<Value>) -> McpResult<JsonRpcResponse> {
        let watcher = self.watcher.as_ref()
            .ok_or_else(|| McpError::InvalidRequest {
                message: "Watcher not initialized".to_string(),
            })?;

        watcher.stop().await?;

        let result = json!({
            "status": "stopped", 
            "message": "File watching stopped successfully",
            "watcher_active": false
        });

        JsonRpcResponse::success(result, request_id.clone())
    }

    /// Get watcher status and statistics
    pub async fn get_watcher_status(&self, params: &Value, request_id: Option<Value>) -> McpResult<JsonRpcResponse> {
        let watcher = self.watcher.as_ref()
            .ok_or_else(|| McpError::InvalidRequest {
                message: "Watcher not initialized".to_string(),
            })?;

        let stats = watcher.get_stats().await?;
        
        let result = json!({
            "watcher_stats": {
                "is_active": stats.is_active,
                "watched_path": stats.watched_path,
                "events_processed": stats.events_processed,
                "active_subscribers": stats.active_subscribers,
                "last_event_time": stats.last_event_time,
                "error_count": stats.error_count
            },
            "status": "ok"
        });

        JsonRpcResponse::success(result, request_id.clone())
    }

    /// Subscribe client to watcher events
    pub async fn subscribe_events(&self, params: &Value, request_id: Option<Value>) -> McpResult<JsonRpcResponse> {
        let watcher = self.watcher.as_ref()
            .ok_or_else(|| McpError::InvalidRequest {
                message: "Watcher not initialized".to_string(),
            })?;

        // Extract client ID from params
        let client_id = params.get("client_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidRequest {
                message: "client_id parameter is required".to_string(),
            })?
            .to_string();

        // Extract optional event filter
        let event_filter = if let Some(filter_value) = params.get("event_filter") {
            let filter: EventFilter = serde_json::from_value(filter_value.clone())
                .map_err(|e| McpError::InvalidRequest {
                    message: format!("Invalid event_filter format: {}", e),
                })?;
            Some(filter)
        } else {
            None
        };

        let _receiver = watcher.subscribe_client(client_id.clone(), event_filter).await?;

        let result = json!({
            "status": "subscribed",
            "client_id": client_id,
            "message": "Successfully subscribed to watcher events",
            "subscription_active": true
        });

        JsonRpcResponse::success(result, request_id.clone())
    }

    /// Unsubscribe client from watcher events
    pub async fn unsubscribe_events(&self, params: &Value, request_id: Option<Value>) -> McpResult<JsonRpcResponse> {
        let watcher = self.watcher.as_ref()
            .ok_or_else(|| McpError::InvalidRequest {
                message: "Watcher not initialized".to_string(),
            })?;

        let client_id = params.get("client_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidRequest {
                message: "client_id parameter is required".to_string(),
            })?;

        watcher.unsubscribe_client(client_id).await?;

        let result = json!({
            "status": "unsubscribed",
            "client_id": client_id,
            "message": "Successfully unsubscribed from watcher events",
            "subscription_active": false
        });

        JsonRpcResponse::success(result, request_id.clone())
    }

    /// Trigger manual index update
    pub async fn trigger_manual_update(&self, params: &Value, request_id: Option<Value>) -> McpResult<JsonRpcResponse> {
        let watcher = self.watcher.as_ref()
            .ok_or_else(|| McpError::InvalidRequest {
                message: "Watcher not initialized".to_string(),
            })?;

        watcher.trigger_manual_update().await?;

        let result = json!({
            "status": "triggered",
            "message": "Manual index update triggered successfully",
            "update_initiated": true
        });

        JsonRpcResponse::success(result, request_id.clone())
    }

    /// Reset watcher error count
    pub async fn reset_errors(&self, params: &Value, request_id: Option<Value>) -> McpResult<JsonRpcResponse> {
        let watcher = self.watcher.as_ref()
            .ok_or_else(|| McpError::InvalidRequest {
                message: "Watcher not initialized".to_string(),
            })?;

        watcher.reset_error_count().await?;

        let result = json!({
            "status": "reset",
            "message": "Watcher error count reset successfully",
            "error_count": 0
        });

        JsonRpcResponse::success(result, request_id.clone())
    }

    /// Check if watcher is initialized and active
    pub async fn is_available(&self) -> bool {
        match &self.watcher {
            Some(watcher) => watcher.is_active().await,
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use crate::search::unified::UnifiedSearcher;

    #[tokio::test]
    async fn test_watcher_tool_lifecycle() {
        let temp_dir = TempDir::new().unwrap();
        
        // Initialize config first
        if let Err(_) = crate::config::Config::init() {
            // Already initialized
        }

        let searcher = Arc::new(RwLock::new(
            UnifiedSearcher::new(temp_dir.path().to_path_buf(), temp_dir.path().join(".embed"))
                .await
                .unwrap()
        ));

        let mut tool = WatcherTool::new(searcher);
        
        // Tool should not be available initially
        assert!(!tool.is_available().await);
        
        // Initialize watcher
        tool.initialize_watcher(temp_dir.path().to_path_buf()).await.unwrap();
        
        // Start watching
        let start_result = tool.start_watching(&json!({}), Some(json!(1))).await.unwrap();
        assert!(start_result.result.is_some());
        
        // Check status
        let status_result = tool.get_watcher_status(&json!({}), Some(json!(2))).await.unwrap();
        assert!(status_result.result.is_some());
        
        // Stop watching  
        let stop_result = tool.stop_watching(&json!({}), Some(json!(3))).await.unwrap();
        assert!(stop_result.result.is_some());
    }

    #[tokio::test]
    async fn test_client_subscription() {
        let temp_dir = TempDir::new().unwrap();
        
        if let Err(_) = crate::config::Config::init() {
            // Already initialized
        }

        let searcher = Arc::new(RwLock::new(
            UnifiedSearcher::new(temp_dir.path().to_path_buf(), temp_dir.path().join(".embed"))
                .await
                .unwrap()
        ));

        let mut tool = WatcherTool::new(searcher);
        tool.initialize_watcher(temp_dir.path().to_path_buf()).await.unwrap();
        
        let params = json!({
            "client_id": "test_client",
            "event_filter": {
                "file_patterns": ["*.rs"],
                "event_types": ["file_modified"]
            }
        });
        
        let result = tool.subscribe_events(&params, Some(json!(1))).await.unwrap();
        assert!(result.result.is_some());
        
        // Unsubscribe
        let unsubscribe_params = json!({
            "client_id": "test_client"
        });
        
        let result = tool.unsubscribe_events(&unsubscribe_params, Some(json!(2))).await.unwrap();
        assert!(result.result.is_some());
    }

    #[tokio::test]
    async fn test_manual_update() {
        let temp_dir = TempDir::new().unwrap();
        
        if let Err(_) = crate::config::Config::init() {
            // Already initialized
        }

        let searcher = Arc::new(RwLock::new(
            UnifiedSearcher::new(temp_dir.path().to_path_buf(), temp_dir.path().join(".embed"))
                .await
                .unwrap()
        ));

        let mut tool = WatcherTool::new(searcher);
        tool.initialize_watcher(temp_dir.path().to_path_buf()).await.unwrap();
        tool.start_watching(&json!({}), None).await.unwrap();
        
        let result = tool.trigger_manual_update(&json!({}), Some(json!(1))).await;
        assert!(result.is_ok());
    }
}