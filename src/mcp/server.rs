use std::sync::Arc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use std::path::PathBuf;
use tokio::sync::RwLock;

use crate::mcp::{
    error::{McpError, McpResult},
    protocol::{JsonRpcRequest, JsonRpcResponse, ProtocolHandler, RpcMethod},
    config::McpConfig,
    types::{
        McpCapabilities, SearchCapabilities, IndexingCapabilities,
        StatsCapabilities, ServerInfo, PerformanceStats,
    },
};
use crate::search::unified::UnifiedSearcher;
use crate::mcp::tools::ToolRegistry;

/// MCP server implementation that integrates with UnifiedSearcher
pub struct McpServer {
    config: McpConfig,
    searcher: Arc<RwLock<UnifiedSearcher>>,
    tool_registry: ToolRegistry,
    protocol_handler: ProtocolHandler,
    start_time: Instant,
    request_count: Arc<tokio::sync::Mutex<u64>>,
    error_count: Arc<tokio::sync::Mutex<u64>>,
    total_search_time: Arc<tokio::sync::Mutex<u64>>,
    total_index_time: Arc<tokio::sync::Mutex<u64>>,
    total_searches: Arc<tokio::sync::Mutex<u64>>,
    total_indexes: Arc<tokio::sync::Mutex<u64>>,
}

impl McpServer {
    /// Create a new MCP server with the provided searcher
    pub async fn new(
        searcher: UnifiedSearcher, 
        config: McpConfig
    ) -> McpResult<Self> {
        // Validate the configuration first
        config.validate()
            .map_err(|e| McpError::ConfigError {
                message: format!("MCP configuration validation failed: {}", e),
            })?;
        let searcher_arc = Arc::new(RwLock::new(searcher));
        let tool_registry = ToolRegistry::new(searcher_arc.clone());
        
        Ok(Self {
            config,
            searcher: searcher_arc,
            tool_registry,
            protocol_handler: ProtocolHandler::new(),
            start_time: Instant::now(),
            request_count: Arc::new(tokio::sync::Mutex::new(0)),
            error_count: Arc::new(tokio::sync::Mutex::new(0)),
            total_search_time: Arc::new(tokio::sync::Mutex::new(0)),
            total_index_time: Arc::new(tokio::sync::Mutex::new(0)),
            total_searches: Arc::new(tokio::sync::Mutex::new(0)),
            total_indexes: Arc::new(tokio::sync::Mutex::new(0)),
        })
    }

    /// Create MCP server with configuration loaded from files
    pub async fn with_project_path(project_path: PathBuf) -> McpResult<Self> {
        // Load MCP configuration first
        let mcp_config = McpConfig::from_base_config()
            .map_err(|e| McpError::ConfigError {
                message: format!("Failed to load MCP configuration: {}", e),
            })?;
        
        let db_path = project_path.join(".embed-search");
        let searcher = UnifiedSearcher::new(project_path, db_path)
            .await
            .map_err(|e| McpError::InternalError {
                message: format!("Failed to create UnifiedSearcher: {}", e),
            })?;

        Self::new(searcher, mcp_config).await
    }
    
    /// Create MCP server with explicit MCP configuration file
    pub async fn with_config_file<P: AsRef<std::path::Path>>(
        project_path: PathBuf,
        mcp_config_path: P
    ) -> McpResult<Self> {
        let mcp_config = McpConfig::load_from_file(mcp_config_path)
            .map_err(|e| McpError::ConfigError {
                message: format!("Failed to load MCP configuration from file: {}", e),
            })?;
        
        let db_path = project_path.join(".embed-search");
        let searcher = UnifiedSearcher::new(project_path, db_path)
            .await
            .map_err(|e| McpError::InternalError {
                message: format!("Failed to create UnifiedSearcher: {}", e),
            })?;

        Self::new(searcher, mcp_config).await
    }

