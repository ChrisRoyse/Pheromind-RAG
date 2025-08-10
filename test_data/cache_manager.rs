use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tokio::sync::RwLock as AsyncRwLock;
use serde::{Serialize, Deserialize};
use anyhow::Result;

/// High-performance LRU cache with TTL support and async operations
/// Optimized for concurrent access patterns in web applications
#[derive(Debug, Clone)]
pub struct CacheManager<K, V> 
where 
    K: Clone + Eq + std::hash::Hash + Send + Sync,
    V: Clone + Send + Sync,
{
    store: Arc<AsyncRwLock<HashMap<K, CacheEntry<V>>>>,
    access_order: Arc<RwLock<Vec<K>>>,
    max_capacity: usize,
    default_ttl: Duration,
    stats: Arc<RwLock<CacheStats>>,
}

#[derive(Debug, Clone)]
struct CacheEntry<V> {
    value: V,
    created_at: Instant,
    expires_at: Option<Instant>,
    access_count: u64,
    last_accessed: Instant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub expired_entries: u64,
    pub current_size: usize,
}

impl<K, V> CacheManager<K, V>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync,
    V: Clone + Send + Sync,
{
    /// Create a new cache manager with specified capacity and default TTL
    pub fn new(max_capacity: usize, default_ttl: Duration) -> Self {
        Self {
            store: Arc::new(AsyncRwLock::new(HashMap::new())),
            access_order: Arc::new(RwLock::new(Vec::new())),
            max_capacity,
            default_ttl,
            stats: Arc::new(RwLock::new(CacheStats {
                hits: 0,
                misses: 0,
                evictions: 0,
                expired_entries: 0,
                current_size: 0,
            })),
        }
    }

    /// Insert a value with default TTL
    pub async fn insert(&self, key: K, value: V) -> Result<()> {
        self.insert_with_ttl(key, value, Some(self.default_ttl)).await
    }

    /// Insert a value with custom TTL (None for no expiration)
    pub async fn insert_with_ttl(&self, key: K, value: V, ttl: Option<Duration>) -> Result<()> {
        let mut store = self.store.write().await;
        let mut access_order = self.access_order.write().unwrap();
        let mut stats = self.stats.write().unwrap();

        let now = Instant::now();
        let expires_at = ttl.map(|duration| now + duration);

        // Remove key from access order if it already exists
        access_order.retain(|k| k != &key);

        // Check if we need to evict entries
        if store.len() >= self.max_capacity && !store.contains_key(&key) {
            self.evict_lru(&mut store, &mut access_order, &mut stats);
        }

        let entry = CacheEntry {
            value,
            created_at: now,
            expires_at,
            access_count: 0,
            last_accessed: now,
        };

        store.insert(key.clone(), entry);
        access_order.push(key);
        stats.current_size = store.len();

        Ok(())
    }

    /// Get a value from cache, returns None if not found or expired
    pub async fn get(&self, key: &K) -> Option<V> {
        let mut store = self.store.write().await;
        let mut access_order = self.access_order.write().unwrap();
        let mut stats = self.stats.write().unwrap();

        if let Some(entry) = store.get_mut(key) {
            let now = Instant::now();

            // Check if entry has expired
            if let Some(expires_at) = entry.expires_at {
                if now > expires_at {
                    store.remove(key);
                    access_order.retain(|k| k != key);
                    stats.expired_entries += 1;
                    stats.misses += 1;
                    stats.current_size = store.len();
                    return None;
                }
            }

            // Update access information
            entry.access_count += 1;
            entry.last_accessed = now;

            // Move to end of access order (most recently used)
            access_order.retain(|k| k != key);
            access_order.push(key.clone());

            stats.hits += 1;
            Some(entry.value.clone())
        } else {
            stats.misses += 1;
            None
        }
    }

    /// Check if a key exists in cache (without updating access time)
    pub async fn contains_key(&self, key: &K) -> bool {
        let store = self.store.read().await;
        if let Some(entry) = store.get(key) {
            // Check expiration
            if let Some(expires_at) = entry.expires_at {
                return Instant::now() <= expires_at;
            }
            true
        } else {
            false
        }
    }

    /// Remove a key from cache
    pub async fn remove(&self, key: &K) -> Option<V> {
        let mut store = self.store.write().await;
        let mut access_order = self.access_order.write().unwrap();
        let mut stats = self.stats.write().unwrap();

        access_order.retain(|k| k != key);
        if let Some(entry) = store.remove(key) {
            stats.current_size = store.len();
            Some(entry.value)
        } else {
            None
        }
    }

    /// Clear all entries from cache
    pub async fn clear(&self) {
        let mut store = self.store.write().await;
        let mut access_order = self.access_order.write().unwrap();
        let mut stats = self.stats.write().unwrap();

        store.clear();
        access_order.clear();
        stats.current_size = 0;
    }

    /// Get cache statistics
    pub fn get_stats(&self) -> CacheStats {
        self.stats.read().unwrap().clone()
    }

    /// Clean up expired entries
    pub async fn cleanup_expired(&self) -> usize {
        let mut store = self.store.write().await;
        let mut access_order = self.access_order.write().unwrap();
        let mut stats = self.stats.write().unwrap();

        let now = Instant::now();
        let initial_size = store.len();

        store.retain(|key, entry| {
            if let Some(expires_at) = entry.expires_at {
                if now > expires_at {
                    access_order.retain(|k| k != key);
                    false
                } else {
                    true
                }
            } else {
                true
            }
        });

        let removed_count = initial_size - store.len();
        stats.expired_entries += removed_count as u64;
        stats.current_size = store.len();

        removed_count
    }

    /// Get cache hit rate as percentage
    pub fn hit_rate(&self) -> f64 {
        let stats = self.stats.read().unwrap();
        let total_requests = stats.hits + stats.misses;
        if total_requests == 0 {
            0.0
        } else {
            (stats.hits as f64 / total_requests as f64) * 100.0
        }
    }

    fn evict_lru(&self, 
                 store: &mut HashMap<K, CacheEntry<V>>, 
                 access_order: &mut Vec<K>, 
                 stats: &mut CacheStats) {
        if let Some(lru_key) = access_order.first().cloned() {
            store.remove(&lru_key);
            access_order.remove(0);
            stats.evictions += 1;
        }
    }
}

