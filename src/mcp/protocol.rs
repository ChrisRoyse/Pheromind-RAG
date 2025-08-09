use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use crate::mcp::error::{McpError, McpResult};

/// JSON-RPC 2.0 request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: Option<JsonValue>,
    pub id: Option<JsonValue>,
}

/// JSON-RPC 2.0 response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub result: Option<JsonValue>,
    pub error: Option<JsonRpcError>,
    pub id: Option<JsonValue>,
}

/// JSON-RPC 2.0 error structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    pub data: Option<JsonValue>,
}

/// Supported RPC methods
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RpcMethod {
    /// Initialize MCP connection and negotiate capabilities
    Initialize,
    /// Perform search query
    Search,
    /// Index files or directories
    Index,
    /// Get server and index statistics
    Stats,
    /// Clear index data
    Clear,
    /// Get server capabilities
    Capabilities,
    /// Health check
    Ping,
    /// Shutdown server gracefully
    Shutdown,
    /// Start file watching
    WatcherStart,
    /// Stop file watching  
    WatcherStop,
    /// Get watcher status
    WatcherStatus,
    /// Subscribe to watcher events
    WatcherSubscribe,
    /// Unsubscribe from watcher events
    WatcherUnsubscribe,
    /// Trigger manual index update
    WatcherManualUpdate,
    /// Reset watcher error count
    WatcherResetErrors,
}

impl RpcMethod {
    /// Parse method name from string
    pub fn from_str(method: &str) -> McpResult<Self> {
        match method {
            "initialize" => Ok(Self::Initialize),
            "search" => Ok(Self::Search),
            "index" => Ok(Self::Index),
            "stats" => Ok(Self::Stats),
            "clear" => Ok(Self::Clear),
            "capabilities" => Ok(Self::Capabilities),
            "ping" => Ok(Self::Ping),
            "shutdown" => Ok(Self::Shutdown),
            "watcher/start" => Ok(Self::WatcherStart),
            "watcher/stop" => Ok(Self::WatcherStop),
            "watcher/status" => Ok(Self::WatcherStatus),
            "watcher/subscribe" => Ok(Self::WatcherSubscribe),
            "watcher/unsubscribe" => Ok(Self::WatcherUnsubscribe),
            "watcher/manual_update" => Ok(Self::WatcherManualUpdate),
            "watcher/reset_errors" => Ok(Self::WatcherResetErrors),
            _ => Err(McpError::MethodNotFound {
                method: method.to_string(),
            }),
        }
    }

    /// Convert method to string
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Initialize => "initialize",
            Self::Search => "search",
            Self::Index => "index",
            Self::Stats => "stats",
            Self::Clear => "clear",
            Self::Capabilities => "capabilities",
            Self::Ping => "ping",
            Self::Shutdown => "shutdown",
            Self::WatcherStart => "watcher/start",
            Self::WatcherStop => "watcher/stop",
            Self::WatcherStatus => "watcher/status",
            Self::WatcherSubscribe => "watcher/subscribe",
            Self::WatcherUnsubscribe => "watcher/unsubscribe",
            Self::WatcherManualUpdate => "watcher/manual_update",
            Self::WatcherResetErrors => "watcher/reset_errors",
        }
    }
}

impl JsonRpcRequest {
    /// Create a new JSON-RPC request
    pub fn new<T: Serialize>(method: RpcMethod, params: Option<T>, id: Option<JsonValue>) -> McpResult<Self> {
        let params = match params {
            Some(p) => Some(serde_json::to_value(p)?),
            None => None,
        };

        Ok(Self {
            jsonrpc: "2.0".to_string(),
            method: method.as_str().to_string(),
            params,
            id,
        })
    }

    /// Validate JSON-RPC 2.0 compliance
    pub fn validate(&self) -> McpResult<()> {
        if self.jsonrpc != "2.0" {
            return Err(McpError::InvalidRequest {
                message: format!("Invalid JSON-RPC version: expected '2.0', got '{}'", self.jsonrpc),
            });
        }

        if self.method.is_empty() {
            return Err(McpError::InvalidRequest {
                message: "Method name cannot be empty".to_string(),
            });
        }

        // Validate that method exists
        RpcMethod::from_str(&self.method)?;

        Ok(())
    }

