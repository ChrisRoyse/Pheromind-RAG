# Task 3.018: Add Benchmark Suite

**Time Estimate**: 10 minutes
**Priority**: LOW
**Dependencies**: task_017
**File(s) to Modify**: `benches/tantivy_benchmarks.rs` (new file)

## Objective
Create a comprehensive benchmark suite to measure Tantivy performance and detect regressions.

## Success Criteria
- [ ] Indexing performance benchmarks
- [ ] Search performance benchmarks
- [ ] Memory usage benchmarks
- [ ] Scalability benchmarks
- [ ] Regression detection capability

## Instructions

### Step 1: Add benchmark dependencies
```toml
# Add to Cargo.toml
[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }
rand = "0.8"
lipsum = "0.9"  # For generating test content

[[bench]]
name = "tantivy_benchmarks"
harness = false
```

### Step 2: Create main benchmark file
```rust
// benches/tantivy_benchmarks.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use embed::search::tantivy_search::*;
use tempfile::tempdir;
use std::time::Duration;

// Benchmark indexing performance
fn bench_indexing(c: &mut Criterion) {
    let mut group = c.benchmark_group("indexing");
    
    // Set measurement time
    group.measurement_time(Duration::from_secs(10));
    
    // Test different document sizes
    for doc_size in [100, 1000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::new("add_document", doc_size),
            doc_size,
            |b, &size| {
                let dir = tempdir().unwrap();
                let mut tantivy = TantivySearch::new_optimized(dir.path()).unwrap();
                let content = generate_content(size);
                
                b.iter(|| {
                    let doc = Document {
                        content: content.clone(),
                        path: "benchmark.txt".to_string(),
                        chunk_index: 0,
                        start_line: 1,
                        end_line: 1,
                    };
                    
                    black_box(tantivy.add_document(doc).unwrap());
                });
            },
        );
    }
    
    // Benchmark batch indexing
    group.bench_function("batch_indexing_1000", |b| {
        b.iter_custom(|iters| {
            let mut total_time = Duration::new(0, 0);
            
            for _ in 0..iters {
                let dir = tempdir().unwrap();
                let mut tantivy = TantivySearch::new_optimized(dir.path()).unwrap();
                
                let start = std::time::Instant::now();
                
                for i in 0..1000 {
                    let doc = Document {
                        content: format!("Benchmark document number {} with content", i),
                        path: format!("bench_{}.txt", i),
                        chunk_index: 0,
                        start_line: 1,
                        end_line: 1,
                    };
                    tantivy.add_document(doc).unwrap();
                }
                tantivy.commit().unwrap();
                
                total_time += start.elapsed();
            }
            
            total_time
        });
    });
    
    group.finish();
}

// Benchmark search performance
fn bench_searching(c: &mut Criterion) {
    let mut group = c.benchmark_group("searching");
    
    // Prepare test index
    let dir = tempdir().unwrap();
    let mut tantivy = TantivySearch::new_optimized(dir.path()).unwrap();
    
    // Index test documents
    for i in 0..10000 {
        let doc = Document {
            content: generate_search_content(i),
            path: format!("search_test_{}.txt", i),
            chunk_index: 0,
            start_line: 1,
            end_line: 1,
        };
        tantivy.add_document(doc).unwrap();
        
        if i % 1000 == 0 {
            tantivy.commit().unwrap();
        }
    }
    tantivy.commit().unwrap();
    
    // Benchmark different search types
    let queries = vec![
        ("single_term", "document"),
        ("two_terms", "test document"),
        ("phrase", "\"test document\""),
        ("complex", "(test OR sample) AND document"),
    ];
    
    for (name, query) in queries {
        group.bench_with_input(
            BenchmarkId::new("search", name),
            &query,
            |b, &query| {
                b.iter(|| {
                    black_box(tantivy.search(query, 10).unwrap());
                });
            },
        );
    }
    
    // Benchmark fuzzy search
    group.bench_function("fuzzy_search", |b| {
        b.iter(|| {
            black_box(tantivy.search_fuzzy("documen", 1).unwrap());
        });
    });
    
    // Benchmark search with different result limits
    for limit in [1, 10, 100, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("search_limit", limit),
            limit,
            |b, &limit| {
                b.iter(|| {
                    black_box(tantivy.search("document", limit).unwrap());
                });
            },
        );
    }
    
    group.finish();
}

// Benchmark memory usage
fn bench_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory");
    
    // Benchmark memory usage with different dataset sizes
    for doc_count in [1000, 10000, 50000].iter() {
        group.bench_with_input(
            BenchmarkId::new("memory_usage", doc_count),
            doc_count,
            |b, &count| {
                b.iter_custom(|iters| {
                    let mut total_time = Duration::new(0, 0);
                    
                    for _ in 0..iters {
                        let dir = tempdir().unwrap();
                        let mut tantivy = TantivySearch::new_with_metrics(dir.path()).unwrap();
                        
                        let start = std::time::Instant::now();
                        
                        // Index documents and measure
                        for i in 0..count {
                            let doc = Document {
                                content: generate_content(500), // 500 chars each
                                path: format!("memory_test_{}.txt", i),
                                chunk_index: 0,
                                start_line: 1,
                                end_line: 1,
                            };
                            tantivy.add_document(doc).unwrap();
                        }
                        tantivy.commit().unwrap();
                        
                        // Measure memory usage
                        let memory_usage = tantivy.get_memory_usage();
                        println!("Memory usage for {} docs: {} bytes", count, memory_usage);
                        
                        total_time += start.elapsed();
                    }
                    
                    total_time
                });
            },
        );
    }
    
    group.finish();
}

// Benchmark concurrent operations
fn bench_concurrency(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrency");
    
    // Prepare shared index
    let dir = tempdir().unwrap();
    let mut tantivy = TantivySearch::new_optimized(dir.path()).unwrap();
    
    // Pre-populate with data
    for i in 0..5000 {
        let doc = Document {
            content: generate_search_content(i),
            path: format!("concurrent_{}.txt", i),
            chunk_index: 0,
            start_line: 1,
            end_line: 1,
        };
        tantivy.add_document(doc).unwrap();
    }
    tantivy.commit().unwrap();
    
    let tantivy = std::sync::Arc::new(tantivy);
    
    // Benchmark concurrent searches
    for thread_count in [1, 2, 4, 8].iter() {
        group.bench_with_input(
            BenchmarkId::new("concurrent_search", thread_count),
            thread_count,
            |b, &threads| {
                b.iter_custom(|iters| {
                    let mut total_time = Duration::new(0, 0);
                    
                    for _ in 0..iters {
                        let start = std::time::Instant::now();
                        
                        let mut handles = vec![];
                        
                        for _ in 0..threads {
                            let tantivy_clone = Arc::clone(&tantivy);
                            let handle = std::thread::spawn(move || {
                                for i in 0..10 {
                                    let query = format!("document {}", i % 100);
                                    let _ = tantivy_clone.search(&query, 10).unwrap();
                                }
                            });
                            handles.push(handle);
                        }
                        
                        for handle in handles {
                            handle.join().unwrap();
                        }
                        
                        total_time += start.elapsed();
                    }
                    
                    total_time
                });
            },
        );
    }
    
    group.finish();
}

// Benchmark scalability
fn bench_scalability(c: &mut Criterion) {
    let mut group = c.benchmark_group("scalability");
    
    // Test search performance with different index sizes
    let sizes = vec![1000, 5000, 10000, 25000, 50000];
    
    for size in sizes {
        group.bench_with_input(
            BenchmarkId::new("search_scalability", size),
            &size,
            |b, &doc_count| {
                // Create index with specified number of documents
                let dir = tempdir().unwrap();
                let mut tantivy = TantivySearch::new_optimized(dir.path()).unwrap();
                
                for i in 0..doc_count {
                    let doc = Document {
                        content: generate_scalability_content(i, doc_count),
                        path: format!("scale_{}.txt", i),
                        chunk_index: 0,
                        start_line: 1,
                        end_line: 1,
                    };
                    tantivy.add_document(doc).unwrap();
                    
                    if i % 1000 == 0 {
                        tantivy.commit().unwrap();
                    }
                }
                tantivy.commit().unwrap();
                
                // Benchmark search on this index size
                b.iter(|| {
                    black_box(tantivy.search("document", 10).unwrap());
                });
            },
        );
    }
    
    group.finish();
}

// Helper functions for generating test content
fn generate_content(size: usize) -> String {
    use lipsum::lipsum;
    lipsum(size / 10) // Approximate words to reach target size
}

fn generate_search_content(index: usize) -> String {
    let templates = vec![
        "This is a test document number {}",
        "Sample content for document {} with keywords",
        "Benchmark data file {} containing search terms",
        "Document {} has unique content for testing",
        "Search test file number {} with various words",
    ];
    
    let template = &templates[index % templates.len()];
    format!(
        "{} Additional content with random words: {} {} {}",
        template.replace("{}", &index.to_string()),
        lipsum::lipsum(5),
        if index % 3 == 0 { "important" } else { "normal" },
        if index % 5 == 0 { "special" } else { "regular" }
    )
}

fn generate_scalability_content(index: usize, total: usize) -> String {
    format!(
        "Scalability test document {} of {}. Content includes: {} Progress: {:.1}%",
        index,
        total,
        lipsum::lipsum(10),
        (index as f64 / total as f64) * 100.0
    )
}

criterion_group!(
    benches,
    bench_indexing,
    bench_searching,
    bench_memory_usage,
    bench_concurrency,
    bench_scalability
);
criterion_main!(benches);
```

