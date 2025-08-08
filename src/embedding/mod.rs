// Embedding cache is always available for flexibility
pub mod cache;

// Core embedding functionality requires ML feature
#[cfg(feature = "ml")]
pub mod nomic;

// Lazy loading wrapper for memory-safe initialization
pub mod lazy_embedder;

// Re-export embedding types only with ML feature
#[cfg(feature = "ml")]
pub use nomic::NomicEmbedder;

// Cache types are always available for compatibility
pub use cache::{EmbeddingCache, CacheEntry, CacheStats};

// Export lazy embedder for memory-safe initialization
pub use lazy_embedder::LazyEmbedder;

// Note: LazyEmbedder provides no-op implementation when ml feature is disabled