    /// Parse method from request
    pub fn get_method(&self) -> McpResult<RpcMethod> {
        RpcMethod::from_str(&self.method)
    }

    /// Extract typed parameters from request
    pub fn get_params<T>(&self) -> McpResult<Option<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        match &self.params {
            Some(params) => {
                let typed_params = serde_json::from_value(params.clone())
                    .map_err(|e| McpError::InvalidParams {
                        message: format!("Failed to parse parameters for method '{}': {}", self.method, e),
                    })?;
                Ok(Some(typed_params))
            }
            None => Ok(None),
        }
    }

    /// Check if this is a notification (no id field)
    pub fn is_notification(&self) -> bool {
        self.id.is_none()
    }
}

impl std::fmt::Display for JsonRpcResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match serde_json::to_string(self) {
            Ok(json) => write!(f, "{}", json),
            Err(_) => write!(f, "{{\"jsonrpc\":\"2.0\",\"error\":{{\"code\":-32603,\"message\":\"Internal error\"}},\"id\":null}}"),
        }
    }
}

impl JsonRpcResponse {
    /// Create a successful response
    pub fn success<T: Serialize>(result: T, id: Option<JsonValue>) -> McpResult<Self> {
        Ok(Self {
            jsonrpc: "2.0".to_string(),
            result: Some(serde_json::to_value(result)?),
            error: None,
            id,
        })
    }

    /// Create an error response
    pub fn error(error: McpError, id: Option<JsonValue>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(JsonRpcError {
                code: error.code(),
                message: error.to_string(),
                data: serde_json::to_value(&error).ok(),
            }),
            id,
        }
    }

    /// Create a parse error response (when request cannot be parsed)
    pub fn parse_error() -> Self {
        Self::error(
            McpError::ParseError {
                message: "Invalid JSON was received by the server".to_string(),
            },
            None,
        )
    }

    /// Validate JSON-RPC 2.0 compliance
    pub fn validate(&self) -> McpResult<()> {
        if self.jsonrpc != "2.0" {
            return Err(McpError::InvalidRequest {
                message: format!("Invalid JSON-RPC version in response: expected '2.0', got '{}'", self.jsonrpc),
            });
        }

        // Either result or error must be present, but not both
        match (&self.result, &self.error) {
            (Some(_), Some(_)) => Err(McpError::InvalidRequest {
                message: "Response cannot have both result and error fields".to_string(),
            }),
            (None, None) => Err(McpError::InvalidRequest {
                message: "Response must have either result or error field".to_string(),
            }),
            _ => Ok(()),
        }
    }

    /// Check if this response indicates success
    pub fn is_success(&self) -> bool {
        self.error.is_none() && self.result.is_some()
    }

    /// Check if this response indicates an error
    pub fn is_error(&self) -> bool {
        self.error.is_some()
    }

    /// Extract typed result from successful response
    pub fn get_result<T>(&self) -> McpResult<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        match &self.result {
            Some(result) => serde_json::from_value(result.clone())
                .map_err(|e| McpError::InternalError {
                    message: format!("Failed to deserialize result: {}", e),
                }),
            None => Err(McpError::InternalError {
                message: "No result field in response".to_string(),
            }),
        }
    }

    /// Convert response to JSON string
    pub fn to_json(&self) -> McpResult<String> {
        serde_json::to_string(self).map_err(|e| McpError::InternalError {
            message: format!("Failed to serialize response to JSON: {}", e),
        })
    }
}

/// Protocol handler for parsing and validating JSON-RPC messages
#[derive(Debug, Default)]
pub struct ProtocolHandler {
    /// Track request IDs to prevent replay attacks
    processed_ids: std::collections::HashSet<String>,
}

impl ProtocolHandler {
    pub fn new() -> Self {
        Self::default()
    }

