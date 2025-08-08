use std::collections::HashSet;
use serde::{Serialize, Deserialize};
use crate::search::ExactMatch;
use crate::error::SearchError;
#[cfg(feature = "vectordb")]
use crate::storage::lancedb_storage::LanceEmbeddingRecord;
#[cfg(feature = "tree-sitter")]
use crate::search::symbol_index::Symbol;
use crate::search::bm25::BM25Match;

/// Configuration constants for fusion scoring
#[derive(Debug, Clone)]
pub struct FusionConfig {
    /// Maximum number of results to return
    pub max_results: usize,
    /// Score cap for normalized BM25 scores (prevents dominance over other match types)
    pub bm25_score_cap: f32,
    /// Minimum score threshold below which BM25 results are filtered out
    pub bm25_min_threshold: f32,
    /// Percentile used for dynamic normalization (e.g., 95th percentile)
    pub normalization_percentile: f32,
    /// Semantic match score multiplier to balance against exact matches
    pub semantic_score_factor: f32,
}

impl Default for FusionConfig {
    fn default() -> Self {
        Self {
            max_results: 20,
            bm25_score_cap: 0.9,
            bm25_min_threshold: 0.01,
            normalization_percentile: 0.95,
            semantic_score_factor: 0.8,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MatchType {
    Exact,
    Semantic,
    Symbol,
    Statistical,  // BM25/TF-IDF matches
}

#[derive(Debug, Clone)]
pub struct FusedResult {
    pub file_path: String,
    pub line_number: Option<usize>,
    pub chunk_index: Option<usize>,
    pub score: f32,
    pub match_type: MatchType,
    pub content: String,
    pub start_line: usize,
    pub end_line: usize,
}

pub struct SimpleFusion {
    config: FusionConfig,
}

impl SimpleFusion {
    pub fn new() -> Self {
        Self {
            config: FusionConfig::default(),
        }
    }
    
    pub fn with_config(config: FusionConfig) -> Self {
        Self { config }
    }
    
    /// Calculate dynamic normalization factor based on score distribution
    /// Uses percentile-based normalization to handle varying BM25 score ranges
    fn calculate_bm25_normalization(&self, scores: &[f32]) -> f32 {
        if scores.is_empty() {
            return 1.0;
        }
        
        // Create sorted copy for percentile calculation
        let mut sorted_scores = scores.to_vec();
        sorted_scores.sort_by(|a, b| {
            a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        // Calculate the specified percentile (default 95th)
        let percentile_index = ((sorted_scores.len() as f32 - 1.0) * self.config.normalization_percentile) as usize;
        let percentile_score = sorted_scores[percentile_index.min(sorted_scores.len() - 1)];
        
        // Avoid division by zero or very small values
        if percentile_score <= 0.0001 {
            return 1.0;
        }
        
        // Calculate normalization factor to map percentile score to the cap
        self.config.bm25_score_cap / percentile_score
    }
    
    /// Parse document ID in format "filepath-chunkindex"
    /// Returns (file_path, chunk_index) or error if format is invalid
    fn parse_doc_id(doc_id: &str) -> Result<(String, usize), SearchError> {
        let parts: Vec<&str> = doc_id.rsplitn(2, '-').collect();
        if parts.len() != 2 {
            return Err(SearchError::InvalidDocId {
                doc_id: doc_id.to_string(),
                expected_format: "filepath-chunkindex".to_string(),
            });
        }
        
        let file_path = parts[1].to_string();
        let chunk_index = parts[0].parse::<usize>().map_err(|_| {
            SearchError::InvalidDocId {
                doc_id: doc_id.to_string(),
                expected_format: "filepath-chunkindex (chunk index must be numeric)".to_string(),
            }
        })?;
        
        Ok((file_path, chunk_index))
    }
    
    #[cfg(feature = "vectordb")]
    pub fn fuse_results(
        &self,
        exact_matches: Vec<ExactMatch>,
        semantic_matches: Vec<LanceEmbeddingRecord>,
    ) -> Result<Vec<FusedResult>, SearchError> {
        let mut seen = HashSet::new();
        let mut results = Vec::new();
        
        // Process exact matches first (higher priority)
        for exact in exact_matches {
            let key = format!("{}-{}", exact.file_path, exact.line_number);
            if seen.insert(key) {
                results.push(FusedResult {
                    file_path: exact.file_path,
                    line_number: Some(exact.line_number),
                    chunk_index: None,
                    score: 1.0, // Exact matches get perfect score
                    match_type: MatchType::Exact,
                    content: exact.content,
                    start_line: exact.line_number,
                    end_line: exact.line_number,
                });
            }
        }
        
        // Add semantic matches with lower scores
        for (_idx, semantic) in semantic_matches.into_iter().enumerate() {
            // Check if we already have an exact match for this file
            // FIXED: Replace .map_or(false, |line| {...}) with explicit Option handling
            let file_has_exact = results.iter().any(|r| {
                r.file_path == semantic.file_path && 
                r.match_type == MatchType::Exact &&
                match r.line_number {
                    Some(line) => {
                        line >= semantic.start_line as usize && 
                        line <= semantic.end_line as usize
                    }
                    None => {
                        // Exact match must have a line number - if missing, this is corrupted data
                        log::error!("Exact match found without line number in file '{}'. This indicates corrupted search results.", r.file_path);
                        false
                    }
                }
            });
            
            if !file_has_exact {
                let key = format!("{}-{}", semantic.file_path, semantic.chunk_index);
                if seen.insert(key) {
                    // Use actual similarity score from vector search - fail if missing
                    let similarity = match semantic.similarity_score {
                        Some(score) => score,
                        None => {
                            return Err(SearchError::MissingSimilarityScore {
                                file_path: semantic.file_path,
                                chunk_index: semantic.chunk_index as u32,
                            });
                        }
                    };
                    
                    results.push(FusedResult {
                        file_path: semantic.file_path,
                        line_number: None,
                        chunk_index: Some(semantic.chunk_index as usize),
                        score: similarity * self.config.semantic_score_factor, // Configurable semantic score factor
                        match_type: MatchType::Semantic,
                        content: semantic.content,
                        start_line: semantic.start_line as usize,
                        end_line: semantic.end_line as usize,
                    });
                }
            }
        }
        
        // Validate all scores for NaN/Infinity before sorting - fail on corrupted data
        for result in &results {
            if !result.score.is_finite() {
                return Err(SearchError::CorruptedData {
                    description: format!("Invalid score {} detected for file '{}'. Search results cannot contain NaN or infinite values.", result.score, result.file_path)
                });
            }
        }
        
        // Sort by score descending with explicit error handling
        results.sort_by(|a, b| {
            b.score.partial_cmp(&a.score).ok_or_else(|| {
                SearchError::CorruptedData {
                    description: format!("Score comparison failed during fusion sorting: {} vs {}. This indicates corrupted search result data.", b.score, a.score)
                }
            }).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        // Take top 20 results
        results.truncate(20);
        Ok(results)
    }
    
    #[cfg(all(feature = "vectordb", feature = "tree-sitter"))]
    pub fn fuse_all_results(
        &self,
        exact_matches: Vec<ExactMatch>,
        semantic_matches: Vec<LanceEmbeddingRecord>,
        symbol_matches: Vec<Symbol>,
    ) -> Result<Vec<FusedResult>, SearchError> {
        let mut seen = HashSet::new();
        let mut results = Vec::new();
        
        // Process exact matches first (highest priority)
        for exact in exact_matches {
            let key = format!("{}-{}", exact.file_path, exact.line_number);
            if seen.insert(key) {
                results.push(FusedResult {
                    file_path: exact.file_path,
                    line_number: Some(exact.line_number),
                    chunk_index: None,
                    score: 1.0, // Exact matches get perfect score
                    match_type: MatchType::Exact,
                    content: exact.content,
                    start_line: exact.line_number,
                    end_line: exact.line_number,
                });
            }
        }
        
        // Add symbol matches (high priority for precise code navigation)
        for symbol in symbol_matches {
            let key = format!("{}-{}", symbol.file_path, symbol.line_start);
            if seen.insert(key.clone()) {
                results.push(FusedResult {
                    file_path: symbol.file_path.clone(),
                    line_number: Some(symbol.line_start),
                    chunk_index: None,
                    score: 0.95, // Symbol matches get high score
                    match_type: MatchType::Symbol,
                    content: format!("{} ({:?})", symbol.name, symbol.kind),
                    start_line: symbol.line_start,
                    end_line: symbol.line_end,
                });
            }
        }
        
        // Add semantic matches with lower scores
        for (_idx, semantic) in semantic_matches.into_iter().enumerate() {
            // Skip if we already have an exact or symbol match for this location
            let file_has_better_match = results.iter().any(|r| {
                r.file_path == semantic.file_path && 
                (r.match_type == MatchType::Exact || r.match_type == MatchType::Symbol) &&
                r.start_line <= semantic.end_line as usize &&
                r.end_line >= semantic.start_line as usize
            });
            
            if !file_has_better_match {
                let key = format!("{}-{}", semantic.file_path, semantic.chunk_index);
                if seen.insert(key) {
                    // Use actual similarity score from vector search - fail if missing
                    let similarity = match semantic.similarity_score {
                        Some(score) => score,
                        None => {
                            return Err(SearchError::MissingSimilarityScore {
                                file_path: semantic.file_path,
                                chunk_index: semantic.chunk_index as u32,
                            });
                        }
                    };
                    
                    results.push(FusedResult {
                        file_path: semantic.file_path,
                        line_number: None,
                        chunk_index: Some(semantic.chunk_index as usize),
                        score: similarity * (self.config.semantic_score_factor * 0.875), // Slightly lower than 2-way fusion
                        match_type: MatchType::Semantic,
                        content: semantic.content,
                        start_line: semantic.start_line as usize,
                        end_line: semantic.end_line as usize,
                    });
                }
            }
        }
        
        // Validate all scores for NaN/Infinity before sorting - fail on corrupted data
        for result in &results {
            if !result.score.is_finite() {
                return Err(SearchError::CorruptedData {
                    description: format!("Invalid score {} detected for file '{}'. Search results cannot contain NaN or infinite values.", result.score, result.file_path)
                });
            }
        }
        
        // Sort by score descending with explicit error handling
        results.sort_by(|a, b| {
            b.score.partial_cmp(&a.score).ok_or_else(|| {
                SearchError::CorruptedData {
                    description: format!("Score comparison failed during all-results fusion sorting: {} vs {}. This indicates corrupted search result data.", b.score, a.score)
                }
            }).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        // Take top 20 results
        results.truncate(20);
        Ok(results)
    }
    
    /// Enhanced fusion with BM25 results (4-way fusion)
    #[cfg(all(feature = "vectordb", feature = "tree-sitter"))]
    pub fn fuse_all_results_with_bm25(
        &self,
        exact_matches: Vec<ExactMatch>,
        semantic_matches: Vec<LanceEmbeddingRecord>,
        symbol_matches: Vec<Symbol>,
        bm25_matches: Vec<BM25Match>,
    ) -> Result<Vec<FusedResult>, SearchError> {
        let mut seen = HashSet::new();
        let mut results = Vec::new();
        
        // 1. Process exact matches first (highest priority)
        for exact in exact_matches {
            let key = format!("{}-{}", exact.file_path, exact.line_number);
            if seen.insert(key) {
                results.push(FusedResult {
                    file_path: exact.file_path,
                    line_number: Some(exact.line_number),
                    chunk_index: None,
                    score: 1.0, // Exact matches get perfect score
                    match_type: MatchType::Exact,
                    content: exact.content,
                    start_line: exact.line_number,
                    end_line: exact.line_number,
                });
            }
        }
        
        // 2. Process BM25 matches (high priority for statistical relevance)
        for bm25 in bm25_matches {
            // Extract file path and chunk index from doc_id (format: "filepath-chunkindex")
            let (file_path, chunk_index) = Self::parse_doc_id(&bm25.doc_id)?;
            let chunk_index = Some(chunk_index);
            
            let key = format!("bm25-{}", bm25.doc_id);
            if seen.insert(key) {
                // Apply dynamic normalization - will be calculated later with full score distribution
                let normalized_score = bm25.score; // Temporary - will be normalized after collecting all scores
                
                // BM25 matches must have valid chunk indices
                let chunk_idx = chunk_index.ok_or_else(|| SearchError::InvalidDocId {
                    doc_id: bm25.doc_id.clone(),
                    expected_format: "BM25 matches require chunk index".to_string(),
                })?;
                
                results.push(FusedResult {
                    file_path,
                    line_number: None,
                    chunk_index: Some(chunk_idx),
                    score: normalized_score,
                    match_type: MatchType::Statistical,
                    content: format!("BM25 match (score: {:.2})", bm25.score),
                    start_line: chunk_idx,
                    end_line: chunk_idx,
                });
            }
        }
        
        // 3. Process symbol matches
        for symbol in symbol_matches {
            let key = format!("{}-{}", symbol.file_path, symbol.line_start);
            if seen.insert(key.clone()) {
                results.push(FusedResult {
                    file_path: symbol.file_path.clone(),
                    line_number: Some(symbol.line_start),
                    chunk_index: None,
                    score: 0.95, // Symbol matches get high score
                    match_type: MatchType::Symbol,
                    content: format!("{} ({:?})", symbol.name, symbol.kind),
                    start_line: symbol.line_start,
                    end_line: symbol.line_end,
                });
            }
        }
        
        // 4. Process semantic matches
        for (_idx, semantic) in semantic_matches.into_iter().enumerate() {
            // Skip if we already have a better match for this location
            let file_has_better_match = results.iter().any(|r| {
                r.file_path == semantic.file_path && 
                (r.match_type == MatchType::Exact || 
                 r.match_type == MatchType::Symbol ||
                 r.match_type == MatchType::Statistical) &&
                r.start_line <= semantic.end_line as usize &&
                r.end_line >= semantic.start_line as usize
            });
            
            if !file_has_better_match {
                let key = format!("{}-{}", semantic.file_path, semantic.chunk_index);
                if seen.insert(key) {
                    // Use actual similarity score from vector search - fail if missing
                    let similarity = match semantic.similarity_score {
                        Some(score) => score,
                        None => {
                            return Err(SearchError::MissingSimilarityScore {
                                file_path: semantic.file_path,
                                chunk_index: semantic.chunk_index as u32,
                            });
                        }
                    };
                    
                    results.push(FusedResult {
                        file_path: semantic.file_path,
                        line_number: None,
                        chunk_index: Some(semantic.chunk_index as usize),
                        score: similarity * (self.config.semantic_score_factor * 0.875), // Configurable semantic reduction
                        match_type: MatchType::Semantic,
                        content: semantic.content,
                        start_line: semantic.start_line as usize,
                        end_line: semantic.end_line as usize,
                    });
                }
            }
        }
        
        // Apply dynamic BM25 normalization before weighted fusion
        self.apply_dynamic_bm25_normalization(&mut results)?;
        
        // Apply weighted fusion scoring
        self.apply_weighted_fusion(&mut results, 0.4, 0.25, 0.25, 0.1);
        
        // Validate all scores before sorting
        for result in &results {
            if result.score.is_nan() || result.score.is_infinite() {
                return Err(SearchError::QueryInvalid {
                    message: format!(
                        "Invalid score detected in search results: {} for file {}",
                        result.score,
                        result.file_path
                    ),
                    query: format!("Score validation failed"),
                });
            }
        }
        
        // Sort by final score with robust error handling
        results.sort_by(|a, b| {
            b.score.partial_cmp(&a.score).ok_or_else(|| {
                SearchError::CorruptedData {
                    description: format!("Score comparison failed during BM25 fusion sorting: {} vs {}. All scores were validated but comparison still failed.", b.score, a.score)
                }
            }).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        // Take top 20 results
        results.truncate(20);
        Ok(results)
    }
    
    /// Core fusion functionality (BM25 + Exact matches only) - no fallbacks
    pub fn fuse_results_core(
        &self,
        exact_matches: Vec<ExactMatch>,
        bm25_matches: Vec<BM25Match>,
    ) -> Result<Vec<FusedResult>, SearchError> {
        let mut seen = HashSet::new();
        let mut results = Vec::new();
        
        // Process exact matches first (highest priority)
        for exact in exact_matches {
            let key = format!("{}-{}", exact.file_path, exact.line_number);
            if seen.insert(key) {
                results.push(FusedResult {
                    file_path: exact.file_path,
                    line_number: Some(exact.line_number),
                    chunk_index: None,
                    score: 1.0, // Exact matches get perfect score
                    match_type: MatchType::Exact,
                    content: exact.content,
                    start_line: exact.line_number,
                    end_line: exact.line_number,
                });
            }
        }
        
        // Process BM25 matches
        for bm25 in bm25_matches {
            // Extract file path and chunk index from doc_id (format: "filepath-chunkindex")
            let (file_path, chunk_index) = Self::parse_doc_id(&bm25.doc_id)?;
            let chunk_index = Some(chunk_index);
            
            let key = format!("bm25-{}", bm25.doc_id);
            if seen.insert(key) {
                // Apply dynamic normalization - will be calculated later with full score distribution
                let normalized_score = bm25.score; // Temporary - will be normalized after collecting all scores
                
                // BM25 matches must have valid chunk indices
                let chunk_idx = chunk_index.ok_or_else(|| SearchError::InvalidDocId {
                    doc_id: bm25.doc_id.clone(),
                    expected_format: "BM25 matches require chunk index".to_string(),
                })?;
                
                results.push(FusedResult {
                    file_path,
                    line_number: None,
                    chunk_index: Some(chunk_idx),
                    score: normalized_score,
                    match_type: MatchType::Statistical,
                    content: format!("BM25 match (score: {:.2})", bm25.score),
                    start_line: chunk_idx,
                    end_line: chunk_idx,
                });
            }
        }
        
        // Validate all scores for NaN/Infinity before sorting - fail on corrupted data
        for result in &results {
            if !result.score.is_finite() {
                return Err(SearchError::CorruptedData {
                    description: format!("Invalid score {} detected for file '{}'. Search results cannot contain NaN or infinite values.", result.score, result.file_path)
                });
            }
        }
        
        // Apply dynamic BM25 normalization
        self.apply_dynamic_bm25_normalization(&mut results)?;
        
        // Sort by score descending with explicit error handling
        results.sort_by(|a, b| {
            b.score.partial_cmp(&a.score).ok_or_else(|| {
                SearchError::CorruptedData {
                    description: format!("Score comparison failed during core fusion sorting: {} vs {}. This indicates corrupted search result data.", b.score, a.score)
                }
            }).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        // Take top results based on configuration
        results.truncate(self.config.max_results);
        Ok(results)
    }
    
    /// Apply weighted fusion to combine scores from different search types
    #[allow(dead_code)]
    fn apply_weighted_fusion(
        &self,
        results: &mut Vec<FusedResult>,
        exact_weight: f32,
        bm25_weight: f32,
        semantic_weight: f32,
        symbol_weight: f32,
    ) {
        for result in results.iter_mut() {
            let base_score = result.score;
            result.score = match result.match_type {
                MatchType::Exact => base_score * exact_weight,
                MatchType::Statistical => base_score * bm25_weight,
                MatchType::Semantic => base_score * semantic_weight,
                MatchType::Symbol => base_score * symbol_weight,
            };
        }
    }
    
    pub fn optimize_ranking(&self, results: &mut Vec<FusedResult>, query: &str) -> Result<(), SearchError> {
        let query_lower = query.to_lowercase();
        let query_words: Vec<&str> = query_lower.split_whitespace().collect();
        
        for result in results.iter_mut() {
            let content_lower = result.content.to_lowercase();
            let file_path_lower = result.file_path.to_lowercase();
            
            // Deprioritize test files 
            let is_test_file = self.is_test_file(&result.file_path)?;
            if is_test_file {
                result.score *= 0.5; // Moderate penalty for test files
            }
            
            // Directory-based ranking boosts
            let path_parts: Vec<&str> = result.file_path.split(['/', '\\']).collect();
            if let Some(dir_name) = path_parts.iter().rev().nth(1) {
                let dir_lower = dir_name.to_lowercase();
                // Boost for implementation directories (generic, not biased)
                if matches!(dir_lower.as_str(), "src" | "lib" | "core" | "main" | "app" | "backend" | "frontend") {
                    result.score *= 1.2; // Reduced boost, more neutral
                }
                // Penalty for test directories
                if matches!(dir_lower.as_str(), "tests" | "test" | "spec" | "specs" | "__tests__") {
                    result.score *= 0.6; // Lighter penalty
                }
                // Penalty for deprecated/legacy code
                if matches!(dir_lower.as_str(), "legacy" | "deprecated" | "old" | "archive") {
                    result.score *= 0.7;
                }
            }
            
            // STRONG boost for exact filename matches
            let filename = match std::path::Path::new(&result.file_path)
                .file_name()
                .and_then(|n| n.to_str()) {
                    Some(name) => name,
                    None => {
                        return Err(SearchError::InvalidFilePath {
                            path: result.file_path.clone(),
                        });
                    }
                };
            let filename_lower = filename.to_lowercase();
            
            if filename_lower.contains(&query_lower) {
                result.score *= 2.0; // Strong boost for filename matches
            }
            
            // Boost for each query word that appears in filename
            for word in &query_words {
                if word.len() > 1 && filename_lower.contains(word) {
                    result.score *= 1.3;
                }
            }
            
            // Boost for query appearing in file path
            if file_path_lower.contains(&query_lower) {
                result.score *= 1.4;
            }
            
            // Enhanced content matching
            let lines: Vec<&str> = result.content.lines().collect();
            
            // Very strong boost for function/class/method names that match query
            for line in &lines {
                let line_lower = line.trim().to_lowercase();
                
                // Function definitions
                if (line_lower.starts_with("fn ") || 
                    line_lower.starts_with("function ") ||
                    line_lower.starts_with("def ") ||
                    line_lower.starts_with("class ") ||
                    line_lower.starts_with("interface ") ||
                    line_lower.starts_with("struct ") ||
                    line_lower.starts_with("enum ") ||
                    line_lower.contains("public ") ||
                    line_lower.contains("private ") ||
                    line_lower.contains("protected ")) && 
                   line_lower.contains(&query_lower) {
                    result.score *= 2.2; // Very strong boost for definitions
                }
                
                // Check each query word in function/class names
                for word in &query_words {
                    if word.len() > 2 && line_lower.contains(word) {
                        // Extra boost if it's a camelCase or snake_case match
                        if self.is_identifier_match(line, word) {
                            result.score *= 1.5;
                        }
                    }
                }
            }
            
            // Boost if query appears in content (general)
            if content_lower.contains(&query_lower) {
                result.score *= 1.2;
            }
            
            // Boost if match is at beginning of content (likely important definition)
            let first_lines = lines
                .iter()
                .take(5)
                .map(|line| line.trim())
                .collect::<Vec<_>>()
                .join("\n")
                .to_lowercase();
                
            if first_lines.contains(&query_lower) {
                result.score *= 1.3;
            }
            
            // Boost for multiple query word matches
            let word_matches = query_words.iter()
                .filter(|word| word.len() > 1 && content_lower.contains(*word))
                .count();
            if word_matches > 1 {
                result.score *= 1.0 + (word_matches as f32 * 0.1);
            }
            
            // Slight penalty for very large chunks (less focused)
            let chunk_size = result.content.lines().count();
            if chunk_size > 200 {
                result.score *= 0.9;
            } else if chunk_size < 10 {
                result.score *= 1.05; // Small boost for focused chunks
            }
            
            // Boost for code files over documentation
            if self.is_code_file(&result.file_path) {
                result.score *= 1.1;
            }
            
            // Validate semantic scores - fail if invalid instead of capping
            if result.match_type == MatchType::Semantic && result.score > 1.5 {
                return Err(SearchError::CorruptedData {
                    description: format!("Semantic score {} exceeds expected maximum (1.5) for file '{}'. This indicates corrupted similarity calculation.", result.score, result.file_path)
                });
            }
            
            // Ensure exact matches stay above semantic matches
            if result.match_type == MatchType::Exact {
                result.score = result.score.max(1.6); // Ensure minimum boost for exact matches
            }
        }
        
        // Validate all scores again before re-sorting - fail on corrupted data
        for result in results.iter() {
            if !result.score.is_finite() {
                return Err(SearchError::CorruptedData {
                    description: format!("Invalid score {} detected after optimization for file '{}'. Rankings contain corrupted data.", result.score, result.file_path)
                });
            }
        }
        
        // Re-sort after optimization with explicit error handling
        results.sort_by(|a, b| {
            b.score.partial_cmp(&a.score).ok_or_else(|| {
                SearchError::CorruptedData {
                    description: format!("Score comparison failed during ranking optimization: {} vs {}. Post-optimization scores are corrupted.", b.score, a.score)
                }
            }).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        Ok(())
    }
    
    /// Apply dynamic normalization to BM25 scores based on their distribution
    fn apply_dynamic_bm25_normalization(&self, results: &mut Vec<FusedResult>) -> Result<(), SearchError> {
        // Collect all BM25 scores for normalization calculation
        let bm25_scores: Vec<f32> = results
            .iter()
            .filter(|r| r.match_type == MatchType::Statistical)
            .map(|r| r.score)
            .collect();
        
        if bm25_scores.is_empty() {
            return Ok(()); // No BM25 results to normalize
        }
        
        // Calculate normalization factor based on score distribution
        let normalization_factor = self.calculate_bm25_normalization(&bm25_scores);
        
        // Apply normalization and filtering to BM25 results
        for result in results.iter_mut() {
            if result.match_type == MatchType::Statistical {
                // Apply dynamic normalization
                result.score *= normalization_factor;
                
                // Apply configured cap to prevent BM25 from dominating other match types
                result.score = result.score.min(self.config.bm25_score_cap);
                
                // Validate normalized score is finite
                if !result.score.is_finite() {
                    return Err(SearchError::CorruptedData {
                        description: format!(
                            "BM25 normalization produced invalid score {} for file '{}'. \
                             Original score was normalized by factor {}.",
                            result.score, result.file_path, normalization_factor
                        )
                    });
                }
            }
        }
        
        // Filter out BM25 results below minimum threshold
        results.retain(|r| {
            r.match_type != MatchType::Statistical || r.score >= self.config.bm25_min_threshold
        });
        
        Ok(())
    }
    
    fn is_identifier_match(&self, line: &str, word: &str) -> bool {
        let line_lower = line.to_lowercase();
        let word_lower = word.to_lowercase();
        
        // Check for camelCase patterns
        if line_lower.contains(&format!("{}[", word_lower)) || // function calls
           line_lower.contains(&format!("{} ", word_lower)) ||  // spaces around
           line_lower.contains(&format!("{}(", word_lower)) ||  // function definitions
           line_lower.contains(&format!("{}_", word_lower)) ||  // snake_case
           line_lower.contains(&format!("_{}", word_lower)) {
            return true;
        }
        
        false
    }
    
    fn is_code_file(&self, path: &str) -> bool {
        match std::path::Path::new(path).extension().and_then(|s| s.to_str()) {
            Some(ext) => matches!(ext.to_lowercase().as_str(), 
                "rs" | "py" | "js" | "ts" | "jsx" | "tsx" | 
                "go" | "java" | "cpp" | "c" | "h" | "hpp" |
                "rb" | "php" | "swift" | "kt" | "scala" | "cs" |
                "sql"
            ),
            None => false,
        }
    }
    
    fn is_test_file(&self, path: &str) -> Result<bool, SearchError> {
        let path_lower = path.to_lowercase();
        let filename = match std::path::Path::new(&path)
            .file_name()
            .and_then(|n| n.to_str()) {
                Some(name) => name.to_lowercase(),
                None => {
                    return Err(SearchError::InvalidFilePath {
                        path: path.to_string(),
                    });
                }
            };
        
        // Check for test indicators in path or filename
        Ok(path_lower.contains("/test") || 
        path_lower.contains("\\test") ||
        path_lower.contains("/tests/") ||
        path_lower.contains("\\tests\\") ||
        filename.contains("test") ||
        filename.contains("spec") ||
        path_lower.contains("_test.") ||
        path_lower.contains("test_") ||
        path_lower.contains("_spec.") ||
        path_lower.contains("spec_"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustc_hash::FxHashMap;
    
    #[cfg(feature = "vectordb")]
    #[test]
    fn test_fusion_prioritizes_exact_matches() {
        let fusion = SimpleFusion::new();
        
        let exact_matches = vec![
            ExactMatch {
                file_path: "test.rs".to_string(),
                line_number: 10,
                content: "fn test()".to_string(),
                line_content: "fn test()".to_string(),
            }
        ];
        
        let semantic_matches = vec![
            LanceEmbeddingRecord {
                id: "test-0".to_string(),
                file_path: "test.rs".to_string(),
                chunk_index: 0,
                content: "some other content".to_string(),
                embedding: vec![0.1; 768],
                start_line: 5,
                end_line: 15,
                similarity_score: Some(0.8),
            }
        ];
        
        let results = fusion.fuse_results(exact_matches, semantic_matches).unwrap();
        
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].match_type, MatchType::Exact);
        assert_eq!(results[0].score, 1.0);
    }
    
    #[test]
    fn test_dynamic_bm25_normalization() {
        let fusion = SimpleFusion::new();
        
        // Create BM25 matches with varying score ranges
        let bm25_matches = vec![
            BM25Match {
                doc_id: "file1.rs-0".to_string(),
                score: 25.5, // High score
                term_scores: FxHashMap::default(),
                matched_terms: vec!["test".to_string()],
            },
            BM25Match {
                doc_id: "file2.rs-0".to_string(),
                score: 15.2, // Medium score
                term_scores: FxHashMap::default(),
                matched_terms: vec!["test".to_string()],
            },
            BM25Match {
                doc_id: "file3.rs-0".to_string(),
                score: 5.1, // Low score
                term_scores: FxHashMap::default(),
                matched_terms: vec!["test".to_string()],
            },
        ];
        
        let results = fusion.fuse_results_core(vec![], bm25_matches).unwrap();
        
        // Verify all scores are normalized and capped appropriately
        assert_eq!(results.len(), 3);
        for result in &results {
            // All BM25 scores should be capped at 0.9 and properly normalized
            assert!(result.score <= 0.9, "BM25 score {} exceeds cap of 0.9", result.score);
            assert!(result.score > 0.0, "BM25 score should be positive");
            assert!(result.score.is_finite(), "BM25 score should be finite");
        }
        
        // Higher original scores should still result in higher normalized scores
        assert!(results[0].score > results[2].score, "Score ordering should be preserved");
    }
    
    #[test]
    fn test_fusion_config_customization() {
        let config = FusionConfig {
            max_results: 5,
            bm25_score_cap: 0.7,
            bm25_min_threshold: 0.1,
            normalization_percentile: 0.90,
            semantic_score_factor: 0.9,
        };
        let fusion = SimpleFusion::with_config(config);
        
        // Create BM25 matches with high scores
        let bm25_matches = vec![
            BM25Match {
                doc_id: "file1.rs-0".to_string(),
                score: 100.0, // Very high score to test capping
                term_scores: FxHashMap::default(),
                matched_terms: vec!["test".to_string()],
            },
        ];
        
        let results = fusion.fuse_results_core(vec![], bm25_matches).unwrap();
        
        // Verify custom cap is applied
        assert_eq!(results.len(), 1);
        assert!(results[0].score <= 0.7, "Custom BM25 cap should be applied");
    }
    
    #[test]
    fn test_fusion_deduplicates_results() {
        let fusion = SimpleFusion::new();
        
        let exact_matches = vec![
            ExactMatch {
                file_path: "test.rs".to_string(),
                line_number: 10,
                content: "fn test()".to_string(),
                line_content: "fn test()".to_string(),
            },
            ExactMatch {
                file_path: "test.rs".to_string(),
                line_number: 10,
                content: "fn test()".to_string(),
                line_content: "fn test()".to_string(),
            }
        ];
        
        let results = fusion.fuse_results_core(exact_matches, vec![]).unwrap();
        
        assert_eq!(results.len(), 1); // Duplicates removed
    }
}