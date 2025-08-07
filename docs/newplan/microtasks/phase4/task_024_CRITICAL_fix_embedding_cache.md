# Task 024 - CRITICAL: Fix Embedding Cache Type Mismatches

## Priority: CRITICAL
## Estimated Time: 10 minutes
## Dependencies: Task 023

## Objective
Resolve type mismatches in the embedding cache system causing compilation errors.

## Current Issue
- Type conflicts between cache key types
- Async handling in cache operations
- HashMap vs LruCache inconsistencies

## Tasks
1. **Fix cache key types** (4 min)
   ```rust
   // In src/ml/embedding_cache.rs
   use std::collections::HashMap;
   use std::sync::Arc;
   use tokio::sync::RwLock;
   use anyhow::Result;
   use crate::types::EmbeddingVector;
   
   #[derive(Debug, Clone)]
   pub struct CacheKey {
       pub content_hash: String,
       pub model_version: String,
       pub chunk_id: Option<String>,
   }
   
   impl std::fmt::Display for CacheKey {
       fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
           write!(f, "{}:{}:{}", 
               self.content_hash, 
               self.model_version,
               self.chunk_id.as_deref().unwrap_or("default")
           )
       }
   }
   
   impl std::hash::Hash for CacheKey {
       fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
           self.content_hash.hash(state);
           self.model_version.hash(state);
           self.chunk_id.hash(state);
       }
   }
   
   impl PartialEq for CacheKey {
       fn eq(&self, other: &Self) -> bool {
           self.content_hash == other.content_hash &&
           self.model_version == other.model_version &&
           self.chunk_id == other.chunk_id
       }
   }
   
   impl Eq for CacheKey {}
   ```

2. **Implement type-safe cache** (5 min)
   ```rust
   #[derive(Debug, Clone)]
   pub struct CachedEmbedding {
       pub embedding: EmbeddingVector,
       pub timestamp: chrono::DateTime<chrono::Utc>,
       pub hit_count: u32,
   }
   
   pub struct EmbeddingCache {
       cache: Arc<RwLock<HashMap<CacheKey, CachedEmbedding>>>,
       max_size: usize,
       ttl_seconds: u64,
   }
   
   impl EmbeddingCache {
       pub fn new(max_size: usize, ttl_seconds: u64) -> Self {
           Self {
               cache: Arc::new(RwLock::new(HashMap::new())),
               max_size,
               ttl_seconds,
           }
       }
       
       pub async fn get(&self, key: &CacheKey) -> Option<EmbeddingVector> {
           let mut cache = self.cache.write().await;
           
           if let Some(cached) = cache.get_mut(key) {
               // Check if entry is still valid
               let now = chrono::Utc::now();
               let age = now.signed_duration_since(cached.timestamp).num_seconds() as u64;
               
               if age <= self.ttl_seconds {
                   cached.hit_count += 1;
                   return Some(cached.embedding.clone());
               } else {
                   // Remove expired entry
                   cache.remove(key);
               }
           }
           
           None
       }
       
       pub async fn put(&self, key: CacheKey, embedding: EmbeddingVector) -> Result<()> {
           let mut cache = self.cache.write().await;
           
           // Evict old entries if cache is full
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
       
       async fn evict_lru(&self, cache: &mut HashMap<CacheKey, CachedEmbedding>) {
           // Find oldest entry
           let mut oldest_key = None;
           let mut oldest_time = chrono::Utc::now();
           
           for (key, cached) in cache.iter() {
               if cached.timestamp < oldest_time {
                   oldest_time = cached.timestamp;
                   oldest_key = Some(key.clone());
               }
           }
           
           if let Some(key) = oldest_key {
               cache.remove(&key);
           }
       }
       
       pub async fn clear(&self) {
           let mut cache = self.cache.write().await;
           cache.clear();
       }
       
       pub async fn size(&self) -> usize {
           let cache = self.cache.read().await;
           cache.len()
       }
       
       pub async fn cleanup_expired(&self) {
           let mut cache = self.cache.write().await;
           let now = chrono::Utc::now();
           
           cache.retain(|_key, cached| {
               let age = now.signed_duration_since(cached.timestamp).num_seconds() as u64;
               age <= self.ttl_seconds
           });
       }
   }
   ```

3. **Fix async integration** (1 min)
   ```rust
   // Update embedding service integration
   impl EmbeddingService {
       pub async fn get_embedding_with_cache(
           &self,
           text: &str,
           chunk_id: Option<String>,
       ) -> Result<EmbeddingVector> {
           // Create cache key
           let content_hash = self.hash_content(text)?;
           let cache_key = CacheKey {
               content_hash,
               model_version: "nomic-embed-text-v1.5".to_string(),
               chunk_id,
           };
           
           // Check cache first
           if let Some(cached_embedding) = self.cache.get(&cache_key).await {
               return Ok(cached_embedding);
           }
           
           // Generate new embedding
           let embedding = self.generate_embedding(text).await?;
           
           // Cache the result
           self.cache.put(cache_key, embedding.clone()).await?;
           
           Ok(embedding)
       }
       
       fn hash_content(&self, text: &str) -> Result<String> {
           use sha2::{Sha256, Digest};
           
           let mut hasher = Sha256::new();
           hasher.update(text.as_bytes());
           let result = hasher.finalize();
           Ok(format!("{:x}", result))
       }
   }
   ```

## Success Criteria
- [ ] Cache types compile without errors
- [ ] Async operations work correctly
- [ ] Cache key hashing is consistent
- [ ] LRU eviction functions properly
- [ ] TTL cleanup works

## Files to Create
- `src/ml/embedding_cache.rs`

## Files to Modify
- `src/ml/embedding_service.rs`
- `src/ml/mod.rs`
- `Cargo.toml` (add sha2 dependency)

## Dependencies to Add
```toml
[dependencies]
sha2 = "0.10"
```

## Validation
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_cache_operations() {
        let cache = EmbeddingCache::new(100, 3600);
        
        let key = CacheKey {
            content_hash: "test_hash".to_string(),
            model_version: "v1".to_string(),
            chunk_id: None,
        };
        
        let embedding = vec![0.1, 0.2, 0.3];
        
        // Test put and get
        cache.put(key.clone(), embedding.clone()).await.unwrap();
        let retrieved = cache.get(&key).await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), embedding);
    }
    
    #[tokio::test]
    async fn test_cache_expiry() {
        let cache = EmbeddingCache::new(100, 1); // 1 second TTL
        
        let key = CacheKey {
            content_hash: "expire_test".to_string(),
            model_version: "v1".to_string(),
            chunk_id: None,
        };
        
        cache.put(key.clone(), vec![1.0]).await.unwrap();
        
        // Wait for expiry
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        
        let retrieved = cache.get(&key).await;
        assert!(retrieved.is_none());
    }
}
```

## Next Task
→ Task 025: Update cache implementation for better performance