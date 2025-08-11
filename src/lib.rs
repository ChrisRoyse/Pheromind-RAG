// Balanced architecture - sophisticated but not over-engineered

pub mod error;
pub mod chunking;
pub mod search;
pub mod cache;
pub mod utils;
pub mod config;
pub mod indexer;
pub mod symbol_extractor;
pub mod semantic_chunker;
pub mod fusion;
pub mod embedding_cache;

// Simple modules for core functionality
pub mod simple_embedder;
pub mod llama_bindings;
pub mod llama_wrapper;
pub mod llama_wrapper_simple;
pub mod simple_storage;
pub mod simple_search;
pub mod advanced_search;

// Re-export key types
pub use error::{SearchError, Result};
pub use chunking::{Chunk, ChunkContext};
pub use search::bm25_fixed::BM25Engine;
pub use fusion::{FusionConfig, SearchResult};
pub use cache::BoundedCache;
pub use config::Config;
pub use indexer::IncrementalIndexer;
pub use symbol_extractor::{SymbolExtractor, Symbol, SymbolKind};

// Main hybrid search interface
pub use simple_search::HybridSearch;
pub use advanced_search::{AdvancedHybridSearch, AdvancedSearchResult};