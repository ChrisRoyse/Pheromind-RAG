use std::sync::{Arc, Mutex};
use std::path::Path;
use std::fs;
use std::io::Write;
use lru::LruCache;
use sha2::{Sha256, Digest};
use serde::{Serialize, Deserialize};
use anyhow::Result;
#[cfg(feature = "ml")]
use crate::config::Config;

/// Cache entry containing embedding vector and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub embedding: Vec<f32>,
    pub content_hash: String,
    pub timestamp: u64,
}

/// Thread-safe LRU cache for embeddings
pub struct EmbeddingCache {
    cache: Arc<Mutex<LruCache<String, CacheEntry>>>,
    max_size: usize,
    cache_dir: Option<std::path::PathBuf>,
}

impl EmbeddingCache {
    /// Create a new embedding cache using configuration values
    /// Returns an error if configuration is not properly initialized
    #[cfg(feature = "ml")]
    pub fn from_config() -> Result<Self, crate::error::EmbedError> {
        let config = Config::get()?;
        Self::new_with_persistence(config.embedding_cache_size, &config.cache_dir)
    }

    /// Create a new embedding cache with specified maximum size
    pub fn new(max_size: usize) -> Result<Self, crate::error::EmbedError> {
        let capacity = std::num::NonZeroUsize::new(max_size)
            .ok_or_else(|| crate::error::EmbedError::Internal {
                message: format!("Cache max_size must be greater than 0, got: {}", max_size),
                backtrace: None,
            })?;
        
        Ok(Self {
            cache: Arc::new(Mutex::new(LruCache::new(capacity))),
            max_size,
            cache_dir: None,
        })
    }

    /// Create a new embedding cache with persistence support
    pub fn new_with_persistence(max_size: usize, cache_dir: impl AsRef<Path>) -> Result<Self, crate::error::EmbedError> {
        let cache_dir = cache_dir.as_ref().to_path_buf();
        
        let capacity = std::num::NonZeroUsize::new(max_size)
            .ok_or_else(|| crate::error::EmbedError::Internal {
                message: format!("Cache max_size must be greater than 0, got: {}", max_size),
                backtrace: None,
            })?;
        
        let mut cache = Self {
            cache: Arc::new(Mutex::new(LruCache::new(capacity))),
            max_size,
            cache_dir: Some(cache_dir),
        };
        
        // Load existing cache if it exists - corruption or read failures are fatal
        match cache.load_from_disk() {
            Ok(()) => {
                // Successfully loaded or no cache file exists
            }
            Err(e) => {
                // This indicates corruption or IO issues with an existing cache file
                return Err(crate::error::EmbedError::Internal {
                    message: format!("Failed to load existing cache from disk: {}. Cache corruption is not acceptable.", e),
                    backtrace: None,
                });
            }
        }
        
        Ok(cache)
    }

