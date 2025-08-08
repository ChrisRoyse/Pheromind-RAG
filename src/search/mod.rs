use serde::{Deserialize, Serialize};

/// A search match result containing file information and match details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExactMatch {
    pub file_path: String,
    pub line_number: usize,
    pub content: String,
    pub line_content: String,
}

// Declare modules in dependency order (no circular refs)
pub mod native_search;     // ✅ No dependencies
pub mod preprocessing;     // ✅ No dependencies
pub mod bm25;              // ✅ No dependencies
pub mod text_processor;    // ✅ No dependencies
pub mod inverted_index;    // ✅ Depends only on bm25
pub mod cache;             // ✅ Depends only on basic types
pub mod fusion;            // ✅ Depends on bm25
pub mod config;            // ✅ Search configuration
pub mod simple_searcher;   // ✅ Basic searcher (BM25+Tantivy) with graceful degradation - NO Tree-sitter by design
pub mod bm25_fixed;        // ✅ Fixed BM25 implementation
pub mod unified;           // ✅ Depends on everything above
#[cfg(feature = "tree-sitter")]
pub mod symbol_index;
#[cfg(feature = "tree-sitter")]
pub mod symbol_enhanced_searcher;
#[cfg(feature = "tantivy")]
pub mod tantivy_search;
#[cfg(feature = "tantivy")]
pub mod search_adapter;

// Re-export ONLY non-circular items
pub use native_search::{NativeSearcher, SearchMatch};
pub use preprocessing::QueryPreprocessor;
pub use bm25::{BM25Engine, BM25Match, BM25Document, Token as BM25Token};
pub use text_processor::{CodeTextProcessor, ProcessedToken, TokenType};
pub use inverted_index::{InvertedIndex, DocumentMetadata};
pub use fusion::{SimpleFusion, FusedResult, MatchType};
pub use unified::UnifiedSearcher;
pub use config::SearchConfig;
pub use simple_searcher::{SimpleSearcher, SimpleSearchResult};
pub use bm25_fixed::BM25Engine as BM25EngineFixed;
pub use cache::SearchResult;
// DO NOT re-export cache::SearchCache - causes circular deps
#[cfg(feature = "tree-sitter")]
pub use symbol_index::{SymbolIndexer, SymbolDatabase, Symbol, SymbolKind};
#[cfg(feature = "tantivy")]
pub use tantivy_search::TantivySearcher;
#[cfg(feature = "tantivy")]
pub use search_adapter::{TextSearcher, create_text_searcher, create_text_searcher_with_root};