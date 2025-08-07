# Task 020: CRITICAL - Health Check Endpoints

## Objective
Implement comprehensive health check endpoints for monitoring system health, dependencies, and readiness.

## Time Estimate
10 minutes

## Priority
CRITICAL - Essential for load balancers and orchestration systems

## Dependencies
- HTTP server infrastructure
- Resource monitoring systems

## Implementation Steps

### 1. Create Health Check Framework (4 min)
```rust
// src/monitoring/health.rs
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::collections::HashMap;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Critical,
}

#[derive(Debug, Clone, Serialize)]
pub struct HealthCheckResult {
    pub component: String,
    pub status: HealthStatus,
    pub message: String,
    pub details: HashMap<String, serde_json::Value>,
    pub response_time: Duration,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[async_trait]
pub trait HealthChecker: Send + Sync {
    async fn check_health(&self) -> HealthCheckResult;
    fn component_name(&self) -> &str;
    fn timeout(&self) -> Duration {
        Duration::from_secs(5)
    }
}

pub struct HealthService {
    checkers: Arc<Vec<Arc<dyn HealthChecker>>>,
    cache_ttl: Duration,
    last_check: Arc<std::sync::Mutex<Option<Instant>>>,
    cached_results: Arc<std::sync::Mutex<Option<SystemHealth>>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SystemHealth {
    pub status: HealthStatus,
    pub checks: Vec<HealthCheckResult>,
    pub overall_response_time: Duration,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub version: String,
    pub uptime: Duration,
}

impl HealthService {
    pub fn new(cache_ttl: Duration) -> Self {
        Self {
            checkers: Arc::new(Vec::new()),
            cache_ttl,
            last_check: Arc::new(std::sync::Mutex::new(None)),
            cached_results: Arc::new(std::sync::Mutex::new(None)),
        }
    }
    
    pub fn register_checker(&mut self, checker: Arc<dyn HealthChecker>) {
        Arc::get_mut(&mut self.checkers).unwrap().push(checker);
    }
    
    pub async fn check_system_health(&self) -> SystemHealth {
        // Check cache first
        {
            let last_check = self.last_check.lock().unwrap();
            if let Some(last_time) = *last_check {
                if last_time.elapsed() < self.cache_ttl {
                    if let Some(cached) = self.cached_results.lock().unwrap().as_ref() {
                        return cached.clone();
                    }
                }
            }
        }
        
        let start_time = Instant::now();
        let mut check_results = Vec::new();
        
        // Run all health checks in parallel
        let mut handles = Vec::new();
        for checker in self.checkers.iter() {
            let checker_clone = checker.clone();
            let timeout = checker.timeout();
            
            let handle = tokio::spawn(async move {
                tokio::time::timeout(timeout, checker_clone.check_health()).await
                    .unwrap_or_else(|_| HealthCheckResult {
                        component: checker_clone.component_name().to_string(),
                        status: HealthStatus::Critical,
                        message: "Health check timeout".to_string(),
                        details: HashMap::new(),
                        response_time: timeout,
                        timestamp: chrono::Utc::now(),
                    })
            });
            handles.push(handle);
        }
        
        // Collect results
        for handle in handles {
            if let Ok(result) = handle.await {
                check_results.push(result);
            }
        }
        
        // Determine overall status
        let overall_status = self.determine_overall_status(&check_results);
        let overall_response_time = start_time.elapsed();
        
        let system_health = SystemHealth {
            status: overall_status,
            checks: check_results,
            overall_response_time,
            timestamp: chrono::Utc::now(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            uptime: self.get_uptime(),
        };
        
        // Update cache
        {
            *self.last_check.lock().unwrap() = Some(start_time);
            *self.cached_results.lock().unwrap() = Some(system_health.clone());
        }
        
        system_health
    }
    
    fn determine_overall_status(&self, results: &[HealthCheckResult]) -> HealthStatus {
        if results.is_empty() {
            return HealthStatus::Unhealthy;
        }
        
        let mut has_critical = false;
        let mut has_unhealthy = false;
        let mut has_degraded = false;
        
        for result in results {
            match result.status {
                HealthStatus::Critical => has_critical = true,
                HealthStatus::Unhealthy => has_unhealthy = true,
                HealthStatus::Degraded => has_degraded = true,
                HealthStatus::Healthy => {},
            }
        }
        
        if has_critical {
            HealthStatus::Critical
        } else if has_unhealthy {
            HealthStatus::Unhealthy
        } else if has_degraded {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        }
    }
    
    fn get_uptime(&self) -> Duration {
        // Simple uptime calculation - in production you'd want to track actual start time
        static START_TIME: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();
        let start_time = START_TIME.get_or_init(|| Instant::now());
        start_time.elapsed()
    }
}
```

