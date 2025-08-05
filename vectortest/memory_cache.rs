// Rust In-Memory Cache Implementation
use std::collections::{HashMap, BTreeMap};
use std::sync::{Arc, RwLock, Mutex};
use std::time::{Duration, Instant};
use std::hash::Hash;
use std::fmt::Debug;
use tokio::time::interval;
use tokio::sync::mpsc;
use serde::{Serialize, Deserialize};

/// Thread-safe in-memory cache with TTL support
pub struct MemoryCache<K, V> 
where 
    K: Eq + Hash + Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    storage: Arc<RwLock<HashMap<K, CacheEntry<V>>>>,
    expiry_index: Arc<RwLock<BTreeMap<Instant, Vec<K>>>>,
    config: CacheConfig,
    stats: Arc<Mutex<CacheStats>>,
    eviction_tx: mpsc::Sender<EvictionCommand<K>>,
}

#[derive(Clone)]
struct CacheEntry<V> {
    value: V,
    inserted_at: Instant,
    last_accessed: Instant,
    access_count: u64,
    expires_at: Option<Instant>,
}

#[derive(Clone, Debug)]
pub struct CacheConfig {
    pub max_size: usize,
    pub default_ttl: Option<Duration>,
    pub eviction_policy: EvictionPolicy,
    pub cleanup_interval: Duration,
}

#[derive(Clone, Debug)]
pub enum EvictionPolicy {
    LRU,  // Least Recently Used
    LFU,  // Least Frequently Used
    FIFO, // First In First Out
    TTL,  // Time To Live only
}

#[derive(Debug, Default)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub expirations: u64,
}

enum EvictionCommand<K> {
    Cleanup,
    CheckSize,
    Remove(K),
}

