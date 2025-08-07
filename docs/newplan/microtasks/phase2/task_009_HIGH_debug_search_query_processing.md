# Task 2.009: Debug BM25 Search Query Processing

**Time Estimate**: 10 minutes
**Priority**: HIGH
**Dependencies**: task_008
**File(s) to Modify**: `src/search/bm25.rs`

## Objective
Debug the BM25 search method to understand why queries are not finding indexed terms, focusing on query tokenization and term matching.

## Success Criteria
- [ ] Query terms are properly tokenized
- [ ] Term matching in inverted index works
- [ ] BM25 scoring produces non-zero scores
- [ ] Search returns expected results

## Instructions

### Step 1: Add comprehensive BM25 search debugging
```rust
// In src/search/bm25.rs, update the search method with detailed logging
impl BM25Engine {
    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<BM25Match>, anyhow::Error> {
        println!("🔍 BM25 DEBUG: Starting search for query: '{}'", query);
        println!("🔍 BM25 DEBUG: Index stats - docs: {}, terms: {}", 
                 self.total_docs, self.term_frequencies.len());
        
        // Tokenize query (simple whitespace split for now)
        let query_terms: Vec<String> = query
            .split_whitespace()
            .map(|s| s.to_lowercase())
            .filter(|s| !s.is_empty())
            .collect();
        
        println!("🔍 BM25 DEBUG: Query terms: {:?}", query_terms);
        
        if query_terms.is_empty() {
            return Err(anyhow::anyhow!("Empty query provided to BM25 search"));
        }
        
        // Check if any query terms exist in our index
        for term in &query_terms {
            if let Some(doc_terms) = self.inverted_index.get(term) {
                println!("🔍 BM25 DEBUG: Term '{}' found in {} documents", term, doc_terms.len());
                for (i, doc_term) in doc_terms.iter().take(3).enumerate() {
                    println!("   Doc {}: id={}, freq={}", i, doc_term.doc_id, doc_term.term_frequency);
                }
            } else {
                println!("🔍 BM25 DEBUG: Term '{}' NOT found in inverted index", term);
            }
        }
        
        // Debug: Show some terms that ARE in the index
        println!("🔍 BM25 DEBUG: Sample terms in index: {:?}", 
                 self.inverted_index.keys().take(10).collect::<Vec<_>>());
        
        // Find all documents that contain at least one query term
        let mut candidate_docs: HashMap<String, Vec<String>> = HashMap::new();
        
        for term in &query_terms {
            if let Some(doc_terms) = self.inverted_index.get(term) {
                println!("🔍 BM25 DEBUG: Processing term '{}' with {} docs", term, doc_terms.len());
                for doc_term in doc_terms {
                    candidate_docs.entry(doc_term.doc_id.clone())
                        .or_insert_with(Vec::new)
                        .push(term.clone());
                }
            }
        }
        
        println!("🔍 BM25 DEBUG: Found {} candidate documents", candidate_docs.len());
        
        // Calculate BM25 scores for candidate documents
        let mut matches: Vec<BM25Match> = Vec::new();
        
        for (doc_id, matched_terms) in candidate_docs {
            println!("🔍 BM25 DEBUG: Calculating score for document: {}", doc_id);
            
            let score = self.calculate_bm25_score(&query_terms, &doc_id)
                .with_context(|| format!("BM25 calculation failed for document '{}'", doc_id))?;
            
            println!("🔍 BM25 DEBUG: Document '{}' scored: {}", doc_id, score);
            
            if score > 0.0 {
                // Calculate individual term contributions for debugging
                let mut term_scores = HashMap::new();
                for term in &query_terms {
                    let single_term_score = self.calculate_bm25_score(&[term.clone()], &doc_id)?;
                    if single_term_score > 0.0 {
                        term_scores.insert(term.clone(), single_term_score);
                    }
                }
                
                matches.push(BM25Match {
                    doc_id,
                    score,
                    term_scores,
                    matched_terms,
                });
                
                println!("🔍 BM25 DEBUG: Added match with score: {}", score);
            } else {
                println!("🔍 BM25 DEBUG: Document '{}' scored 0, skipping", doc_id);
            }
        }
        
        println!("🔍 BM25 DEBUG: Total matches before sorting: {}", matches.len());
        
        // Validate all scores are finite before sorting
        for (idx, match_result) in matches.iter().enumerate() {
            if !match_result.score.is_finite() {
                return Err(anyhow::anyhow!(
                    "BM25 score calculation produced invalid result for document '{}' (index {}). Score: {}",
                    match_result.doc_id, idx, match_result.score
                ));
            }
        }
        
        // Sort by score descending
        matches.sort_by(|a, b| {
            b.score.partial_cmp(&a.score).unwrap()
        });
        
        // Return top results
        matches.truncate(limit);
        
        println!("🔍 BM25 DEBUG: Returning {} final matches", matches.len());
        for (i, m) in matches.iter().enumerate() {
            println!("   Match {}: doc={}, score={}", i, m.doc_id, m.score);
        }
        
        Ok(matches)
    }
}
```

