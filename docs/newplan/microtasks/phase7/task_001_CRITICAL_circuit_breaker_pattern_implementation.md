# Task 001: CRITICAL - Implement Circuit Breaker Pattern

## Objective
Implement a robust circuit breaker pattern to prevent cascade failures and provide graceful degradation when search methods fail.

## Time Estimate
10 minutes

## Priority
CRITICAL - Required for production stability

## Dependencies
- Core search methods must be implemented

## Implementation Steps

### 1. Create Circuit Breaker Core (3 min)
```rust
// src/production/circuit_breaker.rs
use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub enum CircuitState {
    Closed,    // Normal operation
    Open,      // Failing, rejecting requests
    HalfOpen,  // Testing if service recovered
}

pub struct CircuitBreaker {
    state: Arc<Mutex<CircuitState>>,
    failure_count: Arc<Mutex<u32>>,
    last_failure: Arc<Mutex<Option<Instant>>>,
    failure_threshold: u32,
    timeout: Duration,
    recovery_timeout: Duration,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u32, timeout: Duration) -> Self {
        Self {
            state: Arc::new(Mutex::new(CircuitState::Closed)),
            failure_count: Arc::new(Mutex::new(0)),
            last_failure: Arc::new(Mutex::new(None)),
            failure_threshold,
            timeout,
            recovery_timeout: timeout * 2,
        }
    }
}
```

### 2. Implement State Management (4 min)
```rust
impl CircuitBreaker {
    pub async fn call<F, T, E>(&self, operation: F) -> Result<T, CircuitBreakerError<E>>
    where
        F: Future<Output = Result<T, E>>,
    {
        if self.should_reject_request().await? {
            return Err(CircuitBreakerError::CircuitOpen);
        }

        match operation.await {
            Ok(result) => {
                self.on_success().await;
                Ok(result)
            }
            Err(err) => {
                self.on_failure().await;
                Err(CircuitBreakerError::OperationFailed(err))
            }
        }
    }

    async fn should_reject_request(&self) -> Result<bool, CircuitBreakerError<()>> {
        let state = *self.state.lock().unwrap();
        match state {
            CircuitState::Closed => Ok(false),
            CircuitState::Open => {
                if self.should_attempt_reset().await {
                    self.transition_to_half_open().await;
                    Ok(false)
                } else {
                    Ok(true)
                }
            }
            CircuitState::HalfOpen => Ok(false),
        }
    }
}
```

### 3. Add Error Types and Metrics (3 min)
```rust
#[derive(Debug, thiserror::Error)]
pub enum CircuitBreakerError<E> {
    #[error("Circuit breaker is open - rejecting request")]
    CircuitOpen,
    #[error("Operation failed: {0}")]
    OperationFailed(E),
    #[error("Circuit breaker internal error: {0}")]
    InternalError(String),
}

// Add metrics
pub struct CircuitBreakerMetrics {
    pub total_requests: u64,
    pub failed_requests: u64,
    pub rejected_requests: u64,
    pub state_transitions: u64,
    pub current_state: CircuitState,
}
```

## Validation
- [ ] Circuit breaker transitions correctly between states
- [ ] Failure threshold triggers state change
- [ ] Recovery timeout works properly
- [ ] Metrics are collected accurately
- [ ] Error types are properly propagated

## Success Criteria
- Circuit breaker pattern implemented with all three states
- Configurable failure thresholds and timeouts
- Proper error handling and propagation
- Basic metrics collection
- Thread-safe implementation

## Next Task
task_002 - Apply circuit breaker to vector search method
