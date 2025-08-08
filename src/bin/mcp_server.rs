/// MCP Server Binary - Complete stdio-based MCP server implementation
/// 
/// This binary creates a fully functional MCP server that communicates over
/// stdin/stdout using line-delimited JSON-RPC messages.
/// 
/// Usage:
///   cargo run --bin mcp_server -- [project_path]
///   
/// The server will:
/// 1. Initialize the embed-search system for the given project
/// 2. Set up stdio transport for JSON-RPC communication  
/// 3. Handle MCP protocol methods (search, index, stats, etc.)
/// 4. Run until stdin is closed or shutdown is requested

use std::path::PathBuf;
use std::env;
use clap::Parser;
use tokio::signal;

use embed_search::config::Config;
use embed_search::mcp::{
    StdioMcpServerBuilder, McpResult,
    McpConfig,
};

#[derive(Parser, Debug)]
#[command(name = "mcp-server")]
#[command(about = "MCP server for embed-search system")]
#[command(version = "0.1.0")]
struct Args {
    /// Project directory to index and search
    #[arg(help = "Path to project directory")]
    project_path: Option<PathBuf>,
    
    /// Server name (defaults to embed-search-mcp)
    #[arg(long, default_value = "embed-search-mcp")]
    server_name: String,
    
    /// Enable debug logging
    #[arg(long, short, action)]
    debug: bool,
    
    /// Maximum concurrent requests
    #[arg(long, default_value_t = 100)]
    max_requests: u32,
    
    /// Request timeout in milliseconds
    #[arg(long, default_value_t = 30000)]
    timeout_ms: u64,
    
    /// Disable indexing (search-only mode)
    #[arg(long, action)]
    no_indexing: bool,
    
    /// Disable statistics collection
    #[arg(long, action)]
    no_stats: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    
    // Initialize logging
    setup_logging(args.debug)?;
    
    log::info!("Starting MCP server for embed-search");
    log::info!("Server name: {}", args.server_name);
    
    // Initialize embed-search config
    if let Err(e) = Config::init() {
        log::warn!("Config initialization failed, using defaults: {}", e);
    }
    
    // Determine project path
    let project_path = args.project_path
        .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    
    log::info!("Using project path: {}", project_path.display());
    
    // Validate project path
    if !project_path.exists() {
        eprintln!("Error: Project path does not exist: {}", project_path.display());
        std::process::exit(1);
    }
    
    if !project_path.is_dir() {
        eprintln!("Error: Project path is not a directory: {}", project_path.display());
        std::process::exit(1);
    }
    
    // Create MCP configuration
    let mcp_config = McpConfig::new_test_config();
    // Override with command-line arguments
    let mcp_config = McpConfig {
        server_name: args.server_name.clone(),
        server_version: "0.1.0".to_string(),
        server_description: "Embed-search MCP server".to_string(),
        transport: mcp_config.transport,
        tools: embed_search::mcp::McpToolsConfig {
            enable_search: true,
            enable_index: !args.no_indexing,
            enable_status: !args.no_stats,
            enable_clear: true,
            enable_orchestrated_search: true,
            max_results_per_call: 100,
            default_search_timeout_ms: args.timeout_ms,
            max_concurrent_operations: args.max_requests as usize,
        },
        performance: embed_search::mcp::McpPerformanceConfig {
            max_concurrent_requests: args.max_requests,
            request_timeout_ms: args.timeout_ms,
            max_request_size_bytes: 1048576,
            max_response_size_bytes: 10485760,
            enable_metrics: !args.no_stats,
            metrics_interval_secs: 30,
        },
        #[cfg(feature = "ml")]
        embedding: mcp_config.embedding,
        security: mcp_config.security,
        mcp_log_level: if args.debug { "debug".to_string() } else { "info".to_string() },
        enable_request_logging: true,
        enable_performance_logging: !args.no_stats,
    };
    
    log::info!("Configuration: indexing={}, stats={}, max_requests={}, timeout={}ms", 
        mcp_config.tools.enable_index, 
        mcp_config.tools.enable_status,
        mcp_config.performance.max_concurrent_requests,
        mcp_config.performance.request_timeout_ms);
    
    // Build and start the MCP server
    let mut event_loop = match create_mcp_server(project_path, mcp_config).await {
        Ok(server) => server,
        Err(e) => {
            eprintln!("Failed to create MCP server: {}", e);
            std::process::exit(1);
        }
    };
    
    log::info!("MCP server initialized successfully");
    log::info!("Ready to accept JSON-RPC requests on stdin/stdout");
    
    // Set up graceful shutdown
    let shutdown_handle = tokio::spawn(async {
        match signal::ctrl_c().await {
            Ok(()) => {
                log::info!("Received Ctrl+C, initiating graceful shutdown");
            }
            Err(err) => {
                log::error!("Failed to listen for shutdown signal: {}", err);
            }
        }
    });
    
