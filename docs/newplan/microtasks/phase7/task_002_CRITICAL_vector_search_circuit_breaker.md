# Task 002: CRITICAL - Apply Circuit Breaker to Vector Search

## Objective
Integrate circuit breaker pattern with vector search method to prevent embedding API failures from cascading.

## Time Estimate
10 minutes

## Priority
CRITICAL - Vector search is most failure-prone due to external API dependency

## Dependencies
- task_001 - Circuit breaker pattern implemented
- Vector search method must exist

## Implementation Steps

### 1. Wrap Vector Search with Circuit Breaker (4 min)
```rust
// src/search/vector.rs
use crate::production::circuit_breaker::{CircuitBreaker, CircuitBreakerError};
use std::time::Duration;
use std::sync::Arc;

pub struct VectorSearchWithCircuitBreaker {
    inner: VectorSearch,
    circuit_breaker: Arc<CircuitBreaker>,
}

impl VectorSearchWithCircuitBreaker {
    pub fn new(vector_search: VectorSearch) -> Self {
        let circuit_breaker = Arc::new(CircuitBreaker::new(
            5, // failure threshold
            Duration::from_secs(30), // timeout
        ));
        
        Self {
            inner: vector_search,
            circuit_breaker,
        }
    }

    pub async fn search(&self, query: &str) -> Result<Vec<SearchResult>, VectorSearchError> {
        self.circuit_breaker
            .call(async {
                self.inner.search(query).await
            })
            .await
            .map_err(|e| match e {
                CircuitBreakerError::CircuitOpen => {
                    VectorSearchError::ServiceUnavailable("Vector search circuit breaker open".to_string())
                }
                CircuitBreakerError::OperationFailed(inner_err) => inner_err,
                CircuitBreakerError::InternalError(msg) => {
                    VectorSearchError::InternalError(msg)
                }
            })
    }
}
```

### 2. Add Specific Error Handling (3 min)
```rust
// Extend VectorSearchError
#[derive(Debug, thiserror::Error)]
pub enum VectorSearchError {
    #[error("Embedding API error: {0}")]
    EmbeddingError(String),
    #[error("Vector database error: {0}")]
    DatabaseError(String),
    #[error("Service temporarily unavailable: {0}")]
    ServiceUnavailable(String),
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    #[error("Internal error: {0}")]
    InternalError(String),
}

// Classify errors for circuit breaker
impl VectorSearchError {
    pub fn is_transient(&self) -> bool {
        matches!(self, 
            VectorSearchError::EmbeddingError(_) |
            VectorSearchError::DatabaseError(_) |
            VectorSearchError::RateLimitExceeded
        )
    }

    pub fn should_trigger_circuit_breaker(&self) -> bool {
        self.is_transient()
    }
}
```

### 3. Configure Circuit Breaker Parameters (3 min)
```rust
// src/config/circuit_breaker.rs
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,
    pub timeout_seconds: u64,
    pub recovery_timeout_seconds: u64,
    pub half_open_max_calls: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            timeout_seconds: 30,
            recovery_timeout_seconds: 60,
            half_open_max_calls: 3,
        }
    }
}

// Vector search specific config
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VectorSearchCircuitBreakerConfig {
    pub embedding_api: CircuitBreakerConfig,
    pub vector_db: CircuitBreakerConfig,
}

impl Default for VectorSearchCircuitBreakerConfig {
    fn default() -> Self {
        Self {
            embedding_api: CircuitBreakerConfig {
                failure_threshold: 3, // More sensitive for API
                timeout_seconds: 15,
                ..Default::default()
            },
            vector_db: CircuitBreakerConfig::default(),
        }
    }
}
```

## Validation
- [ ] Circuit breaker triggers on embedding API failures
- [ ] Proper error classification (transient vs permanent)
- [ ] Configuration is externalized and adjustable
- [ ] Fallback behavior works correctly
- [ ] Circuit breaker state transitions properly

## Success Criteria
- Vector search wrapped with circuit breaker
- Specific error handling for embedding API failures
- Configurable thresholds and timeouts
- Proper error propagation and classification
- Graceful degradation when circuit is open

## Next Task
task_003 - Apply circuit breaker to keyword search method
