use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};
use crate::config::Config;
use crate::chunking::{ChunkContext};
// Use MatchType from fusion module
use crate::search::fusion::MatchType;

// Define SearchResult locally to avoid circular dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub file: String,
    pub three_chunk_context: ChunkContext,
    pub score: f32,
    pub match_type: MatchType,
    // Backward compatibility fields for tests
    #[serde(skip)]
    pub file_path: String,
    #[serde(skip)]
    pub content: String,
}

impl SearchResult {
    /// Create a new SearchResult with backward compatibility fields populated
    pub fn new(file: String, three_chunk_context: ChunkContext, score: f32, match_type: MatchType) -> Self {
        let file_path = file.clone();
        let content = three_chunk_context.target.content.clone();
        
        Self {
            file,
            three_chunk_context,
            score,
            match_type,
            file_path,
            content,
        }
    }
    
    // Backward compatibility methods for tests
    pub fn file_path(&self) -> &str {
        &self.file
    }
    
    pub fn content(&self) -> &str {
        &self.three_chunk_context.target.content
    }
}

struct CacheEntry {
    results: Vec<SearchResult>,
    timestamp: Instant,
}

pub struct SearchCache {
    cache: Arc<Mutex<HashMap<String, CacheEntry>>>,
    max_size: usize,
    ttl: Duration,
}

impl SearchCache {
    /// Create a new search cache using configuration values
    /// Returns an error if configuration is not properly initialized
    pub fn from_config() -> Result<Self, crate::error::EmbedError> {
        let config = Config::get()?;
        Ok(Self::new(config.search_cache_size))
    }

    pub fn new(max_size: usize) -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::default())),
            max_size,
            ttl: Duration::from_secs(300), // 5 minutes TTL
        }
    }
    
    pub fn with_ttl(max_size: usize, ttl_seconds: u64) -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::default())),
            max_size,
            ttl: Duration::from_secs(ttl_seconds),
        }
    }
    
    pub fn get(&self, query: &str) -> Option<Vec<SearchResult>> {
        let mut cache = match self.cache.lock() {
            Ok(cache) => cache,
            Err(_) => {
                // Cache mutex is poisoned - cannot proceed
                panic!("FATAL: Search cache mutex is poisoned. Cache operations cannot continue.");
            }
        };
        
        if let Some(entry) = cache.get(query) {
            // Check if entry is still valid
            if entry.timestamp.elapsed() < self.ttl {
                return Some(entry.results.clone());
            } else {
                // Remove expired entry
                cache.remove(query);
            }
        }
        
        None
    }
    
    pub fn insert(&self, query: String, results: Vec<SearchResult>) -> Result<(), crate::error::EmbedError> {
        let mut cache = match self.cache.lock() {
            Ok(cache) => cache,
            Err(e) => {
                return Err(crate::error::EmbedError::Concurrency {
                    message: format!("Failed to acquire cache lock for insert operation: {}. Cache is poisoned.", e),
                    operation: Some("cache_insert".to_string()),
                });
            }
        };
        
        // Implement simple LRU by removing oldest entries if at capacity
        if cache.len() >= self.max_size {
            // Find and remove the oldest entry
            if let Some(oldest_key) = cache
                .iter()
                .min_by_key(|(_, entry)| entry.timestamp)
                .map(|(key, _)| key.clone())
            {
                cache.remove(&oldest_key);
            }
        }
        
        cache.insert(query, CacheEntry {
            results,
            timestamp: Instant::now(),
        });
        
        Ok(())
    }
    
    pub fn clear(&self) -> Result<(), crate::error::EmbedError> {
        let mut cache = match self.cache.lock() {
            Ok(cache) => cache,
            Err(e) => {
                return Err(crate::error::EmbedError::Concurrency {
                    message: format!("Failed to acquire cache lock for clear operation: {}. Cache is poisoned.", e),
                    operation: Some("cache_clear".to_string()),
                });
            }
        };
        cache.clear();
        Ok(())
    }
    
    pub fn stats(&self) -> Result<CacheStats, crate::error::EmbedError> {
        let cache = self.cache.lock().map_err(|_| {
            crate::error::EmbedError::Internal {
                message: "FATAL: Search cache mutex is poisoned. Cache operations cannot continue.".to_string(),
                backtrace: None,
            }
        })?;
        let valid_entries = cache
            .values()
            .filter(|entry| entry.timestamp.elapsed() < self.ttl)
            .count();
        
        Ok(CacheStats {
            total_entries: cache.len(),
            valid_entries,
            max_size: self.max_size,
        })
    }
}

pub struct CacheStats {
    pub total_entries: usize,
    pub valid_entries: usize,
    pub max_size: usize,
}

impl std::fmt::Display for CacheStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Cache Stats: {}/{} entries ({} valid)",
            self.total_entries, self.max_size, self.valid_entries
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunking::{Chunk, ChunkContext};
    
    fn create_test_result() -> SearchResult {
        SearchResult::new(
            "test.rs".to_string(),
            ChunkContext {
                above: None,
                target: Chunk {
                    content: "test content".to_string(),
                    start_line: 1,
                    end_line: 1,
                },
                below: None,
                target_index: 0,
            },
            1.0,
            crate::search::fusion::MatchType::Exact,
        )
    }
    
    #[test]
    fn test_cache_basic_operations() {
        let cache = SearchCache::new(10);
        let query = "test query";
        let results = vec![create_test_result()];
        
        // Test insert and get
        cache.insert(query.to_string(), results.clone()).unwrap();
        let cached = cache.get(query);
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().len(), 1);
        
        // Test cache miss
        let missing = cache.get("nonexistent");
        assert!(missing.is_none());
    }
    
    #[test]
    fn test_cache_expiration() {
        let cache = SearchCache::with_ttl(10, 0); // 0 second TTL
        let query = "test query";
        let results = vec![create_test_result()];
        
        cache.insert(query.to_string(), results).unwrap();
        
        // Sleep to ensure expiration
        std::thread::sleep(Duration::from_millis(10));
        
        let cached = cache.get(query);
        assert!(cached.is_none()); // Should be expired
    }
    
    #[test]
    fn test_cache_size_limit() {
        let cache = SearchCache::new(2);
        
        cache.insert("query1".to_string(), vec![create_test_result()]).unwrap();
        cache.insert("query2".to_string(), vec![create_test_result()]).unwrap();
        cache.insert("query3".to_string(), vec![create_test_result()]).unwrap();
        
        let stats = cache.stats()
            .expect("Cache stats should be available");
        assert!(stats.total_entries <= 2); // Should not exceed max size
    }
}