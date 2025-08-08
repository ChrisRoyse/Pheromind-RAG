/// Basic transport layer tests that don't require full server setup
/// 
/// These tests focus on the transport mechanisms independently

use serde_json::Value as JsonValue;

use embed_search::mcp::{
    StdioTransport, Transport, TransportMessage, TransportResponse, TransportInfo,
    TransportFactory, TransportEventLoop, MessageHandler, McpResult, McpError
};

/// Mock transport for testing that doesn't use actual stdin/stdout
#[derive(Debug)]
struct MockTransport {
    messages: Vec<String>,
    current_index: usize,
    responses: Vec<String>,
    active: bool,
}

impl MockTransport {
    fn new(messages: Vec<String>) -> Self {
        Self {
            messages,
            current_index: 0,
            responses: Vec::new(),
            active: false,
        }
    }
    
    pub fn get_responses(&self) -> &[String] {
        &self.responses
    }
}

#[async_trait::async_trait]
impl Transport for MockTransport {
    async fn start(&mut self) -> McpResult<()> {
        self.active = true;
        Ok(())
    }
    
    async fn stop(&mut self) -> McpResult<()> {
        self.active = false;
        Ok(())
    }
    
    async fn read_message(&mut self) -> McpResult<Option<TransportMessage>> {
        if !self.active {
            return Err(McpError::ServerNotReady {
                reason: "Transport not started".to_string(),
            });
        }
        
        if self.current_index >= self.messages.len() {
            return Ok(None); // EOF
        }
        
        let message = self.messages[self.current_index].clone();
        self.current_index += 1;
        
        Ok(Some(TransportMessage {
            content: message,
            message_id: Some(format!("test-{}", self.current_index)),
        }))
    }
    
    async fn send_response(&mut self, response: &TransportResponse) -> McpResult<()> {
        if !self.active {
            return Err(McpError::ServerNotReady {
                reason: "Transport not started".to_string(),
            });
        }
        
        self.responses.push(response.content.clone());
        Ok(())
    }
    
    fn is_active(&self) -> bool {
        self.active
    }
    
    fn info(&self) -> TransportInfo {
        TransportInfo {
            transport_type: "mock".to_string(),
            connection_info: format!("mock transport with {} messages", self.messages.len()),
            is_bidirectional: true,
            supports_streaming: false,
        }
    }
}

/// Simple echo handler for testing
struct EchoHandler;

#[async_trait::async_trait]
impl MessageHandler for EchoHandler {
    async fn handle_message(&mut self, message: &TransportMessage) -> McpResult<Option<TransportResponse>> {
        // Just echo back the message with some transformation
        let echo_content = format!("Echo: {}", message.content);
        
        Ok(Some(TransportResponse {
            content: echo_content,
            message_id: message.message_id.clone(),
        }))
    }
    
    async fn handle_error(&mut self, error: McpError) -> McpResult<()> {
        eprintln!("Echo handler error: {}", error);
        Ok(())
    }
}

/// JSON-RPC response handler for testing
struct JsonRpcHandler;

#[async_trait::async_trait]
impl MessageHandler for JsonRpcHandler {
    async fn handle_message(&mut self, message: &TransportMessage) -> McpResult<Option<TransportResponse>> {
        // Parse as JSON-RPC and create appropriate response
        let request: Result<JsonValue, _> = serde_json::from_str(&message.content);
        
        let response_json = match request {
            Ok(json) => {
                if let Some(id) = json.get("id") {
                    // Valid request with ID - return success response
                    serde_json::json!({
                        "jsonrpc": "2.0",
                        "result": "success",
                        "id": id
                    })
                } else {
                    // Notification - no response needed
                    return Ok(None);
                }
            }
            Err(_) => {
                // Invalid JSON - return parse error
                serde_json::json!({
                    "jsonrpc": "2.0",
                    "error": {
                        "code": -32700,
                        "message": "Parse error"
                    },
                    "id": null
                })
            }
        };
        
        Ok(Some(TransportResponse {
            content: response_json.to_string(),
            message_id: message.message_id.clone(),
        }))
    }
    
    async fn handle_error(&mut self, error: McpError) -> McpResult<()> {
        eprintln!("JsonRpc handler error: {}", error);
        Ok(())
    }
}

#[tokio::test]
async fn test_transport_factory() {
    let transport = TransportFactory::create_stdio();
    assert!(transport.is_ok());
    
    let transport = transport.unwrap();
    let info = transport.info();
    assert_eq!(info.transport_type, "stdio");
    assert!(info.is_bidirectional);
}

#[tokio::test]
async fn test_transport_factory_invalid_config() {
    let result = TransportFactory::create_from_config("invalid");
    assert!(result.is_err());
    
    if let Err(McpError::ConfigError { message }) = result {
        assert!(message.contains("Unsupported transport type"));
    } else {
        panic!("Expected ConfigError");
    }
}

