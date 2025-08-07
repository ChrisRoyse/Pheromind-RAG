# Task 011: Connection Pooling for External Services

## Objective
Implement connection pooling for external services (embedding APIs, databases) to optimize resource usage and improve performance.

## Time Estimate
10 minutes

## Priority
HIGH - Critical for external service reliability and performance

## Dependencies
- External service clients (HTTP, database)

## Implementation Steps

### 1. Create Generic Connection Pool (4 min)
```rust
// src/resources/connection_pool.rs
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use async_trait::async_trait;
use tokio::sync::Semaphore;

#[async_trait]
pub trait Connection: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;
    
    async fn is_healthy(&self) -> bool;
    async fn reset(&mut self) -> Result<(), Self::Error>;
    fn last_used(&self) -> Instant;
    fn set_last_used(&mut self, time: Instant);
    fn usage_count(&self) -> u64;
    fn increment_usage(&mut self);
}

#[derive(Debug, Clone)]
pub struct ConnectionPoolConfig {
    pub min_connections: usize,
    pub max_connections: usize,
    pub max_idle_time: Duration,
    pub connection_timeout: Duration,
    pub health_check_interval: Duration,
    pub max_lifetime: Duration,
}

impl Default for ConnectionPoolConfig {
    fn default() -> Self {
        Self {
            min_connections: 2,
            max_connections: 20,
            max_idle_time: Duration::from_secs(300),
            connection_timeout: Duration::from_secs(10),
            health_check_interval: Duration::from_secs(30),
            max_lifetime: Duration::from_secs(3600),
        }
    }
}

pub struct ConnectionPool<T: Connection> {
    config: ConnectionPoolConfig,
    connections: Arc<Mutex<VecDeque<PooledConnection<T>>>>,
    semaphore: Arc<Semaphore>,
    active_connections: Arc<Mutex<usize>>,
    factory: Arc<dyn ConnectionFactory<T>>,
    last_health_check: Arc<Mutex<Instant>>,
}

#[async_trait]
pub trait ConnectionFactory<T: Connection>: Send + Sync {
    async fn create_connection(&self) -> Result<T, Box<dyn std::error::Error + Send + Sync>>;
}

#[derive(Debug)]
struct PooledConnection<T: Connection> {
    connection: T,
    created_at: Instant,
    id: String,
}

impl<T: Connection> ConnectionPool<T> {
    pub fn new(
        config: ConnectionPoolConfig,
        factory: Arc<dyn ConnectionFactory<T>>,
    ) -> Self {
        Self {
            config,
            connections: Arc::new(Mutex::new(VecDeque::new())),
            semaphore: Arc::new(Semaphore::new(config.max_connections)),
            active_connections: Arc::new(Mutex::new(0)),
            factory,
            last_health_check: Arc::new(Mutex::new(Instant::now())),
        }
    }

    pub async fn get_connection(&self) -> Result<ConnectionGuard<T>, ConnectionPoolError> {
        // Maybe run health checks
        self.maybe_health_check().await;

        // Try to get existing connection
        if let Some(pooled_conn) = self.try_get_existing_connection().await {
            return Ok(ConnectionGuard::new(
                pooled_conn.connection,
                Arc::downgrade(&Arc::new(self.clone())),
            ));
        }

        // Create new connection if under limit
        let _permit = self.semaphore.acquire().await
            .map_err(|_| ConnectionPoolError::PoolShutdown)?;

        let connection = self.factory.create_connection().await
            .map_err(|e| ConnectionPoolError::ConnectionCreationFailed(e.to_string()))?;

        *self.active_connections.lock().unwrap() += 1;

        Ok(ConnectionGuard::new(
            connection,
            Arc::downgrade(&Arc::new(self.clone())),
        ))
    }

    async fn try_get_existing_connection(&self) -> Option<PooledConnection<T>> {
        let mut connections = self.connections.lock().unwrap();
        
        // Find a healthy connection
        for _ in 0..connections.len() {
            if let Some(mut pooled) = connections.pop_front() {
                // Check if connection is still fresh
                if pooled.created_at.elapsed() < self.config.max_lifetime {
                    if pooled.connection.is_healthy().await {
                        pooled.connection.set_last_used(Instant::now());
                        pooled.connection.increment_usage();
                        return Some(pooled);
                    }
                }
                // Connection is stale or unhealthy, don't put it back
                *self.active_connections.lock().unwrap() -= 1;
            } else {
                break;
            }
        }
        
        None
    }

    pub async fn return_connection(&self, mut connection: T) {
        // Reset connection state
        if connection.reset().await.is_ok() {
            let pooled = PooledConnection {
                connection,
                created_at: Instant::now(),
                id: uuid::Uuid::new_v4().to_string(),
            };
            
            let mut connections = self.connections.lock().unwrap();
            connections.push_back(pooled);
        } else {
            // Connection couldn't be reset, discard it
            *self.active_connections.lock().unwrap() -= 1;
        }
    }
}
```

