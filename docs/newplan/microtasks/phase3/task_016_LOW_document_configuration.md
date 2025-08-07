# Task 3.016: Document Configuration Options

**Time Estimate**: 8 minutes
**Priority**: LOW
**Dependencies**: task_015
**File(s) to Modify**: `docs/tantivy_configuration.md` (new file)

## Objective
Create comprehensive documentation for all Tantivy configuration options and tuning parameters.

## Success Criteria
- [ ] All configuration options documented
- [ ] Performance tuning guidelines provided
- [ ] Examples for different use cases
- [ ] Troubleshooting guide included
- [ ] Best practices documented

## Instructions

### Step 1: Create configuration documentation
```markdown
# Tantivy Search Configuration Guide

## Basic Configuration

### Index Settings

```rust
let settings = IndexSettings {
    docstore_compression: Compressor::Lz4,  // or Compressor::Brotli, Compressor::None
    docstore_blocksize: 16384,              // 16KB blocks (default), can be 8192, 32768
};
```

**Options:**
- `docstore_compression`:
  - `Lz4`: Fast compression, good for real-time indexing
  - `Brotli`: Better compression ratio, slower
  - `None`: No compression, fastest but largest size
- `docstore_blocksize`:
  - `8192` (8KB): Less memory, more I/O
  - `16384` (16KB): Balanced (recommended)
  - `32768` (32KB): More memory, less I/O

### Writer Configuration

```rust
let num_threads = num_cpus::get();        // Use all CPU cores
let memory_budget = 50_000_000;           // 50MB buffer

let writer = index.writer_with_num_threads(num_threads, memory_budget)?;
```

**Parameters:**
- `num_threads`: Number of indexing threads
  - `1`: Single-threaded, predictable performance
  - `num_cpus::get()`: Use all cores (recommended for batch indexing)
  - `num_cpus::get() / 2`: Leave cores for other processes
- `memory_budget`: RAM for indexing buffer
  - `10MB`: Minimum for small datasets
  - `50MB`: Recommended default
  - `200MB+`: For high-throughput indexing

### Reader Configuration

```rust
let reader = index.reader_builder()
    .reload_policy(ReloadPolicy::OnCommit)  // or ReloadPolicy::Manual
    .try_into()?;
```

**Options:**
- `ReloadPolicy::OnCommit`: Automatic reload after commits (recommended)
- `ReloadPolicy::Manual`: Manual reload control for custom scenarios

## Performance Tuning

### For High-Throughput Indexing

```rust
// Optimize for batch indexing
let settings = IndexSettings {
    docstore_compression: Compressor::Lz4,  // Fast compression
    docstore_blocksize: 32768,              // Larger blocks
};

let writer = index.writer_with_num_threads(
    num_cpus::get(),     // All cores
    200_000_000,         // 200MB buffer
)?;
```

**Best Practices:**
- Batch commits (every 1000-10000 documents)
- Use larger memory buffers
- Consider SSD storage for index
- Monitor CPU and I/O utilization

### For Low-Latency Search

```rust
// Optimize for search speed
let settings = IndexSettings {
    docstore_compression: Compressor::Lz4,  // Fast decompression
    docstore_blocksize: 16384,              // Balanced blocks
};

// Warm up reader
let reader = index.reader()?;
let searcher = reader.searcher();
```

**Best Practices:**
- Keep reader instances alive
- Pre-warm caches with common queries
- Use appropriate result limits
- Consider query complexity vs. speed

### For Memory-Constrained Environments

```rust
// Optimize for low memory usage
let settings = IndexSettings {
    docstore_compression: Compressor::Brotli,  // Better compression
    docstore_blocksize: 8192,                  // Smaller blocks
};

let writer = index.writer_with_num_threads(
    1,           // Single thread
    10_000_000,  // 10MB buffer
)?;
```

**Best Practices:**
- Use higher compression ratios
- Smaller memory buffers
- More frequent commits
- Monitor memory usage

## Schema Configuration

### Text Fields

```rust
let schema = Schema::builder()
    .add_text_field("title", TEXT | STORED)     // Searchable and retrievable
    .add_text_field("body", TEXT)              // Searchable only
    .add_text_field("metadata", STORED)        // Retrievable only
    .build();
```

**Field Options:**
- `TEXT`: Makes field searchable (indexed)
- `STORED`: Makes field retrievable in results
- `TEXT | STORED`: Both searchable and retrievable
- Neither: Field is ignored (not recommended)

### Numeric Fields

```rust
let schema = Schema::builder()
    .add_u64_field("timestamp", INDEXED | STORED)  // Searchable timestamp
    .add_u64_field("file_size", STORED)           // Just for display
    .build();
