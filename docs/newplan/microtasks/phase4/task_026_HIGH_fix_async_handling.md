# Task 026 - HIGH: Fix Async Handling in Cache Operations

## Priority: HIGH
## Estimated Time: 10 minutes
## Dependencies: Task 025

## Objective
Resolve async/await issues in cache operations and ensure proper concurrent access patterns.

## Current Issue
- Deadlock potential with read/write locks
- Async operations not properly handled
- Race conditions in cache operations

## Tasks
1. **Fix async lock handling** (4 min)
   ```rust
   // Update src/ml/embedding_cache.rs
   use tokio::sync::{RwLock, Mutex};
   use std::sync::Arc;
   
   pub struct EmbeddingCache {
       cache: Arc<RwLock<HashMap<CacheKey, CachedEmbedding>>>,
       max_size: usize,
       ttl_seconds: u64,
       stats: Arc<CacheStats>,
       memory_limit_mb: usize,
       cleanup_lock: Arc<Mutex<()>>, // Prevent concurrent cleanup
   }
   
   impl EmbeddingCache {
       pub fn new(max_size: usize, ttl_seconds: u64, memory_limit_mb: usize) -> Self {
           Self {
               cache: Arc::new(RwLock::new(HashMap::new())),
               max_size,
               ttl_seconds,
               stats: Arc::new(CacheStats::default()),
               memory_limit_mb,
               cleanup_lock: Arc::new(Mutex::new(())),
           }
       }
       
       pub async fn get(&self, key: &CacheKey) -> Option<EmbeddingVector> {
           // First, try a read lock to check if the key exists and is valid
           {
               let cache_read = self.cache.read().await;
               
               if let Some(cached) = cache_read.get(key) {
                   let now = chrono::Utc::now();
                   let age = now.signed_duration_since(cached.timestamp).num_seconds() as u64;
                   
                   if age <= self.ttl_seconds {
                       self.stats.hits.fetch_add(1, Ordering::Relaxed);
                       return Some(cached.embedding.clone());
                   }
                   // Entry is expired, need to remove it
               }
           }
           
           // If we reach here, either key doesn't exist or is expired
           // Use write lock to remove expired entry if it exists
           {
               let mut cache_write = self.cache.write().await;
               
               if let Some(cached) = cache_write.get(key) {
                   let now = chrono::Utc::now();
                   let age = now.signed_duration_since(cached.timestamp).num_seconds() as u64;
                   
                   if age <= self.ttl_seconds {
                       // Entry is still valid (race condition avoided)
                       self.stats.hits.fetch_add(1, Ordering::Relaxed);
                       return Some(cached.embedding.clone());
                   } else {
                       // Remove expired entry
                       cache_write.remove(key);
                       self.stats.expired_entries.fetch_add(1, Ordering::Relaxed);
                   }
               }
           }
           
           self.stats.misses.fetch_add(1, Ordering::Relaxed);
           None
       }
   ```

2. **Fix concurrent put operations** (4 min)
   ```rust
   impl EmbeddingCache {
       pub async fn put(&self, key: CacheKey, embedding: EmbeddingVector) -> Result<()> {
           let _cleanup_guard = self.cleanup_lock.lock().await;
           let mut cache = self.cache.write().await;
           
           // Check if we need to evict before inserting
           let needs_eviction = self.should_evict(&cache, &embedding).await;
           
           if needs_eviction {
               self.evict_entries(&mut cache).await;
           }
           
           // Insert or update the entry
           let cached_embedding = CachedEmbedding {
               embedding,
               timestamp: chrono::Utc::now(),
               hit_count: 0,
           };
           
           cache.insert(key, cached_embedding);
           Ok(())
       }
       
       async fn should_evict(
           &self,
           cache: &HashMap<CacheKey, CachedEmbedding>,
           new_embedding: &EmbeddingVector,
       ) -> bool {
           // Check size limit
           if cache.len() >= self.max_size {
               return true;
           }
           
           // Check memory limit
           let estimated_memory_mb = self.estimate_memory_usage(cache, new_embedding);
           if estimated_memory_mb > self.memory_limit_mb {
               return true;
           }
           
           false
       }
       
       async fn evict_entries(&self, cache: &mut HashMap<CacheKey, CachedEmbedding>) {
           // Remove expired entries first
           let now = chrono::Utc::now();
           let expired_keys: Vec<_> = cache
               .iter()
               .filter_map(|(key, cached)| {
                   let age = now.signed_duration_since(cached.timestamp).num_seconds() as u64;
                   if age > self.ttl_seconds {
                       Some(key.clone())
                   } else {
                       None
                   }
               })
               .collect();
           
           for key in expired_keys {
               cache.remove(&key);
               self.stats.expired_entries.fetch_add(1, Ordering::Relaxed);
           }
           
           // If still over limits, use LRU eviction
           while cache.len() >= self.max_size {
               self.evict_lru_single(cache).await;
           }
           
           // If still over memory limit, evict by size
           while self.estimate_memory_usage(cache, &vec![]).await > self.memory_limit_mb {
               self.evict_by_size_single(cache).await;
           }
       }
       
       async fn evict_lru_single(&self, cache: &mut HashMap<CacheKey, CachedEmbedding>) {
           let lru_key = cache
               .iter()
               .min_by_key(|(_, cached)| cached.timestamp)
               .map(|(key, _)| key.clone());
           
           if let Some(key) = lru_key {
               cache.remove(&key);
               self.stats.evictions.fetch_add(1, Ordering::Relaxed);
           }
       }
       
       async fn evict_by_size_single(&self, cache: &mut HashMap<CacheKey, CachedEmbedding>) {
           let largest_key = cache
               .iter()
               .max_by_key(|(_, cached)| cached.embedding.len())
               .map(|(key, _)| key.clone());
           
           if let Some(key) = largest_key {
               cache.remove(&key);
               self.stats.evictions.fetch_add(1, Ordering::Relaxed);
           }
       }
   }
   ```

