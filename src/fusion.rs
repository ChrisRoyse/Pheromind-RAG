// Configurable fusion algorithm for hybrid search
// Production-ready implementation based on research

// anyhow::Result temporarily removed
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FusionConfig {
    /// Weight for BM25/text search results (0.0 - 1.0)
    pub text_weight: f32,
    
    /// Weight for vector/semantic search results (0.0 - 1.0)
    pub vector_weight: f32,
    
    /// Weight for symbol/AST search results (0.0 - 1.0)
    pub symbol_weight: f32,
    
    /// Weight for fuzzy match results (0.0 - 1.0)
    pub fuzzy_weight: f32,
    
    /// RRF k parameter (typically 60)
    pub rrf_k: f32,
    
    /// Boost factor for results that appear in multiple searches
    pub hybrid_boost: f32,
    
    /// Maximum results to return
    pub max_results: usize,
}

impl Default for FusionConfig {
    fn default() -> Self {
        Self {
            text_weight: 0.25,
            vector_weight: 0.40,
            symbol_weight: 0.25,
            fuzzy_weight: 0.10,
            rrf_k: 60.0,
            hybrid_boost: 1.5,
            max_results: 20,
        }
    }
}

impl FusionConfig {
    /// Create config optimized for code search
    pub fn code_search() -> Self {
        Self {
            text_weight: 0.20,
            vector_weight: 0.35,
            symbol_weight: 0.35,  // Higher weight for symbols in code
            fuzzy_weight: 0.10,
            rrf_k: 60.0,
            hybrid_boost: 1.8,
            max_results: 25,
        }
    }
    
    /// Create config optimized for natural language queries
    pub fn natural_language() -> Self {
        Self {
            text_weight: 0.30,
            vector_weight: 0.50,  // Higher weight for semantic search
            symbol_weight: 0.15,
            fuzzy_weight: 0.05,
            rrf_k: 60.0,
            hybrid_boost: 1.4,
            max_results: 20,
        }
    }
    
    /// Normalize weights to sum to 1.0
    pub fn normalize(&mut self) {
        let sum = self.text_weight + self.vector_weight + self.symbol_weight + self.fuzzy_weight;
        if sum > 0.0 {
            self.text_weight /= sum;
            self.vector_weight /= sum;
            self.symbol_weight /= sum;
            self.fuzzy_weight /= sum;
        }
    }
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub content: String,
    pub file_path: String,
    pub score: f32,
    pub match_type: MatchType,
    pub line_number: Option<usize>,
    pub symbols: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MatchType {
    Text,
    Vector,
    Symbol,
    Fuzzy,
    Hybrid,
}

pub struct FusionEngine {
    config: FusionConfig,
}

impl FusionEngine {
    pub fn new(config: FusionConfig) -> Self {
        let mut config = config;
        config.normalize();
        Self { config }
    }
    
    /// Fuse results from multiple search types using configurable weights
    pub fn fuse_results(
        &self,
        text_results: Vec<SearchResult>,
        vector_results: Vec<SearchResult>,
        symbol_results: Vec<SearchResult>,
        fuzzy_results: Vec<SearchResult>,
    ) -> Vec<SearchResult> {
        let mut score_map: HashMap<String, FusedResult> = HashMap::new();
        
        // Process text search results
        self.add_results_to_map(
            &mut score_map,
            text_results,
            self.config.text_weight,
            MatchType::Text,
        );
        
        // Process vector search results
        self.add_results_to_map(
            &mut score_map,
            vector_results,
            self.config.vector_weight,
            MatchType::Vector,
        );
        
        // Process symbol search results
        self.add_results_to_map(
            &mut score_map,
            symbol_results,
            self.config.symbol_weight,
            MatchType::Symbol,
        );
        
        // Process fuzzy search results
        self.add_results_to_map(
            &mut score_map,
            fuzzy_results,
            self.config.fuzzy_weight,
            MatchType::Fuzzy,
        );
        
        // Convert to final results and sort
        let mut final_results: Vec<SearchResult> = score_map
            .into_values()
            .map(|fused| {
                let match_type = if fused.match_count > 1 {
                    MatchType::Hybrid
                } else {
                    fused.primary_type
                };
                
                let final_score = if match_type == MatchType::Hybrid {
                    fused.combined_score * self.config.hybrid_boost
                } else {
                    fused.combined_score
                };
                
                SearchResult {
                    content: fused.content,
                    file_path: fused.file_path,
                    score: final_score,
                    match_type,
                    line_number: fused.line_number,
                    symbols: fused.symbols,
                }
            })
            .collect();
        
        // Sort by score descending
        final_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        
        // Limit results
        final_results.truncate(self.config.max_results);
        
        final_results
    }
    
