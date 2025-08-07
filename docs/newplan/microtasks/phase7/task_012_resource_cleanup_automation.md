# Task 012: Resource Cleanup Automation

## Objective
Implement automated resource cleanup system to prevent resource leaks and maintain system health over long periods.

## Time Estimate
10 minutes

## Priority
HIGH - Essential for long-running production systems

## Dependencies
- Resource management systems (memory, files, connections)

## Implementation Steps

### 1. Create Resource Cleanup Coordinator (4 min)
```rust
// src/resources/cleanup_coordinator.rs
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::time::interval;
use async_trait::async_trait;

#[async_trait]
pub trait ResourceCleaner: Send + Sync {
    async fn cleanup(&self) -> CleanupResult;
    fn name(&self) -> &str;
    fn cleanup_interval(&self) -> Duration;
}

#[derive(Debug, Clone)]
pub struct CleanupResult {
    pub cleaned_items: usize,
    pub freed_memory: usize,
    pub errors: Vec<String>,
    pub duration: Duration,
}

#[derive(Debug, Clone)]
pub struct CleanupConfig {
    pub global_interval: Duration,
    pub max_cleanup_time: Duration,
    pub enable_aggressive_cleanup: bool,
    pub memory_pressure_threshold: f64,
    pub cleanup_on_low_memory: bool,
}

impl Default for CleanupConfig {
    fn default() -> Self {
        Self {
            global_interval: Duration::from_secs(60),
            max_cleanup_time: Duration::from_secs(30),
            enable_aggressive_cleanup: false,
            memory_pressure_threshold: 0.85,
            cleanup_on_low_memory: true,
        }
    }
}

pub struct CleanupCoordinator {
    config: CleanupConfig,
    cleaners: Arc<Mutex<HashMap<String, Arc<dyn ResourceCleaner>>>>,
    cleanup_stats: Arc<Mutex<HashMap<String, CleanupStats>>>,
    running: Arc<Mutex<bool>>,
}

#[derive(Debug, Default)]
struct CleanupStats {
    total_cleanups: u64,
    total_cleaned_items: u64,
    total_freed_memory: usize,
    total_cleanup_time: Duration,
    last_cleanup: Option<Instant>,
    error_count: u64,
}

impl CleanupCoordinator {
    pub fn new(config: CleanupConfig) -> Self {
        Self {
            config,
            cleaners: Arc::new(Mutex::new(HashMap::new())),
            cleanup_stats: Arc::new(Mutex::new(HashMap::new())),
            running: Arc::new(Mutex::new(false)),
        }
    }

    pub fn register_cleaner(&self, cleaner: Arc<dyn ResourceCleaner>) {
        let name = cleaner.name().to_string();
        self.cleaners.lock().unwrap().insert(name.clone(), cleaner);
        self.cleanup_stats.lock().unwrap().insert(name, CleanupStats::default());
    }

    pub async fn start(&self) {
        let mut running = self.running.lock().unwrap();
        if *running {
            return; // Already running
        }
        *running = true;
        drop(running);

        let coordinator = Arc::new(self.clone());
        tokio::spawn(async move {
            coordinator.cleanup_loop().await;
        });
    }

    pub async fn stop(&self) {
        *self.running.lock().unwrap() = false;
    }

    async fn cleanup_loop(self: Arc<Self>) {
        let mut cleanup_interval = interval(self.config.global_interval);
        
        while *self.running.lock().unwrap() {
            cleanup_interval.tick().await;
            
            // Check if we need aggressive cleanup due to memory pressure
            let aggressive_cleanup = if self.config.cleanup_on_low_memory {
                self.check_memory_pressure().await
            } else {
                false
            };
            
            if aggressive_cleanup || self.config.enable_aggressive_cleanup {
                self.run_aggressive_cleanup().await;
            } else {
                self.run_normal_cleanup().await;
            }
        }
    }

    async fn run_normal_cleanup(&self) {
        let cleaners = self.cleaners.lock().unwrap().clone();
        
        for (name, cleaner) in cleaners {
            // Check if this cleaner should run based on its interval
            let should_run = {
                let stats = self.cleanup_stats.lock().unwrap();
                let cleaner_stats = stats.get(&name).unwrap();
                
                match cleaner_stats.last_cleanup {
                    Some(last) => last.elapsed() >= cleaner.cleanup_interval(),
                    None => true, // First run
                }
            };
            
            if should_run {
                self.run_cleaner(&name, cleaner, false).await;
            }
        }
    }

    async fn run_aggressive_cleanup(&self) {
        let cleaners = self.cleaners.lock().unwrap().clone();
        
        // Run all cleaners in parallel for aggressive cleanup
        let mut handles = Vec::new();
        
        for (name, cleaner) in cleaners {
            let coordinator = Arc::clone(&Arc::new(self.clone()));
            let handle = tokio::spawn(async move {
                coordinator.run_cleaner(&name, cleaner, true).await;
            });
            handles.push(handle);
        }
        
        // Wait for all cleaners with timeout
        let timeout = tokio::time::timeout(
            self.config.max_cleanup_time,
            futures::future::join_all(handles)
        );
        
        if timeout.await.is_err() {
            eprintln!("Cleanup timeout exceeded - some cleaners may still be running");
        }
    }
}
```

