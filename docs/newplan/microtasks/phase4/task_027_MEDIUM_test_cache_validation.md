# Task 027 - MEDIUM: Test Cache Hits and Misses Validation

## Priority: MEDIUM
## Estimated Time: 10 minutes
## Dependencies: Task 026

## Objective
Create comprehensive tests to validate cache hit/miss behavior and performance characteristics.

## Current Issue
- Need to verify cache behavior is correct
- Performance validation required
- Edge cases need testing

## Tasks
1. **Create cache behavior tests** (5 min)
   ```rust
   // In src/ml/tests/cache_validation.rs
   use super::*;
   use crate::ml::embedding_cache::*;
   use std::time::Duration;
   use tokio::time::sleep;
   
   #[tokio::test]
   async fn test_basic_cache_operations() {
       let cache = EmbeddingCache::new(100, 3600, 50); // 50MB limit
       
       let key = CacheKey {
           content_hash: "test_content_hash".to_string(),
           model_version: "nomic-embed-text-v1.5".to_string(),
           chunk_id: Some("chunk_1".to_string()),
       };
       
       let embedding = vec![0.1, 0.2, 0.3, 0.4, 0.5]; // Small test embedding
       
       // Test initial miss
       let result = cache.get(&key).await;
       assert!(result.is_none(), "Expected cache miss on first access");
       assert_eq!(cache.stats().misses.load(std::sync::atomic::Ordering::Relaxed), 1);
       assert_eq!(cache.stats().hits.load(std::sync::atomic::Ordering::Relaxed), 0);
       
       // Test put operation
       let put_result = cache.put(key.clone(), embedding.clone()).await;
       assert!(put_result.is_ok(), "Put operation should succeed");
       
       // Test cache hit
       let result = cache.get(&key).await;
       assert!(result.is_some(), "Expected cache hit after put");
       assert_eq!(result.unwrap(), embedding, "Retrieved embedding should match original");
       assert_eq!(cache.stats().hits.load(std::sync::atomic::Ordering::Relaxed), 1);
       
       // Verify hit rate calculation
       let hit_rate = cache.stats().hit_rate();
       assert!((hit_rate - 0.5).abs() < 0.01, "Hit rate should be 50% (1 hit, 1 miss)");
   }
   
   #[tokio::test]
   async fn test_cache_expiry() {
       let cache = EmbeddingCache::new(100, 2, 50); // 2 second TTL
       
       let key = CacheKey {
           content_hash: "expiry_test".to_string(),
           model_version: "v1".to_string(),
           chunk_id: None,
       };
       
       let embedding = vec![1.0; 10];
       
       // Add to cache
       cache.put(key.clone(), embedding.clone()).await.unwrap();
       
       // Verify it's there
       assert!(cache.get(&key).await.is_some());
       
       // Wait for expiry
       sleep(Duration::from_secs(3)).await;
       
       // Should be expired now
       let result = cache.get(&key).await;
       assert!(result.is_none(), "Entry should have expired");
       
       // Check that expired entries counter increased
       let expired_count = cache.stats().expired_entries.load(std::sync::atomic::Ordering::Relaxed);
       assert!(expired_count > 0, "Expired entries counter should be > 0");
   }
   ```

2. **Test cache eviction policies** (3 min)
   ```rust
   #[tokio::test]
   async fn test_lru_eviction() {
       let cache = EmbeddingCache::new(3, 3600, 100); // Only 3 entries max
       
       // Fill cache to capacity
       for i in 0..3 {
           let key = CacheKey {
               content_hash: format!("item_{}", i),
               model_version: "v1".to_string(),
               chunk_id: None,
           };
           let embedding = vec![i as f32; 10];
           cache.put(key, embedding).await.unwrap();
       }
       
       assert_eq!(cache.size().await, 3);
       
       // Access item_1 to make it more recently used
       let key1 = CacheKey {
           content_hash: "item_1".to_string(),
           model_version: "v1".to_string(),
           chunk_id: None,
       };
       cache.get(&key1).await;
       
       // Add one more item, should evict item_0 (least recently used)
       let new_key = CacheKey {
           content_hash: "item_new".to_string(),
           model_version: "v1".to_string(),
           chunk_id: None,
       };
       cache.put(new_key.clone(), vec![99.0; 10]).await.unwrap();
       
       // Cache should still have 3 items
       assert_eq!(cache.size().await, 3);
       
       // item_0 should be evicted, item_1 should still be there
       let key0 = CacheKey {
           content_hash: "item_0".to_string(),
           model_version: "v1".to_string(),
           chunk_id: None,
       };
       assert!(cache.get(&key0).await.is_none(), "item_0 should be evicted");
       assert!(cache.get(&key1).await.is_some(), "item_1 should still be cached");
       assert!(cache.get(&new_key).await.is_some(), "new item should be cached");
   }
   
   #[tokio::test]
   async fn test_memory_based_eviction() {
       // Create cache with very small memory limit
       let cache = EmbeddingCache::new(100, 3600, 1); // 1MB limit
       
       // Create large embedding that exceeds memory limit
       let large_embedding = vec![1.0; 100000]; // ~400KB embedding
       
       // Add multiple large embeddings
       for i in 0..5 {
           let key = CacheKey {
               content_hash: format!("large_item_{}", i),
               model_version: "v1".to_string(),
               chunk_id: None,
           };
           
           cache.put(key, large_embedding.clone()).await.unwrap();
       }
       
       // Should have triggered memory-based eviction
       let final_size = cache.size().await;
       assert!(final_size < 5, "Memory eviction should have removed some entries");
       
       let evictions = cache.stats().evictions.load(std::sync::atomic::Ordering::Relaxed);
       assert!(evictions > 0, "Should have recorded evictions");
   }
   ```

