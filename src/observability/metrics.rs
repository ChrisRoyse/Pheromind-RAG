use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::{debug, info};

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

    pub fn mean(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            self.sum / self.count as f64
        }
    }

    pub fn count(&self) -> u64 {
        self.count
    }

    pub fn sum(&self) -> f64 {
        self.sum
    }

    pub fn percentile(&self, percentile: f64) -> f64 {
        if self.count == 0 {
            return 0.0;
        }

        let target_count = (self.count as f64 * percentile / 100.0) as u64;
        let mut cumulative_count = 0;

        for (i, &count) in self.counts.iter().enumerate() {
            cumulative_count += count;
            if cumulative_count >= target_count {
                if i == 0 {
                    return 0.0;
                } else if i <= self.buckets.len() {
                    return self.buckets[i - 1];
                } else {
                    return self.buckets[self.buckets.len() - 1];
                }
            }
        }

        self.buckets[self.buckets.len() - 1]
    }
}

/// Metrics for search operations
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

    pub fn success_rate(&self) -> f64 {
        let total = self.search_count + self.failed_searches;
        if total == 0 {
            0.0
        } else {
            self.search_count as f64 / total as f64
        }
    }
}

impl Default for SearchMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Metrics for embedding operations
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

    pub fn record_embedding(&mut self, duration: Duration, from_cache: bool) {
        self.embedding_duration.observe(duration.as_secs_f64());
        self.embedding_count += 1;

        if from_cache {
            self.cache_hits += 1;
        } else {
            self.cache_misses += 1;
        }
    }

    pub fn cache_hit_rate(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total == 0 {
            0.0
        } else {
            self.cache_hits as f64 / total as f64
        }
    }

    pub fn set_embedding_dimension(&mut self, dimension: usize) {
        self.embedding_dimension = Some(dimension);
    }
}

