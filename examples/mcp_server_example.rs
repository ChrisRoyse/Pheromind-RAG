// Example demonstrating MCP server usage with embed-search
use embed_search::mcp::{McpServer, McpConfig};
use std::path::PathBuf;
use tempfile::TempDir;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize configuration
    embed_search::config::Config::init()?;
    
    // Create temporary project directory for testing
    let temp_dir = TempDir::new()?;
    let project_path = temp_dir.path().to_path_buf();
    
    // Create a sample file to index
    let sample_file = project_path.join("sample.rs");
    tokio::fs::write(&sample_file, r#"
fn hello_world() {
    println!("Hello, world!");
}

fn fibonacci(n: u32) -> u32 {
    match n {
        0 => 0,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}
"#).await?;

    // Create MCP server
    let mut server = McpServer::with_project_path(project_path).await
        .map_err(|e| anyhow::anyhow!("Failed to create MCP server: {}", e))?;

    println!("🚀 MCP Server initialized successfully!");
    println!("Server config: {:?}", server.config());

    // Test capabilities request
    let capabilities_request = r#"{"jsonrpc":"2.0","method":"capabilities","id":1}"#;
    let capabilities_response = server.handle_request(capabilities_request).await;
    println!("\n📋 Capabilities response:");
    println!("{}", capabilities_response);

    // Test ping request
    let ping_request = r#"{"jsonrpc":"2.0","method":"ping","id":2}"#;
    let ping_response = server.handle_request(ping_request).await;
    println!("\n🏓 Ping response:");
    println!("{}", ping_response);

    // Test index request
    let index_request = format!(
        r#"{{"jsonrpc":"2.0","method":"index","params":{{"paths":["{}"]}},"id":3}}"#,
        sample_file.to_string_lossy()
    );
    let index_response = server.handle_request(&index_request).await;
    println!("\n📚 Index response:");
    println!("{}", index_response);

    // Test search request
    let search_request = r#"{"jsonrpc":"2.0","method":"search","params":{"query":"fibonacci","max_results":5},"id":4}"#;
    let search_response = server.handle_request(search_request).await;
    println!("\n🔍 Search response:");
    println!("{}", search_response);

    // Test stats request
    let stats_request = r#"{"jsonrpc":"2.0","method":"stats","id":5}"#;
    let stats_response = server.handle_request(stats_request).await;
    println!("\n📊 Stats response:");
    println!("{}", stats_response);

    // Test error handling - invalid method
    let invalid_request = r#"{"jsonrpc":"2.0","method":"invalid_method","id":6}"#;
    let error_response = server.handle_request(invalid_request).await;
    println!("\n❌ Error response (expected):");
    println!("{}", error_response);

    println!("\n✅ All MCP server tests completed successfully!");
    
    Ok(())
}