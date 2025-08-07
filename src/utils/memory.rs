use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use sysinfo::{System, Pid};
use tracing::{debug, warn, info};
use lru::LruCache;
use std::num::NonZeroUsize;

/// Memory usage information
#[derive(Debug, Clone)]
pub struct MemoryUsage {
    pub total_memory: u64,
    pub available_memory: u64,
    pub used_memory: u64,
    pub process_memory: u64,
    pub memory_pressure: MemoryPressure,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MemoryPressure {
    Low,    // < 70% memory usage
    Medium, // 70-85% memory usage
    High,   // 85-95% memory usage
    Critical, // > 95% memory usage
}

impl MemoryPressure {
    fn from_usage_percent(usage_percent: f64) -> Self {
        if usage_percent < 70.0 {
            MemoryPressure::Low
        } else if usage_percent < 85.0 {
            MemoryPressure::Medium
        } else if usage_percent < 95.0 {
            MemoryPressure::High
        } else {
            MemoryPressure::Critical
        }
    }
}

/// Memory monitor that tracks system and process memory usage
pub struct MemoryMonitor {
    system: Arc<Mutex<System>>,
    last_update: Arc<Mutex<Instant>>,
    update_interval: Duration,
    process_pid: Pid,
}

impl MemoryMonitor {
    pub fn new() -> Result<Self, crate::error::EmbedError> {
        let mut system = System::new_all();
        system.refresh_all();
        
        let process_pid = sysinfo::get_current_pid()
            .map_err(|_| crate::error::EmbedError::Internal {
                message: "Unable to get current process PID for memory monitoring".to_string(),
                backtrace: None,
            })?;
        
        Ok(Self {
            system: Arc::new(Mutex::new(system)),
            last_update: Arc::new(Mutex::new(Instant::now())),
            update_interval: Duration::from_secs(5),
            process_pid,
        })
    }

    pub fn with_update_interval(mut self, interval: Duration) -> Self {
        self.update_interval = interval;
        self
    }

    /// Get current memory usage information
    pub fn get_memory_usage(&self) -> Result<MemoryUsage, crate::error::EmbedError> {
        self.refresh_if_needed()?;
        
        let system = self.system.lock()
            .map_err(|_| crate::error::EmbedError::Internal {
                message: "Failed to acquire system lock for memory monitoring".to_string(),
                backtrace: None,
            })?;
        
        let total_memory = system.total_memory();
        let available_memory = system.available_memory();
        let used_memory = total_memory - available_memory;
        
        let process_memory = system
            .process(self.process_pid)
            .map(|p| p.memory())
            .ok_or_else(|| crate::error::EmbedError::Internal {
                message: format!("Unable to read memory usage for process PID {}", self.process_pid),
                backtrace: None,
            })?;
        
        let usage_percent = (used_memory as f64 / total_memory as f64) * 100.0;
        let memory_pressure = MemoryPressure::from_usage_percent(usage_percent);
        
        Ok(MemoryUsage {
            total_memory,
            available_memory,
            used_memory,
            process_memory,
            memory_pressure,
        })
    }

    /// Check if system is under memory pressure
    pub fn is_under_pressure(&self) -> Result<bool, crate::error::EmbedError> {
        let usage = self.get_memory_usage()?;
        Ok(matches!(usage.memory_pressure, 
                    MemoryPressure::High | MemoryPressure::Critical))
    }

    /// Get memory pressure level
    pub fn get_memory_pressure(&self) -> Result<MemoryPressure, crate::error::EmbedError> {
        Ok(self.get_memory_usage()?.memory_pressure)
    }

