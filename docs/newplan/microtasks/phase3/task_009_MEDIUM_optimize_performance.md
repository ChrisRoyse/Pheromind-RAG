# Task 3.009: Optimize Index Performance Settings

**Time Estimate**: 10 minutes
**Priority**: MEDIUM
**Dependencies**: task_008
**File(s) to Modify**: `src/search/tantivy_search.rs`

## Objective
Configure Tantivy with optimal performance settings for indexing and searching.

## Success Criteria
- [ ] Writer uses all CPU cores efficiently
- [ ] Memory buffer size is optimized
- [ ] Compression settings balance speed/size
- [ ] Reader pool is configured properly
- [ ] Performance meets target metrics

## Instructions

### Step 1: Optimize IndexSettings
```rust
// Update IndexSettings for performance
fn create_optimized_settings() -> IndexSettings {
    IndexSettings {
        docstore_compression: Compressor::Lz4,  // Fast compression
        docstore_blocksize: 32_768,  // 32KB blocks for better performance
    }
}
```

### Step 2: Configure writer with optimal settings
```rust
impl TantivySearch {
    pub fn new_optimized(path: &Path) -> Result<Self, TantivyError> {
        let schema = Self::build_schema();
        let settings = Self::create_optimized_settings();
        
        let index = Index::builder()
            .schema(schema)
            .settings(settings)
            .create_in_dir(path)?;
        
        // Use all CPU cores with generous memory
        let num_threads = num_cpus::get().max(1);
        let memory_budget = 100_000_000; // 100MB buffer
        
        let writer = index.writer_with_num_threads(num_threads, memory_budget)?;
        
        // Configure reader with connection pool
        let reader = index.reader_builder()
            .reload_policy(ReloadPolicy::OnCommit)
            .try_into()?;
        
        let body_field = index.schema().get_field("body").unwrap();
        let path_field = index.schema().get_field("path").unwrap();
        
        Ok(Self {
            index,
            writer: Arc::new(Mutex::new(writer)),
            reader: Arc::new(reader),
            body_field,
            path_field,
            // other fields...
        })
    }
}
```

### Step 3: Add performance monitoring
```rust
use std::time::Instant;

impl TantivySearch {
    pub fn add_document_timed(&mut self, doc: Document) -> Result<std::time::Duration, TantivyError> {
        let start = Instant::now();
        
        self.add_document(doc)?;
        
        let duration = start.elapsed();
        if duration.as_millis() > 50 {  // Log slow operations
            println!("Slow document indexing: {}ms", duration.as_millis());
        }
        
        Ok(duration)
    }
    
    pub fn search_timed(&self, query: &str, limit: usize) -> Result<(Vec<SearchResult>, std::time::Duration), TantivyError> {
        let start = Instant::now();
        
        let results = self.search(query, limit)?;
        
        let duration = start.elapsed();
        if duration.as_millis() > 10 {  // Log slow searches
            println!("Slow search operation: {}ms for '{}'", duration.as_millis(), query);
        }
        
        Ok((results, duration))
    }
}
```

### Step 4: Add required imports
```rust
use tantivy::ReloadPolicy;
use std::sync::{Arc, Mutex};
use std::time::Instant;
```

### Step 5: Create performance test
```rust
#[test]
fn test_performance_benchmarks() {
    let dir = tempdir().unwrap();
    let mut tantivy = TantivySearch::new_optimized(dir.path()).unwrap();
    
    // Benchmark document indexing
    let start = Instant::now();
    for i in 0..1000 {
        let doc = Document {
            content: format!("Test document number {} with some content", i),
            path: format!("file{}.txt", i),
            chunk_index: 0,
            start_line: 1,
            end_line: 1,
        };
        tantivy.add_document(doc).unwrap();
    }
    tantivy.commit().unwrap();
    
    let indexing_time = start.elapsed();
    println!("Indexed 1000 docs in: {:?}", indexing_time);
    assert!(indexing_time.as_secs() < 5, "Indexing too slow");
    
    // Benchmark search
    let (results, search_time) = tantivy.search_timed("document", 10).unwrap();
    println!("Search took: {:?}, found {} results", search_time, results.len());
    assert!(search_time.as_millis() < 20, "Search too slow");
}
```

## Terminal Commands
```bash
cd C:\code\embed
cargo check --features tantivy
cargo test --features tantivy test_performance_benchmarks -v
```

## Performance Targets
- **Indexing**: <10ms per document
- **Search**: <20ms for 10,000 documents
- **Memory**: <500MB for 100,000 documents
- **CPU Usage**: Utilize all available cores

## Troubleshooting
- If num_cpus is missing, add to Cargo.toml
- If memory usage is high, reduce buffer size
- If indexing is slow, check disk I/O performance

## Next Task
task_010 - Test with real file content