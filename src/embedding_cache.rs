// High-performance embedding cache to avoid redundant computations
// Critical for meeting <100ms search latency target

use anyhow::Result;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct CachedEmbedding {
    pub embedding: Vec<f32>,
    pub timestamp: Instant,
}

pub struct EmbeddingCache {
    cache: Arc<RwLock<HashMap<u64, CachedEmbedding>>>,
    max_size: usize,
    ttl: Duration,
    hits: Arc<RwLock<u64>>,
    misses: Arc<RwLock<u64>>,
}

impl EmbeddingCache {
    pub fn new(max_size: usize, ttl_seconds: u64) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            max_size,
            ttl: Duration::from_secs(ttl_seconds),
            hits: Arc::new(RwLock::new(0)),
            misses: Arc::new(RwLock::new(0)),
        }
    }
    
    /// Get embedding from cache if available and not expired
    pub fn get(&self, text: &str) -> Option<Vec<f32>> {
        let key = self.compute_hash(text);
        let cache = self.cache.read();
        
        if let Some(cached) = cache.get(&key) {
            if cached.timestamp.elapsed() < self.ttl {
                *self.hits.write() += 1;
                return Some(cached.embedding.clone());
            }
        }
        
        *self.misses.write() += 1;
        None
    }
    
    /// Store embedding in cache
    pub fn put(&self, text: &str, embedding: Vec<f32>) {
        let key = self.compute_hash(text);
        let mut cache = self.cache.write();
        
        // Evict oldest entries if cache is full
        if cache.len() >= self.max_size {
            self.evict_oldest(&mut cache);
        }
        
        cache.insert(key, CachedEmbedding {
            embedding,
            timestamp: Instant::now(),
        });
    }
    
    /// Batch get embeddings
    pub fn get_batch(&self, texts: &[String]) -> (Vec<Option<Vec<f32>>>, Vec<usize>) {
        let mut results = Vec::with_capacity(texts.len());
        let mut miss_indices = Vec::new();
        
        for (i, text) in texts.iter().enumerate() {
            let embedding = self.get(text);
            if embedding.is_none() {
                miss_indices.push(i);
            }
            results.push(embedding);
        }
        
        (results, miss_indices)
    }
    
    /// Batch put embeddings
    pub fn put_batch(&self, texts: &[String], embeddings: Vec<Vec<f32>>) {
        for (text, embedding) in texts.iter().zip(embeddings.into_iter()) {
            self.put(text, embedding);
        }
    }
    
    /// Clear expired entries
    pub fn clear_expired(&self) {
        let mut cache = self.cache.write();
        let now = Instant::now();
        
        cache.retain(|_, v| now.duration_since(v.timestamp) < self.ttl);
    }
    
    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let hits = *self.hits.read();
        let misses = *self.misses.read();
        let total = hits + misses;
        let hit_rate = if total > 0 {
            (hits as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        
        CacheStats {
            size: self.cache.read().len(),
            max_size: self.max_size,
            hits,
            misses,
            hit_rate,
        }
    }
    
    /// Clear all cache entries
    pub fn clear(&self) {
        self.cache.write().clear();
        *self.hits.write() = 0;
        *self.misses.write() = 0;
    }
    
    fn compute_hash(&self, text: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        hasher.finish()
    }
    
    fn evict_oldest(&self, cache: &mut HashMap<u64, CachedEmbedding>) {
        // Find and remove the oldest entry
        if let Some((&oldest_key, _)) = cache
            .iter()
            .min_by_key(|(_, v)| v.timestamp)
        {
            cache.remove(&oldest_key);
        }
    }
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub size: usize,
    pub max_size: usize,
    pub hits: u64,
    pub misses: u64,
    pub hit_rate: f64,
}

/// Wrapper for embedder with caching
pub struct CachedEmbedder<E> {
    embedder: E,
    cache: EmbeddingCache,
}

impl<E> CachedEmbedder<E> {
    pub fn new(embedder: E, cache_size: usize, ttl_seconds: u64) -> Self {
        Self {
            embedder,
            cache: EmbeddingCache::new(cache_size, ttl_seconds),
        }
    }
    
