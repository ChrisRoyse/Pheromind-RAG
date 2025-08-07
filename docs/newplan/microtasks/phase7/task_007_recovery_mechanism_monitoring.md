# Task 007: Recovery Mechanism Monitoring

## Objective
Implement comprehensive monitoring for circuit breaker recovery mechanisms to track system health and recovery patterns.

## Time Estimate
10 minutes

## Priority
HIGH - Critical for understanding system behavior during failures

## Dependencies
- task_001-006 - Circuit breakers and failure testing implemented

## Implementation Steps

### 1. Create Recovery Metrics Collection (4 min)
```rust
// src/monitoring/recovery_metrics.rs
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryMetrics {
    pub service_name: String,
    pub total_failures: u64,
    pub recovery_attempts: u64,
    pub successful_recoveries: u64,
    pub average_recovery_time: Duration,
    pub circuit_open_duration: Duration,
    pub last_failure_time: Option<Instant>,
    pub last_recovery_time: Option<Instant>,
}

pub struct RecoveryMonitor {
    metrics: Arc<Mutex<HashMap<String, RecoveryMetrics>>>,
    recovery_events: Arc<Mutex<Vec<RecoveryEvent>>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RecoveryEvent {
    pub service: String,
    pub event_type: RecoveryEventType,
    pub timestamp: Instant,
    pub duration_since_last: Option<Duration>,
    pub additional_info: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize)]
pub enum RecoveryEventType {
    CircuitOpened,
    CircuitHalfOpened,
    CircuitClosed,
    RecoveryAttempted,
    RecoverySucceeded,
    RecoveryFailed,
}

impl RecoveryMonitor {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(Mutex::new(HashMap::new())),
            recovery_events: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn record_failure(&self, service: &str) {
        let mut metrics = self.metrics.lock().unwrap();
        let metric = metrics.entry(service.to_string())
            .or_insert_with(|| RecoveryMetrics::new(service));
        
        metric.total_failures += 1;
        metric.last_failure_time = Some(Instant::now());
        
        self.record_event(RecoveryEvent {
            service: service.to_string(),
            event_type: RecoveryEventType::CircuitOpened,
            timestamp: Instant::now(),
            duration_since_last: None,
            additional_info: HashMap::new(),
        });
    }

    pub fn record_recovery_attempt(&self, service: &str) {
        let mut metrics = self.metrics.lock().unwrap();
        let metric = metrics.entry(service.to_string())
            .or_insert_with(|| RecoveryMetrics::new(service));
        
        metric.recovery_attempts += 1;
        
        self.record_event(RecoveryEvent {
            service: service.to_string(),
            event_type: RecoveryEventType::RecoveryAttempted,
            timestamp: Instant::now(),
            duration_since_last: metric.last_failure_time
                .map(|t| Instant::now().duration_since(t)),
            additional_info: HashMap::new(),
        });
    }
}
```

### 2. Integrate with Circuit Breakers (3 min)
```rust
// src/production/circuit_breaker.rs - Add monitoring integration
use crate::monitoring::recovery_metrics::RecoveryMonitor;

impl CircuitBreaker {
    pub fn new_with_monitoring(
        failure_threshold: u32, 
        timeout: Duration,
        monitor: Arc<RecoveryMonitor>,
        service_name: String,
    ) -> Self {
        Self {
            state: Arc::new(Mutex::new(CircuitState::Closed)),
            failure_count: Arc::new(Mutex::new(0)),
            last_failure: Arc::new(Mutex::new(None)),
            failure_threshold,
            timeout,
            recovery_timeout: timeout * 2,
            monitor: Some(monitor),
            service_name,
        }
    }

    async fn on_failure(&self) {
        let mut count = self.failure_count.lock().unwrap();
        *count += 1;
        
        if let Some(ref monitor) = self.monitor {
            monitor.record_failure(&self.service_name);
        }
        
        if *count >= self.failure_threshold {
            self.transition_to_open().await;
        }
    }

    async fn on_success(&self) {
        let mut count = self.failure_count.lock().unwrap();
        *count = 0;
        
        let current_state = *self.state.lock().unwrap();
        if matches!(current_state, CircuitState::HalfOpen) {
            self.transition_to_closed().await;
            
            if let Some(ref monitor) = self.monitor {
                monitor.record_successful_recovery(&self.service_name);
            }
        }
    }

    async fn transition_to_half_open(&self) {
        *self.state.lock().unwrap() = CircuitState::HalfOpen;
        
        if let Some(ref monitor) = self.monitor {
            monitor.record_half_open_transition(&self.service_name);
        }
    }
}
```

