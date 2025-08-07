# Task 008: CRITICAL - Implement Memory Management with Limits and Guards

## Objective
Implement comprehensive memory management system with configurable limits, guards, and automatic cleanup to prevent OOM conditions.

## Time Estimate
10 minutes

## Priority
CRITICAL - Essential for production stability and preventing system crashes

## Dependencies
- Basic system monitoring capabilities

## Implementation Steps

### 1. Create Memory Guard System (4 min)
```rust
// src/resources/memory_guard.rs
use std::sync::{Arc, Mutex, Weak};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use sysinfo::{System, SystemExt};

#[derive(Debug, Clone)]
pub struct MemoryLimits {
    pub max_heap_size: usize,
    pub max_per_operation: usize,
    pub warning_threshold: f64,  // 0.0 - 1.0
    pub critical_threshold: f64, // 0.0 - 1.0
    pub cleanup_threshold: f64,  // 0.0 - 1.0
}

impl Default for MemoryLimits {
    fn default() -> Self {
        Self {
            max_heap_size: 2 * 1024 * 1024 * 1024, // 2GB
            max_per_operation: 512 * 1024 * 1024,   // 512MB
            warning_threshold: 0.75,
            critical_threshold: 0.85,
            cleanup_threshold: 0.9,
        }
    }
}

pub struct MemoryGuard {
    limits: MemoryLimits,
    allocations: Arc<Mutex<HashMap<String, AllocationInfo>>>,
    system: Arc<Mutex<System>>,
    last_cleanup: Arc<Mutex<Instant>>,
}

#[derive(Debug, Clone)]
struct AllocationInfo {
    size: usize,
    timestamp: Instant,
    operation_id: String,
    cleanup_callback: Option<Weak<dyn Fn() + Send + Sync>>,
}

impl MemoryGuard {
    pub fn new(limits: MemoryLimits) -> Self {
        Self {
            limits,
            allocations: Arc::new(Mutex::new(HashMap::new())),
            system: Arc::new(Mutex::new(System::new_all())),
            last_cleanup: Arc::new(Mutex::new(Instant::now())),
        }
    }

    pub async fn request_allocation(
        &self, 
        operation_id: &str, 
        size: usize
    ) -> Result<AllocationGuard, MemoryError> {
        // Check if single operation exceeds limit
        if size > self.limits.max_per_operation {
            return Err(MemoryError::AllocationTooLarge {
                requested: size,
                limit: self.limits.max_per_operation,
            });
        }

        // Check current memory usage
        let current_usage = self.get_current_memory_usage().await?;
        let usage_ratio = current_usage as f64 / self.limits.max_heap_size as f64;

        // Trigger cleanup if necessary
        if usage_ratio > self.limits.cleanup_threshold {
            self.trigger_cleanup().await;
        }

        // Recheck after cleanup
        let current_usage = self.get_current_memory_usage().await?;
        if current_usage + size > self.limits.max_heap_size {
            return Err(MemoryError::InsufficientMemory {
                available: self.limits.max_heap_size.saturating_sub(current_usage),
                requested: size,
            });
        }

        // Record allocation
        let allocation_info = AllocationInfo {
            size,
            timestamp: Instant::now(),
            operation_id: operation_id.to_string(),
            cleanup_callback: None,
        };

        let allocation_id = format!("{}_{}", operation_id, Instant::now().elapsed().as_nanos());
        self.allocations.lock().unwrap().insert(allocation_id.clone(), allocation_info);

        Ok(AllocationGuard {
            id: allocation_id,
            size,
            guard: Arc::downgrade(&Arc::new(self.clone())),
        })
    }
}
```

