// Embedding cache is always available for flexibility
pub mod cache;

// Core embedding functionality requires ML feature
#[cfg(feature = "ml")]
pub mod nomic;

// Re-export embedding types only with ML feature
#[cfg(feature = "ml")]
pub use nomic::NomicEmbedder;

// Cache types are always available for compatibility
pub use cache::{EmbeddingCache, CacheEntry, CacheStats};

// Compile-time error when trying to use embeddings without ML feature
#[cfg(not(feature = "ml"))]
compile_error!("Embedding functionality requires 'ml' feature. Enable with --features=ml or add ml to your Cargo.toml");