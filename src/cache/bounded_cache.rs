// Bounded Cache Implementation - Phase 1: Foundation & Safety
// This module provides memory-safe caching with LRU eviction

use std::sync::Arc;
use std::hash::Hash;
use std::time::{Duration, Instant};
use lru::LruCache;
use parking_lot::RwLock;
use std::num::NonZeroUsize;

use crate::error::{EmbedError, Result};

/// Statistics for cache monitoring
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub insertions: u64,
    pub current_size: usize,
    pub max_size: usize,
}

impl CacheStats {
    pub fn new() -> Self {
        Self {
            hits: 0,
            misses: 0,
            evictions: 0,
            insertions: 0,
            current_size: 0,
            max_size: 0,
        }
    }

    /// Calculate hit rate as a percentage
    /// Returns None if no cache operations have occurred (undefined mathematical state)
    pub fn hit_rate(&self) -> Option<f64> {
        let total = self.hits + self.misses;
        if total == 0 {
            None // Undefined: cannot calculate hit rate with no data
        } else {
            Some((self.hits as f64 / total as f64) * 100.0)
        }
    }
}

/// Thread-safe bounded cache with LRU eviction
pub struct BoundedCache<K, V> 
where 
    K: Hash + Eq + Clone,
    V: Clone,
{
    inner: Arc<RwLock<LruCache<K, CacheEntry<V>>>>,
    stats: Arc<RwLock<CacheStats>>,
    #[allow(dead_code)]
    max_size: NonZeroUsize,
    ttl: Option<Duration>,
}

/// Cache entry with optional TTL
#[derive(Clone)]
struct CacheEntry<V> {
    value: V,
    inserted_at: Instant,
    access_count: u64,
}

impl<K, V> BoundedCache<K, V> 
where 
    K: Hash + Eq + Clone,
    V: Clone,
{
    /// Create a new bounded cache with specified capacity
    pub fn new(capacity: usize) -> Result<Self> {
        let capacity = NonZeroUsize::new(capacity)
            .ok_or_else(|| EmbedError::Configuration {
                message: "Cache capacity must be greater than 0".to_string(),
                source: None,
            })?;
        
        Ok(Self {
            inner: Arc::new(RwLock::new(LruCache::new(capacity))),
            stats: Arc::new(RwLock::new(CacheStats {
                hits: 0,
                misses: 0,
                evictions: 0,
                insertions: 0,
                current_size: 0,
                max_size: capacity.get(),
            })),
            max_size: capacity,
            ttl: None,
        })
    }
    
    /// Create a cache with TTL (time-to-live) for entries
    pub fn with_ttl(capacity: usize, ttl: Duration) -> Result<Self> {
        let mut cache = Self::new(capacity)?;
        cache.ttl = Some(ttl);
        Ok(cache)
    }
    
    /// Get a value from the cache
    pub fn get(&self, key: &K) -> Option<V> {
        let mut cache = self.inner.write();
        let mut stats = self.stats.write();
        
        // Try to get the entry
        if let Some(entry) = cache.get_mut(key) {
            // Check TTL if configured
            if let Some(ttl) = self.ttl {
                if entry.inserted_at.elapsed() > ttl {
                    // Entry expired, remove it
                    cache.pop(key);
                    stats.misses += 1;
                    stats.evictions += 1;
                    return None;
                }
            }
            
            // Update access count and stats
            entry.access_count += 1;
            stats.hits += 1;
            Some(entry.value.clone())
        } else {
            stats.misses += 1;
            None
        }
    }
    
    /// Insert or update a value in the cache
    pub fn put(&self, key: K, value: V) -> Option<V> {
        let mut cache = self.inner.write();
        let mut stats = self.stats.write();
        
        let entry = CacheEntry {
            value: value.clone(),
            inserted_at: Instant::now(),
            access_count: 0,
        };
        
        // Check if we're replacing an existing entry
        let old = cache.push(key, entry);
        
        if old.is_some() {
            // Replaced existing entry
            stats.evictions += 1;
        } else {
            // New entry
            stats.insertions += 1;
        }
        
        stats.current_size = cache.len();
        
        old.map(|(_, entry)| entry.value)
    }
    
    /// Remove a value from the cache
    pub fn remove(&self, key: &K) -> Option<V> {
        let mut cache = self.inner.write();
        let mut stats = self.stats.write();
        
        let removed = cache.pop(key);
        if removed.is_some() {
            stats.evictions += 1;
            stats.current_size = cache.len();
        }
        
        removed.map(|e| e.value)
    }
    
    /// Clear all entries from the cache
    pub fn clear(&self) {
        let mut cache = self.inner.write();
        let mut stats = self.stats.write();
        
        let count = cache.len();
        cache.clear();
        
        stats.evictions += count as u64;
        stats.current_size = 0;
    }
    
    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        self.stats.read().clone()
    }
    
    /// Get the current size of the cache
    pub fn len(&self) -> usize {
        self.inner.read().len()
    }
    
    /// Check if the cache is empty
    pub fn is_empty(&self) -> bool {
        self.inner.read().is_empty()
    }
    
    /// Clean up expired entries (for TTL caches)
    pub fn cleanup_expired(&self) -> usize {
        let ttl = match self.ttl {
            Some(ttl) => ttl,
            None => return 0,
        };
        let mut cache = self.inner.write();
        let mut stats = self.stats.write();
        
        let _expired_keys: Vec<K> = Vec::new();
        
        // Find expired entries
        // Note: This is not the most efficient way, but LruCache doesn't expose iteration
        // In production, consider using a different cache implementation
        let _initial_size = cache.len();
        
        // Create a temporary vector of all keys
        let keys: Vec<K> = cache.iter()
            .filter_map(|(k, entry)| {
                if entry.inserted_at.elapsed() > ttl {
                    Some(k.clone())
                } else {
                    None
                }
            })
            .collect();
        
        // Remove expired entries
        for key in keys.iter() {
            cache.pop(key);
        }
        
        let removed = keys.len();
        stats.evictions += removed as u64;
        stats.current_size = cache.len();
        
        removed
    }
}