### Step 2: Debug BM25 scoring calculation
```rust
// Add debugging to calculate_bm25_score method
pub fn calculate_bm25_score(&self, query_terms: &[String], doc_id: &str) -> Result<f32, anyhow::Error> {
    println!("🔍 SCORE DEBUG: Calculating BM25 for doc '{}' with terms: {:?}", doc_id, query_terms);
    
    let doc_length = *self.document_lengths.get(doc_id)
        .ok_or_else(|| anyhow::anyhow!("Document '{}' not found in BM25 index", doc_id))? as f32;
    
    println!("🔍 SCORE DEBUG: Document length: {}, avg length: {}", doc_length, self.avg_doc_length);
    
    let mut score = 0.0;
    
    for term in query_terms {
        let term_lower = term.to_lowercase();
        let idf = self.calculate_idf(&term_lower);
        
        println!("🔍 SCORE DEBUG: Term '{}' IDF: {}", term_lower, idf);
        
        // Find term frequency in this document
        let tf = match self.inverted_index.get(&term_lower) {
            Some(docs) => {
                match docs.iter().find(|dt| dt.doc_id == doc_id) {
                    Some(doc_term) => {
                        println!("🔍 SCORE DEBUG: Term '{}' TF in doc '{}': {}", 
                                term_lower, doc_id, doc_term.term_frequency);
                        doc_term.term_frequency as f32
                    }
                    None => {
                        println!("🔍 SCORE DEBUG: Term '{}' not found in doc '{}'", term_lower, doc_id);
                        0.0
                    }
                }
            }
            None => {
                println!("🔍 SCORE DEBUG: Term '{}' not in inverted index", term_lower);
                0.0
            }
        };
        
        if tf > 0.0 {
            // BM25 formula: IDF * (tf * (k1 + 1)) / (tf + k1 * (1 - b + b * (doc_len / avg_doc_len)))
            let norm_factor = 1.0 - self.b + self.b * (doc_length / self.avg_doc_length);
            let term_score = idf * (tf * (self.k1 + 1.0)) / (tf + self.k1 * norm_factor);
            
            println!("🔍 SCORE DEBUG: Term '{}' contribution: {} (tf={}, idf={}, norm={})", 
                     term_lower, term_score, tf, idf, norm_factor);
            
            score += term_score;
        }
    }
    
    println!("🔍 SCORE DEBUG: Final score for doc '{}': {}", doc_id, score);
    Ok(score)
}
```

### Step 3: Run search with detailed debugging
```bash
cd C:\code\embed
cargo test test_bm25_exact_terms -- --nocapture
```

### Step 4: Analyze debug output
Look for these potential issues:
- Query terms not matching indexed terms (case/normalization)
- Inverted index empty or missing expected terms
- BM25 scoring returning 0 due to math errors
- Term frequencies not calculated correctly

## Terminal Commands
```bash
cd C:\code\embed
cargo test test_bm25_exact_terms -- --nocapture
cargo test test_bm25_known_terms -- --nocapture
```

## Expected Debug Output Analysis
- Query terms should match some indexed terms
- Candidate documents should be found
- BM25 scores should be positive for matching documents
- Final matches should be returned

## Troubleshooting
- If no terms match: Check tokenization consistency between indexing and search
- If scores are 0: Check IDF and TF calculations
- If no candidates found: Verify inverted index is populated
- If math errors: Check for division by zero or NaN values

## Next Task
task_010 - Fix tokenization consistency between indexing and search