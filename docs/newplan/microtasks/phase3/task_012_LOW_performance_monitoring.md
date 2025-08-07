# Task 3.012: Add Performance Monitoring and Metrics

**Time Estimate**: 8 minutes
**Priority**: LOW
**Dependencies**: task_011
**File(s) to Modify**: `src/search/tantivy_search.rs`

## Objective
Implement comprehensive performance monitoring to track indexing and search operations.

## Success Criteria
- [ ] Track indexing performance metrics
- [ ] Monitor search latency
- [ ] Memory usage tracking
- [ ] Index size monitoring
- [ ] Performance alerts for slow operations

## Instructions

### Step 1: Add performance metrics structure
```rust
use std::time::{Duration, Instant};
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Default)]
pub struct TantivyMetrics {
    pub documents_indexed: AtomicU64,
    pub total_indexing_time: AtomicU64,  // in microseconds
    pub searches_performed: AtomicU64,
    pub total_search_time: AtomicU64,    // in microseconds
    pub index_size_bytes: AtomicU64,
    pub memory_usage_bytes: AtomicU64,
    pub slow_operations: AtomicU64,
}

impl TantivyMetrics {
    pub fn avg_indexing_time_ms(&self) -> f64 {
        let count = self.documents_indexed.load(Ordering::Relaxed);
        if count == 0 { return 0.0; }
        
        let total_us = self.total_indexing_time.load(Ordering::Relaxed);
        (total_us as f64) / (count as f64) / 1000.0
    }
    
    pub fn avg_search_time_ms(&self) -> f64 {
        let count = self.searches_performed.load(Ordering::Relaxed);
        if count == 0 { return 0.0; }
        
        let total_us = self.total_search_time.load(Ordering::Relaxed);
        (total_us as f64) / (count as f64) / 1000.0
    }
    
    pub fn print_summary(&self) {
        println!("=== Tantivy Performance Metrics ===");
        println!("Documents indexed: {}", self.documents_indexed.load(Ordering::Relaxed));
        println!("Avg indexing time: {:.2}ms", self.avg_indexing_time_ms());
        println!("Searches performed: {}", self.searches_performed.load(Ordering::Relaxed));
        println!("Avg search time: {:.2}ms", self.avg_search_time_ms());
        println!("Index size: {} bytes", self.index_size_bytes.load(Ordering::Relaxed));
        println!("Slow operations: {}", self.slow_operations.load(Ordering::Relaxed));
    }
}
```

### Step 2: Integrate metrics into TantivySearch
```rust
impl TantivySearch {
    // Add metrics field to struct
    pub metrics: Arc<TantivyMetrics>,
    
    pub fn add_document_monitored(&mut self, doc: Document) -> Result<()> {
        let start = Instant::now();
        
        let result = self.add_document(doc);
        
        let duration = start.elapsed();
        self.metrics.documents_indexed.fetch_add(1, Ordering::Relaxed);
        self.metrics.total_indexing_time.fetch_add(
            duration.as_micros() as u64, 
            Ordering::Relaxed
        );
        
        // Alert on slow operations (>50ms)
        if duration.as_millis() > 50 {
            self.metrics.slow_operations.fetch_add(1, Ordering::Relaxed);
            warn!("Slow indexing operation: {}ms", duration.as_millis());
        }
        
        result
    }
    
    pub fn search_monitored(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        let start = Instant::now();
        
        let result = self.search(query, limit);
        
        let duration = start.elapsed();
        self.metrics.searches_performed.fetch_add(1, Ordering::Relaxed);
        self.metrics.total_search_time.fetch_add(
            duration.as_micros() as u64,
            Ordering::Relaxed
        );
        
        // Alert on slow searches (>20ms)
        if duration.as_millis() > 20 {
            self.metrics.slow_operations.fetch_add(1, Ordering::Relaxed);
            warn!("Slow search operation: {}ms for '{}'", duration.as_millis(), query);
        }
        
        result
    }
}
```

