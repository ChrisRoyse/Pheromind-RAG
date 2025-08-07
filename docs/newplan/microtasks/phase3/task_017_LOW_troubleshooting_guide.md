# Task 3.017: Create Troubleshooting Guide

**Time Estimate**: 8 minutes
**Priority**: LOW
**Dependencies**: task_016
**File(s) to Modify**: `docs/tantivy_troubleshooting.md` (new file)

## Objective
Create a comprehensive troubleshooting guide for common Tantivy issues and their solutions.

## Success Criteria
- [ ] Common problems documented with solutions
- [ ] Diagnostic procedures provided
- [ ] Error messages explained
- [ ] Recovery procedures outlined
- [ ] Prevention strategies included

## Instructions

### Step 1: Create troubleshooting documentation
```markdown
# Tantivy Search Troubleshooting Guide

## Quick Diagnosis

### Check System Health
```bash
# Run production validation
cargo run --bin production_validation --features tantivy

# Check logs
tail -f tantivy.log

# Monitor resource usage
top -p $(pgrep your_app_name)
```

### Basic Health Check
```rust
// Test basic functionality
let health_status = tantivy.check_performance_health();
if !health_status.is_empty() {
    println!("Issues detected: {:?}", health_status);
}
```

## Common Problems and Solutions

### 1. Compilation Errors

#### Problem: "sort_by_field" not found
**Error:**
```
error[E0560]: struct `IndexSettings` has no field named `sort_by_field`
```

**Cause:** Using Tantivy v0.24+ with old API code

**Solution:**
```rust
// OLD (v0.23 and earlier)
let settings = IndexSettings {
    sort_by_field: None,  // REMOVE THIS
    docstore_compression: Compressor::Lz4,
    docstore_blocksize: 16384,
};

// NEW (v0.24+)
let settings = IndexSettings {
    docstore_compression: Compressor::Lz4,
    docstore_blocksize: 16384,
};
```

#### Problem: Query parser compilation errors
**Error:**
```
error: no method named `parse_query` found for type `QueryParser`
```

**Solution:** Check Tantivy version and update imports:
```rust
use tantivy::query::QueryParser;
use tantivy::query::Query;
```

### 2. Runtime Errors

#### Problem: Index creation fails
**Error:**
```
TantivyError: Failed to create index: Permission denied
```

**Diagnosis:**
```bash
# Check directory permissions
ls -la /path/to/index/directory

# Check disk space
df -h /path/to/index