### 2. Implement Connection Guard and Health Monitoring (3 min)
```rust
use std::sync::Weak;

pub struct ConnectionGuard<T: Connection> {
    connection: Option<T>,
    pool: Weak<ConnectionPool<T>>,
    returned: bool,
}

impl<T: Connection> ConnectionGuard<T> {
    fn new(connection: T, pool: Weak<ConnectionPool<T>>) -> Self {
        Self {
            connection: Some(connection),
            pool,
            returned: false,
        }
    }
    
    pub fn connection(&mut self) -> Option<&mut T> {
        self.connection.as_mut()
    }
    
    pub async fn return_to_pool(mut self) {
        if let Some(connection) = self.connection.take() {
            if let Some(pool) = self.pool.upgrade() {
                pool.return_connection(connection).await;
            }
            self.returned = true;
        }
    }
}

impl<T: Connection> Drop for ConnectionGuard<T> {
    fn drop(&mut self) {
        if !self.returned {
            if let Some(connection) = self.connection.take() {
                if let Some(pool) = self.pool.upgrade() {
                    // Use blocking return in destructor
                    tokio::task::block_in_place(|| {
                        tokio::runtime::Handle::current().block_on(async {
                            pool.return_connection(connection).await;
                        });
                    });
                }
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConnectionPoolError {
    #[error("Connection creation failed: {0}")]
    ConnectionCreationFailed(String),
    #[error("Pool has been shut down")]
    PoolShutdown,
    #[error("Connection timeout")]
    ConnectionTimeout,
    #[error("All connections are unhealthy")]
    NoHealthyConnections,
}

impl<T: Connection> ConnectionPool<T> {
    async fn maybe_health_check(&self) {
        let mut last_check = self.last_health_check.lock().unwrap();
        let now = Instant::now();
        
        if now.duration_since(*last_check) < self.config.health_check_interval {
            return;
        }
        
        *last_check = now;
        drop(last_check);
        
        self.health_check_connections().await;
        self.cleanup_expired_connections().await;
    }
    
    async fn health_check_connections(&self) {
        let mut connections = self.connections.lock().unwrap();
        let mut healthy_connections = VecDeque::new();
        let mut removed_count = 0;
        
        while let Some(pooled) = connections.pop_front() {
            if pooled.connection.is_healthy().await && 
               pooled.created_at.elapsed() < self.config.max_lifetime {
                healthy_connections.push_back(pooled);
            } else {
                removed_count += 1;
            }
        }
        
        *connections = healthy_connections;
        *self.active_connections.lock().unwrap() -= removed_count;
    }
    
    async fn cleanup_expired_connections(&self) {
        let mut connections = self.connections.lock().unwrap();
        let cutoff_time = Instant::now() - self.config.max_idle_time;
        let original_len = connections.len();
        
        connections.retain(|pooled| {
            pooled.connection.last_used() > cutoff_time
        });
        
        let removed = original_len - connections.len();
        *self.active_connections.lock().unwrap() -= removed;
    }
    
    pub fn get_stats(&self) -> ConnectionPoolStats {
        let connections = self.connections.lock().unwrap();
        let active = *self.active_connections.lock().unwrap();
        
        ConnectionPoolStats {
            total_connections: active,
            idle_connections: connections.len(),
            active_connections: active - connections.len(),
            max_connections: self.config.max_connections,
            utilization: active as f64 / self.config.max_connections as f64,
        }
    }
}

#[derive(Debug)]
pub struct ConnectionPoolStats {
    pub total_connections: usize,
    pub idle_connections: usize,
    pub active_connections: usize,
    pub max_connections: usize,
    pub utilization: f64,
}
```

