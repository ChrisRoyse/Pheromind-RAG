/// Stdio transport implementation for MCP server
/// 
/// Handles line-delimited JSON-RPC messages over stdin/stdout, which is the
/// standard transport for MCP servers according to the specification.

use std::io;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::io::{stdin, stdout, Stdin, Stdout};
use async_trait::async_trait;

use crate::mcp::error::{McpError, McpResult};
use super::{Transport, TransportMessage, TransportResponse, TransportInfo};

/// Stdio-based transport for MCP communication
/// 
/// This transport reads JSON-RPC messages line by line from stdin and writes
/// responses to stdout. Each message must be a complete JSON object on a single line.
pub struct StdioTransport {
    reader: Option<BufReader<Stdin>>,
    writer: Option<BufWriter<Stdout>>,
    active: bool,
    message_count: u64,
}

impl std::fmt::Debug for StdioTransport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StdioTransport")
            .field("active", &self.active)
            .field("message_count", &self.message_count)
            .field("reader_initialized", &self.reader.is_some())
            .field("writer_initialized", &self.writer.is_some())
            .finish()
    }
}

impl StdioTransport {
    /// Create a new stdio transport
    pub fn new() -> McpResult<Self> {
        Ok(Self {
            reader: None,
            writer: None,
            active: false,
            message_count: 0,
        })
    }
    
    /// Get the number of messages processed
    pub fn message_count(&self) -> u64 {
        self.message_count
    }
    
    /// Reset message counter
    pub fn reset_counter(&mut self) {
        self.message_count = 0;
    }
}

#[async_trait]
impl Transport for StdioTransport {
    async fn start(&mut self) -> McpResult<()> {
        if self.active {
            return Err(McpError::InternalError {
                message: "Transport is already active".to_string(),
            });
        }
        
        // Initialize async stdin/stdout readers
        self.reader = Some(BufReader::new(stdin()));
        self.writer = Some(BufWriter::new(stdout()));
        self.active = true;
        
        log::info!("Stdio transport started successfully");
        Ok(())
    }
    
    async fn stop(&mut self) -> McpResult<()> {
        if !self.active {
            return Ok(());
        }
        
        // Flush any remaining output
        if let Some(ref mut writer) = self.writer {
            writer.flush().await.map_err(|e| McpError::InternalError {
                message: format!("Failed to flush stdout: {}", e),
            })?;
        }
        
        self.reader = None;
        self.writer = None;
        self.active = false;
        
        log::info!("Stdio transport stopped");
        Ok(())
    }
    
    async fn read_message(&mut self) -> McpResult<Option<TransportMessage>> {
        if !self.active {
            return Err(McpError::ServerNotReady {
                reason: "Transport not started".to_string(),
            });
        }
        
        let reader = self.reader.as_mut().ok_or_else(|| McpError::InternalError {
            message: "Reader not initialized".to_string(),
        })?;
        
        let mut line = String::new();
        match reader.read_line(&mut line).await {
            Ok(0) => {
                // EOF reached
                log::info!("EOF reached on stdin, transport closing");
                Ok(None)
            }
            Ok(bytes_read) => {
                // Remove trailing newline
                line.truncate(line.trim_end().len());
                
                if line.is_empty() {
                    // Empty line, try reading next
                    return self.read_message().await;
                }
                
                log::debug!("Received message ({} bytes): {}", bytes_read, line);
                
                // Generate message ID for tracking
                self.message_count += 1;
                let message_id = format!("msg-{}", self.message_count);
                
                Ok(Some(TransportMessage {
                    content: line,
                    message_id: Some(message_id),
                }))
            }
            Err(e) => {
                match e.kind() {
                    io::ErrorKind::UnexpectedEof => {
                        log::info!("Unexpected EOF on stdin");
                        Ok(None)
                    }
                    io::ErrorKind::BrokenPipe => {
                        log::warn!("Broken pipe on stdin");
                        Ok(None)
                    }
                    _ => Err(McpError::InternalError {
                        message: format!("Failed to read from stdin: {}", e),
                    })
                }
            }
        }
    }
    
    async fn send_response(&mut self, response: &TransportResponse) -> McpResult<()> {
        if !self.active {
            return Err(McpError::ServerNotReady {
                reason: "Transport not started".to_string(),
            });
        }
        
        let writer = self.writer.as_mut().ok_or_else(|| McpError::InternalError {
            message: "Writer not initialized".to_string(),
        })?;
        
        // Write response as single line with newline
        let line = format!("{}\n", response.content);
        
        writer.write_all(line.as_bytes()).await.map_err(|e| {
            match e.kind() {
                io::ErrorKind::BrokenPipe => {
                    log::warn!("Broken pipe on stdout");
                    McpError::InternalError {
                        message: "Output stream closed".to_string(),
                    }
                }
                _ => McpError::InternalError {
                    message: format!("Failed to write to stdout: {}", e),
                }
            }
        })?;
        
        // Flush immediately to ensure message is sent
        writer.flush().await.map_err(|e| McpError::InternalError {
            message: format!("Failed to flush stdout: {}", e),
        })?;
        
        log::debug!("Sent response ({}): {}", 
            response.message_id.as_deref().unwrap_or("no-id"), 
            response.content);
        
        Ok(())
    }
    
    fn is_active(&self) -> bool {
        self.active
    }
    
    fn info(&self) -> TransportInfo {
        TransportInfo {
            transport_type: "stdio".to_string(),
            connection_info: format!("stdin/stdout (processed {} messages)", self.message_count),
            is_bidirectional: true,
            supports_streaming: true,
        }
    }
}

