//! Observability Metrics System
//!
//! **CRITICAL**: This module provides NO fallback or default behavior.
//! All metrics must be explicitly configured and initialized.
//!
//! ## Usage Requirements
//!
//! 1. **Global Metrics**: Must call `init_metrics()` before any access via `metrics()`
//! 2. **Local Metrics**: Must explicitly construct via `SearchMetrics::new()`, etc.
//! 3. **No Defaults**: All `Default` implementations have been removed
//!
//! ## Example
//!
//! ```rust
//! use your_crate::observability::metrics::{init_metrics, metrics, SearchMetrics};
//!
//! // Initialize global metrics (required)
//! init_metrics().expect("Failed to initialize metrics");
//!
//! // Now can access global metrics
//! let collector = metrics();
//!
//! // Or use local metrics directly
//! let search_metrics = SearchMetrics::new();
//! ```

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};
use tracing::{debug, info};
use anyhow::Result;

/// Mathematical validation helpers for safe division operations
mod math_helpers {
    /// Safely divide two numbers, returning None if denominator is zero
    pub fn safe_division(numerator: f64, denominator: f64) -> Option<f64> {
        if denominator == 0.0 {
            None // Mathematically undefined
        } else {
            Some(numerator / denominator)
        }
    }
    
    /// Safely calculate percentage (numerator/denominator * 100), returning None if denominator is zero
    pub fn safe_percentage(numerator: f64, denominator: f64) -> Option<f64> {
        safe_division(numerator, denominator).map(|ratio| ratio * 100.0)
    }
}

/// Simple histogram implementation for tracking durations
#[derive(Debug, Clone)]
pub struct Histogram {
    buckets: Vec<f64>,
    counts: Vec<u64>,
    sum: f64,
    count: u64,
}

impl Histogram {
    pub fn new(buckets: Vec<f64>) -> Self {
        let counts = vec![0; buckets.len() + 1];
        Self {
            buckets,
            counts,
            sum: 0.0,
            count: 0,
        }
    }

    pub fn default_latency_buckets() -> Vec<f64> {
        vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]
    }

    pub fn observe(&mut self, value: f64) {
        self.sum += value;
        self.count += 1;

        // Find the appropriate bucket
        let bucket_index = match self.buckets.iter().position(|&bucket| value <= bucket) {
            Some(index) => index,
            None => {
                // Value exceeds all defined buckets - use overflow bucket at index buckets.len()
                self.buckets.len()
            }
        };
        
        self.counts[bucket_index] += 1;
    }

    /// Calculate the mean of observed values
    /// Returns None if no values have been observed (undefined mathematical state)
    pub fn mean(&self) -> Option<f64> {
        if self.count == 0 {
            None // Undefined: cannot calculate mean of empty dataset
        } else {
            Some(self.sum / self.count as f64)
        }
    }

    pub fn count(&self) -> u64 {
        self.count
    }

    pub fn sum(&self) -> f64 {
        self.sum
    }

    /// Calculate the specified percentile of observed values
    /// Returns None if no values have been observed (undefined mathematical state)
    pub fn percentile(&self, percentile: f64) -> Option<f64> {
        if self.count == 0 {
            return None; // Undefined: cannot calculate percentile of empty dataset
        }

        let target_count = (self.count as f64 * percentile / 100.0) as u64;
        let mut cumulative_count = 0;

        for (i, &count) in self.counts.iter().enumerate() {
            cumulative_count += count;
            if cumulative_count >= target_count {
                if i == 0 {
                    return Some(0.0);
                } else if i <= self.buckets.len() {
                    return Some(self.buckets[i - 1]);
                } else {
                    return Some(self.buckets[self.buckets.len() - 1]);
                }
            }
        }

        Some(self.buckets[self.buckets.len() - 1])
    }
}

/// Metrics for search operations
/// 
/// **IMPORTANT**: No default implementation provided. Must be explicitly constructed via `new()`.
#[derive(Debug, Clone)]
pub struct SearchMetrics {
    pub search_duration: Histogram,
    pub search_count: u64,
    pub results_count: Histogram,
    pub failed_searches: u64,
}

impl SearchMetrics {
    pub fn new() -> Self {
        Self {
            search_duration: Histogram::new(Histogram::default_latency_buckets()),
            search_count: 0,
            results_count: Histogram::new(vec![0.0, 1.0, 5.0, 10.0, 25.0, 50.0, 100.0, 250.0, 500.0]),
            failed_searches: 0,
        }
    }

