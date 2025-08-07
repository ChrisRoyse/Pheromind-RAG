# Task 2.013: Clean Up Debug Logging

**Time Estimate**: 10 minutes
**Priority**: LOW
**Dependencies**: task_012
**File(s) to Modify**: `src/search/bm25.rs`, `src/search/unified.rs`, `tests/bm25_integration_tests.rs`

## Objective
Remove or conditionally compile debug logging statements added during debugging to clean up the code for production.

## Success Criteria
- [ ] Debug println! statements are removed or gated
- [ ] Code is clean and production-ready
- [ ] Tests still pass without debug output
- [ ] Optional debug feature flag works

## Instructions

### Step 1: Add conditional debug logging macro
```rust
// In src/search/bm25.rs, at the top of the file
// Add conditional debug logging
macro_rules! debug_log {
    ($($arg:tt)*) => {
        #[cfg(feature = "debug-search")]
        println!($($arg)*);
    };
}
```

### Step 2: Replace debug statements in BM25Engine
```rust
// In src/search/bm25.rs, replace println! with debug_log!
impl BM25Engine {
    pub fn tokenize_text(&self, text: &str) -> Vec<String> {
        debug_log!("🔍 TOKENIZE: Input text: '{}'", text.chars().take(50).collect::<String>());
        
        let tokens: Vec<String> = text
            .split_whitespace()
            .filter_map(|word| {
                let clean_word = word.chars()
                    .filter(|c| c.is_alphanumeric() || *c == '_')
                    .collect::<String>()
                    .to_lowercase();
                
                if !clean_word.is_empty() && clean_word.len() > 1 {
                    Some(clean_word)
                } else {
                    None
                }
            })
            .collect();
        
        debug_log!("🔍 TOKENIZE: Generated {} tokens", tokens.len());
        tokens
    }
    
    pub fn add_document(&mut self, doc: BM25Document) -> Result<()> {
        let doc_id = doc.id.clone();
        let doc_length = doc.tokens.len();
        
        debug_log!("🔍 INDEX DEBUG: Adding document: {} with {} tokens", doc_id, doc_length);
        
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
            let term = token.text.to_lowercase();
            
            term_positions.entry(term.clone())
                .or_insert_with(Vec::new)
                .push(pos);
            
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
        
        debug_log!("🔍 INDEX DEBUG: Document indexed. Total docs: {}, total terms: {}", 
                  self.total_docs, self.term_frequencies.len());
        
        Ok(())
    }
    
    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<BM25Match>, anyhow::Error> {
        debug_log!("🔍 BM25 DEBUG: Starting search for query: '{}'", query);
        
        let query_terms = self.tokenize_text(query);
        
        debug_log!("🔍 BM25 DEBUG: Tokenized query terms: {:?}", query_terms);
        
        if query_terms.is_empty() {
            return Err(anyhow::anyhow!("Empty query after tokenization"));
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
        
        debug_log!("🔍 BM25 DEBUG: Found {} candidate documents", candidate_docs.len());
        
        // Calculate BM25 scores for candidate documents
        let mut matches: Vec<BM25Match> = Vec::new();
        
        for (doc_id, matched_terms) in candidate_docs {
            if let Ok(score) = self.calculate_bm25_score(&query_terms, &doc_id) {
                if score > 0.0 {
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
                }
            }
        }
        
        // Sort by score descending
        matches.sort_by(|a, b| {
            b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        // Return top results
        matches.truncate(limit);
        
        debug_log!("🔍 BM25 DEBUG: Returning {} final matches", matches.len());
        
        Ok(matches)
    }
    
    pub fn calculate_bm25_score(&self, query_terms: &[String], doc_id: &str) -> Result<f32, anyhow::Error> {
        let doc_length = *self.document_lengths.get(doc_id)
            .ok_or_else(|| anyhow::anyhow!("Document '{}' not found in BM25 index", doc_id))? as f32;
        
        let avg_length = if self.avg_doc_length > 0.0 { self.avg_doc_length } else { doc_length };
        let mut score = 0.0;
        
        for term in query_terms {
            let idf = self.calculate_idf(term);
            
            let tf = match self.inverted_index.get(term) {
                Some(docs) => {
                    match docs.iter().find(|dt| dt.doc_id == doc_id) {
                        Some(doc_term) => doc_term.term_frequency as f32,
                        None => 0.0,
                    }
                }
                None => 0.0,
            };
            
            if tf > 0.0 {
                let norm_factor = 1.0 - self.b + self.b * (doc_length / avg_length);
                let norm_factor = if norm_factor > 0.0 { norm_factor } else { 1.0 };
                let term_score = idf * (tf * (self.k1 + 1.0)) / (tf + self.k1 * norm_factor);
                
                if term_score.is_finite() {
                    score += term_score;
                }
            }
        }
        
        debug_log!("🔍 SCORE DEBUG: Final score for doc '{}': {:.4}", doc_id, score);
        Ok(score)
    }
}
```

