# Task 028 - MEDIUM: Verify Cache Performance Meets Requirements

## Priority: MEDIUM
## Estimated Time: 10 minutes
## Dependencies: Task 027

## Objective
Benchmark and validate that the cache implementation meets performance requirements for production use.

## Current Issue
- Need to verify performance benchmarks
- Identify bottlenecks and optimization opportunities
- Ensure scalability for large datasets

## Tasks
1. **Create comprehensive benchmarks** (6 min)
   ```rust
   // In src/ml/tests/cache_benchmarks.rs
   use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
   use crate::ml::embedding_cache::*;
   use tokio::runtime::Runtime;
   use std::sync::Arc;
   
   fn create_test_cache() -> EmbeddingCache {
       EmbeddingCache::new(
           10000,  // 10k entries
           3600,   // 1 hour TTL
           500,    // 500MB limit
       )
   }
   
   fn create_test_embedding(size: usize) -> Vec<f32> {
       (0..size).map(|i| i as f32 / size as f32).collect()
   }
   
   fn create_test_key(id: usize) -> CacheKey {
       CacheKey {
           content_hash: format!("benchmark_key_{}", id),
           model_version: "nomic-embed-text-v1.5".to_string(),
           chunk_id: Some(format!("chunk_{}", id % 100)),
       }
   }
   
   fn bench_cache_put(c: &mut Criterion) {
       let rt = Runtime::new().unwrap();
       let cache = create_test_cache();
       
       let mut group = c.benchmark_group("cache_put");
       
       // Test different embedding sizes
       for &embedding_size in &[128, 384, 768, 1536] {
           group.bench_with_input(
               BenchmarkId::new("embedding_size", embedding_size),
               &embedding_size,
               |b, &size| {
                   let embedding = create_test_embedding(size);
                   let mut counter = 0;
                   
                   b.iter(|| {
                       let key = create_test_key(counter);
                       counter += 1;
                       
                       rt.block_on(async {
                           cache.put(key, black_box(embedding.clone())).await.unwrap();
                       });
                   });
               },
           );
       }
       
       group.finish();
   }
   
   fn bench_cache_get(c: &mut Criterion) {
       let rt = Runtime::new().unwrap();
       let cache = create_test_cache();
       
       // Pre-populate cache
       rt.block_on(async {
           for i in 0..1000 {
               let key = create_test_key(i);
               let embedding = create_test_embedding(768);
               cache.put(key, embedding).await.unwrap();
           }
       });
       
       let mut group = c.benchmark_group("cache_get");
       
       group.bench_function("hit", |b| {
           let mut counter = 0;
           b.iter(|| {
               let key = create_test_key(counter % 1000); // Always hit
               counter += 1;
               
               rt.block_on(async {
                   let result = cache.get(&key).await;
                   black_box(result);
               });
           });
       });
       
       group.bench_function("miss", |b| {
           let mut counter = 10000; // Keys that don't exist
           b.iter(|| {
               let key = create_test_key(counter);
               counter += 1;
               
               rt.block_on(async {
                   let result = cache.get(&key).await;
                   black_box(result);
               });
           });
       });
       
       group.finish();
   }
   
   fn bench_concurrent_access(c: &mut Criterion) {
       let rt = Runtime::new().unwrap();
       let cache = Arc::new(create_test_cache());
       
       let mut group = c.benchmark_group("concurrent_access");
       
       for &num_threads in &[1, 2, 4, 8, 16] {
           group.bench_with_input(
               BenchmarkId::new("threads", num_threads),
               &num_threads,
               |b, &threads| {
                   b.iter(|| {
                       rt.block_on(async {
                           let mut handles = vec![];
                           
                           for thread_id in 0..threads {
                               let cache_clone = Arc::clone(&cache);
                               let handle = tokio::spawn(async move {
                                   for i in 0..100 {
                                       let key = create_test_key(thread_id * 1000 + i);
                                       let embedding = create_test_embedding(768);
                                       
                                       // 50% put, 50% get operations
                                       if i % 2 == 0 {
                                           cache_clone.put(key, embedding).await.unwrap();
                                       } else {
                                           let _ = cache_clone.get(&key).await;
                                       }
                                   }
                               });
                               handles.push(handle);
                           }
                           
                           for handle in handles {
                               handle.await.unwrap();
                           }
                       });
                   });
               },
           );
       }
       
       group.finish();
   }
   ```