    /// Refresh system information if enough time has passed
    fn refresh_if_needed(&self) -> Result<(), crate::error::EmbedError> {
        let now = Instant::now();
        let mut last_update = self.last_update.lock()
            .map_err(|_| crate::error::EmbedError::Internal {
                message: "Failed to acquire last_update lock for memory monitoring".to_string(),
                backtrace: None,
            })?;
        
        if now.duration_since(*last_update) >= self.update_interval {
            let mut system = self.system.lock()
                .map_err(|_| crate::error::EmbedError::Internal {
                    message: "Failed to acquire system lock for memory monitoring".to_string(),
                    backtrace: None,
                })?;
            system.refresh_memory();
            system.refresh_processes();
            *last_update = now;
        }
        Ok(())
    }
}

// Default implementation removed - MemoryMonitor creation can fail
// and must be handled explicitly

/// Adaptive cache controller that adjusts cache sizes based on memory pressure
pub struct CacheController {
    memory_monitor: MemoryMonitor,
    base_cache_size: usize,
    min_cache_size: usize,
    max_cache_size: usize,
}

impl CacheController {
    pub fn new(base_cache_size: usize) -> Result<Self, crate::error::EmbedError> {
        Ok(Self {
            memory_monitor: MemoryMonitor::new()?,
            base_cache_size,
            min_cache_size: base_cache_size / 4,
            max_cache_size: base_cache_size * 2,
        })
    }

    pub fn with_size_bounds(mut self, min_size: usize, max_size: usize) -> Self {
        self.min_cache_size = min_size;
        self.max_cache_size = max_size;
        self
    }

    /// Calculate optimal cache size based on current memory pressure
    pub fn calculate_optimal_cache_size(&self) -> Result<usize, crate::error::EmbedError> {
        let memory_usage = self.memory_monitor.get_memory_usage()?;
        
        let size = match memory_usage.memory_pressure {
            MemoryPressure::Low => {
                // Low pressure: can use larger cache
                debug!("Memory pressure: Low - using larger cache size");
                self.max_cache_size
            }
            MemoryPressure::Medium => {
                // Medium pressure: use base cache size
                debug!("Memory pressure: Medium - using base cache size");
                self.base_cache_size
            }
            MemoryPressure::High => {
                // High pressure: reduce cache size
                warn!("Memory pressure: High - reducing cache size");
                (self.base_cache_size * 3) / 4
            }
            MemoryPressure::Critical => {
                // Critical pressure: use minimum cache size
                warn!("Memory pressure: Critical - using minimum cache size");
                self.min_cache_size
            }
        };
        Ok(size)
    }

    /// Create an adaptive LRU cache that adjusts its size based on memory pressure
    pub fn create_adaptive_cache<K, V>(&self) -> Result<AdaptiveCache<K, V>, crate::error::EmbedError>
    where
        K: std::hash::Hash + Eq,
    {
        let initial_size = self.calculate_optimal_cache_size()?;
        // Create a new controller with the same configuration
        let new_controller = Self::new(self.base_cache_size)?
            .with_size_bounds(self.min_cache_size, self.max_cache_size);
        Ok(AdaptiveCache::new(initial_size, new_controller)?)
    }

    /// Check if caches should be trimmed due to memory pressure
    pub fn should_trim_caches(&self) -> Result<bool, crate::error::EmbedError> {
        let pressure = self.memory_monitor.get_memory_pressure()?;
        Ok(matches!(pressure, MemoryPressure::High | MemoryPressure::Critical))
    }

