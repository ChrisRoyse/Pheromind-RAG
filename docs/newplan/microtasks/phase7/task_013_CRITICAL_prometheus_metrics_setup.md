# Task 013: CRITICAL - Prometheus Metrics Setup

## Objective
Implement comprehensive Prometheus metrics collection for monitoring system performance, resource usage, and business metrics.

## Time Estimate
10 minutes

## Priority
CRITICAL - Essential for production monitoring and observability

## Dependencies
- Basic HTTP server infrastructure

## Implementation Steps

### 1. Set Up Prometheus Metrics Infrastructure (4 min)
```rust
// src/monitoring/prometheus.rs
use prometheus::{
    Counter, Histogram, Gauge, IntCounter, IntGauge,
    Opts, HistogramOpts, Registry, Encoder, TextEncoder,
    register_counter, register_histogram, register_gauge,
    register_int_counter, register_int_gauge,
};
use std::sync::{Arc, Mutex, OnceLock};
use std::collections::HashMap;
use std::time::Instant;
use warp::{Filter, Reply};

// Global registry for metrics
static METRICS_REGISTRY: OnceLock<Arc<Registry>> = OnceLock::new();

pub struct MetricsCollector {
    registry: Arc<Registry>,
    
    // Search metrics
    search_requests_total: IntCounter,
    search_duration: Histogram,
    search_results_total: IntCounter,
    search_errors_total: IntCounter,
    
    // Resource metrics
    memory_usage_bytes: Gauge,
    file_handles_active: IntGauge,
    connections_active: IntGauge,
    thread_pool_utilization: Gauge,
    
    // Circuit breaker metrics
    circuit_breaker_state: IntGauge,
    circuit_breaker_failures: IntCounter,
    circuit_breaker_successes: IntCounter,
    
    // Performance metrics
    request_rate: Gauge,
    error_rate: Gauge,
    p99_latency: Gauge,
    
    // Custom metrics registry
    custom_counters: Arc<Mutex<HashMap<String, IntCounter>>>,
    custom_gauges: Arc<Mutex<HashMap<String, Gauge>>>,
}

impl MetricsCollector {
    pub fn new() -> Result<Self, prometheus::Error> {
        let registry = Arc::new(Registry::new());
        
        // Initialize core metrics
        let search_requests_total = IntCounter::with_opts(
            Opts::new("search_requests_total", "Total number of search requests")
                .namespace("embed_search")
        )?;
        registry.register(Box::new(search_requests_total.clone()))?;
        
        let search_duration = Histogram::with_opts(
            HistogramOpts::new("search_duration_seconds", "Search request duration")
                .namespace("embed_search")
                .buckets(vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0])
        )?;
        registry.register(Box::new(search_duration.clone()))?;
        
        let search_results_total = IntCounter::with_opts(
            Opts::new("search_results_total", "Total number of search results returned")
                .namespace("embed_search")
        )?;
        registry.register(Box::new(search_results_total.clone()))?;
        
        let search_errors_total = IntCounter::with_opts(
            Opts::new("search_errors_total", "Total number of search errors")
                .namespace("embed_search")
        )?;
        registry.register(Box::new(search_errors_total.clone()))?;
        
        // Resource metrics
        let memory_usage_bytes = Gauge::with_opts(
            Opts::new("memory_usage_bytes", "Current memory usage in bytes")
                .namespace("embed_search")
        )?;
        registry.register(Box::new(memory_usage_bytes.clone()))?;
        
        let file_handles_active = IntGauge::with_opts(
            Opts::new("file_handles_active", "Number of active file handles")
                .namespace("embed_search")
        )?;
        registry.register(Box::new(file_handles_active.clone()))?;
        
        let connections_active = IntGauge::with_opts(
            Opts::new("connections_active", "Number of active connections")
                .namespace("embed_search")
        )?;
        registry.register(Box::new(connections_active.clone()))?;
        
        let thread_pool_utilization = Gauge::with_opts(
            Opts::new("thread_pool_utilization", "Thread pool utilization ratio")
                .namespace("embed_search")
        )?;
        registry.register(Box::new(thread_pool_utilization.clone()))?;
        
        // Circuit breaker metrics
        let circuit_breaker_state = IntGauge::with_opts(
            Opts::new("circuit_breaker_state", "Circuit breaker state (0=closed, 1=open, 2=half-open)")
                .namespace("embed_search")
        )?;
        registry.register(Box::new(circuit_breaker_state.clone()))?;
        
        let circuit_breaker_failures = IntCounter::with_opts(
            Opts::new("circuit_breaker_failures_total", "Total circuit breaker failures")
                .namespace("embed_search")
        )?;
        registry.register(Box::new(circuit_breaker_failures.clone()))?;
        
        let circuit_breaker_successes = IntCounter::with_opts(
            Opts::new("circuit_breaker_successes_total", "Total circuit breaker successes")
                .namespace("embed_search")
        )?;
        registry.register(Box::new(circuit_breaker_successes.clone()))?;
        
        // Performance metrics
        let request_rate = Gauge::with_opts(
            Opts::new("request_rate", "Requests per second")
                .namespace("embed_search")
        )?;
        registry.register(Box::new(request_rate.clone()))?;
        
        let error_rate = Gauge::with_opts(
            Opts::new("error_rate", "Error rate percentage")
                .namespace("embed_search")
        )?;
        registry.register(Box::new(error_rate.clone()))?;
        
        let p99_latency = Gauge::with_opts(
            Opts::new("p99_latency_seconds", "99th percentile latency")
                .namespace("embed_search")
        )?;
        registry.register(Box::new(p99_latency.clone()))?;
        
        // Store registry globally
        METRICS_REGISTRY.set(registry.clone()).map_err(|_| prometheus::Error::Msg("Registry already initialized".to_string()))?;
        
        Ok(Self {
            registry,
            search_requests_total,
            search_duration,
            search_results_total,
            search_errors_total,
            memory_usage_bytes,
            file_handles_active,
            connections_active,
            thread_pool_utilization,
            circuit_breaker_state,
            circuit_breaker_failures,
            circuit_breaker_successes,
            request_rate,
            error_rate,
            p99_latency,
            custom_counters: Arc::new(Mutex::new(HashMap::new())),
            custom_gauges: Arc::new(Mutex::new(HashMap::new())),
        })
    }
    
    // Search metrics methods
    pub fn inc_search_requests(&self) {
        self.search_requests_total.inc();
    }
    
    pub fn observe_search_duration(&self, duration: f64) {
        self.search_duration.observe(duration);
    }
    
    pub fn add_search_results(&self, count: u64) {
        self.search_results_total.inc_by(count);
    }
    
    pub fn inc_search_errors(&self) {
        self.search_errors_total.inc();
    }
}
```

