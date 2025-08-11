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
// Enable working GGUF implementation
pub mod llama_wrapper_working;
pub mod simple_storage;
pub mod simple_search;
pub mod advanced_search;
pub mod markdown_metadata_extractor;

// GGUF embedding modules - now enabled
pub mod embedding_prefixes;
pub mod gguf_embedder;

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
pub use markdown_metadata_extractor::{
    MarkdownMetadataExtractor, EnhancedChunkMetadata, MarkdownSymbol, 
    DocumentOutline, LinkInfo, ImageInfo, SymbolType as MarkdownSymbolType
};

// GGUF embedding interfaces - now enabled
pub use embedding_prefixes::{EmbeddingTask, CodeFormatter, BatchProcessor};
pub use gguf_embedder::{GGUFEmbedder, GGUFEmbedderConfig, EmbedderStats};
pub use llama_wrapper_working::{GGUFModel, GGUFContext};