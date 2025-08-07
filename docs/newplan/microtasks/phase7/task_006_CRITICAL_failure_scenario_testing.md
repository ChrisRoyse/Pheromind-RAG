# Task 006: CRITICAL - Implement Failure Scenario Testing Framework

## Objective
Create comprehensive testing framework for circuit breaker failure scenarios to validate production resilience.

## Time Estimate
10 minutes

## Priority
CRITICAL - Must validate circuit breaker behavior before production

## Dependencies
- task_001-005 - All circuit breakers implemented
- Testing infrastructure must exist

## Implementation Steps

### 1. Create Failure Injection Framework (4 min)
```rust
// src/testing/failure_injection.rs
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use async_trait::async_trait;
use mockall::mock;

#[derive(Debug, Clone)]
pub enum FailureType {
    NetworkTimeout,
    ServiceUnavailable,
    RateLimitExceeded,
    OutOfMemory,
    FileSystemError,
    DatabaseConnectionLost,
    EmbeddingApiDown,
    IndexCorruption,
}

pub struct FailureInjector {
    active_failures: Arc<Mutex<HashMap<String, FailureType>>>,
    failure_rates: Arc<Mutex<HashMap<String, f64>>>,
}

impl FailureInjector {
    pub fn new() -> Self {
        Self {
            active_failures: Arc::new(Mutex::new(HashMap::new())),
            failure_rates: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn inject_failure(&self, service: &str, failure_type: FailureType) {
        self.active_failures.lock().unwrap().insert(service.to_string(), failure_type);
    }

    pub fn set_failure_rate(&self, service: &str, rate: f64) {
        self.failure_rates.lock().unwrap().insert(service.to_string(), rate.clamp(0.0, 1.0));
    }

    pub fn should_fail(&self, service: &str) -> Option<FailureType> {
        let active_failures = self.active_failures.lock().unwrap();
        let failure_rates = self.failure_rates.lock().unwrap();

        // Check for explicit failure injection
        if let Some(failure_type) = active_failures.get(service) {
            return Some(failure_type.clone());
        }

        // Check for probabilistic failure
        if let Some(&rate) = failure_rates.get(service) {
            if rand::random::<f64>() < rate {
                return Some(FailureType::ServiceUnavailable);
            }
        }

        None
    }

    pub fn clear_failures(&self) {
        self.active_failures.lock().unwrap().clear();
        self.failure_rates.lock().unwrap().clear();
    }
}
```

### 2. Create Circuit Breaker Test Suite (3 min)
```rust
// src/testing/circuit_breaker_tests.rs
use crate::production::circuit_breaker::*;
use crate::testing::failure_injection::*;
use tokio::time::{sleep, Duration};

#[tokio::test]
async fn test_circuit_breaker_opens_on_failures() {
    let breaker = CircuitBreaker::new(3, Duration::from_millis(100));
    let injector = FailureInjector::new();
    injector.inject_failure("test_service", FailureType::ServiceUnavailable);

    // Trigger failures to open circuit
    for i in 0..3 {
        let result = breaker.call(async {
            if injector.should_fail("test_service").is_some() {
                Err("Service failed")
            } else {
                Ok("Success")
            }
        }).await;
        
        assert!(result.is_err(), "Expected failure on attempt {}", i + 1);
    }

    // Next call should be rejected immediately
    let result = breaker.call(async {
        Ok("Should not execute")
    }).await;
    
    assert!(matches!(result, Err(CircuitBreakerError::CircuitOpen)));
}

#[tokio::test]
async fn test_circuit_breaker_half_open_recovery() {
    let breaker = CircuitBreaker::new(2, Duration::from_millis(50));
    let injector = FailureInjector::new();
    
    // Trigger failures to open circuit
    injector.inject_failure("test_service", FailureType::ServiceUnavailable);
    for _ in 0..2 {
        let _ = breaker.call(async {
            if injector.should_fail("test_service").is_some() {
                Err("Service failed")
            } else {
                Ok("Success")
            }
        }).await;
    }

    // Wait for timeout
    sleep(Duration::from_millis(60)).await;
    
    // Clear failures for recovery
    injector.clear_failures();
    
    // Should transition to half-open and succeed
    let result = breaker.call(async {
        Ok("Recovery success")
    }).await;
    
    assert!(result.is_ok());
}
```

### 3. Create End-to-End Failure Scenarios (3 min)
```rust
// src/testing/e2e_failure_scenarios.rs
use crate::search::*;
use crate::testing::failure_injection::*;

#[tokio::test]
async fn test_vector_search_embedding_api_failure() {
    let injector = FailureInjector::new();
    let vector_search = VectorSearchWithCircuitBreaker::new_with_injector(
        VectorSearch::new(),
        injector.clone()
    );
    
    // Simulate embedding API failures
    injector.inject_failure("embedding_api", FailureType::EmbeddingApiDown);
    
    let mut failure_count = 0;
    for i in 0..10 {
        match vector_search.search("test query").await {
            Err(VectorSearchError::ServiceUnavailable(_)) => failure_count += 1,
            _ => {}
        }
    }
    
    assert!(failure_count >= 5, "Expected circuit breaker to trigger");
}

#[tokio::test]
async fn test_hybrid_search_partial_failure_recovery() {
    let injector = FailureInjector::new();
    let hybrid_search = create_hybrid_search_with_injector(injector.clone());
    
    // Make vector search fail but keep others working
    injector.inject_failure("vector_search", FailureType::ServiceUnavailable);
    
    let result = hybrid_search.search("test query").await;
    
    match result {
        Ok(hybrid_result) => {
            assert!(hybrid_result.successful_methods.len() >= 1);
            assert!(hybrid_result.failed_methods.contains_key(&SearchMethod::Vector));
            assert!(hybrid_result.confidence_score < 1.0);
        }
        Err(_) => panic!("Hybrid search should provide partial results"),
    }
}

#[tokio::test]
async fn test_cascading_failure_prevention() {
    let injector = FailureInjector::new();
    let hybrid_search = create_hybrid_search_with_injector(injector.clone());
    
    // Simulate high failure rates across all methods
    injector.set_failure_rate("vector_search", 0.8);
    injector.set_failure_rate("keyword_search", 0.6);
    injector.set_failure_rate("fuzzy_search", 0.4);
    
    let mut success_count = 0;
    let mut emergency_mode_count = 0;
    
    for _ in 0..20 {
        match hybrid_search.search("test query").await {
            Ok(_) => success_count += 1,
            Err(HybridSearchError::ServiceUnavailable(_)) => {
                // Try emergency mode
                match hybrid_search.emergency_search("test query").await {
                    Ok(_) => emergency_mode_count += 1,
                    Err(_) => {}
                }
            }
            Err(_) => {}
        }
    }
    
    assert!(success_count + emergency_mode_count >= 5, 
           "System should maintain some level of functionality");
}
```

## Validation
- [ ] Circuit breaker opens after threshold failures
- [ ] Half-open state transitions work correctly
- [ ] Recovery mechanisms function properly
- [ ] Partial failure scenarios provide degraded service
- [ ] Emergency modes prevent complete system failure

## Success Criteria
- Comprehensive failure injection framework
- Circuit breaker state transitions tested
- End-to-end failure scenarios validated
- Partial failure recovery verified
- Emergency fallback modes tested

## Next Task
task_007 - Implement recovery mechanism monitoring
