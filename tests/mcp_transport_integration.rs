/// Integration tests for MCP transport layer with protocol handler
/// 
/// Tests the complete flow: Transport -> Protocol Handler -> MCP Server -> Response

use tempfile::TempDir;
use tokio::time::{timeout, Duration};
use serde_json::Value as JsonValue;

use embed_search::mcp::{
    McpServer, StdioMcpServerBuilder, TransportEventLoop, StdioTransport,
    Transport, TransportMessage, TransportResponse, MessageHandler, McpResult, McpError
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
    
    fn get_responses(&self) -> &[String] {
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
    
    fn info(&self) -> embed_search::mcp::TransportInfo {
        embed_search::mcp::TransportInfo {
            transport_type: "mock".to_string(),
            connection_info: format!("mock transport with {} messages", self.messages.len()),
            is_bidirectional: true,
            supports_streaming: false,
        }
    }
}

/// Helper to create a test MCP server
async fn create_test_server() -> McpResult<McpServer> {
    let temp_dir = TempDir::new().unwrap();
    
    // Initialize config first - set environment variable to ensure it works
    std::env::set_var("EMBED_LOG_LEVEL", "info");
    match embed_search::config::Config::init() {
        Ok(()) => {
            // Successfully initialized
        }
        Err(e) => {
            // Check if it's already initialized
            println!("Config init warning: {}", e);
        }
    }
    
    McpServer::with_project_path(temp_dir.path().to_path_buf()).await
}

#[tokio::test]
async fn test_transport_protocol_integration_ping() {
    let server = create_test_server().await.expect("Failed to create test server");
    
    // Test ping request
    let ping_request = r#"{"jsonrpc":"2.0","method":"ping","id":1}"#;
    let messages = vec![ping_request.to_string()];
    
    let transport = MockTransport::new(messages);
    let handler = TestHandler::new(server);
    let mut event_loop = TransportEventLoop::new(transport).with_handler(Box::new(handler));
    
    // Run the event loop with a timeout
    let result = timeout(Duration::from_secs(5), event_loop.run()).await;
    assert!(result.is_ok(), "Event loop timed out");
    
    // Check responses
    let responses = event_loop.transport.get_responses();
    assert_eq!(responses.len(), 1, "Expected exactly one response");
    
    let response: JsonValue = serde_json::from_str(&responses[0]).unwrap();
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(response["result"].is_object());
    assert_eq!(response["result"]["status"], "ok");
}

#[tokio::test]
async fn test_transport_protocol_integration_capabilities() {
    let server = create_test_server().await.expect("Failed to create test server");
    
    // Test capabilities request
    let capabilities_request = r#"{"jsonrpc":"2.0","method":"capabilities","id":2}"#;
    let messages = vec![capabilities_request.to_string()];
    
    let transport = MockTransport::new(messages);
    let handler = TestHandler::new(server);
    let mut event_loop = TransportEventLoop::new(transport).with_handler(Box::new(handler));
    
    let result = timeout(Duration::from_secs(5), event_loop.run()).await;
    assert!(result.is_ok(), "Event loop timed out");
    
    let responses = event_loop.transport.get_responses();
    assert_eq!(responses.len(), 1);
    
    let response: JsonValue = serde_json::from_str(&responses[0]).unwrap();
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 2);
    assert!(response["result"].is_object());
    
    // Check that capabilities structure is present
    let capabilities = &response["result"];
    assert!(capabilities["search"].is_object());
    assert!(capabilities["indexing"].is_object());
    assert!(capabilities["stats"].is_object());
    assert!(capabilities["server_info"].is_object());
}

#[tokio::test]
async fn test_transport_protocol_integration_invalid_request() {
    let server = create_test_server().await.expect("Failed to create test server");
    
    // Test invalid JSON
    let invalid_request = r#"{"invalid":"json","missing":"required_fields"}"#;
    let messages = vec![invalid_request.to_string()];
    
    let transport = MockTransport::new(messages);
    let handler = TestHandler::new(server);
    let mut event_loop = TransportEventLoop::new(transport).with_handler(Box::new(handler));
    
    let result = timeout(Duration::from_secs(5), event_loop.run()).await;
    assert!(result.is_ok(), "Event loop timed out");
    
    let responses = event_loop.transport.get_responses();
    assert_eq!(responses.len(), 1);
    
    let response: JsonValue = serde_json::from_str(&responses[0]).unwrap();
    assert_eq!(response["jsonrpc"], "2.0");
    assert!(response["error"].is_object());
    
    // Should be an invalid request error
    let error = &response["error"];
    assert!(error["code"].as_i64().unwrap() < 0);
    assert!(error["message"].as_str().unwrap().contains("Invalid"));
}