### Step 3: Create regression detection script
```rust
// src/bin/regression_detector.rs
use std::process::Command;
use std::fs;
use serde_json::Value;

fn main() {
    println!("Running Tantivy performance regression detection...");
    
    // Run benchmarks
    let output = Command::new("cargo")
        .args(["bench", "--features", "tantivy"])
        .output()
        .expect("Failed to run benchmarks");
    
    if !output.status.success() {
        eprintln!("Benchmarks failed to run");
        std::process::exit(1);
    }
    
    // Parse benchmark results
    if let Ok(results) = parse_criterion_results() {
        detect_regressions(results);
    } else {
        println!("Could not parse benchmark results for regression detection");
    }
}

fn parse_criterion_results() -> Result<Vec<BenchmarkResult>, Box<dyn std::error::Error>> {
    // Criterion saves results in target/criterion/
    // This is simplified - in practice you'd parse the actual JSON files
    Ok(vec![])
}

#[derive(Debug)]
struct BenchmarkResult {
    name: String,
    mean_time_ns: f64,
    std_dev_ns: f64,
}

fn detect_regressions(results: Vec<BenchmarkResult>) {
    println!("Analyzing {} benchmark results...", results.len());
    
    // Load historical results (if any)
    let historical = load_historical_results().unwrap_or_default();
    
    let mut regressions = Vec::new();
    
    for result in &results {
        if let Some(historical_result) = historical.iter().find(|h| h.name == result.name) {
            let performance_ratio = result.mean_time_ns / historical_result.mean_time_ns;
            
            // Flag as regression if >20% slower
            if performance_ratio > 1.2 {
                regressions.push(RegressionAlert {
                    benchmark_name: result.name.clone(),
                    old_time_ms: historical_result.mean_time_ns / 1_000_000.0,
                    new_time_ms: result.mean_time_ns / 1_000_000.0,
                    regression_percent: (performance_ratio - 1.0) * 100.0,
                });
            }
        }
    }
    
    if regressions.is_empty() {
        println!("✅ No performance regressions detected");
    } else {
        println!("⚠️  {} performance regressions detected:", regressions.len());
        for regression in &regressions {
            println!(
                "  {} regressed {:.1}% ({:.2}ms → {:.2}ms)",
                regression.benchmark_name,
                regression.regression_percent,
                regression.old_time_ms,
                regression.new_time_ms
            );
        }
        
        // Exit with error code if significant regressions
        if regressions.iter().any(|r| r.regression_percent > 50.0) {
            std::process::exit(1);
        }
    }
    
    // Save current results as new baseline
    save_historical_results(&results).unwrap_or_else(|e| {
        eprintln!("Warning: Could not save benchmark results: {}", e);
    });
}

#[derive(Debug)]
struct RegressionAlert {
    benchmark_name: String,
    old_time_ms: f64,
    new_time_ms: f64,
    regression_percent: f64,
}

fn load_historical_results() -> Result<Vec<BenchmarkResult>, Box<dyn std::error::Error>> {
    let data = fs::read_to_string("benchmark_baseline.json")?;
    let results: Vec<BenchmarkResult> = serde_json::from_str(&data)?;
    Ok(results)
}

fn save_historical_results(results: &[BenchmarkResult]) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string_pretty(results)?;
    fs::write("benchmark_baseline.json", json)?;
    Ok(())
}
```

