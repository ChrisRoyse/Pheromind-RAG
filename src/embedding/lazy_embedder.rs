/// Lazy loading wrapper for NomicEmbedder to prevent memory issues in Node.js environments
/// 
/// This wrapper delays the initialization of the embedder until it's actually needed,
/// preventing V8 heap allocation errors when running in MCP server contexts.

use std::sync::Arc;
use tokio::sync::OnceCell;
use anyhow::Result;

#[cfg(feature = "ml")]
use super::nomic::NomicEmbedder;

/// Thread-safe lazy-loaded embedder wrapper
#[cfg(feature = "ml")]
pub struct LazyEmbedder {
    inner: Arc<OnceCell<Arc<NomicEmbedder>>>,
}

#[cfg(feature = "ml")]
impl LazyEmbedder {
    /// Create a new lazy embedder that won't initialize until first use
    pub fn new() -> Self {
        Self {
            inner: Arc::new(OnceCell::new()),
        }
    }
    
    /// Get or initialize the embedder
    /// This will only load the model on first access, preventing unnecessary memory usage
    pub async fn get_or_init(&self) -> Result<Arc<NomicEmbedder>, crate::error::EmbedError> {
        // Try to get existing embedder first
        if let Some(embedder) = self.inner.get() {
            return Ok(embedder.clone());
        }
        
        // Initialize if not already done
        let embedder = NomicEmbedder::get_global().await?;
        
        // Try to set it, but if someone else already did, use theirs
        match self.inner.set(embedder.clone()) {
            Ok(_) => Ok(embedder),
            Err(_) => {
                // Another thread initialized it, use that one
                Ok(self.inner.get().unwrap().clone())
            }
        }
    }
    
    /// Check if embedder is initialized without triggering initialization
    pub fn is_initialized(&self) -> bool {
        self.inner.get().is_some()
    }
    
    /// Embed text, initializing the embedder if needed
    pub async fn embed(&self, text: &str) -> Result<Vec<f32>, crate::error::EmbedError> {
        let embedder = self.get_or_init().await?;
        embedder.embed(text).map_err(|e| crate::error::EmbedError::Internal {
            message: format!("Failed to embed text: {}", e),
            backtrace: None,
        })
    }
    
    /// Embed batch of texts, initializing the embedder if needed
    pub async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, crate::error::EmbedError> {
        let embedder = self.get_or_init().await?;
        let text_refs: Vec<&str> = texts.iter().map(|s| s.as_str()).collect();
        embedder.embed_batch(&text_refs).map_err(|e| crate::error::EmbedError::Internal {
            message: format!("Failed to embed batch: {}", e),
            backtrace: None,
        })
    }
}

#[cfg(feature = "ml")]
impl Clone for LazyEmbedder {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

// Provide a no-op implementation when ml feature is disabled
#[cfg(not(feature = "ml"))]
pub struct LazyEmbedder;

#[cfg(not(feature = "ml"))]
impl LazyEmbedder {
    pub fn new() -> Self {
        Self
    }
    
    pub async fn get_or_init(&self) -> Result<(), crate::error::EmbedError> {
        Err(crate::error::EmbedError::Internal {
            message: "ML features are not enabled".to_string(),
            backtrace: None,
        })
    }
    
    pub fn is_initialized(&self) -> bool {
        false
    }
    
    pub async fn embed(&self, _text: &str) -> Result<Vec<f32>, crate::error::EmbedError> {
        Err(crate::error::EmbedError::Internal {
            message: "ML features are not enabled".to_string(),
            backtrace: None,
        })
    }
    
    pub async fn embed_batch(&self, _texts: &[String]) -> Result<Vec<Vec<f32>>, crate::error::EmbedError> {
        Err(crate::error::EmbedError::Internal {
            message: "ML features are not enabled".to_string(),
            backtrace: None,
        })
    }
}

#[cfg(not(feature = "ml"))]
impl Clone for LazyEmbedder {
    fn clone(&self) -> Self {
        Self
    }
}