/// Specialized cache for embedding vectors
pub struct EmbeddingCache {
    cache: BoundedCache<String, Vec<f32>>,
    dimension: usize,
}

impl EmbeddingCache {
    /// Create a new embedding cache
    pub fn new(capacity: usize, dimension: usize) -> Result<Self> {
        Ok(Self {
            cache: BoundedCache::new(capacity)?,
            dimension,
        })
    }
    
    /// Get an embedding from the cache
    pub fn get(&self, text: &str) -> Option<Vec<f32>> {
        self.cache.get(&text.to_string())
    }
    
    /// Store an embedding in the cache
    pub fn put(&self, text: String, embedding: Vec<f32>) -> Result<()> {
        // Validate embedding dimension
        if embedding.len() != self.dimension {
            return Err(EmbedError::Validation {
                field: "embedding".to_string(),
                reason: format!(
                    "Embedding dimension {} does not match cache dimension {}",
                    embedding.len(),
                    self.dimension
                ),
                value: Some(embedding.len().to_string()),
            });
        }
        
        self.cache.put(text, embedding);
        Ok(())
    }
    
    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        self.cache.stats()
    }
}

/// Specialized cache for search results
pub struct SearchCache {
    cache: BoundedCache<SearchKey, Vec<SearchResult>>,
}

#[derive(Hash, Eq, PartialEq, Clone)]
struct SearchKey {
    query: String,
    top_k: usize,
}

#[derive(Clone)]
pub struct SearchResult {
    pub id: String,
    pub score: f32,
    pub metadata: Option<String>,
}

impl SearchCache {
    /// Create a new search cache with TTL
    pub fn new(capacity: usize, ttl_seconds: u64) -> Result<Self> {
        Ok(Self {
            cache: BoundedCache::<SearchKey, Vec<SearchResult>>::with_ttl(capacity, Duration::from_secs(ttl_seconds))?,
        })
    }
    