3. **Add timeout handling** (2 min)
   ```rust
   impl EmbeddingCache {
       pub async fn get_with_timeout(
           &self,
           key: &CacheKey,
           timeout_ms: u64,
       ) -> Result<Option<EmbeddingVector>> {
           let timeout = tokio::time::Duration::from_millis(timeout_ms);
           
           match tokio::time::timeout(timeout, self.get(key)).await {
               Ok(result) => Ok(result),
               Err(_) => Err(anyhow::anyhow!("Cache get operation timed out after {}ms", timeout_ms)),
           }
       }
       
       pub async fn put_with_timeout(
           &self,
           key: CacheKey,
           embedding: EmbeddingVector,
           timeout_ms: u64,
       ) -> Result<()> {
           let timeout = tokio::time::Duration::from_millis(timeout_ms);
           
           match tokio::time::timeout(timeout, self.put(key, embedding)).await {
               Ok(result) => result,
               Err(_) => Err(anyhow::anyhow!("Cache put operation timed out after {}ms", timeout_ms)),
           }
       }
       
       pub async fn cleanup_expired_safe(&self) {
           // Non-blocking cleanup that won't interfere with normal operations
           let _cleanup_guard = match self.cleanup_lock.try_lock() {
               Ok(guard) => guard,
               Err(_) => {
                   // Cleanup already in progress, skip
                   return;
               }
           };
           
           let mut cache = self.cache.write().await;
           let now = chrono::Utc::now();
           
           cache.retain(|_key, cached| {
               let age = now.signed_duration_since(cached.timestamp).num_seconds() as u64;
               let is_valid = age <= self.ttl_seconds;
               
               if !is_valid {
                   self.stats.expired_entries.fetch_add(1, Ordering::Relaxed);
               }
               
               is_valid
           });
       }
   }
   ```

## Success Criteria
- [ ] No deadlocks in concurrent access
- [ ] Async operations complete properly
- [ ] Race conditions eliminated
- [ ] Timeout handling works
- [ ] Lock contention minimized

## Files to Modify
- `src/ml/embedding_cache.rs`

## Key Improvements
- Separate read/write lock phases
- Cleanup lock prevents concurrent eviction
- Timeout protection for operations
- Non-blocking cleanup option
- Better error handling

## Validation
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    
    #[tokio::test]
    async fn test_concurrent_access() {
        let cache = Arc::new(EmbeddingCache::new(100, 3600, 100));
        let mut handles = vec![];
        
        // Spawn multiple concurrent operations
        for i in 0..10 {
            let cache_clone = Arc::clone(&cache);
            let handle = tokio::spawn(async move {
                let key = CacheKey {
                    content_hash: format!("test_{}", i),
                    model_version: "v1".to_string(),
                    chunk_id: None,
                };
                
                let embedding = vec![i as f32; 768];
                
                // Concurrent put and get operations
                cache_clone.put(key.clone(), embedding).await.unwrap();
                let result = cache_clone.get(&key).await;
                assert!(result.is_some());
            });
            
            handles.push(handle);
        }
        
        // Wait for all operations to complete
        for handle in handles {
            handle.await.unwrap();
        }
        
        assert_eq!(cache.size().await, 10);
    }
}
```

## Next Task
→ Task 027: Test cache hits and misses validation