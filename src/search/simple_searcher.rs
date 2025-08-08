// SimpleSearcher - A modular search implementation with graceful degradation
// Follows SPARC principles and TDD methodology
//
// ARCHITECTURAL DESIGN DECISION:
// SimpleSearcher is intentionally designed as a basic, robust searcher that provides
// graceful degradation when optional features are unavailable. It implements only:
// - BM25 statistical search (always available, pure Rust)
// - Tantivy exact text search (optional feature)
//
// Tree-sitter symbol search is intentionally NOT implemented here. This is by design.
// For comprehensive search including Tree-sitter symbol analysis, use UnifiedSearcher.
// This separation allows:
// 1. Minimal dependencies for basic search needs
// 2. Graceful degradation when advanced features are unavailable
// 3. Clear separation of concerns between basic and advanced search capabilities
//
// See UnifiedSearcher in src/search/unified.rs for the complete feature set including:
// - Tree-sitter symbol indexing and search (lines 27-28, 42-45, 386-407, 496-531)
// - Semantic search with ML embeddings
// - Advanced fusion algorithms

use anyhow::{Result, Context};
use std::collections::HashSet;
use std::path::PathBuf;
use tracing::{info, warn};

use crate::search::config::SearchConfig;
use crate::search::bm25_fixed::BM25Engine;

#[cfg(feature = "tantivy")]
use crate::search::tantivy_search::TantivySearcher;

// Tree-sitter integration: ARCHITECTURAL DESIGN DECISION
// Tree-sitter functionality is intentionally NOT implemented in SimpleSearcher.
// This is by design - SimpleSearcher provides basic search with graceful degradation,
// while comprehensive Tree-sitter symbol search is implemented in UnifiedSearcher.
// See: src/search/unified.rs lines 27-28, 42-45, 386-407, 496-531 for the actual implementation.
// #[cfg(feature = "tree-sitter")]
// use crate::search::tree_sitter_service::TreeSitterService;

/// Search result from SimpleSearcher
#[derive(Debug, Clone)]
pub struct SimpleSearchResult {
    pub path: String,
    pub content: String,
    pub score: f32,
    pub line_number: Option<usize>,
    pub engines_used: HashSet<String>,
}

/// A simple, modular searcher that gracefully degrades
/// when optional features are unavailable
pub struct SimpleSearcher {
    config: SearchConfig,
    bm25_engine: Option<BM25Engine>,
    #[cfg(feature = "tantivy")]
    tantivy_engine: Option<TantivySearcher>,
    // Tree-sitter service intentionally excluded from SimpleSearcher - see above architectural comment
    // #[cfg(feature = "tree-sitter")]
    // tree_sitter_service: Option<TreeSitterService>,
    engines_available: HashSet<String>,
}

impl SimpleSearcher {
    /// Create a new SimpleSearcher with graceful degradation
    pub async fn new(config: SearchConfig) -> Result<Self> {
        let mut engines_available = HashSet::new();
        
        // Always try to initialize BM25 (pure Rust, no dependencies)
        let bm25_engine = if config.enable_bm25 {
            match BM25Engine::new() {
                Ok(engine) => {
                    info!("BM25 engine initialized successfully");
                    engines_available.insert("bm25".to_string());
                    Some(engine)
                }
                Err(e) => {
                    warn!("BM25 engine failed to initialize: {}", e);
                    None
                }
            }
        } else {
            None
        };
        
        // Try to initialize Tantivy if feature is enabled
        #[cfg(feature = "tantivy")]
        let tantivy_engine = if config.enable_tantivy {
            match TantivySearcher::new_with_path(&config.index_path).await {
                Ok(engine) => {
                    info!("Tantivy engine initialized successfully");
                    engines_available.insert("tantivy".to_string());
                    Some(engine)
                }
                Err(e) => {
                    warn!("Tantivy engine failed to initialize: {}", e);
                    None
                }
            }
        } else {
            None
        };
        
        // Tree-sitter service: INTENTIONAL ARCHITECTURAL EXCLUSION
        // Tree-sitter symbol search is NOT implemented in SimpleSearcher by design.
        // For symbol search capabilities, use UnifiedSearcher which provides full Tree-sitter integration.
        // #[cfg(feature = "tree-sitter")]
        // let tree_sitter_service = if config.enable_tree_sitter { ... };
        
        // Ensure at least one engine is available
        if engines_available.is_empty() {
            return Err(anyhow::anyhow!(
                "No search engines could be initialized. At least one engine is required."
            ));
        }
        
        info!("SimpleSearcher initialized with engines: {:?}", engines_available);
        
        Ok(Self {
            config,
            bm25_engine,
            #[cfg(feature = "tantivy")]
            tantivy_engine,
            // Tree-sitter service excluded by design - use UnifiedSearcher for symbol search
            // #[cfg(feature = "tree-sitter")]
            // tree_sitter_service,
            engines_available,
        })
    }
    