    pub fn record_search(&mut self, duration: Duration, result_count: usize, success: bool) {
        if success {
            self.search_duration.observe(duration.as_secs_f64());
            self.results_count.observe(result_count as f64);
            self.search_count += 1;
        } else {
            self.failed_searches += 1;
        }
    }

    /// Calculate success rate as a fraction (0.0 to 1.0)
    /// Returns None if no search operations have occurred (undefined mathematical state)
    pub fn success_rate(&self) -> Option<f64> {
        let total = self.search_count + self.failed_searches;
        math_helpers::safe_division(self.search_count as f64, total as f64)
    }
}


/// Metrics for embedding operations
/// 
/// **IMPORTANT**: No default implementation provided. Must be explicitly constructed via `new()`.
#[derive(Debug, Clone)]
pub struct EmbeddingMetrics {
    pub embedding_duration: Histogram,
    pub embedding_count: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub embedding_dimension: Option<usize>,
}

impl EmbeddingMetrics {
    pub fn new() -> Self {
        Self {
            embedding_duration: Histogram::new(Histogram::default_latency_buckets()),
            embedding_count: 0,
            cache_hits: 0,
            cache_misses: 0,
            embedding_dimension: None,
        }
    }

    #[cfg(feature = "ml")]
    pub fn record_embedding(&mut self, duration: Duration, from_cache: bool) {
        self.embedding_duration.observe(duration.as_secs_f64());
        self.embedding_count += 1;

        if from_cache {
            self.cache_hits += 1;
        } else {
            self.cache_misses += 1;
        }
    }

    /// Calculate cache hit rate as a fraction (0.0 to 1.0)
    /// Returns None if no cache operations have occurred (undefined mathematical state)
    pub fn cache_hit_rate(&self) -> Option<f64> {
        let total = self.cache_hits + self.cache_misses;
        math_helpers::safe_division(self.cache_hits as f64, total as f64)
    }

    #[cfg(feature = "ml")]
    pub fn set_embedding_dimension(&mut self, dimension: usize) {
        self.embedding_dimension = Some(dimension);
    }
}


/// Metrics for cache operations
#[derive(Debug, Clone)]
pub struct CacheMetrics {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub size: u64,
    pub max_size: u64,
}

impl CacheMetrics {
    pub fn new(max_size: u64) -> Self {
        Self {
            hits: 0,
            misses: 0,
            evictions: 0,
            size: 0,
            max_size,
        }
    }

    pub fn record_hit(&mut self) {
        self.hits += 1;
    }

    pub fn record_miss(&mut self) {
        self.misses += 1;
    }

    pub fn record_eviction(&mut self) {
        self.evictions += 1;
    }

    pub fn update_size(&mut self, size: u64) {
        self.size = size;
    }

    /// Calculate hit rate as a fraction (0.0 to 1.0)
    /// Returns None if no cache operations have occurred (undefined mathematical state)
    pub fn hit_rate(&self) -> Option<f64> {
        let total = self.hits + self.misses;
        math_helpers::safe_division(self.hits as f64, total as f64)
    }

    pub fn utilization(&self) -> f64 {
        if self.max_size == 0 {
            0.0
        } else {
            self.size as f64 / self.max_size as f64
        }
    }
}

