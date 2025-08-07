# Task 025 - MEDIUM: Update Cache Implementation for Better Performance

## Priority: MEDIUM
## Estimated Time: 10 minutes
## Dependencies: Task 024

## Objective
Improve cache implementation with better memory management, statistics, and concurrent access.

## Current Issue
- Cache performance can be improved
- No cache statistics or monitoring
- Memory usage not optimized

## Tasks
1. **Add cache statistics** (4 min)
   ```rust
   // Update src/ml/embedding_cache.rs
   use std::sync::atomic::{AtomicU64, Ordering};
   
   #[derive(Debug, Default)]
   pub struct CacheStats {
       pub hits: AtomicU64,
       pub misses: AtomicU64,
       pub evictions: AtomicU64,
       pub expired_entries: AtomicU64,
   }
   
   impl CacheStats {
       pub fn hit_rate(&self) -> f64 {
           let hits = self.hits.load(Ordering::Relaxed);
           let misses = self.misses.load(Ordering::Relaxed);
           let total = hits + misses;
           
           if total == 0 {
               0.0
           } else {
               hits as f64 / total as f64
           }
       }
       
       pub fn total_requests(&self) -> u64 {
           self.hits.load(Ordering::Relaxed) + self.misses.load(Ordering::Relaxed)
       }
   }
   
   // Update EmbeddingCache
   pub struct EmbeddingCache {
       cache: Arc<RwLock<HashMap<CacheKey, CachedEmbedding>>>,
       max_size: usize,
       ttl_seconds: u64,
       stats: Arc<CacheStats>,
       memory_limit_mb: usize,
   }
   
   impl EmbeddingCache {
       pub fn new(max_size: usize, ttl_seconds: u64, memory_limit_mb: usize) -> Self {
           Self {
               cache: Arc::new(RwLock::new(HashMap::new())),
               max_size,
               ttl_seconds,
               stats: Arc::new(CacheStats::default()),
               memory_limit_mb,
           }
       }
       
       pub fn stats(&self) -> &CacheStats {
           &self.stats
       }
   }
   ```

2. **Optimize memory usage** (4 min)
   ```rust
   impl EmbeddingCache {
       pub async fn get(&self, key: &CacheKey) -> Option<EmbeddingVector> {
           let mut cache = self.cache.write().await;
           
           if let Some(cached) = cache.get_mut(key) {
               let now = chrono::Utc::now();
               let age = now.signed_duration_since(cached.timestamp).num_seconds() as u64;
               
               if age <= self.ttl_seconds {
                   cached.hit_count += 1;
                   cached.timestamp = now; // Update access time for LRU
                   self.stats.hits.fetch_add(1, Ordering::Relaxed);
                   return Some(cached.embedding.clone());
               } else {
                   cache.remove(key);
                   self.stats.expired_entries.fetch_add(1, Ordering::Relaxed);
               }
           }
           
           self.stats.misses.fetch_add(1, Ordering::Relaxed);
           None
       }
       
       pub async fn put(&self, key: CacheKey, embedding: EmbeddingVector) -> Result<()> {
           let mut cache = self.cache.write().await;
           
           // Check memory usage before inserting
           let estimated_memory_mb = self.estimate_memory_usage(&cache, &embedding);
           if estimated_memory_mb > self.memory_limit_mb {
               self.evict_by_memory(&mut cache).await;
           }
           
           // Evict by size if needed
           if cache.len() >= self.max_size {
               self.evict_lru(&mut cache).await;
           }
           
           let cached_embedding = CachedEmbedding {
               embedding,
               timestamp: chrono::Utc::now(),
               hit_count: 0,
           };
           
           cache.insert(key, cached_embedding);
           Ok(())
       }
       
       fn estimate_memory_usage(
           &self,
           cache: &HashMap<CacheKey, CachedEmbedding>,
           new_embedding: &EmbeddingVector,
       ) -> usize {
           let base_memory = cache.len() * (std::mem::size_of::<CacheKey>() + std::mem::size_of::<CachedEmbedding>());
           let embeddings_memory = cache.values()
               .map(|cached| cached.embedding.len() * std::mem::size_of::<f32>())
               .sum::<usize>();
           let new_embedding_memory = new_embedding.len() * std::mem::size_of::<f32>();
           
           (base_memory + embeddings_memory + new_embedding_memory) / (1024 * 1024) // Convert to MB
       }
       
       async fn evict_by_memory(&self, cache: &mut HashMap<CacheKey, CachedEmbedding>) {
           // Remove largest embeddings first
           let mut entries: Vec<_> = cache.iter().collect();
           entries.sort_by_key(|(_, cached)| std::cmp::Reverse(cached.embedding.len()));
           
           // Remove 25% of entries, starting with largest
           let remove_count = (cache.len() / 4).max(1);
           let keys_to_remove: Vec<_> = entries.iter()
               .take(remove_count)
               .map(|(key, _)| (*key).clone())
               .collect();
           
           for key in keys_to_remove {
               cache.remove(&key);
               self.stats.evictions.fetch_add(1, Ordering::Relaxed);
           }
       }
       
       async fn evict_lru(&self, cache: &mut HashMap<CacheKey, CachedEmbedding>) {
           // Find least recently used entry (oldest timestamp)
           let mut lru_key = None;
           let mut lru_time = chrono::Utc::now();
           
           for (key, cached) in cache.iter() {
               if cached.timestamp < lru_time {
                   lru_time = cached.timestamp;
                   lru_key = Some(key.clone());
               }
           }
           
           if let Some(key) = lru_key {
               cache.remove(&key);
               self.stats.evictions.fetch_add(1, Ordering::Relaxed);
           }
       }
   }
   ```