impl<K, V> MemoryCache<K, V>
where
    K: Eq + Hash + Clone + Send + Sync + Debug + 'static,
    V: Clone + Send + Sync + 'static,
{
    /// Creates a new cache instance with the given configuration
    pub fn new(config: CacheConfig) -> Self {
        let storage = Arc::new(RwLock::new(HashMap::new()));
        let expiry_index = Arc::new(RwLock::new(BTreeMap::new()));
        let stats = Arc::new(Mutex::new(CacheStats::default()));
        
        let (tx, rx) = mpsc::channel::<EvictionCommand<K>>(100);
        
        let cache = Self {
            storage: storage.clone(),
            expiry_index: expiry_index.clone(),
            config: config.clone(),
            stats: stats.clone(),
            eviction_tx: tx,
        };
        
        // Start background eviction task
        tokio::spawn(Self::eviction_worker(
            rx,
            storage.clone(),
            expiry_index.clone(),
            config,
            stats.clone(),
        ));
        
        cache
    }
    
    /// Inserts a key-value pair with optional TTL
    pub async fn insert(&self, key: K, value: V, ttl: Option<Duration>) -> Option<V> {
        let now = Instant::now();
        let expires_at = ttl.or(self.config.default_ttl).map(|d| now + d);
        
        let entry = CacheEntry {
            value: value.clone(),
            inserted_at: now,
            last_accessed: now,
            access_count: 0,
            expires_at,
        };
        
        let old_value = {
            let mut storage = self.storage.write().unwrap();
            
            // Remove old expiry index if exists
            if let Some(old_entry) = storage.get(&key) {
                if let Some(old_expires) = old_entry.expires_at {
                    let mut expiry_index = self.expiry_index.write().unwrap();
                    if let Some(keys) = expiry_index.get_mut(&old_expires) {
                        keys.retain(|k| k != &key);
                        if keys.is_empty() {
                            expiry_index.remove(&old_expires);
                        }
                    }
                }
            }
            
            storage.insert(key.clone(), entry).map(|e| e.value)
        };
        
        // Update expiry index
        if let Some(expires_at) = expires_at {
            let mut expiry_index = self.expiry_index.write().unwrap();
            expiry_index.entry(expires_at)
                .or_insert_with(Vec::new)
                .push(key.clone());
        }
        
        // Check if eviction is needed
        let _ = self.eviction_tx.send(EvictionCommand::CheckSize).await;
        
        old_value
    }
    
    /// Gets a value by key, updating access statistics
    pub fn get(&self, key: &K) -> Option<V> {
        let mut storage = self.storage.write().unwrap();
        
        if let Some(entry) = storage.get_mut(key) {
            // Check if expired
            if let Some(expires_at) = entry.expires_at {
                if Instant::now() > expires_at {
                    drop(storage);
                    self.remove(key);
                    self.stats.lock().unwrap().misses += 1;
                    return None;
                }
            }
            
            // Update access info
            entry.last_accessed = Instant::now();
            entry.access_count += 1;
            
            self.stats.lock().unwrap().hits += 1;
            Some(entry.value.clone())
        } else {
            self.stats.lock().unwrap().misses += 1;
            None
        }
    }
    
    /// Removes a key from the cache
    pub fn remove(&self, key: &K) -> Option<V> {
        let mut storage = self.storage.write().unwrap();
        
        if let Some(entry) = storage.remove(key) {
            // Remove from expiry index
            if let Some(expires_at) = entry.expires_at {
                let mut expiry_index = self.expiry_index.write().unwrap();
                if let Some(keys) = expiry_index.get_mut(&expires_at) {
                    keys.retain(|k| k != key);
                    if keys.is_empty() {
                        expiry_index.remove(&expires_at);
                    }
                }
            }
            
            Some(entry.value)
        } else {
            None
        }
    }
    
    /// Clears all entries from the cache
    pub fn clear(&self) {
        self.storage.write().unwrap().clear();
        self.expiry_index.write().unwrap().clear();
        
        let mut stats = self.stats.lock().unwrap();
        stats.hits = 0;
        stats.misses = 0;
        stats.evictions = 0;
        stats.expirations = 0;
    }
    
    /// Returns the current size of the cache
    pub fn len(&self) -> usize {
        self.storage.read().unwrap().len()
    }
    
    /// Checks if the cache is empty
    pub fn is_empty(&self) -> bool {
        self.storage.read().unwrap().is_empty()
    }
    
    /// Gets current cache statistics
    pub fn stats(&self) -> CacheStats {
        self.stats.lock().unwrap().clone()
    }
    
    /// Background worker for handling evictions
    async fn eviction_worker(
        mut rx: mpsc::Receiver<EvictionCommand<K>>,
        storage: Arc<RwLock<HashMap<K, CacheEntry<V>>>>,
        expiry_index: Arc<RwLock<BTreeMap<Instant, Vec<K>>>>,
        config: CacheConfig,
        stats: Arc<Mutex<CacheStats>>,
    ) {
        let mut cleanup_interval = interval(config.cleanup_interval);
        
        loop {
            tokio::select! {
                _ = cleanup_interval.tick() => {
                    Self::cleanup_expired(&storage, &expiry_index, &stats);
                }
                
                Some(cmd) = rx.recv() => {
                    match cmd {
                        EvictionCommand::Cleanup => {
                            Self::cleanup_expired(&storage, &expiry_index, &stats);
                        }
                        EvictionCommand::CheckSize => {
                            Self::evict_if_needed(&storage, &expiry_index, &config, &stats);
                        }
                        EvictionCommand::Remove(key) => {
                            // Already handled in remove method
                        }
                    }
                }
            }
        }
    }
    
    /// Removes expired entries
    fn cleanup_expired(
        storage: &Arc<RwLock<HashMap<K, CacheEntry<V>>>>,
        expiry_index: &Arc<RwLock<BTreeMap<Instant, Vec<K>>>>,
        stats: &Arc<Mutex<CacheStats>>,
    ) {
        let now = Instant::now();
        let mut expired_keys = Vec::new();
        
        {
            let expiry_index = expiry_index.read().unwrap();
            for (expires_at, keys) in expiry_index.iter() {
                if *expires_at <= now {
                    expired_keys.extend(keys.clone());
                } else {
                    break; // BTreeMap is ordered, so we can stop here
                }
            }
        }
        
        if !expired_keys.is_empty() {
            let mut storage = storage.write().unwrap();
            let mut expiry_index = expiry_index.write().unwrap();
            let mut stats = stats.lock().unwrap();
            
            for key in expired_keys {
                storage.remove(&key);
                stats.expirations += 1;
            }
            
            // Remove expired entries from index
            expiry_index = expiry_index.split_off(&now);
        }
    }
    
    /// Evicts entries if cache size exceeds maximum
    fn evict_if_needed(
        storage: &Arc<RwLock<HashMap<K, CacheEntry<V>>>>,
        expiry_index: &Arc<RwLock<BTreeMap<Instant, Vec<K>>>>,
        config: &CacheConfig,
        stats: &Arc<Mutex<CacheStats>>,
    ) {
        let mut storage = storage.write().unwrap();
        
        if storage.len() <= config.max_size {
            return;
        }
        
        let to_evict = storage.len() - config.max_size;
        let keys_to_remove = match config.eviction_policy {
            EvictionPolicy::LRU => {
                let mut entries: Vec<_> = storage.iter()
                    .map(|(k, v)| (k.clone(), v.last_accessed))
                    .collect();
                entries.sort_by_key(|(_, last_accessed)| *last_accessed);
                entries.into_iter()
                    .take(to_evict)
                    .map(|(k, _)| k)
                    .collect::<Vec<_>>()
            }
            
            EvictionPolicy::LFU => {
                let mut entries: Vec<_> = storage.iter()
                    .map(|(k, v)| (k.clone(), v.access_count))
                    .collect();
                entries.sort_by_key(|(_, count)| *count);
                entries.into_iter()
                    .take(to_evict)
                    .map(|(k, _)| k)
                    .collect::<Vec<_>>()
            }
            
            EvictionPolicy::FIFO => {
                let mut entries: Vec<_> = storage.iter()
                    .map(|(k, v)| (k.clone(), v.inserted_at))
                    .collect();
                entries.sort_by_key(|(_, inserted_at)| *inserted_at);
                entries.into_iter()
                    .take(to_evict)
                    .map(|(k, _)| k)
                    .collect::<Vec<_>>()
            }
            
            EvictionPolicy::TTL => {
                // TTL only - don't evict based on size
                vec![]
            }
        };
        
        let mut stats = stats.lock().unwrap();
        let mut expiry_index = expiry_index.write().unwrap();
        
        for key in keys_to_remove {
            if let Some(entry) = storage.remove(&key) {
                stats.evictions += 1;
                
                // Remove from expiry index
                if let Some(expires_at) = entry.expires_at {
                    if let Some(keys) = expiry_index.get_mut(&expires_at) {
                        keys.retain(|k| k != &key);
                        if keys.is_empty() {
                            expiry_index.remove(&expires_at);
                        }
                    }
                }
            }
        }
    }
}

