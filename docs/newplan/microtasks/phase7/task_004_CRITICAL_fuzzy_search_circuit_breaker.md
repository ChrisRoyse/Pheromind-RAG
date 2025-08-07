# Task 004: CRITICAL - Apply Circuit Breaker to Fuzzy Search

## Objective
Integrate circuit breaker pattern with fuzzy search method to handle computational failures and prevent performance degradation.

## Time Estimate
10 minutes

## Priority
CRITICAL - Fuzzy search is computationally intensive and prone to resource exhaustion

## Dependencies
- task_001 - Circuit breaker pattern implemented
- Fuzzy search method must exist

## Implementation Steps

### 1. Wrap Fuzzy Search with Circuit Breaker (4 min)
```rust
// src/search/fuzzy.rs
use crate::production::circuit_breaker::{CircuitBreaker, CircuitBreakerError};
use std::time::Duration;
use std::sync::Arc;

pub struct FuzzySearchWithCircuitBreaker {
    inner: FuzzySearch,
    computation_breaker: Arc<CircuitBreaker>,
    memory_breaker: Arc<CircuitBreaker>,
}

impl FuzzySearchWithCircuitBreaker {
    pub fn new(fuzzy_search: FuzzySearch) -> Self {
        Self {
            inner: fuzzy_search,
            computation_breaker: Arc<new(CircuitBreaker::new(
                3, // Low threshold for computation failures
                Duration::from_secs(5),
            )),
            memory_breaker: Arc::new(CircuitBreaker::new(
                2, // Very low threshold for memory issues
                Duration::from_secs(15),
            )),
        }
    }

    pub async fn search(&self, query: &str) -> Result<Vec<SearchResult>, FuzzySearchError> {
        // Check memory usage before starting
        if self.is_memory_pressure_high().await {
            return self.memory_breaker
                .call(async {
                    Err(FuzzySearchError::MemoryPressure(
                        "High memory pressure detected".to_string()
                    ))
                })
                .await
                .map_err(|e| self.handle_circuit_breaker_error(e, "memory"))?;
        }

        // Execute fuzzy search with computation circuit breaker
        self.computation_breaker
            .call(async {
                self.inner.search_with_timeout(query, Duration::from_secs(10)).await
            })
            .await
            .map_err(|e| self.handle_circuit_breaker_error(e, "computation"))
    }
}
```

### 2. Add Resource-Aware Error Handling (3 min)
```rust
#[derive(Debug, thiserror::Error)]
pub enum FuzzySearchError {
    #[error("Computation timeout: {0}")]
    ComputationTimeout(String),
    #[error("Memory pressure: {0}")]
    MemoryPressure(String),
    #[error("Algorithm error: {0}")]
    AlgorithmError(String),
    #[error("Resource exhausted: {0}")]
    ResourceExhausted(String),
    #[error("Service temporarily unavailable: {0}")]
    ServiceUnavailable(String),
    #[error("Internal error: {0}")]
    InternalError(String),
}

impl FuzzySearchError {
    pub fn is_resource_related(&self) -> bool {
        matches!(self,
            FuzzySearchError::MemoryPressure(_) |
            FuzzySearchError::ResourceExhausted(_) |
            FuzzySearchError::ComputationTimeout(_)
        )
    }

    pub fn should_trigger_circuit_breaker(&self) -> bool {
        self.is_resource_related()
    }

    pub fn get_circuit_breaker_category(&self) -> &'static str {
        match self {
            FuzzySearchError::MemoryPressure(_) => "memory",
            FuzzySearchError::ComputationTimeout(_) => "computation",
            FuzzySearchError::ResourceExhausted(_) => "computation",
            _ => "general",
        }
    }

    pub fn recovery_suggestion(&self) -> &'static str {
        match self {
            FuzzySearchError::MemoryPressure(_) => "Wait for memory to be freed",
            FuzzySearchError::ComputationTimeout(_) => "Reduce query complexity",
            FuzzySearchError::ResourceExhausted(_) => "Wait for resources to be available",
            _ => "Retry after brief delay",
        }
    }
}
```

### 3. Add Resource Monitoring and Adaptive Behavior (3 min)
```rust
impl FuzzySearchWithCircuitBreaker {
    async fn is_memory_pressure_high(&self) -> bool {
        // Simple memory pressure detection
        if let Ok(usage) = self.get_memory_usage().await {
            usage.percent > 85.0
        } else {
            false
        }
    }

    async fn get_memory_usage(&self) -> Result<MemoryUsage, std::io::Error> {
        // Platform-specific memory monitoring
        #[cfg(target_os = "linux")]
        {
            self.get_linux_memory_usage().await
        }
        #[cfg(target_os = "windows")]
        {
            self.get_windows_memory_usage().await
        }
        #[cfg(not(any(target_os = "linux", target_os = "windows")))]
        {
            Ok(MemoryUsage { percent: 50.0 }) // Default safe value
        }
    }

    // Adaptive search with reduced complexity
    pub async fn search_adaptive(&self, query: &str) -> Result<Vec<SearchResult>, FuzzySearchError> {
        // Try normal search first
        match self.search(query).await {
            Ok(results) => Ok(results),
            Err(FuzzySearchError::ServiceUnavailable(_)) => {
                // Fallback to simplified fuzzy search
                self.search_simplified(query).await
            }
            Err(e) => Err(e),
        }
    }

    async fn search_simplified(&self, query: &str) -> Result<Vec<SearchResult>, FuzzySearchError> {
        // Use simpler algorithm with lower resource usage
        self.inner.search_simple(query, 0.7).await // Lower similarity threshold
    }
}

#[derive(Debug)]
struct MemoryUsage {
    percent: f64,
}
```

## Validation
- [ ] Circuit breaker triggers on computation timeouts
- [ ] Memory pressure detection works correctly
- [ ] Separate circuit breakers for computation and memory
- [ ] Adaptive behavior reduces complexity when needed
- [ ] Resource monitoring provides accurate data

## Success Criteria
- Fuzzy search wrapped with resource-aware circuit breakers
- Memory pressure detection prevents system overload
- Adaptive behavior provides fallback with reduced complexity
- Separate handling for computation and memory failures
- Recovery suggestions guide system behavior

## Next Task
task_005 - Apply circuit breaker to hybrid search method
