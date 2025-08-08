// Fixed BM25 implementation with correct IDF calculation
// Following TDD red-green-refactor methodology

use anyhow::Result;
use std::collections::HashSet;
use rustc_hash::FxHashMap;
use std::path::PathBuf;

/// BM25 parameters
const K1: f32 = 1.2; // Term frequency saturation
const B: f32 = 0.75; // Document length normalization

#[derive(Debug, Clone)]
pub struct BM25Match {
    pub path: String,
    pub snippet: String,
    pub score: f32,
    pub line_number: Option<usize>,
}

/// Fixed BM25 search engine with correct IDF calculation
pub struct BM25Engine {
    /// Document collection: doc_id -> (content, token_count)
    documents: FxHashMap<String, (String, usize)>,
    /// Inverted index: term -> set of doc_ids
    inverted_index: FxHashMap<String, HashSet<String>>,
    /// Document frequencies: term -> count of docs containing term
    doc_frequencies: FxHashMap<String, usize>,
    /// Total number of documents
    total_docs: usize,
    /// Average document length
    avg_doc_length: f32,
}

impl BM25Engine {
    pub fn new() -> Result<Self> {
        Ok(Self {
            documents: FxHashMap::default(),
            inverted_index: FxHashMap::default(),
            doc_frequencies: FxHashMap::default(),
            total_docs: 0,
            avg_doc_length: 0.0,
        })
    }
    
    /// Index a document
    pub fn index_document(&mut self, doc_id: &str, content: &str) {
        println!("DEBUG INDEX: Indexing doc_id='{}', content='{}'", doc_id, content);
        
        // Tokenize content
        let tokens = self.tokenize(content);
        let token_count = tokens.len();
        
        println!("DEBUG INDEX: Tokens: {:?}", tokens);
        
        // Store document
        self.documents.insert(doc_id.to_string(), (content.to_string(), token_count));
        
        // Update inverted index and document frequencies
        let unique_terms: HashSet<String> = tokens.iter().cloned().collect();
        println!("DEBUG INDEX: Unique terms: {:?}", unique_terms);
        
        for term in unique_terms {
            self.inverted_index
                .entry(term.clone())
                .or_insert_with(HashSet::new)
                .insert(doc_id.to_string());
            
            let old_freq = *self.doc_frequencies.get(&term).unwrap_or(&0);
            *self.doc_frequencies.entry(term.clone()).or_insert(0) += 1;
            let new_freq = *self.doc_frequencies.get(&term).unwrap();
            println!("DEBUG INDEX: Term '{}' frequency: {} -> {}", term, old_freq, new_freq);
        }
        
        // Update statistics
        self.total_docs += 1;
        self.update_avg_doc_length();
        
        println!("DEBUG INDEX: Total docs now: {}", self.total_docs);
        println!("DEBUG INDEX: Doc frequencies: {:?}", self.doc_frequencies);
    }
    
    /// Calculate IDF (Inverse Document Frequency) - TRULY FIXED VERSION
    pub fn calculate_idf(&self, term: &str) -> f32 {
        let term_lower = term.to_lowercase();
        let doc_freq = self.doc_frequencies.get(&term_lower).unwrap_or(&0);
        
        println!("DEBUG IDF: term='{}', doc_freq={}, total_docs={}", term_lower, doc_freq, self.total_docs);
        
        if *doc_freq == 0 {
            println!("DEBUG IDF: Returning 0.0 for nonexistent term");
            return 0.0;
        }
        
        // BM25 IDF formula: log((N - df + 0.5) / (df + 0.5))
        // Where N = total docs, df = docs containing term
        let n = self.total_docs as f32;
        let df = *doc_freq as f32;
        
        // Calculate the ratio first
        let ratio = (n - df + 0.5) / (df + 0.5);
        
        println!("DEBUG IDF: n={}, df={}, ratio={}", n, df, ratio);
        
        // Apply epsilon protection to ensure positive IDF values
        // For very common terms (high df), ratio approaches 0, so ln(ratio) becomes negative
        // We add smoothing to ensure all terms get positive IDF
        const EPSILON: f32 = 0.01; // Minimum IDF value
        
        if ratio <= 0.0 {
            // If ratio is non-positive (edge case), return small positive value
            println!("DEBUG IDF: Ratio <= 0, returning EPSILON: {}", EPSILON);
            EPSILON
        } else {
            // Standard case: ln(ratio), but ensure minimum positive value
            let ln_ratio = ratio.ln();
            let final_idf = ln_ratio.max(EPSILON);
            println!("DEBUG IDF: ln({}) = {}, final_idf = {}", ratio, ln_ratio, final_idf);
            final_idf
        }
    }
    
    /// Search documents using BM25 scoring
    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<BM25Match>> {
        if query.trim().is_empty() {
            return Ok(Vec::new());
        }
        
        let query_terms = self.tokenize(query);
        let mut scores: FxHashMap<String, f32> = FxHashMap::default();
        
        for term in &query_terms {
            let idf = self.calculate_idf(term);
            
            // Get documents containing this term
            if let Some(doc_ids) = self.inverted_index.get(term) {
                for doc_id in doc_ids {
                    if let Some((content, doc_length)) = self.documents.get(doc_id) {
                        // Calculate term frequency in document
                        let tf = self.calculate_term_frequency(content, term);
                        
                        // BM25 formula
                        let dl = *doc_length as f32;
                        let numerator = tf * (K1 + 1.0);
                        let denominator = tf + K1 * (1.0 - B + B * (dl / self.avg_doc_length));
                        let bm25_score = idf * (numerator / denominator);
                        
                        *scores.entry(doc_id.clone()).or_insert(0.0) += bm25_score;
                    }
                }
            }
        }
        
        // Sort by score and create results
        let mut results: Vec<_> = scores
            .into_iter()
            .map(|(doc_id, score)| {
                let (content, _) = self.documents.get(&doc_id).unwrap();
                BM25Match {
                    path: doc_id.clone(),
                    snippet: self.create_snippet(content, &query_terms),
                    score,
                    line_number: None,
                }
            })
            .collect();
        
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        results.truncate(limit);
        
        Ok(results)
    }
    