    /// Generate a hash key for content
    pub fn hash_content(content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Get embedding from cache if it exists
    pub fn get(&self, content: &str) -> Result<Option<Vec<f32>>, crate::error::EmbedError> {
        let hash = Self::hash_content(content);
        
        let mut cache = self.cache.lock().map_err(|_| {
            crate::error::EmbedError::Internal {
                message: "FATAL: Embedding cache mutex is poisoned. Cache operations cannot continue.".to_string(),
                backtrace: None,
            }
        })?;
        
        if let Some(entry) = cache.get(&hash) {
            Ok(Some(entry.embedding.clone()))
        } else {
            Ok(None)
        }
    }

    /// Store embedding in cache
    pub fn put(&self, content: &str, embedding: Vec<f32>) -> Result<(), crate::error::EmbedError> {
        let hash = Self::hash_content(content);
        let entry = CacheEntry {
            embedding,
            content_hash: hash.clone(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| crate::error::EmbedError::Internal {
                    message: format!("System time error: {}", e),
                    backtrace: None,
                })?
                .as_secs(),
        };

        if let Ok(mut cache) = self.cache.lock() {
            cache.put(hash, entry);
            Ok(())
        } else {
            Err(crate::error::EmbedError::Internal {
                message: "Failed to acquire cache lock".to_string(),
                backtrace: None,
            })
        }
    }

    /// Get cache statistics - fails immediately if mutex is poisoned
    pub fn stats(&self) -> Result<CacheStats, crate::error::EmbedError> {
        let cache = self.cache.lock().map_err(|_| {
            crate::error::EmbedError::Internal {
                message: "FATAL: Embedding cache mutex is poisoned. Cache operations cannot continue.".to_string(),
                backtrace: None,
            }
        })?;
        
        Ok(CacheStats {
            entries: cache.len(),
            max_size: self.max_size,
            hit_ratio: 0.0, // Could implement hit tracking if needed
        })
    }

    /// Clear all cache entries
    pub fn clear(&self) -> Result<(), crate::error::EmbedError> {
        let mut cache = self.cache.lock().map_err(|_| {
            crate::error::EmbedError::Internal {
                message: "FATAL: Embedding cache mutex is poisoned. Cache cannot be cleared.".to_string(),
                backtrace: None,
            }
        })?;
        
        cache.clear();
        Ok(())
    }

    /// Get the number of entries in the cache
    pub fn len(&self) -> Result<usize, crate::error::EmbedError> {
        let cache = self.cache.lock().map_err(|_| {
            crate::error::EmbedError::Internal {
                message: "FATAL: Embedding cache mutex is poisoned. Cache length cannot be determined.".to_string(),
                backtrace: None,
            }
        })?;
        
        Ok(cache.len())
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> Result<bool, crate::error::EmbedError> {
        Ok(self.len()? == 0)
    }

    /// Save cache to disk (if persistence is enabled)
    pub fn save_to_disk(&self) -> Result<()> {
        if let Some(cache_dir) = &self.cache_dir {
            // Create cache directory if it doesn't exist
            fs::create_dir_all(cache_dir)?;
            
            let cache_file = cache_dir.join("embeddings.cache");
            
            let cache = self.cache.lock()
                .map_err(|_| anyhow::anyhow!("Failed to acquire cache lock for save - cache is poisoned"))?;
            
            // Convert LRU cache to a serializable format
            let entries: Vec<(String, CacheEntry)> = cache
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            
            let serialized = serde_json::to_string_pretty(&entries)?;
            let mut file = fs::File::create(&cache_file)?;
            file.write_all(serialized.as_bytes())?;
            
            #[cfg(debug_assertions)]
            println!("💾 Saved {} cache entries to {:?}", entries.len(), cache_file);
        }
        
        Ok(())
    }

    /// Load cache from disk (if persistence is enabled)
    pub fn load_from_disk(&mut self) -> Result<()> {
        if let Some(cache_dir) = &self.cache_dir {
            let cache_file = cache_dir.join("embeddings.cache");
            
            if cache_file.exists() {
                let content = fs::read_to_string(&cache_file)?;
                let entries: Vec<(String, CacheEntry)> = serde_json::from_str(&content)?;
                
                let mut cache = self.cache.lock()
                    .map_err(|_| anyhow::anyhow!("Failed to acquire cache lock for load - cache is poisoned"))?;
                
                for (key, entry) in entries {
                    cache.put(key, entry);
                }
                
                #[cfg(debug_assertions)]
                println!("📂 Loaded {} cache entries from {:?}", cache.len(), cache_file);
            }
        }
        
        Ok(())
    }

    /// Batch get embeddings from cache
    pub fn get_batch(&self, contents: &[&str]) -> Result<Vec<Option<Vec<f32>>>, crate::error::EmbedError> {
        contents.iter()
            .map(|content| self.get(content))
            .collect::<Result<Vec<_>, _>>()
    }

    /// Batch put embeddings into cache
    pub fn put_batch(&self, contents: &[&str], embeddings: Vec<Vec<f32>>) -> Result<(), crate::error::EmbedError> {
        for (content, embedding) in contents.iter().zip(embeddings.into_iter()) {
            self.put(content, embedding)?;
        }
        Ok(())
    }
}

/// Cache statistics
#[derive(Debug)]
pub struct CacheStats {
    pub entries: usize,
    pub max_size: usize,
    pub hit_ratio: f32,
}

impl CacheStats {
    pub fn new() -> Self {
        Self {
            entries: 0,
            max_size: 0,
            hit_ratio: 0.0,
        }
    }
}

impl std::fmt::Display for CacheStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Cache: {}/{} entries (hit ratio: {:.1}%)",
            self.entries, self.max_size, self.hit_ratio * 100.0
        )
    }
}