impl Default for EmbeddingMetrics {
    fn default() -> Self {
        Self::new()
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

    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
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
    pub fn record_cache_hit(&self, cache_name: &str) {
        if let Ok(mut caches) = self.cache_metrics.lock() {
            let entry = caches.entry(cache_name.to_string()).or_insert_with(|| CacheMetrics::new(1000));
            entry.record_hit();
        }
    }

    pub fn record_cache_miss(&self, cache_name: &str) {
        if let Ok(mut caches) = self.cache_metrics.lock() {
            let entry = caches.entry(cache_name.to_string()).or_insert_with(|| CacheMetrics::new(1000));
            entry.record_miss();
        }
    }

    pub fn update_cache_size(&self, cache_name: &str, size: u64, max_size: u64) {
        if let Ok(mut caches) = self.cache_metrics.lock() {
            let entry = caches.entry(cache_name.to_string()).or_insert_with(|| CacheMetrics::new(max_size));
            entry.update_size(size);
            entry.max_size = max_size;
        }
    }

    /// Get search metrics snapshot
    pub fn get_search_metrics(&self) -> SearchMetrics {
        self.search_metrics.lock().unwrap().clone()
    }

    /// Get embedding metrics snapshot
    pub fn get_embedding_metrics(&self) -> EmbeddingMetrics {
        self.embedding_metrics.lock().unwrap().clone()
    }

    /// Get cache metrics snapshot
    pub fn get_cache_metrics(&self, cache_name: &str) -> Option<CacheMetrics> {
        self.cache_metrics.lock().unwrap().get(cache_name).cloned()
    }

    /// Print comprehensive metrics report
    pub fn print_report(&self) {
        let uptime = self.start_time.elapsed();
        let search_metrics = self.get_search_metrics();
        let embedding_metrics = self.get_embedding_metrics();
        
        info!("=== Performance Metrics Report ===");
        info!("Uptime: {:.2}s", uptime.as_secs_f64());
        
        // Search metrics
        info!("Search Operations:");
        info!("  Total searches: {}", search_metrics.search_count);
        info!("  Failed searches: {}", search_metrics.failed_searches);
        info!("  Success rate: {:.2}%", search_metrics.success_rate() * 100.0);
        info!("  Mean latency: {:.3}s", search_metrics.search_duration.mean());
        info!("  95th percentile latency: {:.3}s", search_metrics.search_duration.percentile(95.0));
        info!("  Mean results per search: {:.1}", search_metrics.results_count.mean());
        
        // Embedding metrics
        info!("Embedding Operations:");
        info!("  Total embeddings: {}", embedding_metrics.embedding_count);
        info!("  Cache hit rate: {:.2}%", embedding_metrics.cache_hit_rate() * 100.0);
        info!("  Mean embedding time: {:.3}s", embedding_metrics.embedding_duration.mean());
        info!("  95th percentile embedding time: {:.3}s", embedding_metrics.embedding_duration.percentile(95.0));
        if let Some(dim) = embedding_metrics.embedding_dimension {
            info!("  Embedding dimension: {}", dim);
        }
        
        // Cache metrics
        if let Ok(caches) = self.cache_metrics.lock() {
            if !caches.is_empty() {
                info!("Cache Statistics:");
                for (name, metrics) in caches.iter() {
                    info!("  {}: hit_rate={:.2}%, utilization={:.2}%", 
                          name, metrics.hit_rate() * 100.0, metrics.utilization() * 100.0);
                }
            }
        }
        
        info!("================================");
    }

    /// Get performance summary for logging
    pub fn get_performance_summary(&self) -> String {
        let search_metrics = self.get_search_metrics();
        let embedding_metrics = self.get_embedding_metrics();
        
        format!(
            "searches={} ({}% success, {:.3}s avg), embeddings={} ({:.1}% cache hits, {:.3}s avg)",
            search_metrics.search_count,
            (search_metrics.success_rate() * 100.0) as u32,
            search_metrics.search_duration.mean(),
            embedding_metrics.embedding_count,
            embedding_metrics.cache_hit_rate() * 100.0,
            embedding_metrics.embedding_duration.mean()
        )
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Global metrics instance
static METRICS: once_cell::sync::Lazy<MetricsCollector> = once_cell::sync::Lazy::new(MetricsCollector::new);

/// Get the global metrics collector
pub fn metrics() -> &'static MetricsCollector {
    &METRICS
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
        assert_eq!(hist.mean(), 2.63);
    }

    #[test]
    fn test_search_metrics() {
        let mut metrics = SearchMetrics::new();
        
        metrics.record_search(Duration::from_millis(100), 5, true);
        metrics.record_search(Duration::from_millis(200), 3, true);
        metrics.record_search(Duration::from_millis(50), 0, false);
        
        assert_eq!(metrics.search_count, 2);
        assert_eq!(metrics.failed_searches, 1);
        assert_eq!(metrics.success_rate(), 2.0 / 3.0);
    }

    #[test]
    fn test_embedding_metrics() {
        let mut metrics = EmbeddingMetrics::new();
        
        metrics.record_embedding(Duration::from_millis(50), true);  // cache hit
        metrics.record_embedding(Duration::from_millis(100), false); // cache miss
        metrics.record_embedding(Duration::from_millis(75), true);   // cache hit
        
        assert_eq!(metrics.embedding_count, 3);
        assert_eq!(metrics.cache_hits, 2);
        assert_eq!(metrics.cache_misses, 1);
        assert_eq!(metrics.cache_hit_rate(), 2.0 / 3.0);
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
        assert_eq!(metrics.hit_rate(), 2.0 / 3.0);
        assert_eq!(metrics.utilization(), 0.5);
    }

    #[test]
    fn test_metrics_collector() {
        let collector = MetricsCollector::new();
        
        collector.record_search(Duration::from_millis(100), 5, true);
        collector.record_embedding(Duration::from_millis(50), false);
        collector.record_cache_hit("test_cache");
        
        let search_metrics = collector.get_search_metrics();
        let embedding_metrics = collector.get_embedding_metrics();
        
        assert_eq!(search_metrics.search_count, 1);
        assert_eq!(embedding_metrics.embedding_count, 1);
        
        let cache_metrics = collector.get_cache_metrics("test_cache");
        assert!(cache_metrics.is_some());
        assert_eq!(cache_metrics.unwrap().hits, 1);
    }
}