    /// RRF fusion algorithm with configurable k parameter
    pub fn rrf_fusion(
        &self,
        results_lists: Vec<Vec<SearchResult>>,
        weights: Vec<f32>,
    ) -> Vec<SearchResult> {
        let mut score_map: HashMap<String, FusedResult> = HashMap::new();
        
        for (results, weight) in results_lists.iter().zip(weights.iter()) {
            for (rank, result) in results.iter().enumerate() {
                let rrf_score = weight / (self.config.rrf_k + rank as f32 + 1.0);
                let key = Self::create_result_key(&result);
                
                score_map.entry(key)
                    .and_modify(|e| {
                        e.combined_score += rrf_score;
                        e.match_count += 1;
                    })
                    .or_insert(FusedResult {
                        content: result.content.clone(),
                        file_path: result.file_path.clone(),
                        combined_score: rrf_score,
                        match_count: 1,
                        primary_type: result.match_type.clone(),
                        line_number: result.line_number,
                        symbols: result.symbols.clone(),
                    });
            }
        }
        
        // Convert and sort
        let mut final_results: Vec<SearchResult> = score_map
            .into_values()
            .map(|fused| SearchResult {
                content: fused.content,
                file_path: fused.file_path,
                score: fused.combined_score,
                match_type: if fused.match_count > 1 { MatchType::Hybrid } else { fused.primary_type },
                line_number: fused.line_number,
                symbols: fused.symbols,
            })
            .collect();
        
        final_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        final_results.truncate(self.config.max_results);
        
        final_results
    }
    
    fn add_results_to_map(
        &self,
        score_map: &mut HashMap<String, FusedResult>,
        results: Vec<SearchResult>,
        weight: f32,
        match_type: MatchType,
    ) {
        for (rank, result) in results.iter().enumerate() {
            let rrf_score = weight / (self.config.rrf_k + rank as f32 + 1.0);
            let key = Self::create_result_key(&result);
            
            score_map.entry(key)
                .and_modify(|e| {
                    e.combined_score += rrf_score;
                    e.match_count += 1;
                    // Merge symbols
                    for symbol in &result.symbols {
                        if !e.symbols.contains(symbol) {
                            e.symbols.push(symbol.clone());
                        }
                    }
                })
                .or_insert(FusedResult {
                    content: result.content.clone(),
                    file_path: result.file_path.clone(),
                    combined_score: rrf_score,
                    match_count: 1,
                    primary_type: match_type.clone(),
                    line_number: result.line_number,
                    symbols: result.symbols.clone(),
                });
        }
    }
    
    fn create_result_key(result: &SearchResult) -> String {
        format!(
            "{}:{}:{}",
            result.file_path,
            result.line_number.unwrap_or(0),
            &result.content[..50.min(result.content.len())]
        )
    }
}

#[derive(Debug)]
struct FusedResult {
    content: String,
    file_path: String,
    combined_score: f32,
    match_count: usize,
    primary_type: MatchType,
    line_number: Option<usize>,
    symbols: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_fusion_config_normalization() {
        let mut config = FusionConfig {
            text_weight: 1.0,
            vector_weight: 2.0,
            symbol_weight: 1.0,
            fuzzy_weight: 1.0,
            ..Default::default()
        };
        
        config.normalize();
        
        let sum = config.text_weight + config.vector_weight + config.symbol_weight + config.fuzzy_weight;
        assert!((sum - 1.0).abs() < 0.001);
    }
    
    #[test]
    fn test_rrf_fusion() {
        let config = FusionConfig::default();
        let engine = FusionEngine::new(config);
        
        let text_results = vec![
            SearchResult {
                content: "test content".to_string(),
                file_path: "test.rs".to_string(),
                score: 0.9,
                match_type: MatchType::Text,
                line_number: Some(10),
                symbols: vec![],
            },
        ];
        
        let vector_results = vec![
            SearchResult {
                content: "test content".to_string(),
                file_path: "test.rs".to_string(),
                score: 0.8,
                match_type: MatchType::Vector,
                line_number: Some(10),
                symbols: vec![],
            },
        ];
        
        let results = engine.rrf_fusion(
            vec![text_results, vector_results],
            vec![0.5, 0.5],
        );
        
        assert!(!results.is_empty());
        assert_eq!(results[0].match_type, MatchType::Hybrid);
    }
}