### 2. Implement Individual Resource Cleaners (3 min)
```rust
impl CleanupCoordinator {
    async fn run_cleaner(&self, name: &str, cleaner: Arc<dyn ResourceCleaner>, aggressive: bool) {
        let start_time = Instant::now();
        
        let result = if aggressive {
            // For aggressive cleanup, we might want different behavior
            cleaner.cleanup().await
        } else {
            cleaner.cleanup().await
        };
        
        let cleanup_duration = start_time.elapsed();
        
        // Update stats
        {
            let mut stats = self.cleanup_stats.lock().unwrap();
            let cleaner_stats = stats.get_mut(name).unwrap();
            
            cleaner_stats.total_cleanups += 1;
            cleaner_stats.total_cleaned_items += result.cleaned_items as u64;
            cleaner_stats.total_freed_memory += result.freed_memory;
            cleaner_stats.total_cleanup_time += cleanup_duration;
            cleaner_stats.last_cleanup = Some(start_time);
            
            if !result.errors.is_empty() {
                cleaner_stats.error_count += result.errors.len() as u64;
            }
        }
        
        // Log results
        if !result.errors.is_empty() {
            eprintln!("Cleaner '{}' encountered {} errors: {:?}", name, result.errors.len(), result.errors);
        }
    }
    
    async fn check_memory_pressure(&self) -> bool {
        // Simple memory pressure check
        #[cfg(target_os = "linux")]
        {
            use std::fs;
            if let Ok(meminfo) = fs::read_to_string("/proc/meminfo") {
                let mut mem_total = 0;
                let mut mem_available = 0;
                
                for line in meminfo.lines() {
                    if line.starts_with("MemTotal:") {
                        mem_total = line.split_whitespace().nth(1).and_then(|s| s.parse().ok()).unwrap_or(0);
                    } else if line.starts_with("MemAvailable:") {
                        mem_available = line.split_whitespace().nth(1).and_then(|s| s.parse().ok()).unwrap_or(0);
                    }
                }
                
                if mem_total > 0 {
                    let usage_ratio = 1.0 - (mem_available as f64 / mem_total as f64);
                    return usage_ratio > self.config.memory_pressure_threshold;
                }
            }
        }
        
        false // Default to no pressure if we can't determine
    }
    
    pub fn get_cleanup_report(&self) -> CleanupReport {
        let stats = self.cleanup_stats.lock().unwrap();
        let mut cleaner_reports = Vec::new();
        
        for (name, cleaner_stats) in stats.iter() {
            cleaner_reports.push(CleanerReport {
                name: name.clone(),
                total_cleanups: cleaner_stats.total_cleanups,
                total_cleaned_items: cleaner_stats.total_cleaned_items,
                total_freed_memory: cleaner_stats.total_freed_memory,
                average_cleanup_time: if cleaner_stats.total_cleanups > 0 {
                    cleaner_stats.total_cleanup_time / cleaner_stats.total_cleanups as u32
                } else {
                    Duration::default()
                },
                last_cleanup: cleaner_stats.last_cleanup,
                error_count: cleaner_stats.error_count,
                error_rate: if cleaner_stats.total_cleanups > 0 {
                    cleaner_stats.error_count as f64 / cleaner_stats.total_cleanups as f64
                } else {
                    0.0
                },
            });
        }
        
        CleanupReport {
            cleaners: cleaner_reports,
            generated_at: Instant::now(),
        }
    }
}

#[derive(Debug)]
pub struct CleanupReport {
    pub cleaners: Vec<CleanerReport>,
    pub generated_at: Instant,
}

#[derive(Debug)]
pub struct CleanerReport {
    pub name: String,
    pub total_cleanups: u64,
    pub total_cleaned_items: u64,
    pub total_freed_memory: usize,
    pub average_cleanup_time: Duration,
    pub last_cleanup: Option<Instant>,
    pub error_count: u64,
    pub error_rate: f64,
}
```

