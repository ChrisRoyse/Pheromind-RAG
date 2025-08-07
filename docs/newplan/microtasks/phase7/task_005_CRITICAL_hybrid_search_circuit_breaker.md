# Task 005: CRITICAL - Apply Circuit Breaker to Hybrid Search

## Objective
Integrate circuit breaker pattern with hybrid search orchestrator to handle complex multi-method failures and provide intelligent fallbacks.

## Time Estimate
10 minutes

## Priority
CRITICAL - Hybrid search coordinates all methods and needs sophisticated failure handling

## Dependencies
- task_001-004 - Circuit breakers for individual search methods
- Hybrid search orchestrator must exist

## Implementation Steps

### 1. Create Multi-Method Circuit Breaker Orchestrator (4 min)
```rust
// src/search/hybrid.rs
use crate::production::circuit_breaker::{CircuitBreaker, CircuitBreakerError};
use std::collections::HashMap;
use std::time::Duration;
use std::sync::Arc;

pub struct HybridSearchWithCircuitBreaker {
    vector_search: Arc<VectorSearchWithCircuitBreaker>,
    keyword_search: Arc<KeywordSearchWithCircuitBreaker>,
    fuzzy_search: Arc<FuzzySearchWithCircuitBreaker>,
    orchestrator_breaker: Arc<CircuitBreaker>,
    method_priorities: Vec<SearchMethod>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SearchMethod {
    Vector,
    Keyword,
    Fuzzy,
}

impl HybridSearchWithCircuitBreaker {
    pub fn new(
        vector_search: VectorSearchWithCircuitBreaker,
        keyword_search: KeywordSearchWithCircuitBreaker,
        fuzzy_search: FuzzySearchWithCircuitBreaker,
    ) -> Self {
        Self {
            vector_search: Arc::new(vector_search),
            keyword_search: Arc::new(keyword_search),
            fuzzy_search: Arc::new(fuzzy_search),
            orchestrator_breaker: Arc::new(CircuitBreaker::new(
                2, // Low threshold for orchestration failures
                Duration::from_secs(5),
            )),
            method_priorities: vec![
                SearchMethod::Vector,
                SearchMethod::Keyword,
                SearchMethod::Fuzzy,
            ],
        }
    }

    pub async fn search(&self, query: &str) -> Result<HybridSearchResult, HybridSearchError> {
        self.orchestrator_breaker
            .call(async {
                self.execute_hybrid_search(query).await
            })
            .await
            .map_err(|e| match e {
                CircuitBreakerError::CircuitOpen => {
                    HybridSearchError::ServiceUnavailable(
                        "Hybrid search orchestrator circuit breaker open".to_string()
                    )
                }
                CircuitBreakerError::OperationFailed(inner_err) => inner_err,
                CircuitBreakerError::InternalError(msg) => {
                    HybridSearchError::InternalError(msg)
                }
            })
    }
}
```

### 2. Implement Intelligent Fallback Strategy (3 min)
```rust
impl HybridSearchWithCircuitBreaker {
    async fn execute_hybrid_search(&self, query: &str) -> Result<HybridSearchResult, HybridSearchError> {
        let mut results = HashMap::new();
        let mut errors = HashMap::new();
        let mut successful_methods = Vec::new();

        // Try each method in priority order with fallback
        for method in &self.method_priorities {
            match self.try_search_method(method, query).await {
                Ok(method_results) => {
                    results.insert(method.clone(), method_results);
                    successful_methods.push(method.clone());
                }
                Err(e) => {
                    errors.insert(method.clone(), e);
                    // Continue trying other methods
                }
            }
        }

        // Determine if we have enough results to proceed
        if successful_methods.is_empty() {
            return Err(HybridSearchError::AllMethodsFailed(errors));
        }

        // Merge and rank results from successful methods
        let merged_results = self.merge_results_with_confidence(results, &successful_methods).await?;
        
        Ok(HybridSearchResult {
            results: merged_results,
            successful_methods,
            failed_methods: errors,
            confidence_score: self.calculate_confidence_score(&successful_methods),
        })
    }

    async fn try_search_method(
        &self, 
        method: &SearchMethod, 
        query: &str
    ) -> Result<Vec<SearchResult>, HybridSearchError> {
        match method {
            SearchMethod::Vector => {
                self.vector_search.search(query)
                    .await
                    .map_err(HybridSearchError::VectorSearchFailed)
            }
            SearchMethod::Keyword => {
                self.keyword_search.search(query)
                    .await
                    .map_err(HybridSearchError::KeywordSearchFailed)
            }
            SearchMethod::Fuzzy => {
                self.fuzzy_search.search(query)
                    .await
                    .map_err(HybridSearchError::FuzzySearchFailed)
            }
        }
    }
}
```

