# Phase 7: Production Hardening - Making It Bulletproof

**Duration**: 1 week  
**Goal**: Production-ready system with monitoring, docs, and reliability  
**Success Metric**: Can run 24/7 without issues

## Task 7.1: Error Recovery (1 day)

### Implement Circuit Breakers

```rust
// File: src/reliability/circuit_breaker.rs

pub struct CircuitBreaker {
    failure_threshold: usize,
    reset_timeout: Duration,
    failure_count: AtomicUsize,
    last_failure: RwLock<Option<Instant>>,
    state: AtomicU8,  // 0=Closed, 1=Open, 2=HalfOpen
}

impl CircuitBreaker {
    pub async fn call<F, T>(&self, f: F) -> Result<T>
    where
        F: Future<Output = Result<T>>,
    {
        match self.state.load(Ordering::Relaxed) {
            0 => { // Closed - normal operation
                match f.await {
                    Ok(result) => {
                        self.reset();
                        Ok(result)
                    }
                    Err(e) => {
                        self.record_failure();
                        Err(e)
                    }
                }
            }
            1 => { // Open - fast fail
                Err(EmbedError::CircuitBreakerOpen)
            }
            2 => { // Half-Open - test if recovered
                match f.await {
                    Ok(result) => {
                        self.close();
                        Ok(result)
                    }
                    Err(e) => {
                        self.open();
                        Err(e)
                    }
                }
            }
            _ => unreachable!()
        }
    }
}
```

### Apply to Each Search Method

```rust
impl UnifiedSearcher {
    async fn search_with_breaker(&self, query: &str) -> Result<Vec<SearchResult>> {
        let (bm25_result, tantivy_result, native_result, semantic_result) = tokio::join!(
            self.bm25_breaker.call(self.search_bm25(query)),
            self.tantivy_breaker.call(self.search_tantivy(query)),
            self.native_breaker.call(self.search_native(query)),
            self.semantic_breaker.call(self.search_semantic(query)),
        );
        
        // Continue even if some methods are circuit-broken
        // ...
    }
}
```

## Task 7.2: Resource Management (1 day)

### Memory Limits

```rust
// File: src/resource/memory_guard.rs

pub struct MemoryGuard {
    max_memory_mb: usize,
}

impl MemoryGuard {
    pub fn check(&self) -> Result<()> {
        let usage = self.get_memory_usage()?;
        
        if usage > self.max_memory_mb {
            // Clear caches
            self.clear_caches()?;
            
            // Check again
            let usage = self.get_memory_usage()?;
            if usage > self.max_memory_mb {
                return Err(EmbedError::OutOfMemory(usage));
            }
        }
        
        Ok(())
    }
    
    fn get_memory_usage(&self) -> Result<usize> {
        // Platform-specific memory check
        #[cfg(target_os = "linux")]
        {
            let status = fs::read_to_string("/proc/self/status")?;
            // Parse VmRSS
        }
        
        #[cfg(target_os = "windows")]
        {
            // Use Windows API
        }
        
        Ok(0) // Placeholder
    }
}
```

### File Handle Limits

```rust
pub struct FileHandlePool {
    max_handles: usize,
    active_handles: Arc<AtomicUsize>,
}

impl FileHandlePool {
    pub fn acquire(&self) -> Result<FileHandleGuard> {
        let current = self.active_handles.fetch_add(1, Ordering::Relaxed);
        
        if current >= self.max_handles {
            self.active_handles.fetch_sub(1, Ordering::Relaxed);
            return Err(EmbedError::TooManyOpenFiles);
        }
        
        Ok(FileHandleGuard {
            pool: self.active_handles.clone(),
        })
    }
}
```

## Task 7.3: Monitoring & Metrics (1 day)

### Prometheus Metrics

```rust
// File: src/observability/metrics.rs

use prometheus::{Counter, Histogram, Registry};

pub struct Metrics {
    pub search_requests: Counter,
    pub search_errors: Counter,
    pub search_duration: Histogram,
    pub index_operations: Counter,
    pub cache_hits: Counter,
    pub cache_misses: Counter,
    pub embedding_duration: Histogram,
}

impl Metrics {
    pub fn new(registry: &Registry) -> Result<Self> {
        Ok(Self {
            search_requests: Counter::new(
                "embed_search_requests_total",
                "Total number of search requests"
            )?,
            search_errors: Counter::new(
                "embed_search_errors_total",
                "Total number of search errors"
            )?,
            search_duration: Histogram::with_opts(
                HistogramOpts::new(
                    "embed_search_duration_seconds",
                    "Search request duration"
                ).buckets(vec![0.01, 0.05, 0.1, 0.5, 1.0, 5.0])
            )?,
            // ... more metrics
        })
    }
}
```

### Health Checks

```rust
// File: src/health/health_check.rs

pub struct HealthChecker {
    components: Vec<Box<dyn HealthCheckable>>,
}

#[async_trait]
pub trait HealthCheckable {
    async fn check(&self) -> HealthStatus;
}

pub struct HealthStatus {
    pub healthy: bool,
    pub message: String,
    pub details: HashMap<String, String>,
}

impl HealthChecker {
    pub async fn check_all(&self) -> HealthReport {
        let mut report = HealthReport::new();
        
        for component in &self.components {
            let status = component.check().await;
            report.add_component(status);
        }
        
        report
    }
}

// HTTP endpoint
pub async fn health_endpoint() -> impl warp::Reply {
    let checker = get_health_checker();
    let report = checker.check_all().await;
    
    if report.healthy() {
        warp::reply::with_status(
            warp::reply::json(&report),
            StatusCode::OK,
        )
    } else {
        warp::reply::with_status(
            warp::reply::json(&report),
            StatusCode::SERVICE_UNAVAILABLE,
        )
    }
}
```

