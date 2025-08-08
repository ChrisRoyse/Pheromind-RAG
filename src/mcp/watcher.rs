use std::sync::Arc;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::{broadcast};
use std::sync::{Arc as StdArc, RwLock as StdRwLock};
use serde::{Serialize, Deserialize};

use crate::watcher::{GitWatcher, FileEvent, EventType};
use crate::search::unified::UnifiedSearcher;
use crate::mcp::error::{McpError, McpResult};

/// MCP-specific watcher event that includes additional metadata for clients
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpWatcherEvent {
    pub event_id: String,
    pub file_path: String,
    pub event_type: McpEventType,
    pub timestamp: u64,
    pub index_updated: bool,
    pub affected_backends: Vec<String>,
    pub file_size: Option<u64>,
    pub processing_time_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum McpEventType {
    FileCreated,
    FileModified, 
    FileDeleted,
    IndexUpdated,
    BatchUpdateStarted,
    BatchUpdateCompleted,
    WatcherError,
}

/// MCP integration layer for the GitWatcher
pub struct McpWatcher {
    git_watcher: Arc<tokio::sync::RwLock<GitWatcher>>,
    searcher: Arc<tokio::sync::RwLock<UnifiedSearcher>>,
    event_broadcaster: broadcast::Sender<McpWatcherEvent>,
    event_receiver: broadcast::Receiver<McpWatcherEvent>,
    is_active: Arc<tokio::sync::Mutex<bool>>,
    watched_path: PathBuf,
    event_counter: Arc<tokio::sync::Mutex<u64>>,
    client_subscribers: Arc<tokio::sync::Mutex<Vec<ClientSubscription>>>,
}

#[derive(Debug, Clone)]
struct ClientSubscription {
    client_id: String,
    event_filter: Option<EventFilter>,
    created_at: SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventFilter {
    pub file_patterns: Option<Vec<String>>,
    pub event_types: Option<Vec<McpEventType>>,
    pub min_file_size: Option<u64>,
    pub max_file_size: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatcherStats {
    pub is_active: bool,
    pub watched_path: String,
    pub events_processed: u64,
    pub active_subscribers: u32,
    pub last_event_time: Option<u64>,
    pub error_count: u32,
}

impl McpWatcher {
    /// Create new MCP watcher integration with existing GitWatcher
    pub async fn new(
        repo_path: PathBuf,
        searcher: Arc<tokio::sync::RwLock<UnifiedSearcher>>,
    ) -> McpResult<Self> {
        // Create the underlying GitWatcher
        // GitWatcher expects std::RwLock, but we work with tokio::RwLock
        // For now, create a new UnifiedSearcher instance for the GitWatcher
        // This is not ideal but necessary due to the RwLock type mismatch
        let searcher_for_git_watcher = {
            let db_path = repo_path.join(".embed-git-watcher");
            let new_searcher = UnifiedSearcher::new(repo_path.clone(), db_path)
                .await
                .map_err(|e| McpError::InternalError {
                    message: format!("Failed to create searcher for GitWatcher: {}", e),
                })?;
            Arc::new(std::sync::RwLock::new(new_searcher))
        };
        
        let git_watcher = GitWatcher::new(&repo_path, searcher_for_git_watcher)
            .map_err(|e| McpError::InternalError {
                message: format!("Failed to create GitWatcher: {}", e),
            })?;

        let (event_tx, event_rx) = broadcast::channel(1000);
        
        Ok(Self {
            git_watcher: Arc::new(tokio::sync::RwLock::new(git_watcher)),
            searcher,
            event_broadcaster: event_tx,
            event_receiver: event_rx,
            is_active: Arc::new(tokio::sync::Mutex::new(false)),
            watched_path: repo_path,
            event_counter: Arc::new(tokio::sync::Mutex::new(0)),
            client_subscribers: Arc::new(tokio::sync::Mutex::new(Vec::new())),
        })
    }

    /// Start the MCP watcher integration
    pub async fn start(&self) -> McpResult<()> {
        let mut is_active = self.is_active.lock().await;
        if *is_active {
            return Err(McpError::InvalidRequest {
                message: "MCP watcher is already active".to_string(),
            });
        }

        // Start the underlying GitWatcher
        {
            let mut watcher = self.git_watcher.write().await;
            watcher.start_watching()
                .map_err(|e| McpError::InternalError {
                    message: format!("Failed to start GitWatcher: {}", e),
                })?;
        }

        *is_active = true;

        // Broadcast start event
        let start_event = McpWatcherEvent {
            event_id: self.generate_event_id().await,
            file_path: self.watched_path.to_string_lossy().to_string(),
            event_type: McpEventType::BatchUpdateStarted,
            timestamp: current_timestamp(),
            index_updated: false,
            affected_backends: vec!["git_watcher".to_string()],
            file_size: None,
            processing_time_ms: None,
        };

        // Send event to subscribers (ignore if no subscribers)
        let _ = self.event_broadcaster.send(start_event);

        log::info!("MCP watcher started for path: {:?}", self.watched_path);
        Ok(())
    }

    /// Stop the MCP watcher integration
    pub async fn stop(&self) -> McpResult<()> {
        let mut is_active = self.is_active.lock().await;
        if !*is_active {
            return Err(McpError::InvalidRequest {
                message: "MCP watcher is not active".to_string(),
            });
        }

        // Stop the underlying GitWatcher
        {
            let mut watcher = self.git_watcher.write().await;
            watcher.stop_watching();
        }

        *is_active = false;

        // Broadcast stop event
        let stop_event = McpWatcherEvent {
            event_id: self.generate_event_id().await,
            file_path: self.watched_path.to_string_lossy().to_string(),
            event_type: McpEventType::BatchUpdateCompleted,
            timestamp: current_timestamp(),
            index_updated: false,
            affected_backends: vec!["git_watcher".to_string()],
            file_size: None,
            processing_time_ms: None,
        };

        // Send event to subscribers (ignore if no subscribers)
        let _ = self.event_broadcaster.send(stop_event);

        log::info!("MCP watcher stopped for path: {:?}", self.watched_path);
        Ok(())
    }

    /// Subscribe a client to watcher events
    pub async fn subscribe_client(
        &self,
        client_id: String,
        event_filter: Option<EventFilter>,
    ) -> McpResult<broadcast::Receiver<McpWatcherEvent>> {
        let subscription = ClientSubscription {
            client_id: client_id.clone(),
            event_filter,
            created_at: SystemTime::now(),
        };

        let mut subscribers = self.client_subscribers.lock().await;
        subscribers.push(subscription);
        
        let receiver = self.event_broadcaster.subscribe();
        
        log::debug!("Client {} subscribed to MCP watcher events", client_id);
        Ok(receiver)
    }

    /// Unsubscribe a client from watcher events
    pub async fn unsubscribe_client(&self, client_id: &str) -> McpResult<()> {
        let mut subscribers = self.client_subscribers.lock().await;
        let initial_count = subscribers.len();
        subscribers.retain(|sub| sub.client_id != client_id);
        
        if subscribers.len() == initial_count {
            return Err(McpError::InvalidRequest {
                message: format!("Client {} is not subscribed", client_id),
            });
        }

        log::debug!("Client {} unsubscribed from MCP watcher events", client_id);
        Ok(())
    }

    /// Process a file event from the underlying GitWatcher and broadcast to MCP clients
    pub async fn process_file_event(&self, file_event: FileEvent) -> McpResult<()> {
        let start_time = std::time::Instant::now();
        
        // Convert FileEvent to McpWatcherEvent
        let event_type = match file_event.event_type {
            EventType::Created => McpEventType::FileCreated,
            EventType::Modified => McpEventType::FileModified,
            EventType::Removed => McpEventType::FileDeleted,
        };

        // Get file metadata if file exists
        let file_size = if file_event.path.exists() {
            std::fs::metadata(&file_event.path)
                .ok()
                .map(|m| m.len())
        } else {
            None
        };

        // Determine which backends will be affected
        let mut affected_backends = Vec::new();
        
        // Always affects BM25 (basic search)
        affected_backends.push("bm25".to_string());
        
        // Check for other backends based on compile-time features
        #[cfg(feature = "tantivy")]
        if file_event.is_code_file() {
            affected_backends.push("tantivy".to_string());
        }
        
        #[cfg(feature = "vectordb")]
        if file_event.is_code_file() {
            affected_backends.push("lancedb".to_string());
        }

        let processing_time = start_time.elapsed();

        let mcp_event = McpWatcherEvent {
            event_id: self.generate_event_id().await,
            file_path: file_event.path.to_string_lossy().to_string(),
            event_type,
            timestamp: file_event.timestamp
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            index_updated: true, // Assume index will be updated
            affected_backends,
            file_size,
            processing_time_ms: Some(processing_time.as_millis() as u64),
        };

        // Broadcast to all subscribers (ignore if no subscribers)
        let _ = self.event_broadcaster.send(mcp_event);

        // Increment event counter
        let mut counter = self.event_counter.lock().await;
        *counter += 1;

        Ok(())
    }

    /// Force a manual index update and notify MCP clients
    pub async fn trigger_manual_update(&self) -> McpResult<()> {
        // Check if watcher is active
        let is_active = *self.is_active.lock().await;
        if !is_active {
            return Err(McpError::InvalidRequest {
                message: "Cannot trigger manual update - watcher is not active".to_string(),
            });
        }

        let start_time = std::time::Instant::now();

        // Create batch update started event
        let start_event = McpWatcherEvent {
            event_id: self.generate_event_id().await,
            file_path: self.watched_path.to_string_lossy().to_string(),
            event_type: McpEventType::BatchUpdateStarted,
            timestamp: current_timestamp(),
            index_updated: false,
            affected_backends: vec!["manual_trigger".to_string()],
            file_size: None,
            processing_time_ms: None,
        };

        let _ = self.event_broadcaster.send(start_event);

        // Trigger actual index update through UnifiedSearcher
        let result = {
            let searcher = self.searcher.write().await;
            searcher.index_directory(&self.watched_path).await
        };

        let processing_time = start_time.elapsed();

        // Create completion event and handle result
        match &result {
            Ok(_stats) => {
                let completion_event = McpWatcherEvent {
                    event_id: self.generate_event_id().await,
                    file_path: self.watched_path.to_string_lossy().to_string(),
                    event_type: McpEventType::BatchUpdateCompleted,
                    timestamp: current_timestamp(),
                    index_updated: true,
                    affected_backends: vec!["unified_searcher".to_string()],
                    file_size: None,
                    processing_time_ms: Some(processing_time.as_millis() as u64),
                };
                let _ = self.event_broadcaster.send(completion_event);
            },
            Err(_) => {
                let error_event = McpWatcherEvent {
                    event_id: self.generate_event_id().await,
                    file_path: self.watched_path.to_string_lossy().to_string(),
                    event_type: McpEventType::WatcherError,
                    timestamp: current_timestamp(),
                    index_updated: false,
                    affected_backends: vec!["unified_searcher".to_string()],
                    file_size: None,
                    processing_time_ms: Some(processing_time.as_millis() as u64),
                };
                let _ = self.event_broadcaster.send(error_event);
            }
        }

        result.map_err(|e| McpError::InternalError {
            message: format!("Manual update failed: {}", e),
        })?;

        Ok(())
    }

    /// Get current watcher statistics for MCP clients
    pub async fn get_stats(&self) -> McpResult<WatcherStats> {
        let is_active = *self.is_active.lock().await;
        let event_count = *self.event_counter.lock().await;
        let subscriber_count = self.client_subscribers.lock().await.len() as u32;
        
        let error_count = {
            let watcher = self.git_watcher.read().await;
            watcher.get_error_count()
        };

        Ok(WatcherStats {
            is_active,
            watched_path: self.watched_path.to_string_lossy().to_string(),
            events_processed: event_count,
            active_subscribers: subscriber_count,
            last_event_time: None, // Would need to track this separately
            error_count,
        })
    }

    /// Generate unique event ID
    async fn generate_event_id(&self) -> String {
        let counter = {
            let mut c = self.event_counter.lock().await;
            *c += 1;
            *c
        };
        
        format!("mcp_watcher_{}_{}", 
                current_timestamp(), 
                counter)
    }

    /// Check if watcher is currently active
    pub async fn is_active(&self) -> bool {
        *self.is_active.lock().await
    }

    /// Reset error count on the underlying GitWatcher
    pub async fn reset_error_count(&self) -> McpResult<()> {
        let watcher = self.git_watcher.read().await;
        watcher.reset_error_count();
        Ok(())
    }
}

/// Helper function to get current Unix timestamp
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::time::Duration;
    use tokio::time::timeout;

    #[tokio::test]
    async fn test_mcp_watcher_creation() {
        let temp_dir = TempDir::new().unwrap();
        
        // Initialize config first
        if let Err(_) = crate::config::Config::init() {
            // Config already initialized
        }
        
        let searcher = Arc::new(tokio::sync::RwLock::new(
            UnifiedSearcher::new(temp_dir.path().to_path_buf(), temp_dir.path().join(".embed"))
                .await
                .unwrap()
        ));

        let watcher = McpWatcher::new(temp_dir.path().to_path_buf(), searcher).await;
        assert!(watcher.is_ok());
    }

    #[tokio::test]
    async fn test_client_subscription() {
        let temp_dir = TempDir::new().unwrap();
        
        if let Err(_) = crate::config::Config::init() {
            // Config already initialized
        }
        
        let searcher = Arc::new(tokio::sync::RwLock::new(
            UnifiedSearcher::new(temp_dir.path().to_path_buf(), temp_dir.path().join(".embed"))
                .await
                .unwrap()
        ));

        let watcher = McpWatcher::new(temp_dir.path().to_path_buf(), searcher).await.unwrap();
        
        let result = watcher.subscribe_client("test_client".to_string(), None).await;
        assert!(result.is_ok());

        let stats = watcher.get_stats().await.unwrap();
        assert_eq!(stats.active_subscribers, 1);
        
        watcher.unsubscribe_client("test_client").await.unwrap();
        let stats = watcher.get_stats().await.unwrap();
        assert_eq!(stats.active_subscribers, 0);
    }

    #[tokio::test]
    async fn test_watcher_lifecycle() {
        let temp_dir = TempDir::new().unwrap();
        
        if let Err(_) = crate::config::Config::init() {
            // Config already initialized
        }
        
        let searcher = Arc::new(tokio::sync::RwLock::new(
            UnifiedSearcher::new(temp_dir.path().to_path_buf(), temp_dir.path().join(".embed"))
                .await
                .unwrap()
        ));

        let watcher = McpWatcher::new(temp_dir.path().to_path_buf(), searcher).await.unwrap();
        
        assert!(!watcher.is_active().await);
        
        let result = watcher.start().await;
        assert!(result.is_ok());
        assert!(watcher.is_active().await);
        
        let result = watcher.stop().await;
        assert!(result.is_ok());
        assert!(!watcher.is_active().await);
    }

    #[tokio::test]
    async fn test_event_broadcasting() {
        let temp_dir = TempDir::new().unwrap();
        
        if let Err(_) = crate::config::Config::init() {
            // Config already initialized
        }
        
        let searcher = Arc::new(tokio::sync::RwLock::new(
            UnifiedSearcher::new(temp_dir.path().to_path_buf(), temp_dir.path().join(".embed"))
                .await
                .unwrap()
        ));

        let watcher = McpWatcher::new(temp_dir.path().to_path_buf(), searcher).await.unwrap();
        
        let mut receiver = watcher.subscribe_client("test_client".to_string(), None).await.unwrap();
        
        // Create a test file event
        let test_file = temp_dir.path().join("test.rs");
        let file_event = FileEvent::new(test_file, EventType::Modified);
        
        // Process the event in a separate task
        let watcher_clone = watcher.clone();
        tokio::spawn(async move {
            watcher_clone.process_file_event(file_event).await.unwrap();
        });
        
        // Try to receive the event with a timeout
        let result = timeout(Duration::from_millis(100), receiver.recv()).await;
        assert!(result.is_ok());
        
        let event = result.unwrap().unwrap();
        assert!(matches!(event.event_type, McpEventType::FileModified));
        assert!(event.file_path.ends_with("test.rs"));
    }
}

// Need Clone implementation for the async test
impl Clone for McpWatcher {
    fn clone(&self) -> Self {
        Self {
            git_watcher: self.git_watcher.clone(),
            searcher: self.searcher.clone(),
            event_broadcaster: self.event_broadcaster.clone(),
            event_receiver: self.event_broadcaster.subscribe(),
            is_active: self.is_active.clone(),
            watched_path: self.watched_path.clone(),
            event_counter: self.event_counter.clone(),
            client_subscribers: self.client_subscribers.clone(),
        }
    }
}