### 2. Implement Specific Health Checkers (3 min)
```rust
// Memory health checker
pub struct MemoryHealthChecker {
    memory_guard: Arc<MemoryGuard>,
}

impl MemoryHealthChecker {
    pub fn new(memory_guard: Arc<MemoryGuard>) -> Self {
        Self { memory_guard }
    }
}

#[async_trait]
impl HealthChecker for MemoryHealthChecker {
    async fn check_health(&self) -> HealthCheckResult {
        let start_time = Instant::now();
        let stats = self.memory_guard.get_memory_stats();
        
        let usage_ratio = stats.usage_ratio;
        let (status, message) = if usage_ratio > 0.95 {
            (HealthStatus::Critical, "Memory usage critical")
        } else if usage_ratio > 0.85 {
            (HealthStatus::Unhealthy, "Memory usage high")
        } else if usage_ratio > 0.75 {
            (HealthStatus::Degraded, "Memory usage elevated")
        } else {
            (HealthStatus::Healthy, "Memory usage normal")
        };
        
        let mut details = HashMap::new();
        details.insert("usage_ratio".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(usage_ratio).unwrap()));
        details.insert("used_bytes".to_string(), serde_json::Value::Number(serde_json::Number::from(stats.used_system as u64)));
        details.insert("total_bytes".to_string(), serde_json::Value::Number(serde_json::Number::from(stats.total_system as u64)));
        
        HealthCheckResult {
            component: "memory".to_string(),
            status,
            message: message.to_string(),
            details,
            response_time: start_time.elapsed(),
            timestamp: chrono::Utc::now(),
        }
    }
    
    fn component_name(&self) -> &str {
        "memory"
    }
}

// Circuit breaker health checker
pub struct CircuitBreakerHealthChecker {
    circuit_breakers: HashMap<String, Arc<CircuitBreaker>>,
}

impl CircuitBreakerHealthChecker {
    pub fn new() -> Self {
        Self {
            circuit_breakers: HashMap::new(),
        }
    }
    
    pub fn register_circuit_breaker(&mut self, name: String, breaker: Arc<CircuitBreaker>) {
        self.circuit_breakers.insert(name, breaker);
    }
}

#[async_trait]
impl HealthChecker for CircuitBreakerHealthChecker {
    async fn check_health(&self) -> HealthCheckResult {
        let start_time = Instant::now();
        let mut details = HashMap::new();
        let mut open_breakers = 0;
        let mut half_open_breakers = 0;
        
        for (name, breaker) in &self.circuit_breakers {
            let state = breaker.get_state().await;
            let state_str = match state {
                CircuitState::Closed => "closed",
                CircuitState::Open => {
                    open_breakers += 1;
                    "open"
                }
                CircuitState::HalfOpen => {
                    half_open_breakers += 1;
                    "half_open"
                }
            };
            details.insert(name.clone(), serde_json::Value::String(state_str.to_string()));
        }
        
        let total_breakers = self.circuit_breakers.len();
        let (status, message) = if open_breakers == total_breakers {
            (HealthStatus::Critical, "All circuit breakers are open")
        } else if open_breakers > total_breakers / 2 {
            (HealthStatus::Unhealthy, "Majority of circuit breakers are open")
        } else if open_breakers > 0 || half_open_breakers > 0 {
            (HealthStatus::Degraded, "Some circuit breakers are not healthy")
        } else {
            (HealthStatus::Healthy, "All circuit breakers are healthy")
        };
        
        details.insert("open_count".to_string(), serde_json::Value::Number(serde_json::Number::from(open_breakers)));
        details.insert("half_open_count".to_string(), serde_json::Value::Number(serde_json::Number::from(half_open_breakers)));
        details.insert("total_count".to_string(), serde_json::Value::Number(serde_json::Number::from(total_breakers)));
        
        HealthCheckResult {
            component: "circuit_breakers".to_string(),
            status,
            message: message.to_string(),
            details,
            response_time: start_time.elapsed(),
            timestamp: chrono::Utc::now(),
        }
    }
    
    fn component_name(&self) -> &str {
        "circuit_breakers"
    }
}

// External API health checker
pub struct ExternalApiHealthChecker {
    client: reqwest::Client,
    api_url: String,
}

impl ExternalApiHealthChecker {
    pub fn new(api_url: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_url,
        }
    }
}

#[async_trait]
impl HealthChecker for ExternalApiHealthChecker {
    async fn check_health(&self) -> HealthCheckResult {
        let start_time = Instant::now();
        
        match self.client.head(&self.api_url).send().await {
            Ok(response) => {
                let status_code = response.status().as_u16();
                let mut details = HashMap::new();
                details.insert("status_code".to_string(), serde_json::Value::Number(serde_json::Number::from(status_code)));
                details.insert("url".to_string(), serde_json::Value::String(self.api_url.clone()));
                
                let (status, message) = if response.status().is_success() {
                    (HealthStatus::Healthy, "External API is responding")
                } else if response.status().is_server_error() {
                    (HealthStatus::Unhealthy, "External API server error")
                } else {
                    (HealthStatus::Degraded, "External API client error")
                };
                
                HealthCheckResult {
                    component: "external_api".to_string(),
                    status,
                    message: message.to_string(),
                    details,
                    response_time: start_time.elapsed(),
                    timestamp: chrono::Utc::now(),
                }
            }
            Err(e) => {
                let mut details = HashMap::new();
                details.insert("error".to_string(), serde_json::Value::String(e.to_string()));
                details.insert("url".to_string(), serde_json::Value::String(self.api_url.clone()));
                
                HealthCheckResult {
                    component: "external_api".to_string(),
                    status: HealthStatus::Critical,
                    message: "External API unreachable".to_string(),
                    details,
                    response_time: start_time.elapsed(),
                    timestamp: chrono::Utc::now(),
                }
            }
        }
    }
    
    fn component_name(&self) -> &str {
        "external_api"
    }
    
    fn timeout(&self) -> Duration {
        Duration::from_secs(10)
    }
}
```

