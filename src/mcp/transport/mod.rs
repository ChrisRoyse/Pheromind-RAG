/// Transport layer for MCP server communication
/// 
/// This module provides the transport abstraction and implementations for MCP servers.
/// Currently supports stdio transport for line-delimited JSON-RPC messages.

use async_trait::async_trait;

use crate::mcp::error::{McpError, McpResult};

pub mod stdio;

pub use stdio::{StdioTransport, StdioMcpHandler, StdioMcpServerBuilder};

/// Message received from transport layer
#[derive(Debug, Clone)]
pub struct TransportMessage {
    pub content: String,
    pub message_id: Option<String>,
}

/// Response to be sent via transport layer
#[derive(Debug, Clone)]
pub struct TransportResponse {
    pub content: String,
    pub message_id: Option<String>,
}

/// Transport trait for MCP communication
/// 
/// Implementations handle the low-level communication details while the protocol
/// handler manages JSON-RPC parsing and validation.
#[async_trait]
pub trait Transport: Send + Sync + std::fmt::Debug {
    /// Start the transport and begin listening for messages
    /// Returns when the transport is ready to accept connections
    async fn start(&mut self) -> McpResult<()>;
    
    /// Stop the transport and clean up resources
    async fn stop(&mut self) -> McpResult<()>;
    
    /// Read the next message from the transport
    /// Returns None when the transport is closed or EOF is reached
    async fn read_message(&mut self) -> McpResult<Option<TransportMessage>>;
    
    /// Send a response message via the transport
    async fn send_response(&mut self, response: &TransportResponse) -> McpResult<()>;
    
    /// Check if the transport is currently active and ready for communication
    fn is_active(&self) -> bool;
    
    /// Get transport-specific information (e.g., connection details)
    fn info(&self) -> TransportInfo;
}

/// Information about the current transport configuration
#[derive(Debug, Clone)]
pub struct TransportInfo {
    pub transport_type: String,
    pub connection_info: String,
    pub is_bidirectional: bool,
    pub supports_streaming: bool,
}

/// Transport factory for creating different transport types
pub struct TransportFactory;

impl TransportFactory {
    /// Create a new stdio transport
    pub fn create_stdio() -> McpResult<Box<dyn Transport>> {
        Ok(Box::new(StdioTransport::new()?))
    }
    
    /// Create transport from configuration string
    /// Format: "stdio" or "tcp:host:port" (future extension)
    pub fn create_from_config(config: &str) -> McpResult<Box<dyn Transport>> {
        match config.to_lowercase().as_str() {
            "stdio" => Self::create_stdio(),
            _ => Err(McpError::ConfigError {
                message: format!("Unsupported transport type: {}", config),
            }),
        }
    }
}

/// Transport event loop that integrates with the protocol handler
pub struct TransportEventLoop<T: Transport> {
    pub transport: T,
    message_handler: Option<Box<dyn MessageHandler>>,
}

impl<T: Transport> std::fmt::Debug for TransportEventLoop<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TransportEventLoop")
            .field("transport", &self.transport)
            .field("has_message_handler", &self.message_handler.is_some())
            .finish()
    }
}

/// Handler trait for processing messages received from transport
#[async_trait]
pub trait MessageHandler: Send + Sync {
    /// Process an incoming message and return a response
    async fn handle_message(&mut self, message: &TransportMessage) -> McpResult<Option<TransportResponse>>;
    
    /// Handle transport errors and connection issues
    async fn handle_error(&mut self, error: McpError) -> McpResult<()>;
    
    /// Called when transport is started
    async fn on_start(&mut self) -> McpResult<()> {
        Ok(())
    }
    
    /// Called when transport is stopped
    async fn on_stop(&mut self) -> McpResult<()> {
        Ok(())
    }
}

impl<T: Transport> TransportEventLoop<T> {
    pub fn new(transport: T) -> Self {
        Self {
            transport,
            message_handler: None,
        }
    }
    
    pub fn with_handler(mut self, handler: Box<dyn MessageHandler>) -> Self {
        self.message_handler = Some(handler);
        self
    }
    
    /// Run the event loop until transport is closed or an error occurs
    pub async fn run(&mut self) -> McpResult<()> {
        // Start the transport
        self.transport.start().await?;
        
        // Notify handler
        if let Some(ref mut handler) = self.message_handler {
            handler.on_start().await?;
        }
        
        // Main event loop
        loop {
            match self.transport.read_message().await {
                Ok(Some(message)) => {
                    // Process message with handler
                    if let Some(ref mut handler) = self.message_handler {
                        match handler.handle_message(&message).await {
                            Ok(Some(response)) => {
                                // Send response back
                                if let Err(e) = self.transport.send_response(&response).await {
                                    log::error!("Failed to send response: {}", e);
                                    handler.handle_error(e).await?;
                                }
                            }
                            Ok(None) => {
                                // No response needed (notification)
                            }
                            Err(e) => {
                                log::error!("Message handler error: {}", e);
                                handler.handle_error(e).await?;
                            }
                        }
                    }
                }
                Ok(None) => {
                    // Transport closed gracefully
                    log::info!("Transport closed, shutting down event loop");
                    break;
                }
                Err(e) => {
                    log::error!("Transport error: {}", e);
                    if let Some(ref mut handler) = self.message_handler {
                        handler.handle_error(e).await?;
                    }
                    break;
                }
            }
            
            // Check if transport is still active
            if !self.transport.is_active() {
                log::info!("Transport no longer active, shutting down");
                break;
            }
        }
        
        // Cleanup
        if let Some(ref mut handler) = self.message_handler {
            handler.on_stop().await?;
        }
        
        self.transport.stop().await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_transport_factory_stdio() {
        let transport = TransportFactory::create_stdio();
        assert!(transport.is_ok());
        
        let transport = transport.unwrap();
        let info = transport.info();
        assert_eq!(info.transport_type, "stdio");
        assert!(info.is_bidirectional);
    }
    
    #[test]
    fn test_transport_factory_invalid_config() {
        let result = TransportFactory::create_from_config("invalid");
        assert!(result.is_err());
        
        if let Err(McpError::ConfigError { message }) = result {
            assert!(message.contains("Unsupported transport type"));
        } else {
            panic!("Expected ConfigError");
        }
    }
    
    #[test]
    fn test_transport_message_creation() {
        let msg = TransportMessage {
            content: r#"{"jsonrpc":"2.0","method":"test","id":1}"#.to_string(),
            message_id: Some("test-1".to_string()),
        };
        
        assert!(!msg.content.is_empty());
        assert_eq!(msg.message_id, Some("test-1".to_string()));
    }
}