### Step 3: Add index size monitoring
```rust
impl TantivySearch {
    pub fn update_index_size_metrics(&self) -> Result<()> {
        let index_path = self.index.settings().default_index_path();
        
        fn dir_size(path: &Path) -> u64 {
            let mut size = 0;
            if let Ok(entries) = fs::read_dir(path) {
                for entry in entries {
                    if let Ok(entry) = entry {
                        if let Ok(metadata) = entry.metadata() {
                            if metadata.is_file() {
                                size += metadata.len();
                            } else if metadata.is_dir() {
                                size += dir_size(&entry.path());
                            }
                        }
                    }
                }
            }
            size
        }
        
        if let Some(path) = index_path {
            let size = dir_size(&path);
            self.metrics.index_size_bytes.store(size, Ordering::Relaxed);
        }
        
        Ok(())
    }
    
    pub fn get_memory_usage(&self) -> u64 {
        // This is a simplified memory usage calculation
        // In production, use a proper memory profiler
        let estimated_usage = 
            self.metrics.documents_indexed.load(Ordering::Relaxed) * 1024 +  // Approx per doc
            self.metrics.index_size_bytes.load(Ordering::Relaxed) / 10;       // Index overhead
            
        self.metrics.memory_usage_bytes.store(estimated_usage, Ordering::Relaxed);
        estimated_usage
    }
}
```

### Step 4: Create performance benchmarking
```rust
#[test]
fn test_performance_metrics() {
    let dir = tempdir().unwrap();
    let mut tantivy = TantivySearch::new_with_metrics(dir.path()).unwrap();
    
    // Add test documents and measure performance
    for i in 0..100 {
        let doc = Document {
            content: format!("Test document {} with content", i),
            path: format!("file{}.txt", i),
            chunk_index: 0,
            start_line: 1,
            end_line: 1,
        };
        tantivy.add_document_monitored(doc).unwrap();
    }
    tantivy.commit().unwrap();
    
    // Perform searches and measure
    for _ in 0..50 {
        let _ = tantivy.search_monitored("test", 10).unwrap();
        let _ = tantivy.search_monitored("document", 5).unwrap();
    }
    
    // Update and verify metrics
    tantivy.update_index_size_metrics().unwrap();
    let memory_usage = tantivy.get_memory_usage();
    
    // Print performance summary
    tantivy.metrics.print_summary();
    
    // Verify performance targets
    assert!(tantivy.metrics.avg_indexing_time_ms() < 50.0, "Indexing too slow");
    assert!(tantivy.metrics.avg_search_time_ms() < 30.0, "Search too slow");
    assert!(memory_usage > 0, "Memory usage should be tracked");
    
    println!("Memory usage: {} bytes", memory_usage);
}
```

### Step 5: Add performance alerts
```rust
impl TantivySearch {
    pub fn check_performance_health(&self) -> Vec<String> {
        let mut alerts = Vec::new();
        
        if self.metrics.avg_indexing_time_ms() > 100.0 {
            alerts.push(format!("High indexing latency: {:.2}ms", 
                self.metrics.avg_indexing_time_ms()));
        }
        
        if self.metrics.avg_search_time_ms() > 50.0 {
            alerts.push(format!("High search latency: {:.2}ms", 
                self.metrics.avg_search_time_ms()));
        }
        
        let index_size_mb = self.metrics.index_size_bytes.load(Ordering::Relaxed) / 1024 / 1024;
        if index_size_mb > 1000 {  // 1GB
            alerts.push(format!("Large index size: {}MB", index_size_mb));
        }
        
        let slow_ops_ratio = self.metrics.slow_operations.load(Ordering::Relaxed) as f64 / 
            (self.metrics.documents_indexed.load(Ordering::Relaxed) + 
             self.metrics.searches_performed.load(Ordering::Relaxed)) as f64;
             
        if slow_ops_ratio > 0.1 {  // 10% slow operations
            alerts.push(format!("High slow operation ratio: {:.1}%", slow_ops_ratio * 100.0));
        }
        
        alerts
    }
}
```

## Terminal Commands
```bash
cd C:\code\embed
cargo check --features tantivy
cargo test --features tantivy test_performance_metrics -v
```

## Performance Targets
- **Indexing**: <50ms per document (average)
- **Search**: <30ms (average)
- **Index Size**: <1GB for 100k documents
- **Slow Operations**: <10% of total operations

## Troubleshooting
- If atomic operations fail, check std::sync imports
- If metrics are inaccurate, verify timing calculations
- For production, integrate with proper monitoring tools

## Next Task
task_013 - Create comprehensive integration test suite