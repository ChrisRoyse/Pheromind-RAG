use std::time::Duration;
use tempfile::TempDir;
use tokio::time::timeout;

use embed_search::mcp::{McpServer, McpWatcher, McpEventType};
use embed_search::search::unified::UnifiedSearcher;
use embed_search::watcher::{FileEvent, EventType};

#[tokio::test]
async fn test_mcp_watcher_full_integration() {
    // Initialize config first
    if let Err(_) = embed_search::config::Config::init() {
        // Already initialized
    }

    let temp_dir = TempDir::new().unwrap();
    
    // Create MCP server with watcher capabilities
    let mut server = McpServer::with_project_path(temp_dir.path().to_path_buf())
        .await
        .expect("Failed to create MCP server");

    // Enable watcher via public method
    server.enable_watcher(temp_dir.path().to_path_buf()).await.expect("Failed to enable watcher");
    
    // Test capabilities show watcher support
    let capabilities_request = r#"{"jsonrpc":"2.0","method":"capabilities","id":1}"#;
    let response = server.handle_request(capabilities_request).await;
    
    let response_json: serde_json::Value = serde_json::from_str(&response).unwrap();
    assert_eq!(response_json["jsonrpc"], "2.0");
    
    // Check that file_watching is enabled in capabilities
    let file_watching = response_json["result"]["indexing"]["file_watching"].as_bool();
    assert_eq!(file_watching, Some(true));
}

#[tokio::test] 
async fn test_mcp_watcher_lifecycle() {
    if let Err(_) = embed_search::config::Config::init() {
        // Already initialized
    }

    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_path_buf();
    
    // Create UnifiedSearcher
    let searcher = std::sync::Arc::new(tokio::sync::RwLock::new(
        UnifiedSearcher::new(repo_path.clone(), repo_path.join(".embed"))
            .await
            .unwrap()
    ));

    // Create McpWatcher directly
    let watcher = McpWatcher::new(repo_path, searcher).await.unwrap();
    
    // Test initial state
    assert!(!watcher.is_active().await);
    
    // Test start
    watcher.start().await.expect("Failed to start watcher");
    assert!(watcher.is_active().await);
    
    // Test stats
    let stats = watcher.get_stats().await.expect("Failed to get stats");
    assert!(stats.is_active);
    assert_eq!(stats.active_subscribers, 0);
    
    // Test client subscription
    let mut receiver = watcher
        .subscribe_client("test_client".to_string(), None)
        .await
        .expect("Failed to subscribe client");
    
    let updated_stats = watcher.get_stats().await.expect("Failed to get updated stats");
    assert_eq!(updated_stats.active_subscribers, 1);
    
    // Test stop
    watcher.stop().await.expect("Failed to stop watcher");
    assert!(!watcher.is_active().await);
    
    // Clean up subscription
    watcher.unsubscribe_client("test_client").await.expect("Failed to unsubscribe");
}

#[tokio::test]
async fn test_mcp_watcher_event_processing() {
    if let Err(_) = embed_search::config::Config::init() {
        // Already initialized
    }

    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_path_buf();
    
    let searcher = std::sync::Arc::new(tokio::sync::RwLock::new(
        UnifiedSearcher::new(repo_path.clone(), repo_path.join(".embed"))
            .await
            .unwrap()
    ));

    let watcher = McpWatcher::new(repo_path.clone(), searcher).await.unwrap();
    
    // Subscribe to events before processing
    let mut receiver = watcher
        .subscribe_client("test_client".to_string(), None)
        .await
        .expect("Failed to subscribe");
    
    // Create test file
    let test_file = repo_path.join("test.rs");
    std::fs::write(&test_file, "fn main() { println!(\"Hello, world!\"); }").unwrap();
    
    // Process file event
    let file_event = FileEvent::new(test_file.clone(), EventType::Modified);
    watcher.process_file_event(file_event).await.expect("Failed to process event");
    
    // Try to receive the event (with timeout)
    let result = timeout(Duration::from_millis(100), receiver.recv()).await;
    assert!(result.is_ok(), "Should receive event within timeout");
    
    let event = result.unwrap().expect("Should receive valid event");
    assert!(matches!(event.event_type, McpEventType::FileModified));
    assert!(event.file_path.ends_with("test.rs"));
    assert!(event.index_updated);
    
    // Check that affected backends include expected ones
    assert!(event.affected_backends.contains(&"bm25".to_string()));
}