### 3. Implement HTTP and Database Connection Types (3 min)
```rust
// src/resources/http_connection.rs
use reqwest::Client;
use crate::resources::connection_pool::Connection;
use std::time::Instant;

#[derive(Debug)]
pub struct HttpConnection {
    client: Client,
    last_used: Instant,
    usage_count: u64,
    base_url: String,
}

impl HttpConnection {
    pub fn new(base_url: String) -> Self {
        Self {
            client: Client::new(),
            last_used: Instant::now(),
            usage_count: 0,
            base_url,
        }
    }
    
    pub fn client(&self) -> &Client {
        &self.client
    }
}

#[async_trait]
impl Connection for HttpConnection {
    type Error = reqwest::Error;
    
    async fn is_healthy(&self) -> bool {
        // Simple health check - try to connect to base URL
        match self.client.head(&self.base_url).send().await {
            Ok(response) => response.status().is_success() || response.status().is_client_error(),
            Err(_) => false,
        }
    }
    
    async fn reset(&mut self) -> Result<(), Self::Error> {
        // HTTP connections don't need explicit reset
        self.last_used = Instant::now();
        Ok(())
    }
    
    fn last_used(&self) -> Instant {
        self.last_used
    }
    
    fn set_last_used(&mut self, time: Instant) {
        self.last_used = time;
    }
    
    fn usage_count(&self) -> u64 {
        self.usage_count
    }
    
    fn increment_usage(&mut self) {
        self.usage_count += 1;
    }
}

// HTTP Connection Factory
pub struct HttpConnectionFactory {
    base_url: String,
}

impl HttpConnectionFactory {
    pub fn new(base_url: String) -> Self {
        Self { base_url }
    }
}

#[async_trait]
impl ConnectionFactory<HttpConnection> for HttpConnectionFactory {
    async fn create_connection(&self) -> Result<HttpConnection, Box<dyn std::error::Error + Send + Sync>> {
        Ok(HttpConnection::new(self.base_url.clone()))
    }
}

// Integration with embedding API
pub struct PooledEmbeddingClient {
    pool: Arc<ConnectionPool<HttpConnection>>,
}

impl PooledEmbeddingClient {
    pub fn new(api_url: String, pool_config: ConnectionPoolConfig) -> Self {
        let factory = Arc::new(HttpConnectionFactory::new(api_url));
        let pool = Arc::new(ConnectionPool::new(pool_config, factory));
        
        Self { pool }
    }
    
    pub async fn get_embedding(&self, text: &str) -> Result<Vec<f32>, EmbeddingError> {
        let mut guard = self.pool.get_connection().await
            .map_err(|e| EmbeddingError::ConnectionError(e.to_string()))?;
        
        let connection = guard.connection().unwrap();
        
        let response = connection.client()
            .post(&format!("{}/embeddings", connection.base_url))
            .json(&serde_json::json!({
                "input": text,
                "model": "text-embedding-ada-002"
            }))
            .send()
            .await
            .map_err(|e| EmbeddingError::ApiError(e.to_string()))?;
        
        let embedding_response: EmbeddingResponse = response.json().await
            .map_err(|e| EmbeddingError::ParseError(e.to_string()))?;
        
        Ok(embedding_response.data[0].embedding.clone())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum EmbeddingError {
    #[error("Connection error: {0}")]
    ConnectionError(String),
    #[error("API error: {0}")]
    ApiError(String),
    #[error("Parse error: {0}")]
    ParseError(String),
}

#[derive(serde::Deserialize)]
struct EmbeddingResponse {
    data: Vec<EmbeddingData>,
}

#[derive(serde::Deserialize)]
struct EmbeddingData {
    embedding: Vec<f32>,
}
```

## Validation
- [ ] Connection pool manages resources efficiently
- [ ] Health checks remove unhealthy connections
- [ ] Connection guards automatically return to pool
- [ ] Pool statistics provide visibility
- [ ] Integration with external services works correctly

## Success Criteria
- Generic connection pool handles various connection types
- Automatic health monitoring maintains pool quality
- Resource limits prevent connection exhaustion
- Pool statistics show utilization and health
- Integration with embedding API optimizes performance

## Next Task
task_012 - Implement resource cleanup automation