    /// Parse incoming JSON message into a request
    pub fn parse_request(&mut self, json: &str) -> McpResult<JsonRpcRequest> {
        let request: JsonRpcRequest = serde_json::from_str(json)
            .map_err(|e| McpError::ParseError {
                message: format!("Failed to parse JSON-RPC request: {}", e),
            })?;

        // Validate JSON-RPC compliance
        request.validate()?;

        // Check for duplicate request IDs (if not a notification)
        if let Some(id) = &request.id {
            let id_str = id.to_string();
            if self.processed_ids.contains(&id_str) {
                return Err(McpError::InvalidRequest {
                    message: format!("Duplicate request ID: {}", id_str),
                });
            }
            self.processed_ids.insert(id_str);
        }

        Ok(request)
    }

    /// Serialize response to JSON
    pub fn serialize_response(&self, response: &JsonRpcResponse) -> McpResult<String> {
        // Validate response before serializing
        response.validate()?;

        serde_json::to_string(response)
            .map_err(|e| McpError::InternalError {
                message: format!("Failed to serialize JSON-RPC response: {}", e),
            })
    }

    /// Create a batch request (array of requests)
    pub fn parse_batch_request(&mut self, json: &str) -> McpResult<Vec<JsonRpcRequest>> {
        let value: JsonValue = serde_json::from_str(json)
            .map_err(|e| McpError::ParseError {
                message: format!("Failed to parse JSON: {}", e),
            })?;

        match value {
            JsonValue::Array(array) => {
                if array.is_empty() {
                    return Err(McpError::InvalidRequest {
                        message: "Batch request cannot be empty".to_string(),
                    });
                }

                let mut requests = Vec::with_capacity(array.len());
                for item in array {
                    let request: JsonRpcRequest = serde_json::from_value(item)
                        .map_err(|e| McpError::ParseError {
                            message: format!("Failed to parse batch request item: {}", e),
                        })?;
                    
                    request.validate()?;
                    requests.push(request);
                }

                Ok(requests)
            }
            _ => {
                // Single request, not a batch
                let request: JsonRpcRequest = serde_json::from_value(value)
                    .map_err(|e| McpError::ParseError {
                        message: format!("Failed to parse JSON-RPC request: {}", e),
                    })?;
                
                request.validate()?;
                Ok(vec![request])
            }
        }
    }

    /// Serialize batch response to JSON
    pub fn serialize_batch_response(&self, responses: &[JsonRpcResponse]) -> McpResult<String> {
        // Validate all responses
        for response in responses {
            response.validate()?;
        }

        if responses.len() == 1 {
            // Single response
            self.serialize_response(&responses[0])
        } else {
            // Batch response
            serde_json::to_string(responses)
                .map_err(|e| McpError::InternalError {
                    message: format!("Failed to serialize batch response: {}", e),
                })
        }
    }

    /// Clear processed IDs (for cleanup)
    pub fn clear_processed_ids(&mut self) {
        self.processed_ids.clear();
    }

    /// Get number of processed requests
    pub fn processed_count(&self) -> usize {
        self.processed_ids.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_request_parsing() {
        let mut handler = ProtocolHandler::new();
        let json = r#"{"jsonrpc":"2.0","method":"search","params":{"query":"test"},"id":1}"#;
        
        let request = handler.parse_request(json).unwrap();
        assert_eq!(request.method, "search");
        assert_eq!(request.jsonrpc, "2.0");
        assert!(request.params.is_some());
    }

    #[test]
    fn test_invalid_jsonrpc_version() {
        let mut handler = ProtocolHandler::new();
        let json = r#"{"jsonrpc":"1.0","method":"search","id":1}"#;
        
        let result = handler.parse_request(json);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), McpError::InvalidRequest { .. }));
    }

    #[test]
    fn test_success_response() {
        let response = JsonRpcResponse::success("test result", Some(serde_json::json!(1))).unwrap();
        assert!(response.is_success());
        assert!(!response.is_error());
        assert_eq!(response.jsonrpc, "2.0");
    }

    #[test]
    fn test_error_response() {
        let error = McpError::MethodNotFound { method: "unknown".to_string() };
        let response = JsonRpcResponse::error(error, Some(serde_json::json!(1)));
        assert!(!response.is_success());
        assert!(response.is_error());
    }
}