// Search configuration for modular searcher
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

/// Configuration for search engines with feature flags
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConfig {
    /// Enable BM25 text search
    pub enable_bm25: bool,
    
    /// Enable Tantivy full-text search
    pub enable_tantivy: bool,
    
    /// Enable ML-based semantic search
    pub enable_ml: bool,
    
    /// Enable Tree-sitter symbol search
    pub enable_tree_sitter: bool,
    
    /// Path for search index storage
    pub index_path: PathBuf,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            enable_bm25: true,
            enable_tantivy: cfg!(feature = "tantivy"),
            enable_ml: false, // Disabled by default due to compilation issues
            enable_tree_sitter: cfg!(feature = "tree-sitter"),
            index_path: PathBuf::from(".embed_index"),
        }
    }
}

impl SearchConfig {
    /// Create a minimal configuration with only BM25
    pub fn minimal() -> Self {
        Self {
            enable_bm25: true,
            enable_tantivy: false,
            enable_ml: false,
            enable_tree_sitter: false,
            index_path: PathBuf::from(".embed_index"),
        }
    }
    
    /// Create configuration with all available features
    pub fn with_available_features() -> Self {
        Self {
            enable_bm25: true,
            #[cfg(feature = "tantivy")]
            enable_tantivy: true,
            #[cfg(not(feature = "tantivy"))]
            enable_tantivy: false,
            enable_ml: false, // Disabled due to Windows compilation issues
            #[cfg(feature = "tree-sitter")]
            enable_tree_sitter: true,
            #[cfg(not(feature = "tree-sitter"))]
            enable_tree_sitter: false,
            index_path: PathBuf::from(".embed_index"),
        }
    }
    
    /// Check if at least one search engine is enabled
    pub fn has_enabled_engines(&self) -> bool {
        self.enable_bm25 || self.enable_tantivy || self.enable_ml || self.enable_tree_sitter
    }
}