3. **Add cache monitoring** (2 min)
   ```rust
   impl EmbeddingCache {
       pub async fn get_cache_info(&self) -> CacheInfo {
           let cache = self.cache.read().await;
           let estimated_memory_mb = self.estimate_memory_usage(&cache, &vec![]);
           
           CacheInfo {
               size: cache.len(),
               max_size: self.max_size,
               memory_usage_mb: estimated_memory_mb,
               memory_limit_mb: self.memory_limit_mb,
               hit_rate: self.stats.hit_rate(),
               total_requests: self.stats.total_requests(),
               evictions: self.stats.evictions.load(Ordering::Relaxed),
               expired_entries: self.stats.expired_entries.load(Ordering::Relaxed),
           }
       }
       
       pub async fn periodic_cleanup(&self) {
           // Run cleanup every 5 minutes
           let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(300));
           
           loop {
               interval.tick().await;
               self.cleanup_expired().await;
               
               let info = self.get_cache_info().await;
               println!("Cache stats: {} entries, {:.1}MB memory, {:.2}% hit rate",
                   info.size, info.memory_usage_mb, info.hit_rate * 100.0);
           }
       }
   }
   
   #[derive(Debug)]
   pub struct CacheInfo {
       pub size: usize,
       pub max_size: usize,
       pub memory_usage_mb: usize,
       pub memory_limit_mb: usize,
       pub hit_rate: f64,
       pub total_requests: u64,
       pub evictions: u64,
       pub expired_entries: u64,
   }
   ```

## Success Criteria
- [ ] Cache statistics work correctly
- [ ] Memory usage is optimized
- [ ] LRU eviction improves performance
- [ ] Monitoring provides useful metrics
- [ ] Memory limits are respected

## Files to Modify
- `src/ml/embedding_cache.rs`

## Performance Improvements
- Memory usage tracking and limits
- Smarter eviction strategies
- Access time updates for better LRU
- Automatic cleanup scheduling
- Detailed performance statistics

## Validation
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_cache_stats() {
        let cache = EmbeddingCache::new(100, 3600, 100);
        
        let key1 = CacheKey {
            content_hash: "test1".to_string(),
            model_version: "v1".to_string(),
            chunk_id: None,
        };
        
        // Test miss
        assert!(cache.get(&key1).await.is_none());
        assert_eq!(cache.stats().total_requests(), 1);
        assert_eq!(cache.stats().hit_rate(), 0.0);
        
        // Test hit
        cache.put(key1.clone(), vec![1.0]).await.unwrap();
        assert!(cache.get(&key1).await.is_some());
        assert!(cache.stats().hit_rate() > 0.0);
    }
}
```

## Next Task
→ Task 026: Fix async handling in cache operations