### 3. Add Sophisticated Error Handling and Recovery (3 min)
```rust
#[derive(Debug, thiserror::Error)]
pub enum HybridSearchError {
    #[error("Vector search failed: {0}")]
    VectorSearchFailed(VectorSearchError),
    #[error("Keyword search failed: {0}")]
    KeywordSearchFailed(KeywordSearchError),
    #[error("Fuzzy search failed: {0}")]
    FuzzySearchFailed(FuzzySearchError),
    #[error("All search methods failed: {0:?}")]
    AllMethodsFailed(HashMap<SearchMethod, HybridSearchError>),
    #[error("Service temporarily unavailable: {0}")]
    ServiceUnavailable(String),
    #[error("Orchestration error: {0}")]
    OrchestrationError(String),
    #[error("Internal error: {0}")]
    InternalError(String),
}

#[derive(Debug)]
pub struct HybridSearchResult {
    pub results: Vec<RankedSearchResult>,
    pub successful_methods: Vec<SearchMethod>,
    pub failed_methods: HashMap<SearchMethod, HybridSearchError>,
    pub confidence_score: f64,
}

impl HybridSearchWithCircuitBreaker {
    // Adaptive method selection based on circuit breaker states
    pub async fn get_healthy_methods(&self) -> Vec<SearchMethod> {
        let mut healthy_methods = Vec::new();
        
        // Check each method's circuit breaker status
        if self.is_vector_search_healthy().await {
            healthy_methods.push(SearchMethod::Vector);
        }
        if self.is_keyword_search_healthy().await {
            healthy_methods.push(SearchMethod::Keyword);
        }
        if self.is_fuzzy_search_healthy().await {
            healthy_methods.push(SearchMethod::Fuzzy);
        }
        
        healthy_methods
    }

    // Emergency search mode with minimal resource usage
    pub async fn emergency_search(&self, query: &str) -> Result<Vec<SearchResult>, HybridSearchError> {
        // Use only the most reliable method with simplified parameters
        self.keyword_search.search_with_fallback(query)
            .await
            .map_err(HybridSearchError::KeywordSearchFailed)
    }

    fn calculate_confidence_score(&self, successful_methods: &[SearchMethod]) -> f64 {
        match successful_methods.len() {
            0 => 0.0,
            1 => 0.6, // Single method
            2 => 0.8, // Two methods
            3 => 1.0, // All methods
            _ => 1.0,
        }
    }
}
```

## Validation
- [ ] Orchestrator circuit breaker handles coordination failures
- [ ] Individual method failures don't break entire hybrid search
- [ ] Intelligent fallback provides partial results
- [ ] Confidence scoring reflects result reliability
- [ ] Emergency search mode works with minimal resources

## Success Criteria
- Hybrid search orchestrator wrapped with circuit breaker
- Intelligent fallback strategy uses available methods
- Partial results provided when some methods fail
- Confidence scoring indicates result quality
- Emergency mode provides basic functionality

## Next Task
task_006 - Implement failure scenario testing framework