/// Builder pattern for cache configuration
pub struct CacheBuilder {
    max_size: usize,
    default_ttl: Option<Duration>,
    eviction_policy: EvictionPolicy,
    cleanup_interval: Duration,
}

impl Default for CacheBuilder {
    fn default() -> Self {
        Self {
            max_size: 1000,
            default_ttl: None,
            eviction_policy: EvictionPolicy::LRU,
            cleanup_interval: Duration::from_secs(60),
        }
    }
}

impl CacheBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn max_size(mut self, size: usize) -> Self {
        self.max_size = size;
        self
    }
    
    pub fn default_ttl(mut self, ttl: Duration) -> Self {
        self.default_ttl = Some(ttl);
        self
    }
    
    pub fn eviction_policy(mut self, policy: EvictionPolicy) -> Self {
        self.eviction_policy = policy;
        self
    }
    
    pub fn cleanup_interval(mut self, interval: Duration) -> Self {
        self.cleanup_interval = interval;
        self
    }
    
    pub fn build<K, V>(self) -> MemoryCache<K, V>
    where
        K: Eq + Hash + Clone + Send + Sync + Debug + 'static,
        V: Clone + Send + Sync + 'static,
    {
        let config = CacheConfig {
            max_size: self.max_size,
            default_ttl: self.default_ttl,
            eviction_policy: self.eviction_policy,
            cleanup_interval: self.cleanup_interval,
        };
        
        MemoryCache::new(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    
    #[tokio::test]
    async fn test_basic_operations() {
        let cache = CacheBuilder::new()
            .max_size(10)
            .build::<String, String>();
        
        // Test insert and get
        cache.insert("key1".to_string(), "value1".to_string(), None).await;
        assert_eq!(cache.get(&"key1".to_string()), Some("value1".to_string()));
        
        // Test update
        cache.insert("key1".to_string(), "value2".to_string(), None).await;
        assert_eq!(cache.get(&"key1".to_string()), Some("value2".to_string()));
        
        // Test remove
        assert_eq!(cache.remove(&"key1".to_string()), Some("value2".to_string()));
        assert_eq!(cache.get(&"key1".to_string()), None);
    }
    
    #[tokio::test]
    async fn test_ttl_expiration() {
        let cache = CacheBuilder::new()
            .cleanup_interval(Duration::from_millis(100))
            .build::<String, i32>();
        
        cache.insert("key1".to_string(), 42, Some(Duration::from_millis(200))).await;
        
        // Should exist immediately
        assert_eq!(cache.get(&"key1".to_string()), Some(42));
        
        // Wait for expiration
        tokio::time::sleep(Duration::from_millis(300)).await;
        
        // Should be expired
        assert_eq!(cache.get(&"key1".to_string()), None);
    }
    
    #[tokio::test]
    async fn test_lru_eviction() {
        let cache = CacheBuilder::new()
            .max_size(3)
            .eviction_policy(EvictionPolicy::LRU)
            .build::<i32, String>();
        
        // Fill cache
        cache.insert(1, "one".to_string(), None).await;
        cache.insert(2, "two".to_string(), None).await;
        cache.insert(3, "three".to_string(), None).await;
        
        // Access to update LRU
        cache.get(&1);
        cache.get(&3);
        
        // Insert new item, should evict key 2 (least recently used)
        cache.insert(4, "four".to_string(), None).await;
        
        assert_eq!(cache.get(&1), Some("one".to_string()));
        assert_eq!(cache.get(&2), None); // Evicted
        assert_eq!(cache.get(&3), Some("three".to_string()));
        assert_eq!(cache.get(&4), Some("four".to_string()));
    }
}