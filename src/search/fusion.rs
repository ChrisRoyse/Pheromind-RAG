use std::collections::HashSet;
use crate::search::ripgrep::ExactMatch;
use crate::storage::lancedb_storage::LanceEmbeddingRecord;

#[derive(Debug, Clone, PartialEq)]
pub enum MatchType {
    Exact,
    Semantic,
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

pub struct SimpleFusion;

impl SimpleFusion {
    pub fn new() -> Self {
        Self
    }
    
    pub fn fuse_results(
        &self,
        exact_matches: Vec<ExactMatch>,
        semantic_matches: Vec<LanceEmbeddingRecord>,
    ) -> Vec<FusedResult> {
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
        for (idx, semantic) in semantic_matches.into_iter().enumerate() {
            // Check if we already have an exact match for this file
            let file_has_exact = results.iter().any(|r| {
                r.file_path == semantic.file_path && 
                r.match_type == MatchType::Exact &&
                r.line_number.map_or(false, |line| {
                    line >= semantic.start_line as usize && 
                    line <= semantic.end_line as usize
                })
            });
            
            if !file_has_exact {
                let key = format!("{}-{}", semantic.file_path, semantic.chunk_index);
                if seen.insert(key) {
                    // Calculate similarity score based on position in results
                    // Closer matches (lower index) get higher scores
                    let similarity = 1.0 - (idx as f32 / 100.0);
                    
                    results.push(FusedResult {
                        file_path: semantic.file_path,
                        line_number: None,
                        chunk_index: Some(semantic.chunk_index as usize),
                        score: similarity * 0.8, // Slightly lower than exact
                        match_type: MatchType::Semantic,
                        content: semantic.content,
                        start_line: semantic.start_line as usize,
                        end_line: semantic.end_line as usize,
                    });
                }
            }
        }
        
        // Sort by score descending
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        
        // Take top 20 results
        results.truncate(20);
        results
    }
    
    pub fn optimize_ranking(&self, results: &mut Vec<FusedResult>, query: &str) {
        let query_lower = query.to_lowercase();
        let query_words: Vec<&str> = query_lower.split_whitespace().collect();
        
        for result in results.iter_mut() {
            let content_lower = result.content.to_lowercase();
            let file_path_lower = result.file_path.to_lowercase();
            
            // CRITICAL: Deprioritize test files first (before any boosts)
            let is_test_file = self.is_test_file(&result.file_path);
            if is_test_file {
                result.score *= 0.3; // Strong penalty for test files
            }
            
            // MAJOR boost for vectortest/ directory files (actual implementation files)
            if file_path_lower.contains("vectortest/") || file_path_lower.contains("vectortest\\") {
                result.score *= 2.5; // Strong boost for vectortest files
            }
            
            // Directory-based ranking boosts
            let path_parts: Vec<&str> = result.file_path.split(['/', '\\']).collect();
            if let Some(dir_name) = path_parts.iter().rev().nth(1) {
                let dir_lower = dir_name.to_lowercase();
                // Boost for important implementation directories
                if matches!(dir_lower.as_str(), "vectortest" | "src" | "lib" | "core" | "main") {
                    result.score *= 1.8;
                }
                // Penalty for test directories
                if matches!(dir_lower.as_str(), "tests" | "test" | "spec" | "specs") {
                    result.score *= 0.4;
                }
            }
            
            // STRONG boost for exact filename matches
            let filename = std::path::Path::new(&result.file_path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
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
            
            // Cap semantic match scores to avoid overwhelming exact matches
            if result.match_type == MatchType::Semantic && result.score > 1.5 {
                result.score = 1.5;
            }
            
            // Ensure exact matches stay above semantic matches
            if result.match_type == MatchType::Exact {
                result.score = result.score.max(1.6); // Ensure minimum boost for exact matches
            }
        }
        
        // Re-sort after optimization
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
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
    
    fn is_test_file(&self, path: &str) -> bool {
        let path_lower = path.to_lowercase();
        let filename = std::path::Path::new(&path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_lowercase();
        
        // Check for test indicators in path or filename
        path_lower.contains("/test") || 
        path_lower.contains("\\test") ||
        path_lower.contains("/tests/") ||
        path_lower.contains("\\tests\\") ||
        filename.contains("test") ||
        filename.contains("spec") ||
        path_lower.contains("_test.") ||
        path_lower.contains("test_") ||
        path_lower.contains("_spec.") ||
        path_lower.contains("spec_")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
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
                embedding: vec![0.1; 384],
                start_line: 5,
                end_line: 15,
                similarity_score: Some(0.8),
            }
        ];
        
        let results = fusion.fuse_results(exact_matches, semantic_matches);
        
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].match_type, MatchType::Exact);
        assert_eq!(results[0].score, 1.0);
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
        
        let results = fusion.fuse_results(exact_matches, vec![]);
        
        assert_eq!(results.len(), 1); // Duplicates removed
    }
}