### Step 4: Create performance targets
```rust
// tests/performance_targets.rs
use embed::search::tantivy_search::*;
use tempfile::tempdir;
use std::time::Duration;

#[test]
fn test_indexing_performance_targets() {
    let dir = tempdir().unwrap();
    let mut tantivy = TantivySearch::new_optimized(dir.path()).unwrap();
    
    let start = std::time::Instant::now();
    
    // Index 1000 documents
    for i in 0..1000 {
        let doc = Document {
            content: format!("Performance target test document {}", i),
            path: format!("target_{}.txt", i),
            chunk_index: 0,
            start_line: 1,
            end_line: 1,
        };
        tantivy.add_document(doc).unwrap();
    }
    tantivy.commit().unwrap();
    
    let total_time = start.elapsed();
    let avg_time_ms = total_time.as_millis() as f64 / 1000.0;
    
    // Target: <50ms per document on average
    assert!(avg_time_ms < 50.0, 
        "Indexing performance target missed: {:.2}ms/doc (target: <50ms)", 
        avg_time_ms);
    
    println!("Indexing performance: {:.2}ms/doc (target: <50ms)", avg_time_ms);
}

#[test]
fn test_search_performance_targets() {
    let dir = tempdir().unwrap();
    let mut tantivy = TantivySearch::new_optimized(dir.path()).unwrap();
    
    // Pre-populate index
    for i in 0..10000 {
        let doc = Document {
            content: format!("Search target document {} with content", i),
            path: format!("search_target_{}.txt", i),
            chunk_index: 0,
            start_line: 1,
            end_line: 1,
        };
        tantivy.add_document(doc).unwrap();
    }
    tantivy.commit().unwrap();
    
    // Test search performance
    let queries = vec!["document", "content", "search", "target"];
    let mut total_time = Duration::new(0, 0);
    
    for query in &queries {
        let start = std::time::Instant::now();
        let _results = tantivy.search(query, 10).unwrap();
        total_time += start.elapsed();
    }
    
    let avg_time_ms = total_time.as_millis() as f64 / queries.len() as f64;
    
    // Target: <30ms per search
    assert!(avg_time_ms < 30.0,
        "Search performance target missed: {:.2}ms (target: <30ms)",
        avg_time_ms);
        
    println!("Search performance: {:.2}ms (target: <30ms)", avg_time_ms);
}

#[test]
fn test_memory_usage_targets() {
    let dir = tempdir().unwrap();
    let mut tantivy = TantivySearch::new_with_metrics(dir.path()).unwrap();
    
    // Index 10k documents
    for i in 0..10000 {
        let doc = Document {
            content: "Memory usage test document with standard content length".to_string(),
            path: format!("memory_{}.txt", i),
            chunk_index: 0,
            start_line: 1,
            end_line: 1,
        };
        tantivy.add_document(doc).unwrap();
    }
    tantivy.commit().unwrap();
    
    let memory_usage = tantivy.get_memory_usage();
    let memory_mb = memory_usage as f64 / 1024.0 / 1024.0;
    
    // Target: <50MB for 10k documents
    assert!(memory_mb < 50.0,
        "Memory usage target missed: {:.2}MB (target: <50MB)",
        memory_mb);
        
    println!("Memory usage: {:.2}MB for 10k docs (target: <50MB)", memory_mb);
}
```

### Step 5: Add CI integration
```yaml
# .github/workflows/benchmarks.yml
name: Performance Benchmarks

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

jobs:
  benchmark:
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        override: true
        
    - name: Run performance targets
      run: |
        cargo test --features tantivy --release performance_targets
        
    - name: Run benchmarks
      run: |
        cargo bench --features tantivy
        
    - name: Check for regressions
      run: |
        cargo run --bin regression_detector --features tantivy
```

## Terminal Commands
```bash
cd C:\code\embed
cargo bench --features tantivy
cargo test --features tantivy performance_targets -v
cargo run --bin regression_detector --features tantivy
```

## Benchmark Categories
1. **Indexing**: Document addition and commit performance
2. **Searching**: Query execution speed
3. **Memory**: Memory usage under various loads
4. **Concurrency**: Multi-threaded performance
5. **Scalability**: Performance with different dataset sizes

## Performance Targets
- **Indexing**: <50ms per document average
- **Search**: <30ms for typical queries
- **Memory**: <50MB for 10k documents
- **Scalability**: Linear or sub-linear growth

## Troubleshooting
- If benchmarks are slow, check system resources
- If results are inconsistent, increase measurement time
- For regression detection, ensure consistent test environment

## Next Task
task_019 - Create final validation test