impl Drop for StdioTransport {
    fn drop(&mut self) {
        if self.active {
            log::debug!("Dropping active stdio transport");
            // Can't await in drop, so just mark as inactive
            self.active = false;
        }
    }
}

/// Message handler that integrates StdioTransport with McpServer
pub struct StdioMcpHandler {
    server: crate::mcp::McpServer,
}

impl StdioMcpHandler {
    pub fn new(server: crate::mcp::McpServer) -> Self {
        Self { server }
    }
}

#[async_trait]
impl super::MessageHandler for StdioMcpHandler {
    async fn handle_message(&mut self, message: &TransportMessage) -> McpResult<Option<TransportResponse>> {
        log::debug!("Processing message: {}", message.content);
        
        // Use the MCP server to handle the JSON-RPC request
        let response_json = self.server.handle_request(&message.content).await;
        
        Ok(Some(TransportResponse {
            content: response_json,
            message_id: message.message_id.clone(),
        }))
    }
    
    async fn handle_error(&mut self, error: McpError) -> McpResult<()> {
        log::error!("Transport error: {}", error);
        
        // For critical errors, we might want to shut down
        match error {
            McpError::InternalError { .. } | McpError::ServerNotReady { .. } => {
                log::error!("Critical error, may need to shut down");
            }
            _ => {
                log::warn!("Recoverable error: {}", error);
            }
        }
        
        Ok(())
    }
    
    async fn on_start(&mut self) -> McpResult<()> {
        log::info!("MCP server ready on stdio transport");
        Ok(())
    }
    
    async fn on_stop(&mut self) -> McpResult<()> {
        log::info!("MCP server shutting down");
        Ok(())
    }
}

/// Builder for creating configured stdio MCP servers
pub struct StdioMcpServerBuilder {
    project_path: Option<std::path::PathBuf>,
    config: Option<crate::mcp::config::McpConfig>,
}

impl StdioMcpServerBuilder {
    pub fn new() -> Self {
        Self {
            project_path: None,
            config: None,
        }
    }
    
    pub fn with_project_path(mut self, path: std::path::PathBuf) -> Self {
        self.project_path = Some(path);
        self
    }
    
    pub fn with_config(mut self, config: crate::mcp::config::McpConfig) -> Self {
        self.config = Some(config);
        self
    }
    
    pub async fn build(self) -> McpResult<super::TransportEventLoop<StdioTransport>> {
        // Create MCP server
        let server = if let Some(project_path) = self.project_path {
            crate::mcp::McpServer::with_project_path(project_path).await?
        } else {
            return Err(McpError::ConfigError {
                message: "Project path is required".to_string(),
            });
        };
        
        // Create transport
        let transport = StdioTransport::new()?;
        
        // Create handler
        let handler = StdioMcpHandler::new(server);
        
        // Build event loop
        Ok(super::TransportEventLoop::new(transport).with_handler(Box::new(handler)))
    }
}

impl Default for StdioMcpServerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_stdio_transport_creation() {
        let transport = StdioTransport::new();
        assert!(transport.is_ok());
        
        let transport = transport.unwrap();
        assert!(!transport.is_active());
        assert_eq!(transport.message_count(), 0);
    }
    
    #[test]
    fn test_stdio_transport_info() {
        let transport = StdioTransport::new().unwrap();
        let info = transport.info();
        
        assert_eq!(info.transport_type, "stdio");
        assert!(info.is_bidirectional);
        assert!(info.supports_streaming);
        assert!(info.connection_info.contains("stdin/stdout"));
    }
    
    #[tokio::test]
    async fn test_transport_start_stop() {
        let mut transport = StdioTransport::new().unwrap();
        
        // Initially not active
        assert!(!transport.is_active());
        
        // Start transport
        let result = transport.start().await;
        assert!(result.is_ok(), "Failed to start transport: {:?}", result);
        assert!(transport.is_active());
        
        // Stop transport
        let result = transport.stop().await;
        assert!(result.is_ok(), "Failed to stop transport: {:?}", result);
        assert!(!transport.is_active());
    }
    
    #[tokio::test]
    async fn test_transport_double_start_error() {
        let mut transport = StdioTransport::new().unwrap();
        
        // Start once - should succeed
        assert!(transport.start().await.is_ok());
        
        // Start again - should fail
        let result = transport.start().await;
        assert!(result.is_err());
        
        if let Err(McpError::InternalError { message }) = result {
            assert!(message.contains("already active"));
        } else {
            panic!("Expected InternalError for double start");
        }
    }
    
    #[tokio::test]
    async fn test_message_response_structures() {
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
    }
    
    #[tokio::test]
    async fn test_server_builder() {
        let temp_dir = TempDir::new().unwrap();
        
        // Initialize config (required for tests)
        if let Err(_) = crate::config::Config::init() {
            // Already initialized, that's ok
        }
        
        let builder = StdioMcpServerBuilder::new()
            .with_project_path(temp_dir.path().to_path_buf());
            
        let result = builder.build().await;
        assert!(result.is_ok(), "Failed to build server: {:?}", result);
    }
    
    #[tokio::test]
    async fn test_server_builder_without_project_path() {
        let builder = StdioMcpServerBuilder::new();
        let result = builder.build().await;
        
        assert!(result.is_err());
        if let Err(McpError::ConfigError { message }) = result {
            assert!(message.contains("Project path is required"));
        } else {
            panic!("Expected ConfigError for missing project path");
        }
    }
}