### 2. Implement Allocation Guard and Automatic Cleanup (3 min)
```rust
pub struct AllocationGuard {
    id: String,
    size: usize,
    guard: Weak<MemoryGuard>,
}

impl Drop for AllocationGuard {
    fn drop(&mut self) {
        if let Some(guard) = self.guard.upgrade() {
            guard.allocations.lock().unwrap().remove(&self.id);
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MemoryError {
    #[error("Allocation too large: requested {requested} bytes, limit is {limit} bytes")]
    AllocationTooLarge { requested: usize, limit: usize },
    #[error("Insufficient memory: requested {requested} bytes, available {available} bytes")]
    InsufficientMemory { requested: usize, available: usize },
    #[error("Memory pressure too high: current usage {usage:.1}%")]
    MemoryPressureTooHigh { usage: f64 },
    #[error("System memory monitoring error: {0}")]
    SystemError(String),
}

impl MemoryGuard {
    async fn get_current_memory_usage(&self) -> Result<usize, MemoryError> {
        let mut system = self.system.lock().unwrap();
        system.refresh_memory();
        
        let used_memory = system.used_memory() as usize;
        Ok(used_memory)
    }

    async fn trigger_cleanup(&self) {
        let mut last_cleanup = self.last_cleanup.lock().unwrap();
        let now = Instant::now();
        
        // Rate limit cleanup operations
        if now.duration_since(*last_cleanup) < Duration::from_secs(5) {
            return;
        }
        
        *last_cleanup = now;
        drop(last_cleanup);
        
        // Remove old allocations
        let mut allocations = self.allocations.lock().unwrap();
        let cutoff_time = now - Duration::from_secs(300); // 5 minutes
        
        allocations.retain(|_, info| info.timestamp > cutoff_time);
        
        // Trigger garbage collection
        #[cfg(feature = "jemalloc")]
        {
            use tikv_jemalloc_ctl::{epoch, stats};
            epoch::advance().unwrap();
        }
        
        // Force garbage collection in Rust (limited effect)
        std::hint::black_box(Vec::<u8>::with_capacity(1));
    }

    pub fn get_memory_stats(&self) -> MemoryStats {
        let allocations = self.allocations.lock().unwrap();
        let total_tracked = allocations.values().map(|a| a.size).sum();
        
        let system = self.system.lock().unwrap();
        let system_used = system.used_memory() as usize;
        let system_total = system.total_memory() as usize;
        
        MemoryStats {
            total_system: system_total,
            used_system: system_used,
            tracked_allocations: total_tracked,
            allocation_count: allocations.len(),
            usage_ratio: system_used as f64 / system_total as f64,
            limit_ratio: system_used as f64 / self.limits.max_heap_size as f64,
        }
    }
}

#[derive(Debug)]
pub struct MemoryStats {
    pub total_system: usize,
    pub used_system: usize,
    pub tracked_allocations: usize,
    pub allocation_count: usize,
    pub usage_ratio: f64,
    pub limit_ratio: f64,
}
```

### 3. Integrate with Search Operations (3 min)
```rust
// src/search/memory_aware.rs
use crate::resources::memory_guard::{MemoryGuard, AllocationGuard, MemoryError};
use std::sync::Arc;

#[async_trait::async_trait]
pub trait MemoryAwareOperation {
    async fn estimate_memory_usage(&self) -> usize;
    async fn execute_with_memory_guard(
        &self,
        guard: Arc<MemoryGuard>,
        operation_id: &str,
    ) -> Result<Self::Output, MemoryError>;
}

// Example implementation for vector search
impl MemoryAwareOperation for VectorSearchQuery {
    type Output = Vec<SearchResult>;
    
    async fn estimate_memory_usage(&self) -> usize {
        // Estimate based on query complexity and expected results
        let embedding_size = 1536 * 4; // OpenAI embedding size * float32
        let estimated_results = std::cmp::min(self.limit.unwrap_or(100), 1000);
        let result_overhead = estimated_results * 1024; // ~1KB per result
        
        embedding_size + result_overhead
    }
    
    async fn execute_with_memory_guard(
        &self,
        guard: Arc<MemoryGuard>,
        operation_id: &str,
    ) -> Result<Vec<SearchResult>, MemoryError> {
        let estimated_size = self.estimate_memory_usage().await;
        let _allocation_guard = guard.request_allocation(operation_id, estimated_size).await?;
        
        // Execute the actual search with memory protection
        match self.execute().await {
            Ok(results) => Ok(results),
            Err(e) => Err(MemoryError::SystemError(format!("Search failed: {}", e))),
        }
    }
}

// Memory-aware search wrapper
pub struct MemoryAwareSearchEngine {
    memory_guard: Arc<MemoryGuard>,
    vector_search: VectorSearch,
    keyword_search: KeywordSearch,
    fuzzy_search: FuzzySearch,
}

impl MemoryAwareSearchEngine {
    pub fn new(memory_guard: Arc<MemoryGuard>) -> Self {
        Self {
            memory_guard,
            vector_search: VectorSearch::new(),
            keyword_search: KeywordSearch::new(),
            fuzzy_search: FuzzySearch::new(),
        }
    }
    
    pub async fn search(&self, query: &SearchQuery) -> Result<SearchResults, MemoryError> {
        let operation_id = format!("search_{}", uuid::Uuid::new_v4());
        
        // Check memory before starting
        let stats = self.memory_guard.get_memory_stats();
        if stats.usage_ratio > 0.9 {
            return Err(MemoryError::MemoryPressureTooHigh { 
                usage: stats.usage_ratio * 100.0 
            });
        }
        
        match query.method {
            SearchMethod::Vector => {
                query.execute_with_memory_guard(self.memory_guard.clone(), &operation_id).await
            }
            // ... handle other methods
        }
    }
}
```

## Validation
- [ ] Memory limits are enforced correctly
- [ ] Allocation guards prevent memory leaks
- [ ] Automatic cleanup triggers at thresholds
- [ ] Large allocations are rejected appropriately
- [ ] Memory stats provide accurate information

## Success Criteria
- Memory guard system prevents OOM conditions
- Configurable limits for different operation types
- Automatic cleanup reduces memory pressure
- Integration with search operations works correctly
- Memory statistics provide visibility into usage

## Next Task
task_009 - Implement file handle pooling system
