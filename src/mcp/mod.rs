// MCP (Model Context Protocol) server implementation
// This module implements the core MCP server protocol for the embed-search system

pub mod error;
pub mod protocol;
pub mod types;
pub mod config;
pub mod server;
pub mod transport;
pub mod tools;
pub mod orchestrator;
pub mod integration_example;
pub mod watcher;

// Re-export commonly used types
pub use error::{McpError, McpResult};
pub use protocol::{JsonRpcRequest, JsonRpcResponse, JsonRpcError, RpcMethod};
pub use types::{
    SearchRequest, SearchResponse, IndexRequest, IndexResponse, 
    StatsRequest, StatsResponse, ClearRequest, ClearResponse,
    McpCapabilities, SearchMatch
};
pub use config::{
    McpConfig, McpTransportConfig, McpToolsConfig, McpPerformanceConfig,
    McpEmbeddingConfig, McpSecurityConfig
};
pub use server::McpServer;
pub use transport::{
    Transport, TransportMessage, TransportResponse, TransportInfo,
    TransportFactory, TransportEventLoop, MessageHandler,
    StdioTransport, StdioMcpHandler, StdioMcpServerBuilder
};
pub use orchestrator::{
    SearchOrchestrator, OrchestratedSearchResult, SearchMetrics, 
    SearchExecutionMetrics, BackendStatus, OrchestratorConfig
};
pub use integration_example::{EnhancedMcpServer, run_enhanced_mcp_server_example};
pub use watcher::{
    McpWatcher, McpWatcherEvent, McpEventType, EventFilter, WatcherStats
};