#[tokio::test]
async fn test_mock_transport_basic_flow() {
    let messages = vec!["message1".to_string(), "message2".to_string()];
    let mut transport = MockTransport::new(messages);
    
    // Initially not active
    assert!(!transport.is_active());
    
    // Start transport
    transport.start().await.unwrap();
    assert!(transport.is_active());
    
    // Read first message
    let msg1 = transport.read_message().await.unwrap();
    assert!(msg1.is_some());
    let msg1 = msg1.unwrap();
    assert_eq!(msg1.content, "message1");
    
    // Send response
    let response = TransportResponse {
        content: "response1".to_string(),
        message_id: msg1.message_id.clone(),
    };
    transport.send_response(&response).await.unwrap();
    
    // Read second message
    let msg2 = transport.read_message().await.unwrap();
    assert!(msg2.is_some());
    let msg2 = msg2.unwrap();
    assert_eq!(msg2.content, "message2");
    
    // No more messages
    let msg3 = transport.read_message().await.unwrap();
    assert!(msg3.is_none());
    
    // Check responses
    let responses = transport.get_responses();
    assert_eq!(responses.len(), 1);
    assert_eq!(responses[0], "response1");
    
    // Stop transport
    transport.stop().await.unwrap();
    assert!(!transport.is_active());
}

#[tokio::test]
async fn test_event_loop_with_echo_handler() {
    let messages = vec!["test message".to_string()];
    let transport = MockTransport::new(messages);
    let handler = EchoHandler;
    
    let mut event_loop = TransportEventLoop::new(transport).with_handler(Box::new(handler));
    
    // Run event loop
    event_loop.run().await.unwrap();
    
    // Check responses
    let responses = event_loop.transport.get_responses();
    assert_eq!(responses.len(), 1);
    assert_eq!(responses[0], "Echo: test message");
}

#[tokio::test]
async fn test_event_loop_with_jsonrpc_handler() {
    let messages = vec![
        r#"{"jsonrpc":"2.0","method":"test","id":1}"#.to_string(),
        r#"invalid json"#.to_string(),
        r#"{"jsonrpc":"2.0","method":"notification"}"#.to_string(), // No id = notification
    ];
    
    let transport = MockTransport::new(messages);
    let handler = JsonRpcHandler;
    
    let mut event_loop = TransportEventLoop::new(transport).with_handler(Box::new(handler));
    
    // Run event loop
    event_loop.run().await.unwrap();
    
    // Check responses
    let responses = event_loop.transport.get_responses();
    assert_eq!(responses.len(), 2, "Expected 2 responses (valid request + parse error)");
    
    // Parse first response (valid request)
    let response1: JsonValue = serde_json::from_str(&responses[0]).unwrap();
    assert_eq!(response1["jsonrpc"], "2.0");
    assert_eq!(response1["result"], "success");
    assert_eq!(response1["id"], 1);
    
    // Parse second response (parse error)
    let response2: JsonValue = serde_json::from_str(&responses[1]).unwrap();
    assert_eq!(response2["jsonrpc"], "2.0");
    assert!(response2["error"].is_object());
    assert_eq!(response2["error"]["code"], -32700);
}

#[tokio::test]
async fn test_stdio_transport_creation_and_info() {
    let transport = StdioTransport::new().unwrap();
    
    // Check initial state
    assert!(!transport.is_active());
    assert_eq!(transport.message_count(), 0);
    
    // Check transport info
    let info = transport.info();
    assert_eq!(info.transport_type, "stdio");
    assert!(info.is_bidirectional);
    assert!(info.supports_streaming);
    assert!(info.connection_info.contains("stdin/stdout"));
}

#[tokio::test]
async fn test_transport_error_handling() {
    let mut transport = MockTransport::new(vec![]);
    
    // Try to read without starting - should fail
    let result = transport.read_message().await;
    assert!(result.is_err());
    if let Err(McpError::ServerNotReady { reason }) = result {
        assert_eq!(reason, "Transport not started");
    } else {
        panic!("Expected ServerNotReady error");
    }
    
    // Try to send without starting - should fail
    let response = TransportResponse {
        content: "test".to_string(),
        message_id: None,
    };
    let result = transport.send_response(&response).await;
    assert!(result.is_err());
    if let Err(McpError::ServerNotReady { reason }) = result {
        assert_eq!(reason, "Transport not started");
    } else {
        panic!("Expected ServerNotReady error");
    }
}

#[tokio::test]
async fn test_message_and_response_structures() {
    let msg = TransportMessage {
        content: r#"{"jsonrpc":"2.0","method":"ping","id":1}"#.to_string(),
        message_id: Some("test-1".to_string()),
    };
    
    let resp = TransportResponse {
        content: r#"{"jsonrpc":"2.0","result":"pong","id":1}"#.to_string(),
        message_id: msg.message_id.clone(),
    };
    
    assert_eq!(resp.message_id, Some("test-1".to_string()));
    assert!(resp.content.contains("pong"));
    
    // Verify JSON structure
    let json: JsonValue = serde_json::from_str(&resp.content).unwrap();
    assert_eq!(json["jsonrpc"], "2.0");
    assert_eq!(json["result"], "pong");
    assert_eq!(json["id"], 1);
}