### 3. Create HTTP Health Endpoints (3 min)
```rust
// HTTP endpoints for health checks
use warp::{Filter, Reply};

pub fn create_health_routes(health_service: Arc<HealthService>) -> impl Filter<Extract = impl Reply> + Clone {
    let health_service = warp::any().map(move || health_service.clone());
    
    // Liveness probe - basic health check
    let liveness = warp::path!("health" / "live")
        .and(warp::get())
        .map(|| {
            warp::reply::json(&serde_json::json!({
                "status": "alive",
                "timestamp": chrono::Utc::now().to_rfc3339()
            }))
        });
    
    // Readiness probe - comprehensive health check
    let readiness = warp::path!("health" / "ready")
        .and(warp::get())
        .and(health_service.clone())
        .and_then(|health_service: Arc<HealthService>| async move {
            let health = health_service.check_system_health().await;
            
            let status_code = match health.status {
                HealthStatus::Healthy => warp::http::StatusCode::OK,
                HealthStatus::Degraded => warp::http::StatusCode::OK,
                HealthStatus::Unhealthy => warp::http::StatusCode::SERVICE_UNAVAILABLE,
                HealthStatus::Critical => warp::http::StatusCode::SERVICE_UNAVAILABLE,
            };
            
            Ok::<_, warp::Rejection>(warp::reply::with_status(
                warp::reply::json(&health),
                status_code,
            ))
        });
    
    // Detailed health check
    let health_detailed = warp::path!("health")
        .and(warp::get())
        .and(health_service)
        .and_then(|health_service: Arc<HealthService>| async move {
            let health = health_service.check_system_health().await;
            Ok::<_, warp::Rejection>(warp::reply::json(&health))
        });
    
    liveness.or(readiness).or(health_detailed)
}

// Setup function for health monitoring
pub async fn setup_health_monitoring(
    memory_guard: Arc<MemoryGuard>,
    circuit_breakers: HashMap<String, Arc<CircuitBreaker>>,
    external_apis: Vec<String>,
) -> Arc<HealthService> {
    let mut health_service = HealthService::new(Duration::from_secs(30));
    
    // Register health checkers
    health_service.register_checker(Arc::new(MemoryHealthChecker::new(memory_guard)));
    
    let mut cb_checker = CircuitBreakerHealthChecker::new();
    for (name, breaker) in circuit_breakers {
        cb_checker.register_circuit_breaker(name, breaker);
    }
    health_service.register_checker(Arc::new(cb_checker));
    
    for api_url in external_apis {
        health_service.register_checker(Arc::new(ExternalApiHealthChecker::new(api_url)));
    }
    
    Arc::new(health_service)
}
```

## Validation
- [ ] Health check endpoints return correct HTTP status codes
- [ ] Individual component health checkers work correctly
- [ ] Overall system status reflects component health
- [ ] Health check caching improves performance
- [ ] External API health checks handle timeouts

## Success Criteria
- Comprehensive health check framework
- Multiple health endpoints for different use cases
- Proper HTTP status codes for load balancer integration
- Detailed health information for debugging
- Configurable caching for performance

## Next Task
task_021 - Implement dashboard setup and configuration
