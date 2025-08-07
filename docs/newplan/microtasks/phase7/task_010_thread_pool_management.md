# Task 010: Thread Pool Management System

## Objective
Implement intelligent thread pool management with dynamic sizing, priority queues, and workload balancing.

## Time Estimate
10 minutes

## Priority
HIGH - Essential for optimal resource utilization and performance

## Dependencies
- Basic async runtime infrastructure

## Implementation Steps

### 1. Create Adaptive Thread Pool (4 min)
```rust
// src/resources/thread_pool.rs
use std::sync::{Arc, Mutex};
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};
use tokio::sync::{Semaphore, oneshot};
use tokio::task::JoinHandle;

#[derive(Debug, Clone)]
pub struct ThreadPoolConfig {
    pub min_threads: usize,
    pub max_threads: usize,
    pub idle_timeout: Duration,
    pub queue_size_limit: usize,
    pub load_balancing_interval: Duration,
}

impl Default for ThreadPoolConfig {
    fn default() -> Self {
        let cpu_count = num_cpus::get();
        Self {
            min_threads: cpu_count,
            max_threads: cpu_count * 4,
            idle_timeout: Duration::from_secs(60),
            queue_size_limit: 10000,
            load_balancing_interval: Duration::from_secs(10),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

#[derive(Debug)]
struct PooledTask {
    id: String,
    priority: TaskPriority,
    submitted_at: Instant,
    estimated_duration: Option<Duration>,
    task: Box<dyn FnOnce() -> Box<dyn std::any::Any + Send> + Send>,
    result_sender: oneshot::Sender<Box<dyn std::any::Any + Send>>,
}

pub struct ThreadPool {
    config: ThreadPoolConfig,
    active_threads: Arc<Mutex<usize>>,
    task_queues: Arc<Mutex<HashMap<TaskPriority, VecDeque<PooledTask>>>>,
    semaphore: Arc<Semaphore>,
    shutdown_flag: Arc<Mutex<bool>>,
    stats: Arc<Mutex<ThreadPoolStats>>,
}

#[derive(Debug, Default)]
struct ThreadPoolStats {
    tasks_submitted: u64,
    tasks_completed: u64,
    tasks_failed: u64,
    total_execution_time: Duration,
    average_queue_time: Duration,
    peak_active_threads: usize,
    current_queue_size: usize,
}

impl ThreadPool {
    pub fn new(config: ThreadPoolConfig) -> Self {
        let semaphore = Arc::new(Semaphore::new(config.max_threads));
        
        Self {
            config,
            active_threads: Arc::new(Mutex::new(0)),
            task_queues: Arc::new(Mutex::new({
                let mut queues = HashMap::new();
                queues.insert(TaskPriority::Critical, VecDeque::new());
                queues.insert(TaskPriority::High, VecDeque::new());
                queues.insert(TaskPriority::Normal, VecDeque::new());
                queues.insert(TaskPriority::Low, VecDeque::new());
                queues
            })),
            semaphore,
            shutdown_flag: Arc::new(Mutex::new(false)),
            stats: Arc::new(Mutex::new(ThreadPoolStats::default())),
        }
    }

    pub async fn submit_task<F, R>(
        &self,
        task: F,
        priority: TaskPriority,
        estimated_duration: Option<Duration>,
    ) -> Result<R, ThreadPoolError>
    where
        F: FnOnce() -> R + Send + 'static,
        R: Send + 'static,
    {
        if *self.shutdown_flag.lock().unwrap() {
            return Err(ThreadPoolError::PoolShutdown);
        }

        // Check queue limits
        let current_queue_size = self.get_total_queue_size();
        if current_queue_size >= self.config.queue_size_limit {
            return Err(ThreadPoolError::QueueFull {
                current: current_queue_size,
                limit: self.config.queue_size_limit,
            });
        }

        let (sender, receiver) = oneshot::channel();
        let task_id = uuid::Uuid::new_v4().to_string();
        
        let pooled_task = PooledTask {
            id: task_id.clone(),
            priority,
            submitted_at: Instant::now(),
            estimated_duration,
            task: Box::new(move || Box::new(task())),
            result_sender: sender,
        };

        // Add to appropriate priority queue
        {
            let mut queues = self.task_queues.lock().unwrap();
            let queue = queues.get_mut(&priority).unwrap();
            queue.push_back(pooled_task);
            
            let mut stats = self.stats.lock().unwrap();
            stats.tasks_submitted += 1;
            stats.current_queue_size = self.get_total_queue_size_unlocked(&queues);
        }

        // Spawn worker if needed
        self.maybe_spawn_worker().await;

        // Wait for result
        let result = receiver.await
            .map_err(|_| ThreadPoolError::TaskCancelled)?;
        
        // Downcast result
        result.downcast::<R>()
            .map(|boxed| *boxed)
            .map_err(|_| ThreadPoolError::ResultTypeMismatch)
    }
}
```