/// Drop implementation to save cache on shutdown
impl Drop for EmbeddingCache {
    fn drop(&mut self) {
        if let Err(e) = self.save_to_disk() {
            eprintln!("ERROR: Failed to save cache on shutdown: {}. Embedding cache data may be lost. Check disk space and permissions.", e);
            // Log the error but do not panic - crashing the application is worse than losing cache data
            // The cache is a performance optimization, not critical system data
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_cache_basic_operations() {
        let cache = EmbeddingCache::new(10).unwrap();
        
        // Test empty cache
        assert!(cache.is_empty().unwrap());
        assert_eq!(cache.len().unwrap(), 0);
        assert!(cache.get("test content").unwrap().is_none());
        
        // Test put and get
        let embedding = vec![1.0, 2.0, 3.0];
        cache.put("test content", embedding.clone()).unwrap();
        
        assert_eq!(cache.len().unwrap(), 1);
        assert!(!cache.is_empty().unwrap());
        
        let retrieved = cache.get("test content").unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), embedding);
        
        // Test cache miss
        assert!(cache.get("different content").unwrap().is_none());
    }

    #[test]
    fn test_cache_lru_eviction() {
        let cache = EmbeddingCache::new(2).unwrap();
        
        // Fill cache to capacity
        cache.put("content1", vec![1.0]).unwrap();
        cache.put("content2", vec![2.0]).unwrap();
        assert_eq!(cache.len().unwrap(), 2);
        
        // Add third item, should evict first
        cache.put("content3", vec![3.0]).unwrap();
        assert_eq!(cache.len().unwrap(), 2);
        
        // content1 should be evicted
        assert!(cache.get("content1").unwrap().is_none());
        assert!(cache.get("content2").unwrap().is_some());
        assert!(cache.get("content3").unwrap().is_some());
    }

    #[test]
    fn test_content_hashing() {
        let hash1 = EmbeddingCache::hash_content("test content");
        let hash2 = EmbeddingCache::hash_content("test content");
        let hash3 = EmbeddingCache::hash_content("different content");
        
        // Same content should produce same hash
        assert_eq!(hash1, hash2);
        
        // Different content should produce different hash
        assert_ne!(hash1, hash3);
        
        // Hash should be reasonable length (SHA256 produces 64 hex chars)
        assert_eq!(hash1.len(), 64);
    }

    #[test]
    fn test_batch_operations() {
        let cache = EmbeddingCache::new(10).unwrap();
        
        let contents = vec!["content1", "content2", "content3"];
        let embeddings = vec![
            vec![1.0, 1.1],
            vec![2.0, 2.1], 
            vec![3.0, 3.1],
        ];
        
        // Test batch put
        cache.put_batch(&contents, embeddings.clone()).unwrap();
        assert_eq!(cache.len().unwrap(), 3);
        
        // Test batch get
        let retrieved = cache.get_batch(&contents).unwrap();
        assert_eq!(retrieved.len(), 3);
        
        for (original, retrieved_opt) in embeddings.iter().zip(retrieved.iter()) {
            assert!(retrieved_opt.is_some());
            assert_eq!(retrieved_opt.as_ref().unwrap(), original);
        }
        
        // Test partial cache hits
        let mixed_contents = vec!["content1", "missing", "content3"];
        let mixed_retrieved = cache.get_batch(&mixed_contents).unwrap();
        
        assert!(mixed_retrieved[0].is_some());
        assert!(mixed_retrieved[1].is_none());
        assert!(mixed_retrieved[2].is_some());
    }

    #[tokio::test]
    async fn test_cache_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let cache_dir = temp_dir.path();
        
        // Create cache with persistence
        {
            let cache = EmbeddingCache::new_with_persistence(10, cache_dir).unwrap();
            cache.put("persistent content", vec![1.0, 2.0, 3.0]).unwrap();
            cache.save_to_disk().unwrap();
        }
        
        // Create new cache instance and load from disk
        {
            let cache = EmbeddingCache::new_with_persistence(10, cache_dir).unwrap();
            let retrieved = cache.get("persistent content").unwrap();
            
            assert!(retrieved.is_some());
            assert_eq!(retrieved.unwrap(), vec![1.0, 2.0, 3.0]);
        }
    }
}