### 2. Implement Metrics Collection and Updates (3 min)
```rust
impl MetricsCollector {
    // Resource metrics methods
    pub fn set_memory_usage(&self, bytes: f64) {
        self.memory_usage_bytes.set(bytes);
    }
    
    pub fn set_file_handles_active(&self, count: i64) {
        self.file_handles_active.set(count);
    }
    
    pub fn set_connections_active(&self, count: i64) {
        self.connections_active.set(count);
    }
    
    pub fn set_thread_pool_utilization(&self, ratio: f64) {
        self.thread_pool_utilization.set(ratio);
    }
    
    // Circuit breaker metrics
    pub fn set_circuit_breaker_state(&self, service: &str, state: i64) {
        // Use labels for different services
        self.circuit_breaker_state.set(state);
    }
    
    pub fn inc_circuit_breaker_failures(&self) {
        self.circuit_breaker_failures.inc();
    }
    
    pub fn inc_circuit_breaker_successes(&self) {
        self.circuit_breaker_successes.inc();
    }
    
    // Performance metrics
    pub fn set_request_rate(&self, rate: f64) {
        self.request_rate.set(rate);
    }
    
    pub fn set_error_rate(&self, rate: f64) {
        self.error_rate.set(rate);
    }
    
    pub fn set_p99_latency(&self, latency: f64) {
        self.p99_latency.set(latency);
    }
    
    // Custom metrics
    pub fn inc_custom_counter(&self, name: &str) -> Result<(), prometheus::Error> {
        let mut counters = self.custom_counters.lock().unwrap();
        
        if let Some(counter) = counters.get(name) {
            counter.inc();
        } else {
            let counter = IntCounter::with_opts(
                Opts::new(name, format!("Custom counter: {}", name))
                    .namespace("embed_search_custom")
            )?;
            self.registry.register(Box::new(counter.clone()))?;
            counter.inc();
            counters.insert(name.to_string(), counter);
        }
        
        Ok(())
    }
    
    pub fn set_custom_gauge(&self, name: &str, value: f64) -> Result<(), prometheus::Error> {
        let mut gauges = self.custom_gauges.lock().unwrap();
        
        if let Some(gauge) = gauges.get(name) {
            gauge.set(value);
        } else {
            let gauge = Gauge::with_opts(
                Opts::new(name, format!("Custom gauge: {}", name))
                    .namespace("embed_search_custom")
            )?;
            self.registry.register(Box::new(gauge.clone()))?;
            gauge.set(value);
            gauges.insert(name.to_string(), gauge);
        }
        
        Ok(())
    }
    
    // Get metrics as Prometheus text format
    pub fn get_metrics(&self) -> Result<String, prometheus::Error> {
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer)?;
        String::from_utf8(buffer).map_err(|e| prometheus::Error::Msg(format!("UTF-8 error: {}", e)))
    }
}

// Global metrics instance
pub fn get_metrics() -> &'static MetricsCollector {
    static METRICS: OnceLock<MetricsCollector> = OnceLock::new();
    METRICS.get_or_init(|| {
        MetricsCollector::new().expect("Failed to initialize metrics collector")
    })
}

// Metrics update scheduler
pub struct MetricsUpdater {
    metrics: &'static MetricsCollector,
    memory_guard: Arc<MemoryGuard>,
    file_pool: Arc<FilePool>,
    http_pool: Arc<ConnectionPool<HttpConnection>>,
    thread_pool: Arc<ThreadPool>,
}

impl MetricsUpdater {
    pub fn new(
        memory_guard: Arc<MemoryGuard>,
        file_pool: Arc<FilePool>,
        http_pool: Arc<ConnectionPool<HttpConnection>>,
        thread_pool: Arc<ThreadPool>,
    ) -> Self {
        Self {
            metrics: get_metrics(),
            memory_guard,
            file_pool,
            http_pool,
            thread_pool,
        }
    }
    
    pub async fn start_periodic_updates(&self) {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(10));
        
        loop {
            interval.tick().await;
            self.update_all_metrics().await;
        }
    }
    
    async fn update_all_metrics(&self) {
        // Update memory metrics
        let memory_stats = self.memory_guard.get_memory_stats();
        self.metrics.set_memory_usage(memory_stats.used_system as f64);
        
        // Update file handle metrics
        let file_stats = self.file_pool.get_pool_stats();
        self.metrics.set_file_handles_active(file_stats.total_files as i64);
        
        // Update connection metrics
        let conn_stats = self.http_pool.get_stats();
        self.metrics.set_connections_active(conn_stats.total_connections as i64);
        
        // Update thread pool metrics
        let thread_stats = self.thread_pool.get_stats();
        self.metrics.set_thread_pool_utilization(thread_stats.utilization);
    }
}
```