# Check if directory is writable
touch /path/to/index/test_file && rm /path/to/index/test_file
```

**Solutions:**
1. Fix permissions:
   ```bash
   chmod 755 /path/to/index/directory
   chown user:group /path/to/index/directory
   ```

2. Free up disk space:
   ```bash
   # Clean up old indexes
   rm -rf /path/to/old/indexes/*
   
   # Check for large log files
   find /var/log -name "*.log" -size +100M
   ```

3. Create directory if missing:
   ```rust
   std::fs::create_dir_all(index_path)?;
   ```

#### Problem: "Index locked by another process"
**Error:**
```
TantivyError: IndexLocked
```

**Diagnosis:**
```bash
# Check for stale lock files
find /path/to/index -name "*.lock"

# Check for running processes
ps aux | grep your_app_name
lsof | grep /path/to/index
```

**Solutions:**
1. Wait for other process to finish
2. Remove stale lock files (if safe):
   ```bash
   rm /path/to/index/*.lock
   ```
3. Kill stale processes:
   ```bash
   pkill -f your_app_name
   ```

### 3. Performance Issues

#### Problem: Slow indexing (>100ms per document)
**Symptoms:**
- High indexing latency
- CPU usage below expected
- Frequent disk I/O

**Diagnosis:**
```rust
// Check current performance
let metrics = tantivy.metrics.clone();
println!("Avg indexing time: {:.2}ms", metrics.avg_indexing_time_ms());
println!("Documents indexed: {}", metrics.documents_indexed.load(Ordering::Relaxed));
```

**Solutions:**
1. Increase memory buffer:
   ```rust
   let writer = index.writer_with_num_threads(
       num_cpus::get(),
       200_000_000,  // Increase to 200MB
   )?;
   ```

2. Batch commits:
   ```rust
   for (i, doc) in documents.enumerate() {
       writer.add_document(doc)?;
       if i % 1000 == 0 {  // Commit every 1000 docs
           writer.commit()?;
       }
   }
   ```

3. Use faster compression:
   ```rust
   let settings = IndexSettings {
       docstore_compression: Compressor::Lz4,  // Faster than Brotli
       docstore_blocksize: 32768,              // Larger blocks
   };
   ```

#### Problem: Slow search (>50ms)
**Symptoms:**
- High search latency
- Memory pressure during search
- CPU spikes

**Diagnosis:**
```rust
// Time individual searches
let start = Instant::now();
let results = tantivy.search("query", 10)?;
let duration = start.elapsed();
println!("Search took: {:?}", duration);
```

**Solutions:**
1. Keep reader alive:
   ```rust
   // DON'T create new reader each time
   struct SearchService {
       reader: IndexReader,  // Keep this alive
   }
   ```

2. Limit result count:
   ```rust
   // Instead of large limits
   let results = tantivy.search("query", 1000)?;  // Too many
   
   // Use reasonable limits
   let results = tantivy.search("query", 20)?;    // Better
   ```

3. Optimize query complexity:
   ```rust
   // Simple queries are faster
   let results = tantivy.search("simple term", 10)?;
   
   // Complex queries are slower
   let results = tantivy.search("term1 AND (term2 OR term3) NOT term4", 10)?;
   ```

### 4. Memory Issues

#### Problem: Out of memory during indexing
**Error:**
```
std::alloc::alloc: memory allocation of 1073741824 bytes failed
```

**Diagnosis:**
```bash
# Check memory usage
free -h
top -o %MEM
```

**Solutions:**
1. Reduce memory budget:
   ```rust
   let writer = index.writer_with_num_threads(
       1,           // Fewer threads
       20_000_000,  // Smaller buffer (20MB)
   )?;
   ```

2. More frequent commits:
   ```rust
   for (i, doc) in documents.enumerate() {
       writer.add_document(doc)?;
       if i % 100 == 0 {  // More frequent commits
           writer.commit()?;
       }
   }
   ```

3. Use better compression:
   ```rust
   let settings = IndexSettings {
       docstore_compression: Compressor::Brotli,  // Better compression
       docstore_blocksize: 8192,                 // Smaller blocks
   };
   ```

### 5. Search Quality Issues

#### Problem: Search returns no results
**Symptoms:**
- Known content not found
- Empty result sets
- Inconsistent search behavior

**Diagnosis:**
```rust
// Check if documents are indexed
let reader = index.reader()?;
let searcher = reader.searcher();
let num_docs = searcher.num_docs();
println!("Total documents in index: {}", num_docs);

// Test with broad queries
let results = tantivy.search("*", 10)?;  // Find anything
let results = tantivy.search("a", 10)?;   // Common letter
```

**Solutions:**
1. Verify documents are committed:
   ```rust
   writer.add_document(doc)?;
   writer.commit()?;  // Must commit!
   ```

2. Check tokenization:
   ```rust
   // Debug tokenization
   let tokenizer = TextAnalyzer::from(SimpleTokenizer)
       .filter(LowerCaser);
   
   let mut tokens = Vec::new();
   let mut token_stream = tokenizer.token_stream("your search query");
   while let Some(token) = token_stream.next() {
       tokens.push(token.text.clone());
   }
   println!("Tokens: {:?}", tokens);
   ```

3. Try fuzzy search:
   ```rust
   // If exact search fails, try fuzzy
   let fuzzy_results = tantivy.search_fuzzy("query", 2)?;
   ```

#### Problem: Irrelevant search results
**Symptoms:**
- Results don't match query intent
- Poor ranking
- Too many false positives

**Solutions:**
1. Use phrase queries:
   ```rust
   // Instead of individual terms
   let results = tantivy.search("term1 term2", 10)?;
   
   // Try phrase search
   let results = tantivy.search("\"term1 term2\"", 10)?;
   ```

2. Adjust fuzzy distance:
   ```rust
   // Too fuzzy
   let results = tantivy.search_fuzzy("query", 3)?;
   
   // More precise
   let results = tantivy.search_fuzzy("query", 1)?;
   ```

### 6. Index Corruption

#### Problem: Index corruption errors
**Error:**
```
TantivyError: IndexCorruption { details: "Invalid file format" }
```

**Diagnosis:**
```bash
# Check index file integrity
ls -la /path/to/index/
file /path/to/index/*

# Check for partial writes
find /path/to/index -size 0
```

**Recovery:**
1. Try to open with error tolerance:
   ```rust
   match Index::open_in_dir(path) {
       Ok(index) => /* use index */,
       Err(_) => {
           println!("Index corrupted, rebuilding...");
           rebuild_index(path)?;
       }
   }
   ```

2. Restore from backup:
   ```bash
   rm -rf /path/to/corrupted/index
   cp -r /path/to/backup/index /path/to/index
   ```

3. Rebuild from source:
   ```rust
   // Remove corrupted index
   std::fs::remove_dir_all(index_path)?;
   
   // Create new index
   let mut tantivy = TantivySearch::new(index_path)?;
   
   // Re-index all documents
   for doc in all_documents {
       tantivy.add_document(doc)?;
   }
   tantivy.commit()?;
   ```

## Diagnostic Tools

### Index Inspector
```rust
// src/bin/index_inspector.rs
use tantivy::Index;