3. **Performance and stress tests** (2 min)
   ```rust
   #[tokio::test]
   async fn test_cache_performance() {
       let cache = EmbeddingCache::new(1000, 3600, 100);
       
       // Pre-fill cache with test data
       let test_keys: Vec<_> = (0..500)
           .map(|i| CacheKey {
               content_hash: format!("perf_test_{}", i),
               model_version: "v1".to_string(),
               chunk_id: None,
           })
           .collect();
       
       let test_embedding = vec![0.5; 768];
       
       // Fill cache
       let start = std::time::Instant::now();
       for key in &test_keys {
           cache.put(key.clone(), test_embedding.clone()).await.unwrap();
       }
       let fill_time = start.elapsed();
       println!("Cache fill (500 items) took: {:?}", fill_time);
       
       // Test read performance
       let start = std::time::Instant::now();
       for key in &test_keys {
           let result = cache.get(key).await;
           assert!(result.is_some());
       }
       let read_time = start.elapsed();
       println!("Cache reads (500 items) took: {:?}", read_time);
       
       // Performance assertions
       assert!(fill_time.as_millis() < 1000, "Fill time should be < 1 second");
       assert!(read_time.as_millis() < 100, "Read time should be < 100ms");
       
       // Verify hit rate
       let hit_rate = cache.stats().hit_rate();
       assert!(hit_rate > 0.99, "Hit rate should be > 99% for this test");
   }
   
   #[tokio::test]
   async fn test_concurrent_cache_access() {
       use std::sync::Arc;
       use tokio::task::JoinSet;
       
       let cache = Arc::new(EmbeddingCache::new(1000, 3600, 100));
       let mut join_set = JoinSet::new();
       
       // Spawn multiple concurrent tasks
       for task_id in 0..10 {
           let cache_clone = Arc::clone(&cache);
           
           join_set.spawn(async move {
               for i in 0..50 {
                   let key = CacheKey {
                       content_hash: format!("concurrent_{}_{}", task_id, i),
                       model_version: "v1".to_string(),
                       chunk_id: None,
                   };
                   
                   let embedding = vec![task_id as f32; 100];
                   
                   // Put and immediately try to get
                   cache_clone.put(key.clone(), embedding.clone()).await.unwrap();
                   let result = cache_clone.get(&key).await;
                   assert!(result.is_some());
                   assert_eq!(result.unwrap(), embedding);
               }
           });
       }
       
       // Wait for all tasks to complete
       while let Some(result) = join_set.join_next().await {
           result.unwrap(); // Panic if any task failed
       }
       
       // Verify final state
       assert_eq!(cache.size().await, 500); // 10 tasks * 50 items each
       let total_requests = cache.stats().total_requests();
       assert_eq!(total_requests, 1000); // 500 puts (misses) + 500 gets (hits)
   }
   ```

## Success Criteria
- [ ] Cache hit/miss behavior correct
- [ ] TTL expiry works properly
- [ ] LRU eviction functions correctly
- [ ] Memory-based eviction works
- [ ] Performance meets targets
- [ ] Concurrent access is safe

## Files to Create
- `src/ml/tests/cache_validation.rs`
- `src/ml/tests/mod.rs`

## Performance Targets
- Cache fill: <2ms per item
- Cache read: <0.2ms per item
- Hit rate: >95% for repeated queries
- Memory usage: Within configured limits
- Concurrent safety: No race conditions

## Next Task
→ Task 028: Verify cache performance meets requirements