    /// Log current memory status
    pub fn log_memory_status(&self) -> Result<(), crate::error::EmbedError> {
        let usage = self.memory_monitor.get_memory_usage()?;
        let usage_percent = (usage.used_memory as f64 / usage.total_memory as f64) * 100.0;
        
        info!(
            "Memory status - Total: {}MB, Used: {}MB ({:.1}%), Available: {}MB, Process: {}MB, Pressure: {:?}",
            usage.total_memory / 1024 / 1024,
            usage.used_memory / 1024 / 1024,
            usage_percent,
            usage.available_memory / 1024 / 1024,
            usage.process_memory / 1024 / 1024,
            usage.memory_pressure
        );
        Ok(())
    }
}

// Clone implementation removed - CacheController cannot be safely cloned
// due to memory monitor initialization that can fail

/// Adaptive LRU cache that automatically adjusts its size based on memory pressure
pub struct AdaptiveCache<K, V>
where
    K: std::hash::Hash + Eq,
{
    cache: LruCache<K, V>,
    controller: CacheController,
    last_resize: Instant,
    resize_interval: Duration,
}

impl<K, V> AdaptiveCache<K, V>
where
    K: std::hash::Hash + Eq,
{
    pub fn new(initial_capacity: usize, controller: CacheController) -> Result<Self, crate::error::EmbedError> {
        let capacity = NonZeroUsize::new(initial_capacity.max(1))
            .ok_or_else(|| crate::error::EmbedError::Validation {
                field: "initial_capacity".to_string(),
                reason: "Cache capacity must be greater than 0".to_string(),
                value: Some(initial_capacity.to_string()),
            })?;
        
        Ok(Self {
            cache: LruCache::new(capacity),
            controller,
            last_resize: Instant::now(),
            resize_interval: Duration::from_secs(30),
        })
    }

    /// Get a value from the cache
    pub fn get(&mut self, key: &K) -> Result<Option<&V>, crate::error::EmbedError> {
        self.maybe_resize()?;
        Ok(self.cache.get(key))
    }

    /// Put a value into the cache
    pub fn put(&mut self, key: K, value: V) -> Result<Option<V>, crate::error::EmbedError> {
        self.maybe_resize()?;
        Ok(self.cache.put(key, value))
    }

    /// Get current cache size
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Get cache capacity
    pub fn cap(&self) -> usize {
        self.cache.cap().get()
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    /// Clear the cache
    pub fn clear(&mut self) {
        self.cache.clear();
    }

    /// Manually trigger cache resize based on current memory pressure
    pub fn resize(&mut self) -> Result<(), crate::error::EmbedError> {
        let optimal_size = self.controller.calculate_optimal_cache_size()?;
        let current_size = self.cache.cap().get();
        
        if optimal_size != current_size {
            debug!(
                "Resizing cache from {} to {} entries due to memory pressure",
                current_size, optimal_size
            );
            
            let new_capacity = NonZeroUsize::new(optimal_size.max(1))
                .ok_or_else(|| crate::error::EmbedError::Internal {
                    message: format!("Cache resize capacity {} is invalid", optimal_size),
                    backtrace: None,
                })?;
            self.cache.resize(new_capacity);
        }
        
        self.last_resize = Instant::now();
        Ok(())
    }

    /// Maybe resize the cache if enough time has passed
    fn maybe_resize(&mut self) -> Result<(), crate::error::EmbedError> {
        if self.last_resize.elapsed() >= self.resize_interval {
            self.resize()?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_monitor() {
        let monitor = MemoryMonitor::new().unwrap();
        let usage = monitor.get_memory_usage().unwrap();
        
        assert!(usage.total_memory > 0);
        assert!(usage.available_memory <= usage.total_memory);
        assert!(usage.used_memory <= usage.total_memory);
    }

    #[test]
    fn test_memory_pressure_levels() {
        assert_eq!(MemoryPressure::from_usage_percent(60.0), MemoryPressure::Low);
        assert_eq!(MemoryPressure::from_usage_percent(75.0), MemoryPressure::Medium);
        assert_eq!(MemoryPressure::from_usage_percent(90.0), MemoryPressure::High);
        assert_eq!(MemoryPressure::from_usage_percent(98.0), MemoryPressure::Critical);
    }

    #[test]
    fn test_cache_controller() {
        let controller = CacheController::new(1000).unwrap();
        let cache_size = controller.calculate_optimal_cache_size().unwrap();
        
        // Cache size should be within reasonable bounds
        assert!(cache_size > 0);
        assert!(cache_size <= controller.max_cache_size);
        assert!(cache_size >= controller.min_cache_size);
    }

    #[test]
    fn test_adaptive_cache() {
        let controller = CacheController::new(100).unwrap();
        let mut cache = controller.create_adaptive_cache::<String, i32>().unwrap();
        
        cache.put("key1".to_string(), 1).unwrap();
        cache.put("key2".to_string(), 2).unwrap();
        
        assert_eq!(cache.get(&"key1".to_string()).unwrap(), Some(&1));
        assert_eq!(cache.get(&"key2".to_string()).unwrap(), Some(&2));
        assert_eq!(cache.len(), 2);
    }
}