fn main() {
    let index_path = std::env::args().nth(1).expect("Usage: index_inspector <path>");
    
    match Index::open_in_dir(index_path) {
        Ok(index) => {
            let schema = index.schema();
            println!("Schema: {:?}", schema);
            
            let reader = index.reader().unwrap();
            let searcher = reader.searcher();
            println!("Documents: {}", searcher.num_docs());
            
            // Try a test search
            match searcher.search(&tantivy::query::AllQuery, &tantivy::collector::Count) {
                Ok(count) => println!("Searchable documents: {}", count),
                Err(e) => println!("Search failed: {}", e),
            }
        },
        Err(e) => {
            println!("Failed to open index: {}", e);
            println!("Index may be corrupted or incompatible");
        }
    }
}
```

### Performance Profiler
```rust
// src/bin/performance_profiler.rs
fn main() {
    let mut tantivy = TantivySearch::new("test_index").unwrap();
    
    // Profile indexing
    let start = Instant::now();
    for i in 0..1000 {
        let doc = create_test_document(i);
        tantivy.add_document(doc).unwrap();
    }
    tantivy.commit().unwrap();
    println!("Indexing 1000 docs: {:?}", start.elapsed());
    
    // Profile searching
    let queries = vec!["test", "document", "sample", "content"];
    for query in queries {
        let start = Instant::now();
        let results = tantivy.search(query, 10).unwrap();
        println!("Search '{}': {:?} ({} results)", query, start.elapsed(), results.len());
    }
    
    // Print performance summary
    tantivy.metrics.print_summary();
}
```

## Prevention Strategies

### 1. Regular Health Checks
```rust
// Schedule regular health checks
std::thread::spawn(|| {
    let mut monitor = TantivyMonitor::new();
    
    loop {
        let health = monitor.health_check(&tantivy);
        if health != HealthStatus::Healthy {
            eprintln!("Tantivy health issue: {:?}", health);
        }
        
        std::thread::sleep(Duration::from_secs(60));
    }
});
```

### 2. Backup Strategy
```bash
#!/bin/bash
# backup_tantivy_index.sh