```

**Numeric Options:**
- `INDEXED`: Enables range queries and sorting
- `STORED`: Includes in search results
- `FAST`: Enables fast field access (column storage)

## Environment-Specific Configuration

### Development Environment

```rust
// Fast iteration, debugging friendly
let config = TantivyConfig {
    index_settings: IndexSettings {
        docstore_compression: Compressor::None,
        docstore_blocksize: 8192,
    },
    writer_threads: 1,
    memory_budget: 10_000_000,
    enable_metrics: true,
    log_level: LogLevel::Debug,
};
```

### Production Environment

```rust
// Optimized for performance and reliability
let config = TantivyConfig {
    index_settings: IndexSettings {
        docstore_compression: Compressor::Lz4,
        docstore_blocksize: 16384,
    },
    writer_threads: num_cpus::get(),
    memory_budget: 100_000_000,
    enable_metrics: true,
    log_level: LogLevel::Info,
};
```

### Testing Environment

```rust
// Fast, deterministic, minimal resources
let config = TantivyConfig {
    index_settings: IndexSettings {
        docstore_compression: Compressor::None,
        docstore_blocksize: 8192,
    },
    writer_threads: 1,
    memory_budget: 5_000_000,
    enable_metrics: false,
    log_level: LogLevel::Error,
};
```

## Common Configuration Issues

### Problem: Slow Indexing

**Symptoms:**
- High indexing latency (>100ms per document)
- CPU usage below 100%
- Frequent disk writes

**Solutions:**
```rust
// Increase buffer size and threads
let writer = index.writer_with_num_threads(
    num_cpus::get(),
    200_000_000,  // Larger buffer
)?;

// Batch commits
for (i, doc) in documents.enumerate() {
    writer.add_document(doc)?;
    if i % 1000 == 0 {
        writer.commit()?;
    }
}
```

### Problem: High Memory Usage

**Symptoms:**
- Out of memory errors
- System slowdown
- Excessive swap usage

**Solutions:**
```rust
// Reduce buffer size and use compression
let settings = IndexSettings {
    docstore_compression: Compressor::Brotli,
    docstore_blocksize: 8192,
};

let writer = index.writer_with_num_threads(1, 20_000_000)?;

// More frequent commits
for (i, doc) in documents.enumerate() {
    writer.add_document(doc)?;
    if i % 100 == 0 {  // More frequent
        writer.commit()?;
    }
}
```

### Problem: Slow Search

**Symptoms:**
- Search latency >50ms
- High CPU during search
- Memory pressure

**Solutions:**
```rust
// Keep reader alive and warm up
struct SearchService {
    reader: IndexReader,  // Reuse reader
}

impl SearchService {
    fn search(&self, query: &str) -> Result<Vec<SearchResult>> {
        let searcher = self.reader.searcher();
        // ... perform search
    }
    
    // Warm up with common queries
    fn warmup(&self) {
        let _ = self.search("common");
        let _ = self.search("query");
        let _ = self.search("terms");
    }
}
```

## Monitoring Configuration

```rust
// Enable detailed monitoring
let config = TantivyConfig {
    enable_metrics: true,
    metrics_interval: Duration::from_secs(60),
    slow_query_threshold: Duration::from_millis(100),
    alert_on_errors: true,
};
```

**Monitoring Options:**
- `enable_metrics`: Collect performance statistics
- `metrics_interval`: How often to report metrics
- `slow_query_threshold`: When to log slow queries
- `alert_on_errors`: Enable error alerting

## Configuration Examples

### Small Dataset (<10k documents)
```rust
let config = TantivyConfig::small_dataset();
```

### Medium Dataset (10k-1M documents)
```rust
let config = TantivyConfig::medium_dataset();
```

### Large Dataset (>1M documents)
```rust
let config = TantivyConfig::large_dataset();
```

### Real-time Indexing
```rust
let config = TantivyConfig::realtime_indexing();
```

### Batch Processing
```rust
let config = TantivyConfig::batch_processing();
```
```

### Step 2: Create configuration validation
```rust
// src/config/tantivy_config.rs

#[derive(Debug, Clone)]
pub struct TantivyConfig {
    pub index_settings: IndexSettings,
    pub writer_threads: usize,
    pub memory_budget: usize,
    pub enable_metrics: bool,
    pub log_level: LogLevel,
}

impl TantivyConfig {
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.writer_threads == 0 {
            return Err(ConfigError::InvalidThreadCount);
        }
        
        if self.memory_budget < 1_000_000 {
            return Err(ConfigError::InsufficientMemoryBudget);
        }
        
        if self.index_settings.docstore_blocksize < 1024 {
            return Err(ConfigError::BlockSizeTooSmall);
        }
        
        Ok(())
    }
    
    pub fn small_dataset() -> Self {
        Self {
            index_settings: IndexSettings {
                docstore_compression: Compressor::None,
                docstore_blocksize: 8192,
            },
            writer_threads: 1,
            memory_budget: 10_000_000,
            enable_metrics: true,
            log_level: LogLevel::Info,
        }
    }
    
    pub fn large_dataset() -> Self {
        Self {
            index_settings: IndexSettings {
                docstore_compression: Compressor::Lz4,
                docstore_blocksize: 32768,
            },
            writer_threads: num_cpus::get(),
            memory_budget: 200_000_000,
            enable_metrics: true,
            log_level: LogLevel::Info,
        }
    }
}
```

## Terminal Commands
```bash
cd C:\code\embed
cargo check --features tantivy
cargo test --features tantivy config_validation -v
```

## Configuration Documentation Checklist
- [ ] All IndexSettings options documented
- [ ] Writer configuration explained
- [ ] Reader configuration covered
- [ ] Performance tuning guidelines provided
- [ ] Environment-specific examples included
- [ ] Common issues and solutions documented

## Troubleshooting
- If configuration is unclear, add more examples
- If performance issues persist, review hardware requirements
- Update documentation as new options are added

## Next Task
task_017 - Create troubleshooting guide