    /// Process incoming JSON-RPC request
    pub async fn handle_request(&mut self, json: &str) -> String {
        *self.request_count.lock().await += 1;

        let request = match self.protocol_handler.parse_request(json) {
            Ok(req) => req,
            Err(e) => {
                *self.error_count.lock().await += 1;
                return self.protocol_handler
                    .serialize_response(&JsonRpcResponse::error(e, None))
                    .unwrap_or_else(|_| r#"{"jsonrpc":"2.0","error":{"code":-32603,"message":"Internal error"},"id":null}"#.to_string());
            }
        };

        let response = self.process_request(request).await;
        
        match self.protocol_handler.serialize_response(&response) {
            Ok(json) => json,
            Err(e) => {
                *self.error_count.lock().await += 1;
                JsonRpcResponse::error(e, None).to_string()
            }
        }
    }

    /// Process batch requests
    pub async fn handle_batch_request(&mut self, json: &str) -> String {
        let requests = match self.protocol_handler.parse_batch_request(json) {
            Ok(reqs) => reqs,
            Err(e) => {
                return self.protocol_handler
                    .serialize_response(&JsonRpcResponse::error(e, None))
                    .unwrap_or_else(|_| "{}".to_string());
            }
        };

        let mut responses = Vec::with_capacity(requests.len());
        
        for request in requests {
            *self.request_count.lock().await += 1;
            let response = self.process_request(request).await;
            responses.push(response);
        }

        self.protocol_handler
            .serialize_batch_response(&responses)
            .unwrap_or_else(|_| "[]".to_string())
    }

    async fn process_request(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        let method = match request.get_method() {
            Ok(m) => m,
            Err(e) => {
                *self.error_count.lock().await += 1;
                return JsonRpcResponse::error(e, request.id);
            }
        };

        let result = match method {
            RpcMethod::Initialize => self.handle_initialize(&request).await,
            RpcMethod::Search => self.handle_search(&request).await,
            RpcMethod::Index => self.handle_index(&request).await,
            RpcMethod::Stats => self.handle_stats(&request).await,
            RpcMethod::Clear => self.handle_clear(&request).await,
            RpcMethod::Capabilities => self.handle_capabilities(&request).await,
            RpcMethod::Ping => self.handle_ping(&request).await,
            RpcMethod::Shutdown => self.handle_shutdown(&request).await,
            RpcMethod::WatcherStart => self.handle_watcher_start(&request).await,
            RpcMethod::WatcherStop => self.handle_watcher_stop(&request).await,
            RpcMethod::WatcherStatus => self.handle_watcher_status(&request).await,
            RpcMethod::WatcherSubscribe => self.handle_watcher_subscribe(&request).await,
            RpcMethod::WatcherUnsubscribe => self.handle_watcher_unsubscribe(&request).await,
            RpcMethod::WatcherManualUpdate => self.handle_watcher_manual_update(&request).await,
            RpcMethod::WatcherResetErrors => self.handle_watcher_reset_errors(&request).await,
        };

        match result {
            Ok(response) => response,
            Err(e) => {
                *self.error_count.lock().await += 1;
                JsonRpcResponse::error(e, request.id)
            }
        }
    }

    async fn handle_initialize(&self, request: &JsonRpcRequest) -> McpResult<JsonRpcResponse> {
        let capabilities = self.get_capabilities().await;
        JsonRpcResponse::success(capabilities, request.id.clone())
    }

    async fn handle_search(&self, request: &JsonRpcRequest) -> McpResult<JsonRpcResponse> {
        let params = request.params.as_ref().unwrap_or(&serde_json::Value::Null);
        
        // Update metrics
        let start_time = Instant::now();
        let result = self.tool_registry.execute_search(params, request.id.clone()).await;
        let search_time = start_time.elapsed();
        
        if result.is_ok() {
            *self.total_search_time.lock().await += search_time.as_millis() as u64;
            *self.total_searches.lock().await += 1;
        }
        
        result
    }

    async fn handle_index(&self, request: &JsonRpcRequest) -> McpResult<JsonRpcResponse> {
        let params = request.params.as_ref().unwrap_or(&serde_json::Value::Null);
        
        // Update metrics
        let start_time = Instant::now();
        let result = self.tool_registry.execute_index_directory(params, request.id.clone()).await;
        let index_time = start_time.elapsed();
        
        if result.is_ok() {
            *self.total_index_time.lock().await += index_time.as_millis() as u64;
            *self.total_indexes.lock().await += 1;
        }
        
        result
    }

    async fn handle_stats(&self, request: &JsonRpcRequest) -> McpResult<JsonRpcResponse> {
        let params = request.params.as_ref().unwrap_or(&serde_json::Value::Null);
        self.tool_registry.execute_get_status(params, request.id.clone()).await
    }

    async fn handle_clear(&self, request: &JsonRpcRequest) -> McpResult<JsonRpcResponse> {
        let params = request.params.as_ref().unwrap_or(&serde_json::Value::Null);
        self.tool_registry.execute_clear_index(params, request.id.clone()).await
    }

    async fn handle_capabilities(&self, request: &JsonRpcRequest) -> McpResult<JsonRpcResponse> {
        let capabilities = self.get_capabilities().await;
        JsonRpcResponse::success(capabilities, request.id.clone())
    }

    async fn handle_ping(&self, request: &JsonRpcRequest) -> McpResult<JsonRpcResponse> {
        let response = serde_json::json!({
            "status": "ok",
            "timestamp": SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            "uptime_seconds": self.start_time.elapsed().as_secs()
        });

        JsonRpcResponse::success(response, request.id.clone())
    }

    async fn handle_shutdown(&self, request: &JsonRpcRequest) -> McpResult<JsonRpcResponse> {
        let response = serde_json::json!({
            "status": "shutting_down",
            "message": "Server shutdown initiated"
        });

        JsonRpcResponse::success(response, request.id.clone())
    }

    async fn handle_watcher_start(&self, request: &JsonRpcRequest) -> McpResult<JsonRpcResponse> {
        let params = request.params.as_ref().unwrap_or(&serde_json::Value::Null);
        self.tool_registry.execute_watcher_start(params, request.id.clone()).await
    }

    async fn handle_watcher_stop(&self, request: &JsonRpcRequest) -> McpResult<JsonRpcResponse> {
        let params = request.params.as_ref().unwrap_or(&serde_json::Value::Null);
        self.tool_registry.execute_watcher_stop(params, request.id.clone()).await
    }

    async fn handle_watcher_status(&self, request: &JsonRpcRequest) -> McpResult<JsonRpcResponse> {
        let params = request.params.as_ref().unwrap_or(&serde_json::Value::Null);
        self.tool_registry.execute_watcher_status(params, request.id.clone()).await
    }

    async fn handle_watcher_subscribe(&self, request: &JsonRpcRequest) -> McpResult<JsonRpcResponse> {
        let params = request.params.as_ref().unwrap_or(&serde_json::Value::Null);
        self.tool_registry.execute_watcher_subscribe(params, request.id.clone()).await
    }

    async fn handle_watcher_unsubscribe(&self, request: &JsonRpcRequest) -> McpResult<JsonRpcResponse> {
        let params = request.params.as_ref().unwrap_or(&serde_json::Value::Null);
        self.tool_registry.execute_watcher_unsubscribe(params, request.id.clone()).await
    }

    async fn handle_watcher_manual_update(&self, request: &JsonRpcRequest) -> McpResult<JsonRpcResponse> {
        let params = request.params.as_ref().unwrap_or(&serde_json::Value::Null);
        self.tool_registry.execute_watcher_manual_update(params, request.id.clone()).await
    }

    async fn handle_watcher_reset_errors(&self, request: &JsonRpcRequest) -> McpResult<JsonRpcResponse> {
        let params = request.params.as_ref().unwrap_or(&serde_json::Value::Null);
        self.tool_registry.execute_watcher_reset_errors(params, request.id.clone()).await
    }

    async fn get_capabilities(&self) -> McpCapabilities {
        // Detect available features based on compile-time configuration
        let features = vec![
            #[cfg(feature = "ml")]
            "semantic_search".to_string(),
            #[cfg(feature = "tantivy")]
            "exact_search".to_string(),
            #[cfg(feature = "tree-sitter")]
            "symbol_search".to_string(),
            "statistical_search".to_string(), // BM25 is always available
        ]
        .into_iter()
        .collect();

        let backends = vec![
            #[cfg(feature = "tantivy")]
            "tantivy".to_string(),
            "bm25".to_string(),
            #[cfg(feature = "vectordb")]
            "lancedb".to_string(),
        ]
        .into_iter()
        .collect();

        McpCapabilities {
            search: SearchCapabilities {
                semantic_search: cfg!(feature = "ml"),
                exact_search: cfg!(feature = "tantivy"),
                symbol_search: cfg!(feature = "tree-sitter"),
                statistical_search: true, // BM25 always available
                fuzzy_search: cfg!(feature = "tantivy"),
                max_results: 1000,
                supported_file_types: vec![
                    "rs".to_string(), "py".to_string(), "js".to_string(),
                    "ts".to_string(), "go".to_string(), "java".to_string(),
                    "cpp".to_string(), "c".to_string(), "md".to_string(),
                ],
            },
            indexing: IndexingCapabilities {
                batch_indexing: true,
                incremental_updates: true,
                file_watching: true, // Now available via MCP watcher integration
                symbol_extraction: cfg!(feature = "tree-sitter"),
                max_file_size_mb: 10, // Reasonable default
            },
            stats: StatsCapabilities {
                index_stats: true,
                search_metrics: true,
                performance_metrics: true,
                cache_stats: true,
            },
            server_info: ServerInfo {
                name: self.config.server_name.clone(),
                version: self.config.server_version.clone(),
                features,
                supported_backends: backends,
            },
        }
    }

    /// Get server configuration
    pub fn config(&self) -> &McpConfig {
        &self.config
    }

    /// Get current performance statistics
    pub async fn performance_stats(&self) -> PerformanceStats {
        let total_searches = *self.total_searches.lock().await;
        let total_indexes = *self.total_indexes.lock().await;
        let total_search_time = *self.total_search_time.lock().await;
        let total_index_time = *self.total_index_time.lock().await;

        PerformanceStats {
            avg_search_time_ms: if total_searches > 0 {
                total_search_time as f64 / total_searches as f64
            } else {
                0.0
            },
            avg_index_time_ms: if total_indexes > 0 {
                total_index_time as f64 / total_indexes as f64
            } else {
                0.0
            },
            total_searches,
            total_indexes,
            uptime_seconds: self.start_time.elapsed().as_secs(),
        }
    }

    /// Enable watcher functionality for a specific repository path
    pub async fn enable_watcher(&mut self, repo_path: PathBuf) -> McpResult<()> {
        self.tool_registry.enable_watcher(repo_path).await
    }

    /// Check if watcher is available and active
    pub async fn is_watcher_available(&self) -> bool {
        self.tool_registry.is_watcher_available().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use serde_json::Value as JsonValue;

    #[tokio::test]
    async fn test_mcp_server_creation() {
        let temp_dir = TempDir::new().unwrap();
        
        // Initialize config first (required for UnifiedSearcher)
        std::env::set_var("EMBED_LOG_LEVEL", "info");
        if let Err(_) = crate::config::Config::init() {
            // Config already initialized, that's ok
        }
        
        let result = McpServer::with_project_path(temp_dir.path().to_path_buf()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_capabilities_request() {
        let temp_dir = TempDir::new().unwrap();
        
        // Initialize config first
        if let Err(_) = crate::config::Config::init() {
            // Config already initialized, that's ok
        }
        
        let mut server = McpServer::with_project_path(temp_dir.path().to_path_buf()).await.unwrap();
        
        let request_json = r#"{"jsonrpc":"2.0","method":"capabilities","id":1}"#;
        let response_json = server.handle_request(request_json).await;
        
        // Response should be valid JSON and contain capabilities
        let response: JsonValue = serde_json::from_str(&response_json).unwrap();
        assert_eq!(response["jsonrpc"], "2.0");
        assert!(response["result"].is_object());
    }

    #[tokio::test]
    async fn test_ping_request() {
        let temp_dir = TempDir::new().unwrap();
        
        // Initialize config first
        if let Err(_) = crate::config::Config::init() {
            // Config already initialized, that's ok
        }
        
        let mut server = McpServer::with_project_path(temp_dir.path().to_path_buf()).await.unwrap();
        
        let request_json = r#"{"jsonrpc":"2.0","method":"ping","id":1}"#;
        let response_json = server.handle_request(request_json).await;
        
        let response: JsonValue = serde_json::from_str(&response_json).unwrap();
        assert_eq!(response["jsonrpc"], "2.0");
        assert_eq!(response["result"]["status"], "ok");
    }
}