    /// Index a directory recursively
    pub fn index_directory(&mut self, dir: &PathBuf) -> Result<()> {
        use std::fs;
        use walkdir::WalkDir;
        
        for entry in WalkDir::new(dir)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file() {
                // Skip non-text files
                if let Some(ext) = path.extension() {
                    let ext_str = ext.to_str().unwrap_or("");
                    if !Self::is_text_file(ext_str) {
                        continue;
                    }
                }
                
                // Read and index file
                if let Ok(content) = fs::read_to_string(path) {
                    let doc_id = path.strip_prefix(dir)
                        .unwrap_or(path)
                        .to_string_lossy()
                        .to_string();
                    self.index_document(&doc_id, &content);
                }
            }
        }
        
        Ok(())
    }
    
    /// Simple tokenization (lowercase and split on non-alphanumeric)
    fn tokenize(&self, text: &str) -> Vec<String> {
        text.to_lowercase()
            .split(|c: char| !c.is_alphanumeric())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect()
    }
    
    /// Calculate term frequency in a document
    fn calculate_term_frequency(&self, content: &str, term: &str) -> f32 {
        let tokens = self.tokenize(content);
        let term_lower = term.to_lowercase();
        tokens.iter().filter(|t| *t == &term_lower).count() as f32
    }
    
    /// Update average document length
    fn update_avg_doc_length(&mut self) {
        if self.total_docs == 0 {
            self.avg_doc_length = 0.0;
            return;
        }
        
        let total_length: usize = self.documents.values().map(|(_, len)| len).sum();
        self.avg_doc_length = total_length as f32 / self.total_docs as f32;
    }
    
    /// Create a snippet around query terms
    fn create_snippet(&self, content: &str, query_terms: &[String]) -> String {
        let words: Vec<&str> = content.split_whitespace().collect();
        
        // Find first occurrence of any query term
        let mut best_pos = 0;
        for (i, word) in words.iter().enumerate() {
            let word_lower = word.to_lowercase();
            if query_terms.iter().any(|term| word_lower.contains(term)) {
                best_pos = i;
                break;
            }
        }
        
        // Create snippet around the found position
        let start = best_pos.saturating_sub(10);
        let end = (best_pos + 20).min(words.len());
        
        let snippet = words[start..end].join(" ");
        if start > 0 {
            format!("...{}", snippet)
        } else {
            snippet
        }
    }
    
    /// Check if file extension indicates text file
    fn is_text_file(ext: &str) -> bool {
        matches!(ext, 
            "rs" | "toml" | "md" | "txt" | "json" | "yaml" | "yml" | 
            "js" | "ts" | "jsx" | "tsx" | "py" | "go" | "java" | 
            "c" | "cpp" | "h" | "hpp" | "cs" | "rb" | "php" | "sh"
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_idf_calculation_fixed() {
        let mut engine = BM25Engine::new().unwrap();
        
        // Create documents with different term frequencies
        engine.index_document("doc1", "cat cat cat cat cat");
        engine.index_document("doc2", "dog dog dog");
        engine.index_document("doc3", "cat dog");
        engine.index_document("doc4", "mouse");
        engine.index_document("doc5", "cat");
        
        // Calculate IDF values
        let cat_idf = engine.calculate_idf("cat"); // appears in 3 docs
        let dog_idf = engine.calculate_idf("dog"); // appears in 2 docs
        let mouse_idf = engine.calculate_idf("mouse"); // appears in 1 doc
        
        println!("cat IDF: {}", cat_idf);
        println!("dog IDF: {}", dog_idf);
        println!("mouse IDF: {}", mouse_idf);
        
        // Verify IDF ordering (rarer terms have higher IDF)
        assert!(mouse_idf > dog_idf, "Mouse (rare) should have higher IDF than dog");
        assert!(dog_idf > cat_idf, "Dog should have higher IDF than cat (common)");
        assert!(cat_idf > 0.0, "Common terms should still have positive IDF");
    }
    
    #[test]
    fn test_relevance_scoring_fixed() {
        let mut engine = BM25Engine::new().unwrap();
        
        // Index test documents
        engine.index_document("auth_service", "authentication user login password secure");
        engine.index_document("test_file", "test example demo sample");
        engine.index_document("user_model", "user profile data model");
        
        // Search for "authentication user"
        let results = engine.search("authentication user", 10).unwrap();
        
        // Verify correct ranking
        assert!(!results.is_empty(), "Should return results");
        assert_eq!(results[0].path, "auth_service", 
            "Document with both query terms should rank first");
        
        // Verify score ordering
        for i in 1..results.len() {
            assert!(results[i-1].score >= results[i].score, 
                "Results should be sorted by score");
        }
    }
}