### 3. Create Specific Cleaners for Different Resources (3 min)
```rust
// Memory cleaner
pub struct MemoryCleaner {
    memory_guard: Arc<MemoryGuard>,
}

impl MemoryCleaner {
    pub fn new(memory_guard: Arc<MemoryGuard>) -> Self {
        Self { memory_guard }
    }
}

#[async_trait]
impl ResourceCleaner for MemoryCleaner {
    async fn cleanup(&self) -> CleanupResult {
        let start_time = Instant::now();
        let before_stats = self.memory_guard.get_memory_stats();
        
        // Trigger memory cleanup
        self.memory_guard.trigger_cleanup().await;
        
        let after_stats = self.memory_guard.get_memory_stats();
        let freed_memory = before_stats.used_system.saturating_sub(after_stats.used_system);
        
        CleanupResult {
            cleaned_items: 1,
            freed_memory,
            errors: Vec::new(),
            duration: start_time.elapsed(),
        }
    }
    
    fn name(&self) -> &str {
        "memory_cleaner"
    }
    
    fn cleanup_interval(&self) -> Duration {
        Duration::from_secs(120) // Clean every 2 minutes
    }
}

// File handle cleaner
pub struct FileHandleCleaner {
    file_pool: Arc<FilePool>,
}

impl FileHandleCleaner {
    pub fn new(file_pool: Arc<FilePool>) -> Self {
        Self { file_pool }
    }
}

#[async_trait]
impl ResourceCleaner for FileHandleCleaner {
    async fn cleanup(&self) -> CleanupResult {
        let start_time = Instant::now();
        let before_stats = self.file_pool.get_pool_stats();
        
        // Force cleanup of expired files
        self.file_pool.force_cleanup().await;
        
        let after_stats = self.file_pool.get_pool_stats();
        let cleaned_files = before_stats.pooled_files.saturating_sub(after_stats.pooled_files);
        
        CleanupResult {
            cleaned_items: cleaned_files,
            freed_memory: cleaned_files * 1024, // Estimate
            errors: Vec::new(),
            duration: start_time.elapsed(),
        }
    }
    
    fn name(&self) -> &str {
        "file_handle_cleaner"
    }
    
    fn cleanup_interval(&self) -> Duration {
        Duration::from_secs(300) // Clean every 5 minutes
    }
}

// Connection pool cleaner
pub struct ConnectionPoolCleaner<T: Connection> {
    pool: Arc<ConnectionPool<T>>,
}

impl<T: Connection> ConnectionPoolCleaner<T> {
    pub fn new(pool: Arc<ConnectionPool<T>>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl<T: Connection + 'static> ResourceCleaner for ConnectionPoolCleaner<T> {
    async fn cleanup(&self) -> CleanupResult {
        let start_time = Instant::now();
        let before_stats = self.pool.get_stats();
        
        // Trigger connection cleanup
        self.pool.health_check_connections().await;
        
        let after_stats = self.pool.get_stats();
        let cleaned_connections = before_stats.idle_connections.saturating_sub(after_stats.idle_connections);
        
        CleanupResult {
            cleaned_items: cleaned_connections,
            freed_memory: cleaned_connections * 4096, // Estimate
            errors: Vec::new(),
            duration: start_time.elapsed(),
        }
    }
    
    fn name(&self) -> &str {
        "connection_pool_cleaner"
    }
    
    fn cleanup_interval(&self) -> Duration {
        Duration::from_secs(180) // Clean every 3 minutes
    }
}

// Integration example
pub async fn setup_cleanup_system(
    memory_guard: Arc<MemoryGuard>,
    file_pool: Arc<FilePool>,
    http_pool: Arc<ConnectionPool<HttpConnection>>,
) -> Arc<CleanupCoordinator> {
    let config = CleanupConfig {
        enable_aggressive_cleanup: false,
        cleanup_on_low_memory: true,
        memory_pressure_threshold: 0.80,
        ..Default::default()
    };
    
    let coordinator = Arc::new(CleanupCoordinator::new(config));
    
    // Register all cleaners
    coordinator.register_cleaner(Arc::new(MemoryCleaner::new(memory_guard)));
    coordinator.register_cleaner(Arc::new(FileHandleCleaner::new(file_pool)));
    coordinator.register_cleaner(Arc::new(ConnectionPoolCleaner::new(http_pool)));
    
    // Start cleanup loop
    coordinator.start().await;
    
    coordinator
}
```

## Validation
- [ ] Cleanup coordinator manages multiple resource types
- [ ] Individual cleaners run at appropriate intervals
- [ ] Aggressive cleanup triggers under memory pressure
- [ ] Cleanup statistics are collected accurately
- [ ] Error handling prevents cleanup failures

## Success Criteria
- Automated cleanup prevents resource leaks
- Memory pressure triggers aggressive cleanup
- Individual cleaners maintain their resources
- Comprehensive statistics track cleanup effectiveness
- System maintains health over extended periods

## Next Task
task_013 - Implement memory profiling system
