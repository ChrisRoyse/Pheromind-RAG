use std::collections::HashMap;
use anyhow::{Result, Context};
use serde::{Serialize, Deserialize};

/// High-performance BM25 implementation optimized for code search
#[derive(Debug, Clone)]
pub struct BM25Engine {
    /// Term frequency saturation parameter (default: 1.2)
    k1: f32,
    /// Length normalization parameter (default: 0.75)
    b: f32,
    
    /// Document collection statistics
    total_docs: usize,
    avg_doc_length: f32,
    
    /// Term statistics: term -> (document_frequency, total_frequency)
    term_frequencies: HashMap<String, TermStats>,
    /// Document lengths: doc_id -> length
    document_lengths: HashMap<String, usize>,
    
    /// Inverted index for fast lookups: term -> documents containing it
    inverted_index: HashMap<String, Vec<DocumentTerm>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TermStats {
    /// How many documents contain this term
    pub document_frequency: usize,
    /// Total occurrences across all documents
    pub total_frequency: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentTerm {
    /// Document identifier
    pub doc_id: String,
    /// Occurrences in this document
    pub term_frequency: usize,
    /// Word positions for phrase queries
    pub positions: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct BM25Match {
    pub doc_id: String,
    pub score: f32,
    /// Individual term contributions for debugging
    pub term_scores: HashMap<String, f32>,
    pub matched_terms: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct BM25Document {
    pub id: String,
    pub file_path: String,
    pub chunk_index: usize,
    pub tokens: Vec<Token>,
    pub start_line: usize,
    pub end_line: usize,
    pub language: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    pub text: String,
    pub position: usize,
    pub importance_weight: f32,
}

// BM25Engine must be explicitly configured with parameters - no default fallback allowed
// Use BM25Engine::with_params(k1, b) or BM25Engine::new() explicitly

impl BM25Engine {
    pub fn new() -> Self {
        Self::with_params(1.2, 0.75)
    }
    
    pub fn with_params(k1: f32, b: f32) -> Self {
        Self {
            k1,
            b,
            total_docs: 0,
            avg_doc_length: 0.0,
            term_frequencies: HashMap::new(),
            document_lengths: HashMap::new(),
            inverted_index: HashMap::new(),
        }
    }
    
    /// Add a document to the BM25 index
    pub fn add_document(&mut self, doc: BM25Document) -> Result<()> {
        let doc_id = doc.id.clone();
        let doc_length = doc.tokens.len();
        
        // Update document count and average length
        let total_length = (self.avg_doc_length * self.total_docs as f32) + doc_length as f32;
        self.total_docs += 1;
        self.avg_doc_length = total_length / self.total_docs as f32;
        
        // Store document length
        self.document_lengths.insert(doc_id.clone(), doc_length);
        
        // Process tokens and update inverted index
        let mut term_positions: HashMap<String, Vec<usize>> = HashMap::new();
        let mut term_counts: HashMap<String, usize> = HashMap::new();
        
        for (pos, token) in doc.tokens.iter().enumerate() {
            // Always normalize terms to lowercase for consistency
            let term = token.text.to_lowercase();
            
            // Track positions for this term
            term_positions.entry(term.clone())
                .or_insert_with(Vec::new)
                .push(pos);
            
            // Count term frequency
            *term_counts.entry(term.clone()).or_insert(0) += 1;
        }
        
        // Update inverted index and term statistics
        for (term, positions) in term_positions {
            let freq = term_counts[&term];
            
            // Update term statistics
            let stats = self.term_frequencies.entry(term.clone())
                .or_insert(TermStats {
                    document_frequency: 0,
                    total_frequency: 0,
                });
            stats.document_frequency += 1;
            stats.total_frequency += freq;
            
            // Add to inverted index
            let doc_term = DocumentTerm {
                doc_id: doc_id.clone(),
                term_frequency: freq,
                positions,
            };
            
            self.inverted_index.entry(term)
                .or_insert_with(Vec::new)
                .push(doc_term);
        }
        
        Ok(())
    }
    
    /// Calculate IDF (Inverse Document Frequency) for a term
    pub fn calculate_idf(&self, term: &str) -> f32 {
        let term_lower = term.to_lowercase();
        
        if let Some(stats) = self.term_frequencies.get(&term_lower) {
            let n = self.total_docs as f32;
            let df = stats.document_frequency as f32;
            // BM25 IDF formula: log((N - df + 0.5) / (df + 0.5))
            let raw_idf = ((n - df + 0.5) / (df + 0.5)).ln();
            
            // FIX: Handle negative IDF (when df > N/2) which causes reverse ranking
            // - Negative IDF causes higher TF to score lower (inverted ranking)
            // - Use max(epsilon, raw_idf) to floor negative values while preserving positive IDF ordering
            // - This ensures rare terms (positive IDF) still rank higher than common terms (floored at epsilon)
            let epsilon = 0.001f32; // Small positive value for very common terms
            let idf = epsilon.max(raw_idf);
            idf
        } else {
            // Term not in any document, return high IDF
            (self.total_docs as f32 + 1.0).ln()
        }
    }
    
    /// Calculate BM25 score for a document given query terms
    pub fn calculate_bm25_score(&self, query_terms: &[String], doc_id: &str) -> Result<f32, anyhow::Error> {
        let doc_length = *self.document_lengths.get(doc_id)
            .ok_or_else(|| anyhow::anyhow!("Document '{}' not found in BM25 index. Document must be indexed before scoring.", doc_id))? as f32;
        
        let mut score = 0.0;
        
        for term in query_terms {
            let term_lower = term.to_lowercase();
            let idf = self.calculate_idf(&term_lower);
            
            // Find term frequency in this document
            // If term doesn't exist in index OR document doesn't contain term, tf = 0.0 (legitimate BM25 behavior)
            let tf = match self.inverted_index.get(&term_lower) {
                Some(docs) => {
                    // Term exists in index, check if this specific document contains it
                    match docs.iter().find(|dt| dt.doc_id == doc_id) {
                        Some(doc_term) => doc_term.term_frequency as f32,
                        None => 0.0, // Document doesn't contain this term - legitimate 0.0 contribution for BM25
                    }
                }
                None => {
                    // Term not found in any document - legitimate 0.0 contribution for BM25
                    0.0
                }
            };
            
            if tf > 0.0 {
                // BM25 formula: IDF * (tf * (k1 + 1)) / (tf + k1 * (1 - b + b * (doc_len / avg_doc_len)))
                let norm_factor = 1.0 - self.b + self.b * (doc_length / self.avg_doc_length);
                let term_score = idf * (tf * (self.k1 + 1.0)) / (tf + self.k1 * norm_factor);
                score += term_score;
            }
        }
        
        Ok(score)
    }
    
    /// Search for documents matching the query
    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<BM25Match>, anyhow::Error> {
        // Tokenize query (simple whitespace split for now)
        let query_terms: Vec<String> = query
            .split_whitespace()
            .map(|s| s.to_lowercase())
            .filter(|s| !s.is_empty())
            .collect();
        
        if query_terms.is_empty() {
            return Err(anyhow::anyhow!("Empty query provided to BM25 search. Query must contain valid search terms."));
        }
        
        // Find all documents that contain at least one query term
        let mut candidate_docs: HashMap<String, Vec<String>> = HashMap::new();
        
        for term in &query_terms {
            if let Some(doc_terms) = self.inverted_index.get(term) {
                for doc_term in doc_terms {
                    candidate_docs.entry(doc_term.doc_id.clone())
                        .or_insert_with(Vec::new)
                        .push(term.clone());
                }
            }
        }
        
        // Calculate BM25 scores for candidate documents
        let mut matches: Vec<BM25Match> = Vec::new();
        
        for (doc_id, matched_terms) in candidate_docs {
            let score = self.calculate_bm25_score(&query_terms, &doc_id)
                .with_context(|| format!("BM25 calculation failed for document '{}' - mathematical integrity compromised", doc_id))?;
                
            // Calculate individual term contributions for debugging
            let mut term_scores = HashMap::new();
            for term in &query_terms {
                let single_term_score = self.calculate_bm25_score(&[term.clone()], &doc_id)
                    .with_context(|| format!("Single term BM25 calculation failed for term '{}' in document '{}' - mathematical integrity compromised", term, doc_id))?;
                
                if single_term_score != 0.0 && single_term_score.is_finite() {
                    term_scores.insert(term.clone(), single_term_score);
                }
                // Note: Zero or negative scores are mathematically valid results, not errors
            }
            
            if score != 0.0 && score.is_finite() {
                matches.push(BM25Match {
                    doc_id,
                    score,
                    term_scores,
                    matched_terms,
                });
            }
        }
        
        // Validate all scores are finite before sorting - PRINCIPLE 0: No NaN fallbacks
        for (idx, match_result) in matches.iter().enumerate() {
            if !match_result.score.is_finite() {
                return Err(anyhow::anyhow!(
                    "BM25 score calculation produced invalid result (NaN or infinite) for document '{}' (index {}). Score: {}. This indicates mathematical corruption in BM25 computation and cannot be recovered from.",
                    match_result.doc_id, idx, match_result.score
                ));
            }
        }
        
        // Sort by score descending - safe after validation
        matches.sort_by(|a, b| {
            b.score.partial_cmp(&a.score).unwrap() // Safe after validation
        });
        
        // Return top results
        matches.truncate(limit);
        Ok(matches)
    }
    
    /// Get statistics about the index
    pub fn get_stats(&self) -> IndexStats {
        IndexStats {
            total_documents: self.total_docs,
            total_terms: self.term_frequencies.len(),
            avg_document_length: self.avg_doc_length,
            k1: self.k1,
            b: self.b,
        }
    }
    
    /// Clear the entire index
    pub fn clear(&mut self) {
        self.total_docs = 0;
        self.avg_doc_length = 0.0;
        self.term_frequencies.clear();
        self.document_lengths.clear();
        self.inverted_index.clear();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexStats {
    pub total_documents: usize,
    pub total_terms: usize,
    pub avg_document_length: f32,
    pub k1: f32,
    pub b: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_bm25_basic() {
        let mut engine = BM25Engine::new();
        
        // Add test documents
        let doc1 = BM25Document {
            id: "doc1".to_string(),
            file_path: "test1.rs".to_string(),
            chunk_index: 0,
            tokens: vec![
                Token { text: "quick".to_string(), position: 0, importance_weight: 1.0 },
                Token { text: "brown".to_string(), position: 1, importance_weight: 1.0 },
                Token { text: "fox".to_string(), position: 2, importance_weight: 1.0 },
            ],
            start_line: 0,
            end_line: 10,
            language: Some("rust".to_string()),
        };
        
        let doc2 = BM25Document {
            id: "doc2".to_string(),
            file_path: "test2.rs".to_string(),
            chunk_index: 0,
            tokens: vec![
                Token { text: "quick".to_string(), position: 0, importance_weight: 1.0 },
                Token { text: "quick".to_string(), position: 1, importance_weight: 1.0 },
                Token { text: "dog".to_string(), position: 2, importance_weight: 1.0 },
            ],
            start_line: 0,
            end_line: 10,
            language: Some("rust".to_string()),
        };
        
        engine.add_document(doc1).unwrap();
        engine.add_document(doc2).unwrap();
        
        // Search for "quick"
        let results = engine.search("quick", 10).expect("Search must succeed in test");
        assert_eq!(results.len(), 2);
        
        // doc2 should score higher due to higher term frequency
        assert!(results[0].doc_id == "doc2");
        assert!(results[0].score > results[1].score);
        
        // Search for "fox"
        let results = engine.search("fox", 10).expect("Search must succeed in test");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].doc_id, "doc1");
    }
    
    #[test]
    fn test_idf_calculation() {
        let mut engine = BM25Engine::new();
        
        // Add documents with varying term frequencies
        for i in 0..10 {
            let mut tokens = vec![
                Token { text: "common".to_string(), position: 0, importance_weight: 1.0 },
            ];
            
            if i < 2 {
                tokens.push(Token { text: "rare".to_string(), position: 1, importance_weight: 1.0 });
            }
            
            let doc = BM25Document {
                id: format!("doc{}", i),
                file_path: format!("test{}.rs", i),
                chunk_index: 0,
                tokens,
                start_line: 0,
                end_line: 10,
                language: Some("rust".to_string()),
            };
            
            engine.add_document(doc).unwrap();
        }
        
        // IDF of common term should be lower than rare term
        let idf_common = engine.calculate_idf("common");
        let idf_rare = engine.calculate_idf("rare");
        
        assert!(idf_rare > idf_common);
    }
}