    /// Get search results from cache
    pub fn get(&self, query: &str, top_k: usize) -> Option<Vec<SearchResult>> {
        let key = SearchKey {
            query: query.to_string(),
            top_k,
        };
        self.cache.get(&key)
    }
    
    /// Store search results in cache
    pub fn put(&self, query: String, top_k: usize, results: Vec<SearchResult>) {
        let key = SearchKey { query, top_k };
        self.cache.put(key, results);
    }
    
    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        self.cache.stats()
    }
    
    /// Clean up expired entries
    pub fn cleanup(&self) -> usize {
        self.cache.cleanup_expired()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    
    #[test]
    fn test_bounded_cache_basic() {
        let cache: BoundedCache<String, i32> = BoundedCache::new(3).unwrap();
        
        // Insert items
        cache.put("a".to_string(), 1);
        cache.put("b".to_string(), 2);
        cache.put("c".to_string(), 3);
        
        // Check they exist
        assert_eq!(cache.get(&"a".to_string()), Some(1));
        assert_eq!(cache.get(&"b".to_string()), Some(2));
        assert_eq!(cache.get(&"c".to_string()), Some(3));
        
        // Add one more - should evict least recently used
        cache.put("d".to_string(), 4);
        
        // "a" should be evicted (it was least recently used)
        assert_eq!(cache.get(&"a".to_string()), None);
        assert_eq!(cache.get(&"d".to_string()), Some(4));
    }
    
    #[test]
    fn test_cache_stats() {
        let cache: BoundedCache<String, i32> = BoundedCache::new(2).unwrap();
        
        cache.put("a".to_string(), 1);
        cache.put("b".to_string(), 2);
        
        // Some hits and misses
        cache.get(&"a".to_string());  // hit
        cache.get(&"b".to_string());  // hit
        cache.get(&"c".to_string());  // miss
        
        let stats = cache.stats();
        assert_eq!(stats.hits, 2);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.insertions, 2);
        assert_eq!(stats.current_size, 2);
        assert!((stats.hit_rate().unwrap() - 66.66666666666667).abs() < 0.001);
    }
    
    #[test]
    fn test_cache_with_ttl() {
        let cache: BoundedCache<String, i32> = 
            BoundedCache::with_ttl(10, Duration::from_millis(100)).unwrap();
        
        cache.put("a".to_string(), 1);
        
        // Should exist immediately
        assert_eq!(cache.get(&"a".to_string()), Some(1));
        
        // Wait for TTL to expire
        thread::sleep(Duration::from_millis(150));
        
        // Should be expired
        assert_eq!(cache.get(&"a".to_string()), None);
    }
    
    #[test]
    fn test_thread_safety() {
        let cache = Arc::new(BoundedCache::<String, i32>::new(100).unwrap());
        let mut handles = vec![];
        
        // Spawn multiple threads doing concurrent operations
        for i in 0..10 {
            let cache_clone = cache.clone();
            handles.push(thread::spawn(move || {
                for j in 0..100 {
                    let key = format!("key_{}_{}", i, j);
                    cache_clone.put(key.clone(), i * 100 + j);
                    cache_clone.get(&key);
                }
            }));
        }
        
        // Wait for all threads
        for handle in handles {
            handle.join().unwrap();
        }
        
        // Cache should still be in valid state
        let stats = cache.stats();
        assert!(stats.hits > 0);
        assert!(stats.insertions > 0);
    }
    
    #[test]
    fn test_embedding_cache() {
        let cache = EmbeddingCache::new(10, 768).unwrap();
        
        let embedding = vec![1.0; 768];
        cache.put("test text".to_string(), embedding.clone()).unwrap();
        
        assert_eq!(cache.get("test text"), Some(embedding));
        assert_eq!(cache.get("other text"), None);
        
        // Wrong dimension should fail
        let wrong_embedding = vec![1.0; 512];
        assert!(cache.put("wrong".to_string(), wrong_embedding).is_err());
    }
}