    /// Search using all available engines and combine results
    pub async fn search(&self, query: &str) -> Result<Vec<SimpleSearchResult>> {
        if query.trim().is_empty() {
            return Ok(Vec::new());
        }
        
        let mut all_results = Vec::new();
        let mut engines_used = HashSet::new();
        
        // Search with BM25 if available
        if let Some(ref engine) = self.bm25_engine {
            match engine.search(query, 20) {
                Ok(results) => {
                    engines_used.insert("bm25".to_string());
                    for result in results {
                        all_results.push(SimpleSearchResult {
                            path: result.path.clone(),
                            content: result.snippet.clone(),
                            score: result.score,
                            line_number: result.line_number,
                            engines_used: engines_used.clone(),
                        });
                    }
                }
                Err(e) => {
                    warn!("BM25 search failed: {}", e);
                }
            }
        }
        
        // Search with Tantivy if available
        #[cfg(feature = "tantivy")]
        if let Some(ref engine) = self.tantivy_engine {
            match engine.search(query).await {
                Ok(results) => {
                    engines_used.insert("tantivy".to_string());
                    for result in results {
                        all_results.push(SimpleSearchResult {
                            path: result.file_path.clone(),
                            content: result.content.clone(),
                            score: 1.0, // Tantivy doesn't provide score in ExactMatch
                            line_number: Some(result.line_number),
                            engines_used: engines_used.clone(),
                        });
                    }
                }
                Err(e) => {
                    warn!("Tantivy search failed: {}", e);
                }
            }
        }
        
        // Tree-sitter search: INTENTIONAL OMISSION
        // Symbol search with Tree-sitter is not implemented in SimpleSearcher by design.
        // SimpleSearcher focuses on graceful degradation with basic search engines (BM25 + Tantivy).
        // For comprehensive symbol search, use UnifiedSearcher which includes full Tree-sitter integration.
        // #[cfg(feature = "tree-sitter")]
        // if let Some(ref service) = self.tree_sitter_service { ... }
        
        // Deduplicate and sort by score
        let mut seen = HashSet::new();
        all_results.retain(|r| seen.insert(r.path.clone()));
        all_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        
        // Limit results
        all_results.truncate(50);
        
        Ok(all_results)
    }
    
    /// Index a project directory
    pub async fn index_project(&mut self, project_path: &PathBuf) -> Result<()> {
        info!("Indexing project: {:?}", project_path);
        
        // Index with BM25
        if let Some(ref mut engine) = self.bm25_engine {
            engine.index_directory(project_path)
                .context("Failed to index with BM25")?;
        }
        
        // Index with Tantivy
        #[cfg(feature = "tantivy")]
        if let Some(ref mut engine) = self.tantivy_engine {
            engine.index_directory(project_path).await
                .context("Failed to index with Tantivy")?;
        }
        
        Ok(())
    }
    
    /// Check if a query looks like a symbol name
    fn looks_like_symbol(query: &str) -> bool {
        // Simple heuristic: contains :: or starts with uppercase or contains _
        query.contains("::") || 
        query.chars().next().map_or(false, |c| c.is_uppercase()) ||
        query.contains('_')
    }
    
    /// Get available search engines
    pub fn available_engines(&self) -> &HashSet<String> {
        &self.engines_available
    }
}