### Step 3: Clean up UnifiedSearcher debug statements
```rust
// In src/search/unified.rs, add the same debug macro and clean up
macro_rules! debug_log {
    ($($arg:tt)*) => {
        #[cfg(feature = "debug-search")]
        println!($($arg)*);
    };
}

impl UnifiedSearcher {
    pub async fn verify_bm25_index(&self) -> Result<(), SearchError> {
        if let Some(ref bm25_arc) = self.bm25_engine {
            let bm25_engine = bm25_arc.lock()
                .map_err(|_| SearchError::SearchExecutionError("BM25 engine lock poisoned".to_string()))?;
            
            let stats = bm25_engine.get_stats();
            debug_log!("🔍 INDEX STATS: Total documents: {}", stats.total_documents);
            debug_log!("🔍 INDEX STATS: Total terms: {}", stats.total_terms);
            debug_log!("🔍 INDEX STATS: Average document length: {:.2}", stats.avg_document_length);
            
            if stats.total_documents == 0 {
                return Err(SearchError::IndexingError("BM25 index is empty".to_string()));
            }
            
            if stats.total_terms == 0 {
                return Err(SearchError::IndexingError("BM25 index has no terms".to_string()));
            }
            
            debug_log!("✅ INDEX VERIFICATION: BM25 index appears healthy");
        }
        
        Ok(())
    }
    
    pub async fn index_file(&self, file_path: &Path) -> Result<(), SearchError> {
        debug_log!("🔍 UNIFIED DEBUG: Indexing file: {:?}", file_path);
        
        if let Some(ref bm25_arc) = self.bm25_engine {
            let doc = self.convert_file_to_bm25_document(file_path).await?;
            
            debug_log!("🔍 UNIFIED DEBUG: Document created - ID: {}, tokens: {}", 
                      doc.id, doc.tokens.len());
            
            let mut bm25_engine = bm25_arc.lock()
                .map_err(|_| SearchError::IndexingError("BM25 engine lock poisoned".to_string()))?;
            
            bm25_engine.add_document(doc)
                .map_err(|e| SearchError::IndexingError(format!("BM25 indexing failed: {}", e)))?;
            
            debug_log!("🔍 UNIFIED DEBUG: Document added successfully");
        }
        
        Ok(())
    }
    
    pub async fn search(&self, query: &str) -> Result<Vec<SearchResult>, SearchError> {
        debug_log!("🔍 UNIFIED DEBUG: Search query received: '{}'", query);
        
        let mut all_results = Vec::new();
        
        if let Some(ref bm25_arc) = self.bm25_engine {
            let bm25_engine = bm25_arc.lock()
                .map_err(|_| SearchError::SearchExecutionError("BM25 engine lock poisoned".to_string()))?;
            
            match bm25_engine.search(query, 50) {
                Ok(bm25_results) => {
                    debug_log!("🔍 UNIFIED DEBUG: BM25 returned {} results", bm25_results.len());
                    
                    drop(bm25_engine);
                    
                    for bm25_match in bm25_results {
                        let search_result = self.convert_bm25_to_search_result(bm25_match).await?;
                        all_results.push(search_result);
                    }
                }
                Err(e) => {
                    debug_log!("🔍 UNIFIED DEBUG: BM25 search failed: {}", e);
                }
            }
        }
        
        debug_log!("🔍 UNIFIED DEBUG: Returning {} total results", all_results.len());
        Ok(all_results)
    }
}
```

### Step 4: Clean up test files
```rust
// In tests/bm25_integration_tests.rs, remove excessive debug output
// Keep only essential test output
#[tokio::test]
async fn test_bm25_basic_search() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let project_path = temp_dir.path().to_path_buf();
    let db_path = project_path.join(".embed_db");
    
    create_test_codebase(&project_path).await?;
    
    let searcher = UnifiedSearcher::new_with_backend(
        project_path.clone(),
        db_path,
        SearchBackend::Tantivy
    ).await?;
    
    // Index files
    for entry in std::fs::read_dir(&project_path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            searcher.index_file(&path).await?;
        }
    }
    
    // Test search
    let results = searcher.search("database connection").await?;
    
    // This assertion should now pass
    assert!(!results.is_empty(), "Should find results for 'database connection'");
    
    println!("✅ BM25 basic search test passed with {} results", results.len());
    
    Ok(())
}
```

### Step 5: Add debug feature to Cargo.toml
```toml
# In Cargo.toml, add debug feature
[features]
default = ["core"]
core = []
vectordb = []
debug-search = []  # Enable debug logging for search
```

### Step 6: Test clean code
```bash
cd C:\code\embed
# Test without debug output
cargo test test_bm25_basic_search

# Test with debug output
cargo test test_bm25_basic_search --features debug-search -- --nocapture
```

## Terminal Commands
```bash
cd C:\code\embed
cargo check --features core
cargo test test_bm25_basic_search
cargo test test_bm25_basic_search --features debug-search -- --nocapture
```

## Expected Results
- Tests pass without debug output (clean)
- Tests pass with debug output when feature is enabled
- Code is clean and production-ready

## Next Task
task_014 - Add comprehensive BM25 unit tests