## Task 7.4: Logging & Tracing (1 day)

### Structured Logging

```rust
// File: src/observability/logging.rs

use tracing::{info, warn, error, instrument};

#[instrument(skip(self))]
pub async fn search(&self, query: &str) -> Result<Vec<SearchResult>> {
    info!(query = %query, "Starting search");
    
    let start = Instant::now();
    
    let result = self.search_internal(query).await;
    
    match &result {
        Ok(results) => {
            info!(
                query = %query,
                result_count = results.len(),
                duration_ms = start.elapsed().as_millis(),
                "Search completed successfully"
            );
        }
        Err(e) => {
            error!(
                query = %query,
                error = %e,
                duration_ms = start.elapsed().as_millis(),
                "Search failed"
            );
        }
    }
    
    result
}
```

### OpenTelemetry Integration

```rust
use opentelemetry::{global, sdk::propagation::TraceContextPropagator};
use tracing_subscriber::prelude::*;

pub fn init_telemetry() -> Result<()> {
    global::set_text_map_propagator(TraceContextPropagator::new());
    
    let tracer = opentelemetry_jaeger::new_pipeline()
        .with_service_name("embed-search")
        .install_batch(opentelemetry::runtime::Tokio)?;
    
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
    
    let subscriber = tracing_subscriber::registry()
        .with(telemetry)
        .with(tracing_subscriber::fmt::layer());
    
    tracing::subscriber::set_global_default(subscriber)?;
    
    Ok(())
}
```

## Task 7.5: API Documentation (1 day)

### OpenAPI Specification

```yaml
# File: api/openapi.yaml
openapi: 3.0.0
info:
  title: Embed Search API
  version: 1.0.0
  description: Multi-modal search system with ML capabilities

paths:
  /search:
    post:
      summary: Perform search
      requestBody:
        required: true
        content:
          application/json:
            schema:
              type: object
              properties:
                query:
                  type: string
                  description: Search query
                limit:
                  type: integer
                  default: 10
                methods:
                  type: array
                  items:
                    type: string
                    enum: [bm25, tantivy, native, semantic]
      responses:
        200:
          description: Search results
          content:
            application/json:
              schema:
                type: object
                properties:
                  results:
                    type: array
                    items:
                      $ref: '#/components/schemas/SearchResult'
                  total:
                    type: integer
                  duration_ms:
                    type: number

  /health:
    get:
      summary: Health check
      responses:
        200:
          description: System healthy
        503:
          description: System unhealthy
```

### Code Documentation

```rust
/// Unified search orchestrator that coordinates multiple search backends.
/// 
/// # Example
/// ```rust
/// let searcher = UnifiedSearcher::new(config).await?;
/// let results = searcher.search("async fn").await?;
/// ```
/// 
/// # Errors
/// Returns `EmbedError` if all search methods fail or configuration is invalid.
pub struct UnifiedSearcher {
    // ...
}
```

## Task 7.6: Deployment Configuration (1 day)

### Docker Setup

```dockerfile
# File: Dockerfile
FROM rust:1.75 as builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Build with all features
RUN cargo build --release --features full-system

# Runtime image
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/embed-search /usr/local/bin/
COPY models /models

ENV RUST_LOG=info
ENV MODEL_PATH=/models/nomic-embed-text-v1.5.Q4_K_M.gguf

EXPOSE 8080

CMD ["embed-search", "serve"]
```

### Kubernetes Deployment

```yaml
# File: k8s/deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: embed-search
spec:
  replicas: 3
  selector:
    matchLabels:
      app: embed-search
  template:
    metadata:
      labels:
        app: embed-search
    spec:
      containers:
      - name: embed-search
        image: embed-search:latest
        ports:
        - containerPort: 8080
        resources:
          requests:
            memory: "2Gi"
            cpu: "1"
          limits:
            memory: "4Gi"
            cpu: "2"
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
```

## Task 7.7: Performance Tuning (1 day)

### Profile and Optimize

```bash
# Profile with flamegraph
cargo flamegraph --release --features full-system -- search "test query"

# Profile with perf
perf record -g ./target/release/embed-search search "test"
perf report
```

### Optimization Checklist

- [ ] Enable LTO in release builds
- [ ] Use jemalloc for better memory performance
- [ ] Optimize index structures
- [ ] Tune cache sizes based on memory
- [ ] Enable SIMD for embeddings
- [ ] Use connection pooling for DB
- [ ] Implement request batching

## Success Criteria

- [ ] Circuit breakers prevent cascade failures
- [ ] Memory usage stays within limits
- [ ] Metrics exported to Prometheus
- [ ] Health checks accurate
- [ ] Logs structured and traceable
- [ ] API fully documented
- [ ] Docker image builds and runs
- [ ] Kubernetes deployment works
- [ ] Performance optimized

## Production Checklist

- [ ] All tests pass
- [ ] Documentation complete
- [ ] Security scan passed
- [ ] Load testing completed
- [ ] Monitoring configured
- [ ] Alerts set up
- [ ] Backup strategy defined
- [ ] Rollback procedure documented
- [ ] SLA defined

## Final Validation

Run continuous load test for 24 hours:
```bash
# Start system
docker-compose up -d

# Run load test
ab -n 100000 -c 100 http://localhost:8080/search

# Check metrics
curl http://localhost:8080/metrics

# Check health
curl http://localhost:8080/health
```

System must:
- Handle 100 req/s sustained
- P99 latency < 1 second
- Zero crashes or panics
- Memory stable (no leaks)
- All health checks green