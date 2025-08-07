# Task 2.011: Implement Complete BM25 Search Logic

**Time Estimate**: 10 minutes
**Priority**: HIGH
**Dependencies**: task_010
**File(s) to Modify**: `src/search/bm25.rs`

## Objective
Complete the BM25 search implementation by filling in the candidate document finding and scoring logic that was simplified in the previous task.

## Success Criteria
- [ ] Search finds all candidate documents containing query terms
- [ ] BM25 scores are calculated for all candidates
- [ ] Results are sorted by score
- [ ] Non-zero scores are returned

## Instructions

### Step 1: Complete the search method implementation
```rust
// In src/search/bm25.rs, complete the search method
pub fn search(&self, query: &str, limit: usize) -> Result<Vec<BM25Match>, anyhow::Error> {
    println!("🔍 BM25 DEBUG: Starting search for query: '{}'", query);
    
    // Use consistent tokenization
    let query_terms = self.tokenize_text(query);
    
    println!("🔍 BM25 DEBUG: Tokenized query terms: {:?}", query_terms);
    
    if query_terms.is_empty() {
        return Err(anyhow::anyhow!("Empty query after tokenization"));
    }
    
    // Check if any query terms exist in our index
    let mut terms_found = 0;
    for term in &query_terms {
        if let Some(doc_terms) = self.inverted_index.get(term) {
            println!("🔍 BM25 DEBUG: ✅ Term '{}' found in {} documents", term, doc_terms.len());
            terms_found += 1;
        } else {
            println!("🔍 BM25 DEBUG: ❌ Term '{}' NOT found in inverted index", term);
        }
    }
    
    println!("🔍 BM25 DEBUG: Found {}/{} query terms in index", terms_found, query_terms.len());
    
    if terms_found == 0 {
        println!("🔍 BM25 DEBUG: No query terms found in index - returning empty results");
        return Ok(Vec::new());
    }
    
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
    for (doc_id, terms) in candidate_docs.iter().take(3) {
        println!("   Candidate: {} with terms: {:?}", doc_id, terms);
    }
    
    // Calculate BM25 scores for candidate documents
    let mut matches: Vec<BM25Match> = Vec::new();
    
    for (doc_id, matched_terms) in candidate_docs {
        println!("🔍 BM25 DEBUG: Calculating score for document: {}", doc_id);
        
        match self.calculate_bm25_score(&query_terms, &doc_id) {
            Ok(score) => {
                println!("🔍 BM25 DEBUG: Document '{}' scored: {}", doc_id, score);
                
                if score > 0.0 {
                    // Calculate individual term contributions for debugging
                    let mut term_scores = HashMap::new();
                    for term in &query_terms {
                        if let Ok(single_term_score) = self.calculate_bm25_score(&[term.clone()], &doc_id) {
                            if single_term_score > 0.0 {
                                term_scores.insert(term.clone(), single_term_score);
                            }
                        }
                    }
                    
                    matches.push(BM25Match {
                        doc_id,
                        score,
                        term_scores,
                        matched_terms,
                    });
                    
                    println!("🔍 BM25 DEBUG: ✅ Added match with score: {}", score);
                } else {
                    println!("🔍 BM25 DEBUG: ❌ Document '{}' scored 0, skipping", doc_id);
                }
            }
            Err(e) => {
                println!("🔍 BM25 DEBUG: ❌ Error calculating score for '{}': {}", doc_id, e);
            }
        }
    }
    
    println!("🔍 BM25 DEBUG: Total matches before sorting: {}", matches.len());
    
    // Validate all scores are finite
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
        b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal)
    });
    
    // Return top results
    matches.truncate(limit);
    
    println!("🔍 BM25 DEBUG: Returning {} final matches", matches.len());
    for (i, m) in matches.iter().enumerate() {
        println!("   Match {}: doc={}, score={:.4}", i, m.doc_id, m.score);
    }
    
    Ok(matches)
}
```