### 2. Implement Worker Management and Load Balancing (3 min)
```rust
impl ThreadPool {
    async fn maybe_spawn_worker(&self) {
        let active = *self.active_threads.lock().unwrap();
        let queue_size = self.get_total_queue_size();
        
        // Dynamic thread scaling based on queue size and load
        let should_spawn = active < self.config.max_threads && 
            (queue_size > active * 2 || active < self.config.min_threads);
        
        if should_spawn {
            if let Ok(_permit) = self.semaphore.clone().try_acquire_owned() {
                *self.active_threads.lock().unwrap() += 1;
                
                let pool_ref = Arc::new(self.clone());
                tokio::spawn(async move {
                    pool_ref.worker_loop().await;
                });
            }
        }
    }
    
    async fn worker_loop(self: Arc<Self>) {
        let mut idle_start: Option<Instant> = None;
        
        loop {
            if *self.shutdown_flag.lock().unwrap() {
                break;
            }
            
            // Try to get next task from priority queues
            let task = self.get_next_task();
            
            match task {
                Some(task) => {
                    idle_start = None;
                    let start_time = Instant::now();
                    
                    // Execute task
                    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        (task.task)()
                    }));
                    
                    let execution_time = start_time.elapsed();
                    let queue_time = start_time.duration_since(task.submitted_at);
                    
                    // Update stats
                    {
                        let mut stats = self.stats.lock().unwrap();
                        match result {
                            Ok(_) => stats.tasks_completed += 1,
                            Err(_) => stats.tasks_failed += 1,
                        }
                        stats.total_execution_time += execution_time;
                        stats.average_queue_time = Duration::from_nanos(
                            (stats.average_queue_time.as_nanos() as u64 + queue_time.as_nanos() as u64) / 2
                        );
                    }
                    
                    // Send result
                    match result {
                        Ok(result) => {
                            let _ = task.result_sender.send(result);
                        }
                        Err(_) => {
                            // Task panicked - don't send result, receiver will get error
                        }
                    }
                }
                None => {
                    // No tasks available
                    if idle_start.is_none() {
                        idle_start = Some(Instant::now());
                    }
                    
                    // Check if we should exit due to idle timeout
                    if let Some(idle_time) = idle_start {
                        if idle_time.elapsed() > self.config.idle_timeout {
                            let active = *self.active_threads.lock().unwrap();
                            if active > self.config.min_threads {
                                break;
                            }
                        }
                    }
                    
                    // Short sleep to avoid busy waiting
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
            }
        }
        
        // Worker is exiting
        *self.active_threads.lock().unwrap() -= 1;
    }
    
    fn get_next_task(&self) -> Option<PooledTask> {
        let mut queues = self.task_queues.lock().unwrap();
        
        // Process in priority order
        for priority in [TaskPriority::Critical, TaskPriority::High, TaskPriority::Normal, TaskPriority::Low] {
            if let Some(queue) = queues.get_mut(&priority) {
                if let Some(task) = queue.pop_front() {
                    return Some(task);
                }
            }
        }
        
        None
    }
    
    fn get_total_queue_size(&self) -> usize {
        let queues = self.task_queues.lock().unwrap();
        self.get_total_queue_size_unlocked(&queues)
    }
    
    fn get_total_queue_size_unlocked(&self, queues: &HashMap<TaskPriority, VecDeque<PooledTask>>) -> usize {
        queues.values().map(|q| q.len()).sum()
    }
}
```

