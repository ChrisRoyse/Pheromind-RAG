// Search configuration for modular searcher
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

/// Comprehensive configuration for search engines and parameters
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
    
    /// Maximum results to return for BM25 search
    pub bm25_max_results: usize,
    
    /// Maximum results to return for semantic search
    pub semantic_max_results: usize,
    
    /// Maximum results to return for exact search
    pub exact_max_results: usize,
    
    /// Maximum results to return for symbol search
    pub symbol_max_results: usize,
    
    /// Search cache size (number of cached queries)
    pub search_cache_size: usize,
    
    /// Term cache size (number of cached lowercase terms)
    pub term_cache_size: usize,
    
    /// BM25 k1 parameter (term frequency saturation)
    pub bm25_k1: f32,
    
    /// BM25 b parameter (length normalization)
    pub bm25_b: f32,
    
    /// Fusion weight for exact matches
    pub fusion_exact_weight: f32,
    
    /// Fusion weight for BM25 matches
    pub fusion_bm25_weight: f32,
    
    /// Fusion weight for semantic matches
    pub fusion_semantic_weight: f32,
    
    /// Fusion weight for symbol matches
    pub fusion_symbol_weight: f32,
    
    /// Maximum results to return for Tantivy search
    pub tantivy_max_results: usize,
    
    /// Maximum results to return for fusion search
    pub fusion_max_results: usize,
    
    /// Maximum results to return for simple search
    pub simple_max_results: usize,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            enable_bm25: true,
            enable_tantivy: cfg!(feature = "tantivy"),
            enable_ml: false, // Disabled by default due to compilation issues
            enable_tree_sitter: cfg!(feature = "tree-sitter"),
            index_path: PathBuf::from(".embed_index"),
            
            // Search result limits - explicit configuration required
            bm25_max_results: 50,
            semantic_max_results: 30,
            exact_max_results: 100,
            symbol_max_results: 25,
            
            // Cache sizes
            search_cache_size: 100,
            term_cache_size: 10000,
            
            // BM25 parameters
            bm25_k1: 1.2,
            bm25_b: 0.75,
            
            // Fusion weights
            fusion_exact_weight: 0.4,
            fusion_bm25_weight: 0.25,
            fusion_semantic_weight: 0.25,
            fusion_symbol_weight: 0.1,
            
            // Additional search limits
            tantivy_max_results: 100,
            fusion_max_results: 20,
            simple_max_results: 50,
        }
    }
}

impl SearchConfig {
    /// Create a minimal configuration with only BM25
    pub fn minimal() -> Self {
        let mut config = Self::default();
        config.enable_bm25 = true;
        config.enable_tantivy = false;
        config.enable_ml = false;
        config.enable_tree_sitter = false;
        config.index_path = PathBuf::from(".embed_index");
        // Keep all other parameters at default values
        config
    }
    
    /// Create configuration with all available features
    pub fn with_available_features() -> Self {
        let mut config = Self::default();
        config.enable_bm25 = true;
        #[cfg(feature = "tantivy")]
        { config.enable_tantivy = true; }
        #[cfg(not(feature = "tantivy"))]
        { config.enable_tantivy = false; }
        config.enable_ml = false; // Disabled due to Windows compilation issues
        #[cfg(feature = "tree-sitter")]
        { config.enable_tree_sitter = true; }
        #[cfg(not(feature = "tree-sitter"))]
        { config.enable_tree_sitter = false; }
        config.index_path = PathBuf::from(".embed_index");
        config
    }
    
    /// Validate search configuration parameters
    pub fn validate(&self) -> Result<(), String> {
        if !self.has_enabled_engines() {
            return Err("At least one search engine must be enabled".to_string());
        }
        
        if self.bm25_max_results == 0 {
            return Err("bm25_max_results must be greater than 0".to_string());
        }
        
        if self.semantic_max_results == 0 {
            return Err("semantic_max_results must be greater than 0".to_string());
        }
        
        if self.exact_max_results == 0 {
            return Err("exact_max_results must be greater than 0".to_string());
        }
        
        if self.symbol_max_results == 0 {
            return Err("symbol_max_results must be greater than 0".to_string());
        }
        
        if self.search_cache_size == 0 {
            return Err("search_cache_size must be greater than 0".to_string());
        }
        
        if self.term_cache_size == 0 {
            return Err("term_cache_size must be greater than 0".to_string());
        }
        
        if self.bm25_k1 <= 0.0 {
            return Err("bm25_k1 must be greater than 0".to_string());
        }
        
        if self.bm25_b < 0.0 || self.bm25_b > 1.0 {
            return Err("bm25_b must be between 0.0 and 1.0".to_string());
        }
        
        // Validate fusion weights sum to reasonable value
        let weight_sum = self.fusion_exact_weight + self.fusion_bm25_weight + 
                        self.fusion_semantic_weight + self.fusion_symbol_weight;
        if weight_sum <= 0.0 {
            return Err("Sum of fusion weights must be greater than 0".to_string());
        }
        
        Ok(())
    }
    
    /// Check if at least one search engine is enabled
    pub fn has_enabled_engines(&self) -> bool {
        self.enable_bm25 || self.enable_tantivy || self.enable_ml || self.enable_tree_sitter
    }
}