### Step 2: Verify BM25 score calculation handles edge cases
```rust
// Update calculate_bm25_score to handle edge cases better
pub fn calculate_bm25_score(&self, query_terms: &[String], doc_id: &str) -> Result<f32, anyhow::Error> {
    println!("🔍 SCORE DEBUG: Calculating BM25 for doc '{}' with terms: {:?}", doc_id, query_terms);
    
    let doc_length = *self.document_lengths.get(doc_id)
        .ok_or_else(|| anyhow::anyhow!("Document '{}' not found in BM25 index", doc_id))? as f32;
    
    println!("🔍 SCORE DEBUG: Document length: {}, avg length: {:.2}", doc_length, self.avg_doc_length);
    
    // Handle edge case: if avg_doc_length is 0
    if self.avg_doc_length <= 0.0 {
        println!("🔍 SCORE DEBUG: Average document length is 0 or negative, using doc length as average");
    }
    
    let avg_length = if self.avg_doc_length > 0.0 { self.avg_doc_length } else { doc_length };
    
    let mut score = 0.0;
    
    for term in query_terms {
        let idf = self.calculate_idf(term);
        println!("🔍 SCORE DEBUG: Term '{}' IDF: {:.4}", term, idf);
        
        // Find term frequency in this document
        let tf = match self.inverted_index.get(term) {
            Some(docs) => {
                match docs.iter().find(|dt| dt.doc_id == doc_id) {
                    Some(doc_term) => {
                        println!("🔍 SCORE DEBUG: Term '{}' TF in doc '{}': {}", 
                                term, doc_id, doc_term.term_frequency);
                        doc_term.term_frequency as f32
                    }
                    None => {
                        println!("🔍 SCORE DEBUG: Term '{}' not found in doc '{}'", term, doc_id);
                        0.0
                    }
                }
            }
            None => {
                println!("🔍 SCORE DEBUG: Term '{}' not in inverted index", term);
                0.0
            }
        };
        
        if tf > 0.0 {
            // BM25 formula: IDF * (tf * (k1 + 1)) / (tf + k1 * (1 - b + b * (doc_len / avg_doc_len)))
            let norm_factor = 1.0 - self.b + self.b * (doc_length / avg_length);
            
            // Ensure norm_factor is not zero
            let norm_factor = if norm_factor > 0.0 { norm_factor } else { 1.0 };
            
            let term_score = idf * (tf * (self.k1 + 1.0)) / (tf + self.k1 * norm_factor);
            
            println!("🔍 SCORE DEBUG: Term '{}' calculation: idf={:.4}, tf={}, k1={}, b={}, norm={:.4}, score={:.4}", 
                     term, idf, tf, self.k1, self.b, norm_factor, term_score);
            
            if term_score.is_finite() {
                score += term_score;
            } else {
                println!("🔍 SCORE DEBUG: Warning - term score not finite for '{}', skipping", term);
            }
        }
    }
    
    println!("🔍 SCORE DEBUG: Final score for doc '{}': {:.4}", doc_id, score);
    Ok(score)
}
```

### Step 3: Test the complete search implementation
```bash
cd C:\code\embed
cargo test test_bm25_exact_terms -- --nocapture
```

### Step 4: Run comprehensive search tests
```bash
cd C:\code\embed
cargo test test_bm25_known_terms -- --nocapture
cargo test test_bm25_basic_search -- --nocapture
```

## Terminal Commands
```bash
cd C:\code\embed
cargo check --features core
cargo test test_bm25_exact_terms -- --nocapture
cargo test test_bm25_known_terms -- --nocapture
```

## Expected Results
- Candidate documents should be found for query terms
- BM25 scores should be calculated successfully
- Matches should be returned with positive scores
- Results should be sorted by score

## Success Indicators
- Debug shows candidate documents > 0
- Debug shows matches with score > 0
- Tests pass with expected result counts

## Troubleshooting
- If no candidates: Check inverted index population
- If scores are 0: Verify TF and IDF calculations
- If math errors: Check for division by zero
- If no matches returned: Verify score > 0 filter

## Next Task
task_012 - Test end-to-end BM25 integration