### 3. Add Monitoring and Error Handling (3 min)
```rust
#[derive(Debug, thiserror::Error)]
pub enum ThreadPoolError {
    #[error("Thread pool has been shut down")]
    PoolShutdown,
    #[error("Task queue is full: {current} tasks queued, limit is {limit}")]
    QueueFull { current: usize, limit: usize },
    #[error("Task was cancelled")]
    TaskCancelled,
    #[error("Result type mismatch")]
    ResultTypeMismatch,
    #[error("Task execution failed: {0}")]
    TaskExecutionFailed(String),
}

impl ThreadPool {
    pub fn get_stats(&self) -> ThreadPoolStats {
        let stats = self.stats.lock().unwrap();
        let active_threads = *self.active_threads.lock().unwrap();
        let queue_size = self.get_total_queue_size();
        
        ThreadPoolStats {
            tasks_submitted: stats.tasks_submitted,
            tasks_completed: stats.tasks_completed,
            tasks_failed: stats.tasks_failed,
            total_execution_time: stats.total_execution_time,
            average_queue_time: stats.average_queue_time,
            peak_active_threads: std::cmp::max(stats.peak_active_threads, active_threads),
            current_queue_size: queue_size,
            active_threads,
            utilization: if self.config.max_threads > 0 {
                active_threads as f64 / self.config.max_threads as f64
            } else {
                0.0
            },
        }
    }
    
    pub async fn shutdown(&self, timeout: Duration) -> Result<(), ThreadPoolError> {
        *self.shutdown_flag.lock().unwrap() = true;
        
        let start = Instant::now();
        while *self.active_threads.lock().unwrap() > 0 && start.elapsed() < timeout {
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        
        if *self.active_threads.lock().unwrap() > 0 {
            Err(ThreadPoolError::TaskExecutionFailed(
                "Some threads did not shut down within timeout".to_string()
            ))
        } else {
            Ok(())
        }
    }
    
    // Health check method
    pub fn health_check(&self) -> ThreadPoolHealth {
        let stats = self.get_stats();
        let queue_utilization = stats.current_queue_size as f64 / self.config.queue_size_limit as f64;
        
        let status = if stats.utilization > 0.9 && queue_utilization > 0.8 {
            HealthStatus::Critical
        } else if stats.utilization > 0.75 || queue_utilization > 0.6 {
            HealthStatus::Warning
        } else {
            HealthStatus::Healthy
        };
        
        ThreadPoolHealth {
            status,
            active_threads: stats.active_threads,
            queue_size: stats.current_queue_size,
            utilization: stats.utilization,
            queue_utilization,
            average_execution_time: if stats.tasks_completed > 0 {
                stats.total_execution_time / stats.tasks_completed as u32
            } else {
                Duration::default()
            },
        }
    }
}

#[derive(Debug)]
pub struct ThreadPoolHealth {
    pub status: HealthStatus,
    pub active_threads: usize,
    pub queue_size: usize,
    pub utilization: f64,
    pub queue_utilization: f64,
    pub average_execution_time: Duration,
}

#[derive(Debug)]
pub enum HealthStatus {
    Healthy,
    Warning,
    Critical,
}

// Integration with search operations
pub struct ThreadPooledSearch {
    thread_pool: Arc<ThreadPool>,
    search_engine: SearchEngine,
}

impl ThreadPooledSearch {
    pub fn new(thread_pool_config: ThreadPoolConfig) -> Self {
        Self {
            thread_pool: Arc::new(ThreadPool::new(thread_pool_config)),
            search_engine: SearchEngine::new(),
        }
    }
    
    pub async fn search_parallel(
        &self, 
        queries: Vec<String>,
        priority: TaskPriority,
    ) -> Result<Vec<SearchResults>, ThreadPoolError> {
        let mut handles = Vec::new();
        
        for query in queries {
            let search_engine = self.search_engine.clone();
            let handle = self.thread_pool.submit_task(
                move || search_engine.search(&query),
                priority,
                Some(Duration::from_secs(5)), // Estimated duration
            );
            handles.push(handle);
        }
        
        let mut results = Vec::new();
        for handle in handles {
            results.push(handle.await?);
        }
        
        Ok(results)
    }
}
```

## Validation
- [ ] Thread pool scales dynamically with load
- [ ] Priority queues process tasks in correct order
- [ ] Idle timeout reduces thread count appropriately
- [ ] Resource limits prevent thread exhaustion
- [ ] Health monitoring provides accurate status

## Success Criteria
- Dynamic thread pool adapts to workload
- Priority-based task scheduling works correctly
- Resource limits prevent system overload
- Comprehensive monitoring and health checks
- Integration with search operations optimizes performance

## Next Task
task_011 - Implement connection pooling for external services
