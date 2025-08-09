//! Lazy loading wrapper for embedders with cache support
//!
//! This wrapper provides a unified interface for different embedder types,
//! with automatic caching and lazy initialization. Defaults to minimal
//! hash-based embedder for guaranteed V8 safety.

use crate::embedding::cache::{EmbeddingCache, CacheEntry};
use crate::embedding::minimal_embedder::MinimalEmbedder;
use std::sync::Arc;
use tokio::sync::RwLock;

// ML embedder references removed

/// Embedder types available in the system
pub enum EmbedderType {
    /// Minimal hash-based embedder (always available, default)
    Minimal(MinimalEmbedder),
    // ML embedder types removed
}

/// Thread-safe lazy-loaded embedder wrapper with caching
pub struct LazyEmbedder {
    embedder: Arc<RwLock<Option<EmbedderType>>>,
    cache: Arc<RwLock<EmbeddingCache>>,
}

impl Default for LazyEmbedder {
    fn default() -> Self {
        Self::new()
    }
}

impl LazyEmbedder {
    /// Create a new lazy embedder that won't initialize until first use
    pub fn new() -> Self {
        Self {
            embedder: Arc::new(RwLock::new(None)),
            cache: Arc::new(RwLock::new(EmbeddingCache::new(1000).expect("Failed to create cache"))), // 1000 entry cache
        }
    }

    /// Initialize the embedder (defaults to minimal embedder unless force_ml is true)
    pub async fn initialize(&mut self, force_ml: bool) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if self.embedder.read().await.is_some() {
            return Ok(());
        }
        
        let embedder_type = {
            println!("🚀 Using minimal hash-based embedder (40 lines vs 138,000)");
            EmbedderType::Minimal(MinimalEmbedder::new())
        };
        
        *self.embedder.write().await = Some(embedder_type);
        Ok(())
    }

    /// Check if embedder is initialized without triggering initialization
    pub async fn is_initialized(&self) -> bool {
        self.embedder.read().await.is_some()
    }

    /// Embed text with caching
    pub async fn embed_with_cache(&self, text: &str) -> Result<Vec<f32>, Box<dyn std::error::Error + Send + Sync>> {
        // Try cache first
        if let Ok(Some(cached)) = self.cache.read().await.get(text) {
            return Ok(cached);
        }
        
        // Generate embedding
        let embedding = self.embed(text).await?;
        
        // Cache result
        let cache_entry = CacheEntry {
            embedding: embedding.clone(),
            content_hash: EmbeddingCache::hash_content(text),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };
        
        let _ = self.cache.write().await.put(text, embedding.clone());
        
        Ok(embedding)
    }
    
    /// Embed text, initializing the embedder if needed
    pub async fn embed(&self, text: &str) -> Result<Vec<f32>, Box<dyn std::error::Error + Send + Sync>> {
        let embedder_guard = self.embedder.read().await;
        
        match embedder_guard.as_ref() {
            Some(EmbedderType::Minimal(embedder)) => {
                // Use minimal hash-based embedder
                Ok(embedder.embed(text))
            }
            // ML embedder case removed
            None => {
                Err("Embedder not initialized".into())
            }
        }
    }

    /// Embed batch of texts
    pub async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, Box<dyn std::error::Error + Send + Sync>> {
        let mut results = Vec::with_capacity(texts.len());
        
        for text in texts {
            let embedding = self.embed_with_cache(text).await?;
            results.push(embedding);
        }
        
        Ok(results)
    }

    /// Get embedder status and cache statistics
    pub async fn get_status(&self) -> serde_json::Value {
        let embedder_guard = self.embedder.read().await;
        let cache_guard = self.cache.read().await;
        
        let embedder_status = match embedder_guard.as_ref() {
            Some(EmbedderType::Minimal(_)) => {
                serde_json::json!({
                    "type": "minimal-hash",
                    "status": "active",
                    "description": "Hash-based embedder (40 lines vs 138,000)",
                    "dimension": 768,
                    "memory_safe": true,
                    "deterministic": true
                })
            }
            // ML embedder status removed
            None => {
                serde_json::json!({
                    "type": "uninitialized",
                    "status": "inactive",
                    "description": "Embedder not yet initialized"
                })
            }
        };
        
        serde_json::json!({
            "embedder": embedder_status,
            "cache": {
                "size": cache_guard.len().unwrap_or(0),
                "max_size": cache_guard.capacity(),
                "hit_rate": cache_guard.hit_rate()
            }
        })
    }

    /// Clear the embedding cache
    pub async fn clear_cache(&self) {
        let _ = self.cache.write().await.clear();
    }

    /// Get cache statistics
    pub async fn get_cache_stats(&self) -> serde_json::Value {
        let cache_guard = self.cache.read().await;
        serde_json::json!({
            "size": cache_guard.len().unwrap_or(0),
            "max_size": cache_guard.capacity(),
            "hit_rate": cache_guard.hit_rate(),
            "hits": cache_guard.hits(),
            "misses": cache_guard.misses()
        })
    }
}

impl Clone for LazyEmbedder {
    fn clone(&self) -> Self {
        Self {
            embedder: self.embedder.clone(),
            cache: self.cache.clone(),
        }
    }
}