/// Central metrics collector
/// 
/// **IMPORTANT**: No default implementation provided. Must be explicitly constructed via `new()`.
/// For global usage, must call `init_metrics()` before accessing via `metrics()` function.
#[derive(Debug)]
pub struct MetricsCollector {
    search_metrics: Arc<Mutex<SearchMetrics>>,
    embedding_metrics: Arc<Mutex<EmbeddingMetrics>>,
    cache_metrics: Arc<Mutex<HashMap<String, CacheMetrics>>>,
    start_time: Instant,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            search_metrics: Arc::new(Mutex::new(SearchMetrics::new())),
            embedding_metrics: Arc::new(Mutex::new(EmbeddingMetrics::new())),
            cache_metrics: Arc::new(Mutex::new(HashMap::new())),
            start_time: Instant::now(),
        }
    }

    /// Record a search operation
    pub fn record_search(&self, duration: Duration, result_count: usize, success: bool) {
        if let Ok(mut metrics) = self.search_metrics.lock() {
            metrics.record_search(duration, result_count, success);
        }
        
        debug!(
            "Search completed: duration={:.3}s, results={}, success={}",
            duration.as_secs_f64(),
            result_count,
            success
        );
    }

    /// Record an embedding operation
    #[cfg(feature = "ml")]
    pub fn record_embedding(&self, duration: Duration, from_cache: bool) {
        if let Ok(mut metrics) = self.embedding_metrics.lock() {
            metrics.record_embedding(duration, from_cache);
        }
        
        debug!(
            "Embedding computed: duration={:.3}s, from_cache={}",
            duration.as_secs_f64(),
            from_cache
        );
    }

    /// Record cache operation
    pub fn record_cache_hit(&self, cache_name: &str) -> Result<()> {
        let mut caches = self.cache_metrics.lock()
            .map_err(|e| anyhow::anyhow!("Failed to acquire cache metrics lock: {}", e))?;
        
        let entry = caches.get_mut(cache_name)
            .ok_or_else(|| anyhow::anyhow!("Cache '{}' not initialized. Call update_cache_size first.", cache_name))?;
        
        entry.record_hit();
        Ok(())
    }

    pub fn record_cache_miss(&self, cache_name: &str) -> Result<()> {
        let mut caches = self.cache_metrics.lock()
            .map_err(|e| anyhow::anyhow!("Failed to acquire cache metrics lock: {}", e))?;
        
        let entry = caches.get_mut(cache_name)
            .ok_or_else(|| anyhow::anyhow!("Cache '{}' not initialized. Call update_cache_size first.", cache_name))?;
        
        entry.record_miss();
        Ok(())
    }

    pub fn update_cache_size(&self, cache_name: &str, size: u64, max_size: u64) -> Result<()> {
        let mut caches = self.cache_metrics.lock()
            .map_err(|e| anyhow::anyhow!("Failed to acquire cache metrics lock: {}", e))?;
        
        // Initialize cache metrics if not present
        if !caches.contains_key(cache_name) {
            caches.insert(cache_name.to_string(), CacheMetrics::new(max_size));
        }
        
        let entry = caches.get_mut(cache_name)
            .ok_or_else(|| anyhow::anyhow!("Failed to access cache metrics for '{}'", cache_name))?;
        
        entry.update_size(size);
        entry.max_size = max_size;
        Ok(())
    }

    /// Get search metrics snapshot
    /// 
    /// # Errors
    /// 
    /// Returns error if the search metrics mutex is poisoned due to a panic in another thread.
    /// This indicates a serious system corruption that must be handled explicitly.
    pub fn get_search_metrics(&self) -> Result<SearchMetrics> {
        self.search_metrics
            .lock()
            .map_err(|e| anyhow::anyhow!("Search metrics mutex poisoned: {}", e))
            .map(|guard| guard.clone())
    }

    /// Get embedding metrics snapshot
    /// 
    /// # Errors
    /// 
    /// Returns error if the embedding metrics mutex is poisoned due to a panic in another thread.
    /// This indicates a serious system corruption that must be handled explicitly.
    #[cfg(feature = "ml")]
    pub fn get_embedding_metrics(&self) -> Result<EmbeddingMetrics> {
        self.embedding_metrics
            .lock()
            .map_err(|e| anyhow::anyhow!("Embedding metrics mutex poisoned: {}", e))
            .map(|guard| guard.clone())
    }

    /// Get cache metrics snapshot
    /// 
    /// # Errors
    /// 
    /// Returns error if the cache metrics mutex is poisoned due to a panic in another thread.
    /// This indicates a serious system corruption that must be handled explicitly.
    pub fn get_cache_metrics(&self, cache_name: &str) -> Result<Option<CacheMetrics>> {
        self.cache_metrics
            .lock()
            .map_err(|e| anyhow::anyhow!("Cache metrics mutex poisoned: {}", e))
            .map(|guard| guard.get(cache_name).cloned())
    }

    /// Print comprehensive metrics report
    pub fn print_report(&self) {
        let uptime = self.start_time.elapsed();
        let search_metrics = match self.get_search_metrics() {
            Ok(metrics) => metrics,
            Err(e) => {
                tracing::error!("Failed to get search metrics: {}", e);
                return;
            }
        };
        #[cfg(feature = "ml")]
        let embedding_metrics = match self.get_embedding_metrics() {
            Ok(metrics) => metrics,
            Err(e) => {
                tracing::error!("Failed to get embedding metrics: {}", e);
                return;
            }
        };
        
        info!("=== Performance Metrics Report ===");
        info!("Uptime: {:.2}s", uptime.as_secs_f64());
        
        // Search metrics
        info!("Search Operations:");
        info!("  Total searches: {}", search_metrics.search_count);
        info!("  Failed searches: {}", search_metrics.failed_searches);
        match search_metrics.success_rate() {
            Some(rate) => info!("  Success rate: {:.2}%", rate * 100.0),
            None => info!("  Success rate: N/A (no searches performed)"),
        }
        match search_metrics.search_duration.mean() {
            Some(mean) => info!("  Mean latency: {:.3}s", mean),
            None => info!("  Mean latency: N/A (no data)"),
        }
        match search_metrics.search_duration.percentile(95.0) {
            Some(p95) => info!("  95th percentile latency: {:.3}s", p95),
            None => info!("  95th percentile latency: N/A (insufficient data)"),
        }
        match search_metrics.results_count.mean() {
            Some(mean) => info!("  Mean results per search: {:.1}", mean),
            None => info!("  Mean results per search: N/A (no data)"),
        }
        
        // Embedding metrics
        #[cfg(feature = "ml")]
        {
            info!("Embedding Operations:");
            info!("  Total embeddings: {}", embedding_metrics.embedding_count);
            match embedding_metrics.cache_hit_rate() {
                Some(rate) => info!("  Cache hit rate: {:.2}%", rate * 100.0),
                None => info!("  Cache hit rate: N/A (no embeddings)"),
            }
            match embedding_metrics.embedding_duration.mean() {
                Some(mean) => info!("  Mean embedding time: {:.3}s", mean),
                None => info!("  Mean embedding time: N/A (no data)"),
            }
            match embedding_metrics.embedding_duration.percentile(95.0) {
                Some(p95) => info!("  95th percentile embedding time: {:.3}s", p95),
                None => info!("  95th percentile embedding time: N/A (insufficient data)"),
            }
            if let Some(dim) = embedding_metrics.embedding_dimension {
                info!("  Embedding dimension: {}", dim);
            }
        }
        
        // Cache metrics
        if let Ok(caches) = self.cache_metrics.lock() {
            if !caches.is_empty() {
                info!("Cache Statistics:");
                for (name, metrics) in caches.iter() {
                    match metrics.hit_rate() {
                        Some(rate) => info!("  {}: hit_rate={:.2}%, utilization={:.2}%", 
                                          name, rate * 100.0, metrics.utilization() * 100.0),
                        None => info!("  {}: hit_rate=N/A, utilization={:.2}%", 
                                      name, metrics.utilization() * 100.0),
                    }
                }
            }
        }
        
        info!("================================");
    }

    /// Get performance summary for logging
    pub fn get_performance_summary(&self) -> Result<String> {
        let search_metrics = self.get_search_metrics()?;
        #[cfg(feature = "ml")]
        let embedding_metrics = self.get_embedding_metrics()?;
        
        #[cfg(feature = "ml")]
        {
            let success_str = search_metrics.success_rate()
                .map(|r| format!("{}%", (r * 100.0) as u32))
                .ok_or_else(|| anyhow::anyhow!("No search success rate data available"))?;
            let search_avg_str = search_metrics.search_duration.mean()
                .map(|m| format!("{:.3}s", m))
                .ok_or_else(|| anyhow::anyhow!("No search duration data available"))?;
            let cache_hit_str = embedding_metrics.cache_hit_rate()
                .map(|r| format!("{:.1}%", r * 100.0))
                .ok_or_else(|| anyhow::anyhow!("No cache hit rate data available"))?;
            let embed_avg_str = embedding_metrics.embedding_duration.mean()
                .map(|m| format!("{:.3}s", m))
                .ok_or_else(|| anyhow::anyhow!("No embedding duration data available"))?;
            
            Ok(format!(
                "searches={} ({} success, {} avg), embeddings={} ({} cache hits, {} avg)",
                search_metrics.search_count,
                success_str,
                search_avg_str,
                embedding_metrics.embedding_count,
                cache_hit_str,
                embed_avg_str
            ))
        }
        #[cfg(not(feature = "ml"))]
        {
            let success_str = search_metrics.success_rate()
                .map(|r| format!("{}%", (r * 100.0) as u32))
                .ok_or_else(|| anyhow::anyhow!("No search success rate data available"))?;
            let search_avg_str = search_metrics.search_duration.mean()
                .map(|m| format!("{:.3}s", m))
                .ok_or_else(|| anyhow::anyhow!("No search duration data available"))?;
            
            Ok(format!(
                "searches={} ({} success, {} avg)",
                search_metrics.search_count,
                success_str,
                search_avg_str
            ))
        }
    }
}