    pub fn cache_stats(&self) -> CacheStats {
        self.cache.stats()
    }
    
    pub fn clear_cache(&self) {
        self.cache.clear()
    }
}

// Implement for NomicEmbedder
use crate::simple_embedder::NomicEmbedder;

impl CachedEmbedder<NomicEmbedder> {
    pub fn embed(&mut self, text: &str) -> Result<Vec<f32>> {
        // Check cache first
        if let Some(embedding) = self.cache.get(text) {
            return Ok(embedding);
        }
        
        // Generate embedding and cache it
        let embedding = self.embedder.embed(text)?;
        self.cache.put(text, embedding.clone());
        
        Ok(embedding)
    }
    
    pub fn embed_query(&mut self, query: &str) -> Result<Vec<f32>> {
        // Queries use different prefix, so cache separately
        let cache_key = format!("query:{}", query);
        
        if let Some(embedding) = self.cache.get(&cache_key) {
            return Ok(embedding);
        }
        
        let embedding = self.embedder.embed_query(query)?;
        self.cache.put(&cache_key, embedding.clone());
        
        Ok(embedding)
    }
    
    pub fn embed_batch(&mut self, texts: Vec<String>) -> Result<Vec<Vec<f32>>> {
        let (cached_results, miss_indices) = self.cache.get_batch(&texts);
        
        if miss_indices.is_empty() {
            // All embeddings were cached
            return Ok(cached_results.into_iter().map(|o| o.unwrap()).collect());
        }
        
        // Get texts that need embedding
        let texts_to_embed: Vec<String> = miss_indices
            .iter()
            .map(|&i| texts[i].clone())
            .collect();
        
        // Generate missing embeddings
        let new_embeddings = self.embedder.embed_batch(texts_to_embed.clone())?;
        
        // Cache the new embeddings
        self.cache.put_batch(&texts_to_embed, new_embeddings.clone());
        
        // Combine cached and new results
        let mut results = Vec::with_capacity(texts.len());
        let mut new_embedding_iter = new_embeddings.into_iter();
        
        for (_i, cached) in cached_results.into_iter().enumerate() {
            if let Some(embedding) = cached {
                results.push(embedding);
            } else {
                results.push(new_embedding_iter.next().unwrap());
            }
        }
        
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cache_operations() {
        let cache = EmbeddingCache::new(100, 60);
        
        // Test put and get
        let embedding = vec![0.1, 0.2, 0.3];
        cache.put("test", embedding.clone());
        
        let retrieved = cache.get("test");
        assert_eq!(retrieved, Some(embedding));
        
        // Test cache miss
        let miss = cache.get("not_cached");
        assert_eq!(miss, None);
        
        // Check stats
        let stats = cache.stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.size, 1);
    }
    
    #[test]
    fn test_cache_eviction() {
        let cache = EmbeddingCache::new(2, 60);
        
        cache.put("text1", vec![0.1]);
        cache.put("text2", vec![0.2]);
        cache.put("text3", vec![0.3]); // Should evict text1
        
        assert_eq!(cache.get("text1"), None);
        assert_eq!(cache.get("text2"), Some(vec![0.2]));
        assert_eq!(cache.get("text3"), Some(vec![0.3]));
    }
    
    #[test]
    fn test_batch_operations() {
        let cache = EmbeddingCache::new(100, 60);
        
        let texts = vec![
            "text1".to_string(),
            "text2".to_string(),
            "text3".to_string(),
        ];
        
        let embeddings = vec![
            vec![0.1],
            vec![0.2],
            vec![0.3],
        ];
        
        cache.put_batch(&texts[..2], embeddings[..2].to_vec());
        
        let (results, miss_indices) = cache.get_batch(&texts);
        
        assert_eq!(miss_indices, vec![2]);
        assert_eq!(results[0], Some(vec![0.1]));
        assert_eq!(results[1], Some(vec![0.2]));
        assert_eq!(results[2], None);
    }
}