INDEX_PATH="/path/to/tantivy/index"
BACKUP_PATH="/path/to/backups/$(date +%Y%m%d_%H%M%S)"

# Create backup
cp -r "$INDEX_PATH" "$BACKUP_PATH"

# Keep only last 7 backups
ls -dt /path/to/backups/* | tail -n +8 | xargs rm -rf
```

### 3. Monitoring Integration
```rust
// Add metrics to your monitoring system
impl TantivySearch {
    pub fn export_metrics(&self) -> HashMap<String, f64> {
        let mut metrics = HashMap::new();
        
        metrics.insert("avg_indexing_time_ms".to_string(), 
                      self.metrics.avg_indexing_time_ms());
        metrics.insert("avg_search_time_ms".to_string(), 
                      self.metrics.avg_search_time_ms());
        metrics.insert("total_documents".to_string(), 
                      self.metrics.documents_indexed.load(Ordering::Relaxed) as f64);
        
        metrics
    }
}
```
```

### Step 2: Create diagnostic utilities
```rust
// src/bin/tantivy_doctor.rs
use embed::search::tantivy_search::*;
use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("Usage: tantivy_doctor <index_path>");
        return;
    }
    
    let index_path = Path::new(&args[1]);
    
    println!("🔍 Tantivy Doctor - Diagnosing index at {:?}", index_path);
    
    // Check 1: Path exists
    if !index_path.exists() {
        println!("❌ Index path does not exist");
        return;
    }
    println!("✅ Index path exists");
    
    // Check 2: Can open index
    match TantivySearch::new(index_path) {
        Ok(tantivy) => {
            println!("✅ Index opens successfully");
            
            // Check 3: Basic functionality
            match test_basic_operations(&tantivy) {
                Ok(()) => println!("✅ Basic operations work"),
                Err(e) => println!("❌ Basic operations failed: {}", e),
            }
            
            // Check 4: Performance
            test_performance(&tantivy);
            
        },
        Err(e) => {
            println!("❌ Cannot open index: {}", e);
            println!("💡 Try rebuilding the index");
        }
    }
}

fn test_basic_operations(tantivy: &TantivySearch) -> Result<(), Box<dyn std::error::Error>> {
    // Try a simple search
    let _results = tantivy.search("test", 1)?;
    Ok(())
}

fn test_performance(tantivy: &TantivySearch) {
    use std::time::Instant;
    
    let start = Instant::now();
    match tantivy.search("performance_test", 10) {
        Ok(results) => {
            let duration = start.elapsed();
            if duration.as_millis() > 50 {
                println!("⚠️  Search is slow: {}ms", duration.as_millis());
            } else {
                println!("✅ Search performance is good: {}ms", duration.as_millis());
            }
            println!("   Found {} results", results.len());
        },
        Err(e) => println!("❌ Performance test failed: {}", e),
    }
}
```

## Terminal Commands
```bash
cd C:\code\embed
cargo run --bin tantivy_doctor --features tantivy -- /path/to/index
cargo run --bin index_inspector --features tantivy -- /path/to/index
cargo run --bin performance_profiler --features tantivy
```

## Emergency Procedures

### Complete Index Rebuild
1. Stop application
2. Backup current index (if possible)
3. Delete corrupted index
4. Restart application (will create new index)
5. Re-index all documents

### Recovery from Backup
1. Stop application
2. Remove corrupted index
3. Restore from backup
4. Restart application
5. Verify functionality

## Getting Help

1. **Check logs** for detailed error messages
2. **Run diagnostic tools** to identify issues
3. **Review configuration** for optimal settings
4. **Monitor resource usage** (CPU, memory, disk)
5. **Test with minimal examples** to isolate problems

## Troubleshooting
- If issues persist, check hardware resources
- For data corruption, always restore from backup
- When in doubt, rebuild the index from source data

## Next Task
task_018 - Add benchmark suite