#[tokio::test]
async fn test_mcp_watcher_manual_update() {
    if let Err(_) = embed_search::config::Config::init() {
        // Already initialized
    }

    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_path_buf();
    
    let searcher = std::sync::Arc::new(tokio::sync::RwLock::new(
        UnifiedSearcher::new(repo_path.clone(), repo_path.join(".embed"))
            .await
            .unwrap()
    ));

    let watcher = McpWatcher::new(repo_path, searcher).await.unwrap();
    
    // Start watcher first
    watcher.start().await.expect("Failed to start watcher");
    
    // Subscribe to events
    let mut receiver = watcher
        .subscribe_client("test_client".to_string(), None)
        .await
        .expect("Failed to subscribe");
    
    // Trigger manual update
    let update_result = watcher.trigger_manual_update().await;
    assert!(update_result.is_ok(), "Manual update should succeed");
    
    // Should receive batch update events
    let start_event_result = timeout(Duration::from_millis(100), receiver.recv()).await;
    assert!(start_event_result.is_ok(), "Should receive start event");
    
    let start_event = start_event_result.unwrap().unwrap();
    assert!(matches!(start_event.event_type, McpEventType::BatchUpdateStarted));
    
    let completion_event_result = timeout(Duration::from_millis(100), receiver.recv()).await;
    assert!(completion_event_result.is_ok(), "Should receive completion event");
    
    let completion_event = completion_event_result.unwrap().unwrap();
    assert!(matches!(completion_event.event_type, McpEventType::BatchUpdateCompleted));
    
    watcher.stop().await.expect("Failed to stop watcher");
}

#[tokio::test]
async fn test_mcp_watcher_error_handling() {
    if let Err(_) = embed_search::config::Config::init() {
        // Already initialized
    }

    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_path_buf();
    
    let searcher = std::sync::Arc::new(tokio::sync::RwLock::new(
        UnifiedSearcher::new(repo_path.clone(), repo_path.join(".embed"))
            .await
            .unwrap()
    ));

    let watcher = McpWatcher::new(repo_path, searcher).await.unwrap();
    
    // Test starting watcher twice should fail
    watcher.start().await.expect("First start should succeed");
    let second_start = watcher.start().await;
    assert!(second_start.is_err(), "Second start should fail");
    
    // Test stopping inactive watcher should fail (after stopping once)
    watcher.stop().await.expect("First stop should succeed");
    let second_stop = watcher.stop().await;
    assert!(second_stop.is_err(), "Second stop should fail");
    
    // Test operations on stopped watcher
    let manual_update = watcher.trigger_manual_update().await;
    assert!(manual_update.is_err(), "Manual update on stopped watcher should fail");
}

#[tokio::test]
async fn test_mcp_watcher_subscription_management() {
    if let Err(_) = embed_search::config::Config::init() {
        // Already initialized
    }

    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_path_buf();
    
    let searcher = std::sync::Arc::new(tokio::sync::RwLock::new(
        UnifiedSearcher::new(repo_path.clone(), repo_path.join(".embed"))
            .await
            .unwrap()
    ));

    let watcher = McpWatcher::new(repo_path, searcher).await.unwrap();
    
    // Test initial state
    let initial_stats = watcher.get_stats().await.expect("Failed to get stats");
    assert_eq!(initial_stats.active_subscribers, 0);
    
    // Subscribe multiple clients
    let _receiver1 = watcher.subscribe_client("client1".to_string(), None).await.unwrap();
    let _receiver2 = watcher.subscribe_client("client2".to_string(), None).await.unwrap();
    
    let stats_after_subscription = watcher.get_stats().await.unwrap();
    assert_eq!(stats_after_subscription.active_subscribers, 2);
    
    // Unsubscribe one client
    watcher.unsubscribe_client("client1").await.expect("Failed to unsubscribe client1");
    
    let stats_after_unsubscribe = watcher.get_stats().await.unwrap();
    assert_eq!(stats_after_unsubscribe.active_subscribers, 1);
    
    // Try to unsubscribe non-existent client
    let invalid_unsubscribe = watcher.unsubscribe_client("non_existent").await;
    assert!(invalid_unsubscribe.is_err(), "Unsubscribing non-existent client should fail");
}