#[tokio::test]
async fn test_transport_protocol_integration_batch_requests() {
    let server = create_test_server().await.expect("Failed to create test server");
    
    // Test batch request (array of requests)
    let batch_request = r#"[
        {"jsonrpc":"2.0","method":"ping","id":1},
        {"jsonrpc":"2.0","method":"capabilities","id":2}
    ]"#;
    let messages = vec![batch_request.to_string()];
    
    let transport = MockTransport::new(messages);
    let handler = TestHandler::new(server);
    let mut event_loop = TransportEventLoop::new(transport).with_handler(Box::new(handler));
    
    let result = timeout(Duration::from_secs(5), event_loop.run()).await;
    assert!(result.is_ok(), "Event loop timed out");
    
    let responses = event_loop.transport.get_responses();
    assert_eq!(responses.len(), 1);
    
    // Parse as batch response (should be array)
    let response: JsonValue = serde_json::from_str(&responses[0]).unwrap();
    if let JsonValue::Array(batch_response) = response {
        assert_eq!(batch_response.len(), 2);
        
        // Check both responses
        for (i, resp) in batch_response.iter().enumerate() {
            assert_eq!(resp["jsonrpc"], "2.0");
            assert_eq!(resp["id"], i + 1);
            assert!(resp["result"].is_object());
        }
    } else {
        panic!("Expected array response for batch request");
    }
}

#[tokio::test]
async fn test_transport_protocol_integration_multiple_messages() {
    let server = create_test_server().await.expect("Failed to create test server");
    
    // Test multiple separate messages
    let messages = vec![
        r#"{"jsonrpc":"2.0","method":"ping","id":1}"#.to_string(),
        r#"{"jsonrpc":"2.0","method":"capabilities","id":2}"#.to_string(),
        r#"{"jsonrpc":"2.0","method":"ping","id":3}"#.to_string(),
    ];
    
    let transport = MockTransport::new(messages);
    let handler = TestHandler::new(server);
    let mut event_loop = TransportEventLoop::new(transport).with_handler(Box::new(handler));
    
    let result = timeout(Duration::from_secs(5), event_loop.run()).await;
    assert!(result.is_ok(), "Event loop timed out");
    
    let responses = event_loop.transport.get_responses();
    assert_eq!(responses.len(), 3, "Expected three separate responses");
    
    // Check all responses are valid JSON-RPC
    for (i, response_str) in responses.iter().enumerate() {
        let response: JsonValue = serde_json::from_str(response_str).unwrap();
        assert_eq!(response["jsonrpc"], "2.0");
        assert_eq!(response["id"], i + 1);
        assert!(response["result"].is_object());
    }
}

#[tokio::test] 
async fn test_stdio_transport_basic_functionality() {
    let transport = StdioTransport::new().unwrap();
    
    // Test initial state
    assert!(!transport.is_active());
    assert_eq!(transport.message_count(), 0);
    
    // Test transport info
    let info = transport.info();
    assert_eq!(info.transport_type, "stdio");
    assert!(info.is_bidirectional);
    assert!(info.supports_streaming);
}

#[tokio::test]
async fn test_stdio_server_builder() {
    let temp_dir = TempDir::new().unwrap();
    
    // Initialize config
    std::env::set_var("EMBED_LOG_LEVEL", "info");
    if let Err(_) = embed_search::config::Config::init() {
        // Already initialized
    }
    
    let builder = StdioMcpServerBuilder::new()
        .with_project_path(temp_dir.path().to_path_buf());
    
    let result = builder.build().await;
    assert!(result.is_ok(), "Failed to build stdio server: {:?}", result);
    
    // The event loop should be created successfully
    let event_loop = result.unwrap();
    assert!(!event_loop.transport.is_active());
}

/// Test message handler that wraps McpServer for testing
struct TestHandler {
    server: McpServer,
}

impl TestHandler {
    fn new(server: McpServer) -> Self {
        Self { server }
    }
}

#[async_trait::async_trait]
impl MessageHandler for TestHandler {
    async fn handle_message(&mut self, message: &TransportMessage) -> McpResult<Option<TransportResponse>> {
        // Delegate to server - handle both single and batch requests
        let response_json = if message.content.trim_start().starts_with('[') {
            // Batch request
            self.server.handle_batch_request(&message.content).await
        } else {
            // Single request
            self.server.handle_request(&message.content).await
        };
        
        Ok(Some(TransportResponse {
            content: response_json,
            message_id: message.message_id.clone(),
        }))
    }
    
    async fn handle_error(&mut self, error: McpError) -> McpResult<()> {
        eprintln!("Test handler error: {}", error);
        Ok(())
    }
}