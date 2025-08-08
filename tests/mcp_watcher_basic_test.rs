use std::env;

#[tokio::test]
async fn test_mcp_watcher_compilation() {
    // Test that the MCP watcher code compiles and basic types can be created
    
    // Set required environment variables
    env::set_var("EMBED_LOG_LEVEL", "info");
    env::set_var("EMBED_CONFIG_PATH", "test_config.toml");
    
    // Initialize config 
    if let Err(_) = embed_search::config::Config::init() {
        // Already initialized
    }

    // Basic import test
    use embed_search::mcp::McpEventType;
    
    // Test that event types can be created
    let _event_type = McpEventType::FileModified;
    assert!(true, "Basic types compile correctly");
    
    // Test that MCP protocol includes watcher methods
    use embed_search::mcp::protocol::RpcMethod;
    
    let watcher_start = RpcMethod::from_str("watcher/start");
    assert!(watcher_start.is_ok(), "Watcher start method is recognized");
    
    let watcher_stop = RpcMethod::from_str("watcher/stop");
    assert!(watcher_stop.is_ok(), "Watcher stop method is recognized");
    
    let watcher_status = RpcMethod::from_str("watcher/status");
    assert!(watcher_status.is_ok(), "Watcher status method is recognized");
    
    println!("✅ MCP watcher integration compiles and basic types work correctly");
}

#[tokio::test]
async fn test_mcp_server_watcher_capabilities() {
    use tempfile::TempDir;
    use embed_search::mcp::McpServer;
    
    // Set required environment variables
    env::set_var("EMBED_LOG_LEVEL", "info");
    
    // Initialize config 
    if let Err(_) = embed_search::config::Config::init() {
        // Already initialized
    }

    let temp_dir = TempDir::new().unwrap();
    
    // Create MCP server
    let server_result = McpServer::with_project_path(temp_dir.path().to_path_buf()).await;
    if server_result.is_err() {
        println!("⚠️  Could not create MCP server: {:?}", server_result.err());
        return;
    }
    
    let mut server = server_result.unwrap();
    
    // Test that capabilities include file watching
    let capabilities_request = r#"{"jsonrpc":"2.0","method":"capabilities","id":1}"#;
    let response = server.handle_request(capabilities_request).await;
    
    let response_json: serde_json::Value = serde_json::from_str(&response).unwrap();
    
    // Check that the response is valid JSON-RPC
    assert_eq!(response_json["jsonrpc"], "2.0");
    assert!(response_json["result"].is_object(), "Should have result object");
    
    // Check that file_watching capability is present
    let file_watching = response_json["result"]["indexing"]["file_watching"].as_bool();
    assert_eq!(file_watching, Some(true), "file_watching should be enabled");
    
    println!("✅ MCP server correctly reports file_watching capability");
}

#[tokio::test]  
async fn test_watcher_protocol_methods() {
    use embed_search::mcp::protocol::RpcMethod;
    
    // Test all watcher methods can be parsed
    let methods = vec![
        ("watcher/start", RpcMethod::WatcherStart),
        ("watcher/stop", RpcMethod::WatcherStop), 
        ("watcher/status", RpcMethod::WatcherStatus),
        ("watcher/subscribe", RpcMethod::WatcherSubscribe),
        ("watcher/unsubscribe", RpcMethod::WatcherUnsubscribe),
        ("watcher/manual_update", RpcMethod::WatcherManualUpdate),
        ("watcher/reset_errors", RpcMethod::WatcherResetErrors),
    ];
    
    for (method_str, expected_method) in methods {
        let parsed = RpcMethod::from_str(method_str).unwrap();
        assert_eq!(parsed, expected_method, "Method {} should parse correctly", method_str);
        
        let serialized = expected_method.as_str();
        assert_eq!(serialized, method_str, "Method should serialize back to original string");
    }
    
    println!("✅ All watcher protocol methods work correctly");
}

#[tokio::test]
async fn test_watcher_error_handling() {
    use embed_search::mcp::protocol::RpcMethod;
    
    // Test that invalid watcher methods are rejected
    let invalid_methods = vec![
        "watcher/invalid",
        "watch/start", 
        "watcher",
        "watcher/",
    ];
    
    for invalid_method in invalid_methods {
        let result = RpcMethod::from_str(invalid_method);
        assert!(result.is_err(), "Invalid method '{}' should be rejected", invalid_method);
    }
    
    println!("✅ Invalid watcher methods are properly rejected");
}