pub mod safe_vectordb;
pub mod simple_vectordb;
pub mod lightweight_storage;
#[cfg(feature = "vectordb")]
pub mod lancedb_storage;
#[cfg(feature = "vectordb")]
pub mod lancedb;

pub use safe_vectordb::{VectorStorage, StorageConfig};
pub use simple_vectordb::{StorageError, EmbeddingRecord, VectorSchema};
#[cfg(feature = "vectordb")]
pub use lancedb_storage::{LanceDBStorage, LanceStorageError, LanceEmbeddingRecord};