    // Run the server event loop
    let server_result = tokio::select! {
        result = event_loop.run() => {
            match result {
                Ok(()) => {
                    log::info!("MCP server event loop completed successfully");
                    Ok(())
                }
                Err(e) => {
                    log::error!("MCP server event loop error: {}", e);
                    Err(e)
                }
            }
        }
        _ = shutdown_handle => {
            log::info!("Shutdown signal received, stopping server");
            Ok(())
        }
    };
    
    match server_result {
        Ok(()) => {
            log::info!("MCP server shutdown complete");
            Ok(())
        }
        Err(e) => {
            log::error!("MCP server error: {}", e);
            Err(e.into())
        }
    }
}

/// Create and configure the MCP server
async fn create_mcp_server(
    project_path: PathBuf, 
    config: McpConfig
) -> McpResult<embed_search::mcp::TransportEventLoop<embed_search::mcp::StdioTransport>> {
    log::debug!("Creating MCP server for project: {}", project_path.display());
    
    let server_builder = StdioMcpServerBuilder::new()
        .with_project_path(project_path)
        .with_config(config);
    
    server_builder.build().await
}

/// Set up logging configuration
fn setup_logging(debug: bool) -> Result<(), Box<dyn std::error::Error>> {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
    
    let env_filter = if debug {
        EnvFilter::new("debug")
    } else {
        EnvFilter::try_from_default_env()
            .or_else(|_| EnvFilter::try_new("info"))?
    };
    
    // Use stderr for logs so stdout is clean for JSON-RPC
    tracing_subscriber::registry()
        .with(env_filter)
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(std::io::stderr)
                .with_target(false)
                .with_thread_ids(false)
                .with_level(true)
                .compact()
        )
        .init();
    
    Ok(())
}

/// Print server capabilities and usage information
#[allow(dead_code)]
fn print_capabilities() {
    eprintln!("MCP Server Capabilities:");
    eprintln!("  - JSON-RPC 2.0 over stdin/stdout");
    eprintln!("  - Line-delimited messages");
    eprintln!();
    eprintln!("Supported Methods:");
    eprintln!("  - initialize: Get server capabilities");
    eprintln!("  - search: Perform semantic/text search");
    eprintln!("  - index: Index files or directories");
    eprintln!("  - stats: Get server and index statistics");
    eprintln!("  - clear: Clear index data");
    eprintln!("  - capabilities: Get detailed capabilities");
    eprintln!("  - ping: Health check");
    eprintln!("  - shutdown: Graceful server shutdown");
    eprintln!();
    eprintln!("Features Available:");
    #[cfg(feature = "ml")]
    eprintln!("  ✓ Semantic search (ML embeddings)");
    #[cfg(not(feature = "ml"))]
    eprintln!("  ✗ Semantic search (disabled - enable 'ml' feature)");
    
    #[cfg(feature = "tantivy")]
    eprintln!("  ✓ Full-text search with Tantivy");
    #[cfg(not(feature = "tantivy"))]
    eprintln!("  ✗ Full-text search (disabled - enable 'tantivy' feature)");
    
    #[cfg(feature = "tree-sitter")]
    eprintln!("  ✓ Symbol extraction and search");
    #[cfg(not(feature = "tree-sitter"))]
    eprintln!("  ✗ Symbol search (disabled - enable 'tree-sitter' feature)");
    
    eprintln!("  ✓ BM25 statistical search (always available)");
    
    #[cfg(feature = "vectordb")]
    eprintln!("  ✓ Vector database storage");
    #[cfg(not(feature = "vectordb"))]
    eprintln!("  ✗ Vector database (disabled - enable 'vectordb' feature)");
    
    eprintln!();
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_server_creation() {
        let temp_dir = TempDir::new().unwrap();
        
        // Initialize config
        if let Err(_) = Config::init() {
            // Already initialized
        }
        
        let config = McpConfig::new_test_config();
        let result = create_mcp_server(temp_dir.path().to_path_buf(), config).await;
        
        assert!(result.is_ok(), "Failed to create server: {:?}", result);
    }
    
    #[test]
    fn test_args_parsing() {
        // Test default args
        let args = Args::try_parse_from(&["mcp_server"]).unwrap();
        assert_eq!(args.server_name, "embed-search-mcp");
        assert_eq!(args.max_requests, 100);
        assert!(!args.debug);
        assert!(!args.no_indexing);
        assert!(!args.no_stats);
    }
    
    #[test]
    fn test_args_parsing_with_options() {
        let args = Args::try_parse_from(&[
            "mcp_server",
            "/tmp/project", 
            "--debug",
            "--server-name", "test-server",
            "--max-requests", "50",
            "--no-indexing"
        ]).unwrap();
        
        assert_eq!(args.project_path, Some(PathBuf::from("/tmp/project")));
        assert_eq!(args.server_name, "test-server");
        assert_eq!(args.max_requests, 50);
        assert!(args.debug);
        assert!(args.no_indexing);
        assert!(!args.no_stats);
    }
}