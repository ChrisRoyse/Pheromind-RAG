// MINIMAL EMBEDDER - The ONLY embedder we need
mod minimal_embedder;
pub mod cache;
pub mod lazy_embedder;

// PRIMARY EXPORT - MinimalEmbedder is the default
pub use minimal_embedder::MinimalEmbedder;

// Cache and lazy loading
pub use cache::{EmbeddingCache, CacheEntry, CacheStats};
pub use lazy_embedder::LazyEmbedder;