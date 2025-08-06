use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use crate::config::Config;
use crate::chunking::{ChunkContext};
// Use MatchType from fusion module
use crate::search::fusion::MatchType;

// Define SearchResult locally to avoid circular dependency
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub file: String,
    pub three_chunk_context: ChunkContext,
    pub score: f32,
    pub match_type: MatchType,
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
    pub fn from_config() -> Self {
        let config = Config::get().unwrap_or_default();
        Self::new(config.search_cache_size)
    }

    pub fn new(max_size: usize) -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
            max_size,
            ttl: Duration::from_secs(300), // 5 minutes TTL
        }
    }
    
    pub fn with_ttl(max_size: usize, ttl_seconds: u64) -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
            max_size,
            ttl: Duration::from_secs(ttl_seconds),
        }
    }
    
    pub fn get(&self, query: &str) -> Option<Vec<SearchResult>> {
        let mut cache = self.cache.lock().unwrap();
        
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
    
    pub fn insert(&self, query: String, results: Vec<SearchResult>) {
        let mut cache = self.cache.lock().unwrap();
        
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
    }
    
    pub fn clear(&self) {
        let mut cache = self.cache.lock().unwrap();
        cache.clear();
    }
    
    pub fn stats(&self) -> CacheStats {
        let cache = self.cache.lock().unwrap();
        let valid_entries = cache
            .values()
            .filter(|entry| entry.timestamp.elapsed() < self.ttl)
            .count();
        
        CacheStats {
            total_entries: cache.len(),
            valid_entries,
            max_size: self.max_size,
        }
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
    use crate::chunking::{Chunk, ThreeChunkExpander, ChunkContext};
    
    fn create_test_result() -> SearchResult {
        SearchResult {
            file: "test.rs".to_string(),
            three_chunk_context: ChunkContext {
                above: None,
                target: Chunk {
                    content: "test content".to_string(),
                    start_line: 1,
                    end_line: 1,
                },
                below: None,
                target_index: 0,
            },
            score: 1.0,
            match_type: crate::search::fusion::MatchType::Exact,
        }
    }
    
    #[test]
    fn test_cache_basic_operations() {
        let cache = SearchCache::new(10);
        let query = "test query";
        let results = vec![create_test_result()];
        
        // Test insert and get
        cache.insert(query.to_string(), results.clone());
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
        
        cache.insert(query.to_string(), results);
        
        // Sleep to ensure expiration
        std::thread::sleep(Duration::from_millis(10));
        
        let cached = cache.get(query);
        assert!(cached.is_none()); // Should be expired
    }
    
    #[test]
    fn test_cache_size_limit() {
        let cache = SearchCache::new(2);
        
        cache.insert("query1".to_string(), vec![create_test_result()]);
        cache.insert("query2".to_string(), vec![create_test_result()]);
        cache.insert("query3".to_string(), vec![create_test_result()]);
        
        let stats = cache.stats();
        assert!(stats.total_entries <= 2); // Should not exceed max size
    }
}