/// Global metrics instance - requires explicit initialization
static METRICS: OnceLock<MetricsCollector> = OnceLock::new();

/// Initialize the global metrics collector
/// 
/// This must be called exactly once before any metrics operations.
/// Subsequent calls will be ignored.
pub fn init_metrics() -> Result<(), &'static str> {
    METRICS
        .set(MetricsCollector::new())
        .map_err(|_| "Metrics already initialized")
}

/// Get the global metrics collector
/// 
/// # Panics
/// 
/// This function will panic if metrics have not been explicitly initialized via `init_metrics()`
pub fn metrics() -> &'static MetricsCollector {
    METRICS.get().expect(
        "METRICS not initialized! Call init_metrics() before accessing metrics."
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_histogram() {
        let mut hist = Histogram::new(vec![0.1, 0.5, 1.0, 5.0]);
        
        hist.observe(0.05);
        hist.observe(0.3);
        hist.observe(0.8);
        hist.observe(2.0);
        hist.observe(10.0);
        
        assert_eq!(hist.count(), 5);
        assert_eq!(hist.mean().unwrap(), 2.63);
    }

    #[test]
    fn test_search_metrics() {
        let mut metrics = SearchMetrics::new();
        
        metrics.record_search(Duration::from_millis(100), 5, true);
        metrics.record_search(Duration::from_millis(200), 3, true);
        metrics.record_search(Duration::from_millis(50), 0, false);
        
        assert_eq!(metrics.search_count, 2);
        assert_eq!(metrics.failed_searches, 1);
        assert_eq!(metrics.success_rate().unwrap(), 2.0 / 3.0);
    }

    #[test]
    #[cfg(feature = "ml")]
    fn test_embedding_metrics() {
        let mut metrics = EmbeddingMetrics::new();
        
        metrics.record_embedding(Duration::from_millis(50), true);  // cache hit
        metrics.record_embedding(Duration::from_millis(100), false); // cache miss
        metrics.record_embedding(Duration::from_millis(75), true);   // cache hit
        
        assert_eq!(metrics.embedding_count, 3);
        assert_eq!(metrics.cache_hits, 2);
        assert_eq!(metrics.cache_misses, 1);
        assert_eq!(metrics.cache_hit_rate().unwrap(), 2.0 / 3.0);
    }

    #[test]
    fn test_cache_metrics() {
        let mut metrics = CacheMetrics::new(100);
        
        metrics.record_hit();
        metrics.record_hit();
        metrics.record_miss();
        metrics.update_size(50);
        
        assert_eq!(metrics.hits, 2);
        assert_eq!(metrics.misses, 1);
        assert_eq!(metrics.hit_rate().unwrap(), 2.0 / 3.0);
        assert_eq!(metrics.utilization(), 0.5);
    }

    #[test]
    fn test_metrics_collector() {
        let collector = MetricsCollector::new();
        
        // Initialize cache metrics first
        collector.update_cache_size("test_cache", 10, 100).expect("Should be able to initialize cache");
        
        collector.record_search(Duration::from_millis(100), 5, true);
        #[cfg(feature = "ml")]
        collector.record_embedding(Duration::from_millis(50), false);
        collector.record_cache_hit("test_cache").expect("Should be able to record cache hit");
        
        let search_metrics = collector.get_search_metrics()
            .expect("Should be able to get search metrics");
        #[cfg(feature = "ml")]
        let embedding_metrics = collector.get_embedding_metrics()
            .expect("Should be able to get embedding metrics");
        
        assert_eq!(search_metrics.search_count, 1);
        #[cfg(feature = "ml")]
        assert_eq!(embedding_metrics.embedding_count, 1);
        
        let cache_metrics = collector.get_cache_metrics("test_cache")
            .expect("Should be able to get cache metrics");
        assert!(cache_metrics.is_some());
        assert_eq!(cache_metrics.unwrap().hits, 1);
    }

    #[test]
    fn test_explicit_metrics_usage() {
        // Test that individual metrics components work without defaults
        let search_metrics = SearchMetrics::new();
        #[cfg(feature = "ml")]
        let embedding_metrics = EmbeddingMetrics::new();
        let cache_metrics = CacheMetrics::new(100);
        
        // Verify no defaults are used - all fields are explicitly set
        assert_eq!(search_metrics.search_count, 0);
        #[cfg(feature = "ml")]
        assert_eq!(embedding_metrics.embedding_count, 0);
        assert_eq!(cache_metrics.max_size, 100);
        
        // Test that MetricsCollector constructor works explicitly
        let collector = MetricsCollector::new();
        collector.record_search(Duration::from_millis(100), 5, true);
        
        let retrieved = collector.get_search_metrics()
            .expect("Should be able to get search metrics");
        assert_eq!(retrieved.search_count, 1);
    }
}