### 3. Create HTTP Metrics Endpoint (3 min)
```rust
// HTTP server for metrics endpoint
use warp::Filter;

pub async fn start_metrics_server(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let metrics_route = warp::path!("metrics")
        .and(warp::get())
        .map(|| {
            match get_metrics().get_metrics() {
                Ok(metrics) => {
                    warp::reply::with_header(
                        warp::reply::with_status(metrics, warp::http::StatusCode::OK),
                        "content-type",
                        "text/plain; version=0.0.4; charset=utf-8",
                    )
                }
                Err(e) => {
                    warp::reply::with_header(
                        warp::reply::with_status(
                            format!("Error gathering metrics: {}", e),
                            warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                        ),
                        "content-type",
                        "text/plain",
                    )
                }
            }
        });
    
    let health_route = warp::path!("health")
        .and(warp::get())
        .map(|| {
            warp::reply::json(&serde_json::json!({
                "status": "healthy",
                "timestamp": chrono::Utc::now().to_rfc3339()
            }))
        });
    
    let routes = metrics_route.or(health_route);
    
    println!("Starting metrics server on port {}", port);
    warp::serve(routes)
        .run(([0, 0, 0, 0], port))
        .await;
    
    Ok(())
}

// Middleware for automatic metrics collection
pub fn with_metrics<F, R>(
    operation_name: &str,
    operation: F,
) -> impl Future<Output = Result<R, Box<dyn std::error::Error>>>
where
    F: Future<Output = Result<R, Box<dyn std::error::Error>>>,
{
    let operation_name = operation_name.to_string();
    async move {
        let start_time = Instant::now();
        let metrics = get_metrics();
        
        // Increment request counter
        metrics.inc_search_requests();
        let _ = metrics.inc_custom_counter(&format!("{}_requests", operation_name));
        
        let result = operation.await;
        
        let duration = start_time.elapsed().as_secs_f64();
        metrics.observe_search_duration(duration);
        
        match &result {
            Ok(_) => {
                let _ = metrics.inc_custom_counter(&format!("{}_successes", operation_name));
            }
            Err(_) => {
                metrics.inc_search_errors();
                let _ = metrics.inc_custom_counter(&format!("{}_errors", operation_name));
            }
        }
        
        result
    }
}

// Integration with search operations
pub struct MetricsAwareSearchEngine {
    inner: SearchEngine,
    metrics: &'static MetricsCollector,
}

impl MetricsAwareSearchEngine {
    pub fn new(search_engine: SearchEngine) -> Self {
        Self {
            inner: search_engine,
            metrics: get_metrics(),
        }
    }
    
    pub async fn search(&self, query: &SearchQuery) -> Result<SearchResults, SearchError> {
        let start_time = Instant::now();
        self.metrics.inc_search_requests();
        
        let result = self.inner.search(query).await;
        
        let duration = start_time.elapsed().as_secs_f64();
        self.metrics.observe_search_duration(duration);
        
        match &result {
            Ok(results) => {
                self.metrics.add_search_results(results.len() as u64);
            }
            Err(_) => {
                self.metrics.inc_search_errors();
            }
        }
        
        result
    }
}
```

## Validation
- [ ] Prometheus metrics are properly registered
- [ ] Metrics endpoint returns valid Prometheus format
- [ ] Resource metrics update correctly
- [ ] Search metrics track requests and performance
- [ ] Custom metrics can be added dynamically

## Success Criteria
- Comprehensive Prometheus metrics for all system components
- HTTP endpoint serves metrics in Prometheus format
- Automatic metrics collection for search operations
- Resource utilization metrics are accurate
- Performance metrics track latency and throughput

## Next Task
task_014 - Implement counter and histogram metrics