// Specialized cache for string-based operations
pub type StringCache = CacheManager<String, String>;

impl StringCache {
    /// Create a string cache optimized for web application use
    pub fn for_web_app() -> Self {
        Self::new(10000, Duration::from_secs(3600)) // 1 hour TTL, 10k capacity
    }

    /// Cache a computation result with automatic key generation
    pub async fn cache_computation<F, Fut>(&self, input: &str, computation: F) -> Result<String>
    where
        F: FnOnce(String) -> Fut,
        Fut: std::future::Future<Output = Result<String>>,
    {
        let cache_key = format!("compute:{}", input);
        
        if let Some(cached_result) = self.get(&cache_key).await {
            return Ok(cached_result);
        }

        let result = computation(input.to_string()).await?;
        self.insert(cache_key, result.clone()).await?;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_basic_cache_operations() {
        let cache = CacheManager::new(3, Duration::from_secs(60));
        
        // Test insert and get
        cache.insert("key1".to_string(), "value1".to_string()).await.unwrap();
        assert_eq!(cache.get(&"key1".to_string()).await, Some("value1".to_string()));
        
        // Test miss
        assert_eq!(cache.get(&"nonexistent".to_string()).await, None);
        
        // Test stats
        let stats = cache.get_stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
    }

    #[tokio::test]
    async fn test_ttl_expiration() {
        let cache = CacheManager::new(10, Duration::from_millis(100));
        
        cache.insert_with_ttl("temp".to_string(), "data".to_string(), Some(Duration::from_millis(50))).await.unwrap();
        assert_eq!(cache.get(&"temp".to_string()).await, Some("data".to_string()));
        
        sleep(Duration::from_millis(60)).await;
        assert_eq!(cache.get(&"temp".to_string()).await, None);
    }

    #[tokio::test]
    async fn test_lru_eviction() {
        let cache = CacheManager::new(2, Duration::from_secs(60));
        
        cache.insert("key1".to_string(), "value1".to_string()).await.unwrap();
        cache.insert("key2".to_string(), "value2".to_string()).await.unwrap();
        cache.insert("key3".to_string(), "value3".to_string()).await.unwrap(); // Should evict key1
        
        assert_eq!(cache.get(&"key1".to_string()).await, None);
        assert_eq!(cache.get(&"key2".to_string()).await, Some("value2".to_string()));
        assert_eq!(cache.get(&"key3".to_string()).await, Some("value3".to_string()));
    }
}