2. **Add memory usage validation** (2 min)
   ```rust
   #[tokio::test]
   async fn test_memory_usage_accuracy() {
       let cache = EmbeddingCache::new(1000, 3600, 100); // 100MB limit
       
       // Add known-size embeddings
       let embedding_size = 768; // f32 = 4 bytes, so 768 * 4 = 3072 bytes per embedding
       let num_embeddings = 100;
       
       for i in 0..num_embeddings {
           let key = CacheKey {
               content_hash: format!("memory_test_{}", i),
               model_version: "v1".to_string(),
               chunk_id: None,
           };
           let embedding = vec![i as f32; embedding_size];
           cache.put(key, embedding).await.unwrap();
       }
       
       let cache_info = cache.get_cache_info().await;
       let expected_memory_mb = (num_embeddings * embedding_size * 4) / (1024 * 1024); // Convert to MB
       
       println!("Estimated memory: {}MB, Expected: {}MB", cache_info.memory_usage_mb, expected_memory_mb);
       
       // Allow some overhead for HashMap and metadata
       assert!(
           cache_info.memory_usage_mb >= expected_memory_mb &&
           cache_info.memory_usage_mb <= expected_memory_mb * 2,
           "Memory usage estimation is significantly off: {} vs expected {}",
           cache_info.memory_usage_mb,
           expected_memory_mb
       );
   }
   
   #[tokio::test]
   async fn test_performance_regression() {
       let cache = EmbeddingCache::new(10000, 3600, 200);
       let embedding = vec![0.5; 768];
       
       // Warm up cache
       for i in 0..1000 {
           let key = create_test_key(i);
           cache.put(key, embedding.clone()).await.unwrap();
       }
       
       // Test put performance
       let start = std::time::Instant::now();
       for i in 1000..2000 {
           let key = create_test_key(i);
           cache.put(key, embedding.clone()).await.unwrap();
       }
       let put_duration = start.elapsed();
       let put_per_sec = 1000.0 / put_duration.as_secs_f64();
       
       println!("Put performance: {:.0} ops/sec", put_per_sec);
       assert!(put_per_sec > 1000.0, "Put performance below 1000 ops/sec: {}", put_per_sec);
       
       // Test get performance
       let start = std::time::Instant::now();
       for i in 0..1000 {
           let key = create_test_key(i);
           let result = cache.get(&key).await;
           assert!(result.is_some());
       }
       let get_duration = start.elapsed();
       let get_per_sec = 1000.0 / get_duration.as_secs_f64();
       
       println!("Get performance: {:.0} ops/sec", get_per_sec);
       assert!(get_per_sec > 5000.0, "Get performance below 5000 ops/sec: {}", get_per_sec);
   }
   ```

3. **Add scalability tests** (2 min)
   ```rust
   #[tokio::test]
   async fn test_large_dataset_scalability() {
       let cache = EmbeddingCache::new(50000, 3600, 1000); // Large cache
       let embedding = vec![0.1; 768];
       
       // Test scaling to 10k entries
       let start = std::time::Instant::now();
       for i in 0..10000 {
           let key = create_test_key(i);
           cache.put(key, embedding.clone()).await.unwrap();
           
           // Progress update
           if i % 1000 == 0 {
               println!("Inserted {} entries", i);
           }
       }
       let insert_time = start.elapsed();
       
       println!("Inserted 10k entries in {:?}", insert_time);
       assert!(insert_time.as_secs() < 30, "Insert time too slow: {:?}", insert_time);
       
       // Test search performance on large dataset
       let start = std::time::Instant::now();
       for i in 0..1000 {
           let key = create_test_key(i * 10); // Spread out access pattern
           let result = cache.get(&key).await;
           assert!(result.is_some());
       }
       let search_time = start.elapsed();
       
       println!("Searched 1k entries in {:?}", search_time);
       assert!(search_time.as_millis() < 500, "Search time too slow: {:?}", search_time);
       
       // Verify cache stats
       let info = cache.get_cache_info().await;
       println!("Final cache info: {:?}", info);
       assert_eq!(info.size, 10000);
       assert!(info.hit_rate > 0.9); // Should be high hit rate for this test
   }
   ```

## Success Criteria
- [ ] Put operations: >1000 ops/sec
- [ ] Get operations: >5000 ops/sec
- [ ] Memory estimation accuracy: ±50%
- [ ] Concurrent access scales linearly
- [ ] Large dataset handling: <30s for 10k entries
- [ ] Memory usage stays within limits

## Files to Create
- `src/ml/tests/cache_benchmarks.rs`

## Files to Modify
- `Cargo.toml` (add criterion for benchmarks)

## Benchmark Dependencies
```toml
[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }
```

## Running Benchmarks
```bash
# Run all cache benchmarks
cargo bench --bench cache_benchmarks

# Run performance regression tests
cargo test test_performance_regression --release

# Run scalability tests
cargo test test_large_dataset_scalability --release
```

## Next Task
→ Task 029: Implement tokenization for text input