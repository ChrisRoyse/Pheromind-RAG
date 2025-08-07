# Task 003: CRITICAL - Apply Circuit Breaker to Keyword Search

## Objective
Integrate circuit breaker pattern with keyword search method to handle file system and indexing failures gracefully.

## Time Estimate
10 minutes

## Priority
CRITICAL - Keyword search failures can cascade to file system issues

## Dependencies
- task_001 - Circuit breaker pattern implemented
- Keyword search method must exist

## Implementation Steps

### 1. Wrap Keyword Search with Circuit Breaker (4 min)
```rust
// src/search/keyword.rs
use crate::production::circuit_breaker::{CircuitBreaker, CircuitBreakerError};
use std::time::Duration;
use std::sync::Arc;

pub struct KeywordSearchWithCircuitBreaker {
    inner: KeywordSearch,
    file_system_breaker: Arc<CircuitBreaker>,
    indexing_breaker: Arc<CircuitBreaker>,
}

impl KeywordSearchWithCircuitBreaker {
    pub fn new(keyword_search: KeywordSearch) -> Self {
        Self {
            inner: keyword_search,
            file_system_breaker: Arc::new(CircuitBreaker::new(
                10, // Higher threshold for FS operations
                Duration::from_secs(10),
            )),
            indexing_breaker: Arc::new(CircuitBreaker::new(
                5, // Lower threshold for indexing
                Duration::from_secs(20),
            )),
        }
    }

    pub async fn search(&self, query: &str) -> Result<Vec<SearchResult>, KeywordSearchError> {
        // First check file system operations
        let file_results = self.file_system_breaker
            .call(async {
                self.inner.search_files(query).await
            })
            .await
            .map_err(|e| self.handle_circuit_breaker_error(e, "file_system"))?;

        // Then check indexing operations
        let indexed_results = self.indexing_breaker
            .call(async {
                self.inner.search_index(query).await
            })
            .await
            .map_err(|e| self.handle_circuit_breaker_error(e, "indexing"))?;

        // Merge results
        Ok(self.merge_results(file_results, indexed_results))
    }
}
```

### 2. Add Error Classification (3 min)
```rust
#[derive(Debug, thiserror::Error)]
pub enum KeywordSearchError {
    #[error("File system error: {0}")]
    FileSystemError(String),
    #[error("Index error: {0}")]
    IndexError(String),
    #[error("I/O error: {0}")]
    IoError(String),
    #[error("Service temporarily unavailable: {0}")]
    ServiceUnavailable(String),
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    #[error("Internal error: {0}")]
    InternalError(String),
}

impl KeywordSearchError {
    pub fn is_file_system_error(&self) -> bool {
        matches!(self, 
            KeywordSearchError::FileSystemError(_) |
            KeywordSearchError::IoError(_) |
            KeywordSearchError::PermissionDenied(_)
        )
    }

    pub fn is_indexing_error(&self) -> bool {
        matches!(self, KeywordSearchError::IndexError(_))
    }

    pub fn should_trigger_circuit_breaker(&self) -> bool {
        self.is_file_system_error() || self.is_indexing_error()
    }

    pub fn get_circuit_breaker_category(&self) -> &'static str {
        if self.is_file_system_error() {
            "file_system"
        } else if self.is_indexing_error() {
            "indexing"
        } else {
            "general"
        }
    }
}
```

### 3. Add Fallback Mechanisms (3 min)
```rust
impl KeywordSearchWithCircuitBreaker {
    fn handle_circuit_breaker_error<E>(
        &self, 
        error: CircuitBreakerError<E>, 
        category: &str
    ) -> KeywordSearchError {
        match error {
            CircuitBreakerError::CircuitOpen => {
                KeywordSearchError::ServiceUnavailable(
                    format!("{} circuit breaker is open", category)
                )
            }
            CircuitBreakerError::OperationFailed(inner_err) => {
                // Convert inner error to KeywordSearchError
                KeywordSearchError::InternalError(
                    format!("{} operation failed", category)
                )
            }
            CircuitBreakerError::InternalError(msg) => {
                KeywordSearchError::InternalError(msg)
            }
        }
    }

    // Fallback to partial results when one circuit is open
    pub async fn search_with_fallback(&self, query: &str) -> SearchResult {
        let mut results = Vec::new();
        let mut errors = Vec::new();

        // Try file system search
        match self.search_files_safe(query).await {
            Ok(file_results) => results.extend(file_results),
            Err(e) => errors.push(format!("File system: {}", e)),
        }

        // Try indexing search
        match self.search_index_safe(query).await {
            Ok(index_results) => results.extend(index_results),
            Err(e) => errors.push(format!("Indexing: {}", e)),
        }

        SearchResult {
            results,
            errors,
            partial: !errors.is_empty(),
        }
    }
}
```

## Validation
- [ ] Circuit breaker triggers on file system failures
- [ ] Separate circuit breakers for FS and indexing
- [ ] Proper error classification and routing
- [ ] Fallback mechanisms provide partial results
- [ ] Circuit breaker states transition correctly

## Success Criteria
- Keyword search wrapped with dual circuit breakers
- File system and indexing failures handled separately
- Fallback mechanisms provide graceful degradation
- Error classification routes to correct circuit breaker
- Partial results available when one subsystem fails

## Next Task
task_004 - Apply circuit breaker to fuzzy search method