### 3. Create Recovery Analytics Dashboard (3 min)
```rust
// src/monitoring/recovery_analytics.rs
use crate::monitoring::recovery_metrics::*;
use std::time::{Duration, Instant};

pub struct RecoveryAnalytics {
    monitor: Arc<RecoveryMonitor>,
}

impl RecoveryAnalytics {
    pub fn new(monitor: Arc<RecoveryMonitor>) -> Self {
        Self { monitor }
    }

    pub fn get_recovery_report(&self) -> RecoveryReport {
        let metrics = self.monitor.metrics.lock().unwrap();
        let events = self.monitor.recovery_events.lock().unwrap();
        
        let mut services = Vec::new();
        for (service, metric) in metrics.iter() {
            services.push(ServiceRecoveryInfo {
                name: service.clone(),
                health_status: self.calculate_health_status(metric),
                recovery_rate: self.calculate_recovery_rate(metric),
                average_downtime: metric.average_recovery_time,
                recent_events: self.get_recent_events(service, &events),
            });
        }
        
        RecoveryReport {
            services,
            overall_system_health: self.calculate_overall_health(&metrics),
            generated_at: Instant::now(),
            recommendations: self.generate_recommendations(&metrics),
        }
    }

    fn calculate_health_status(&self, metric: &RecoveryMetrics) -> HealthStatus {
        let recovery_rate = if metric.recovery_attempts == 0 {
            1.0
        } else {
            metric.successful_recoveries as f64 / metric.recovery_attempts as f64
        };
        
        match recovery_rate {
            r if r >= 0.9 => HealthStatus::Healthy,
            r if r >= 0.7 => HealthStatus::Degraded,
            r if r >= 0.5 => HealthStatus::Unhealthy,
            _ => HealthStatus::Critical,
        }
    }

    fn generate_recommendations(&self, metrics: &HashMap<String, RecoveryMetrics>) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        for (service, metric) in metrics {
            if metric.recovery_attempts > 0 {
                let recovery_rate = metric.successful_recoveries as f64 / metric.recovery_attempts as f64;
                
                if recovery_rate < 0.5 {
                    recommendations.push(format!(
                        "Service '{}' has low recovery rate ({:.1}%). Consider reviewing circuit breaker thresholds.",
                        service, recovery_rate * 100.0
                    ));
                }
                
                if metric.average_recovery_time > Duration::from_secs(60) {
                    recommendations.push(format!(
                        "Service '{}' has long recovery time ({:.1}s). Consider optimizing recovery mechanisms.",
                        service, metric.average_recovery_time.as_secs_f64()
                    ));
                }
            }
        }
        
        recommendations
    }
}

#[derive(Debug, Serialize)]
pub struct RecoveryReport {
    pub services: Vec<ServiceRecoveryInfo>,
    pub overall_system_health: HealthStatus,
    pub generated_at: Instant,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ServiceRecoveryInfo {
    pub name: String,
    pub health_status: HealthStatus,
    pub recovery_rate: f64,
    pub average_downtime: Duration,
    pub recent_events: Vec<RecoveryEvent>,
}

#[derive(Debug, Serialize, Clone)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Critical,
}
```

## Validation
- [ ] Recovery metrics are collected accurately
- [ ] Circuit breaker state transitions are monitored
- [ ] Recovery analytics provide meaningful insights
- [ ] Health status calculations are correct
- [ ] Recommendations are actionable

## Success Criteria
- Comprehensive recovery metrics collection
- Circuit breaker integration with monitoring
- Analytics dashboard for recovery patterns
- Health status calculation for each service
- Actionable recommendations for improvements

## Next Task
task_008 - Implement memory management with limits and guards
