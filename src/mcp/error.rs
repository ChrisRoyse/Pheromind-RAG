use thiserror::Error;
use serde::{Deserialize, Serialize};

/// MCP-specific error types following JSON-RPC 2.0 specification
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
pub enum McpError {
    #[error("Parse error: {message}")]
    ParseError { message: String },
    
    #[error("Invalid request: {message}")]
    InvalidRequest { message: String },
    
    #[error("Method not found: {method}")]
    MethodNotFound { method: String },
    
    #[error("Invalid parameters: {message}")]
    InvalidParams { message: String },
    
    #[error("Internal error: {message}")]
    InternalError { message: String },
    
    #[error("Search error: {message}")]
    SearchError { message: String },
    
    #[error("Index error: {message}")]
    IndexError { message: String },
    
    #[error("Configuration error: {message}")]
    ConfigError { message: String },
    
    #[error("Resource not found: {resource}")]
    ResourceNotFound { resource: String },
    
    #[error("Permission denied: {operation}")]
    PermissionDenied { operation: String },
    
    #[error("Timeout: {operation} exceeded time limit")]
    Timeout { operation: String },
    
    #[error("Rate limit exceeded: {limit} requests per {window}")]
    RateLimitExceeded { limit: u32, window: String },
    
    #[error("Server not ready: {reason}")]
    ServerNotReady { reason: String },
}

impl McpError {
    /// Get the JSON-RPC error code for this error
    pub fn code(&self) -> i32 {
        match self {
            McpError::ParseError { .. } => -32700,
            McpError::InvalidRequest { .. } => -32600,
            McpError::MethodNotFound { .. } => -32601,
            McpError::InvalidParams { .. } => -32602,
            McpError::InternalError { .. } => -32603,
            McpError::SearchError { .. } => -32001,
            McpError::IndexError { .. } => -32002,
            McpError::ConfigError { .. } => -32003,
            McpError::ResourceNotFound { .. } => -32004,
            McpError::PermissionDenied { .. } => -32005,
            McpError::Timeout { .. } => -32006,
            McpError::RateLimitExceeded { .. } => -32007,
            McpError::ServerNotReady { .. } => -32008,
        }
    }
    
    /// Create a JSON-RPC error object
    pub fn to_json_rpc_error(&self, id: Option<serde_json::Value>) -> serde_json::Value {
        serde_json::json!({
            "jsonrpc": "2.0",
            "error": {
                "code": self.code(),
                "message": self.to_string(),
                "data": self.clone()
            },
            "id": id
        })
    }
}

/// Result type alias for MCP operations
pub type McpResult<T> = std::result::Result<T, McpError>;

// Convert from embed-search errors to MCP errors
impl From<crate::error::EmbedError> for McpError {
    fn from(err: crate::error::EmbedError) -> Self {
        match err {
            crate::error::EmbedError::Search { message, .. } => McpError::SearchError { message },
            crate::error::EmbedError::Configuration { message, .. } => McpError::ConfigError { message },
            crate::error::EmbedError::NotFound { ref resource, .. } => McpError::ResourceNotFound { 
                resource: format!("{} ({})", resource, err) 
            },
            crate::error::EmbedError::Timeout { operation, .. } => McpError::Timeout { operation },
            crate::error::EmbedError::Internal { message, .. } => McpError::InternalError { message },
            _ => McpError::InternalError { message: err.to_string() },
        }
    }
}

impl From<anyhow::Error> for McpError {
    fn from(err: anyhow::Error) -> Self {
        McpError::InternalError { 
            message: err.to_string() 
        }
    }
}

impl From<serde_json::Error> for McpError {
    fn from(err: serde_json::Error) -> Self {
        McpError::ParseError { 
            message: err.to_string() 
        }
    }
}