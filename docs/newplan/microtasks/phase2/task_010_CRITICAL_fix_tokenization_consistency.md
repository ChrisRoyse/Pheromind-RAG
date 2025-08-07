# Task 2.010: Fix Tokenization Consistency Between Indexing and Search

**Time Estimate**: 10 minutes
**Priority**: CRITICAL
**Dependencies**: task_009
**File(s) to Modify**: `src/search/bm25.rs`, `src/search/unified.rs`

## Objective
Ensure that query tokenization in the search method uses the same logic as document tokenization during indexing to fix term matching issues.

## Success Criteria
- [ ] Query terms are tokenized identically to document terms
- [ ] Case normalization is consistent
- [ ] Word filtering rules are identical
- [ ] Term matching works correctly

## Instructions

### Step 1: Create consistent tokenization function in BM25
```rust
// In src/search/bm25.rs, add a shared tokenization method
impl BM25Engine {
    /// Consistent tokenization used for both indexing and searching
    pub fn tokenize_text(&self, text: &str) -> Vec<String> {
        println!("🔍 TOKENIZE: Input text: '{}'", text.chars().take(50).collect::<String>());
        
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
        
        println!("🔍 TOKENIZE: Generated {} tokens: {:?}", tokens.len(), 
                 tokens.iter().take(10).collect::<Vec<_>>());
        
        tokens
    }
}
```

### Step 2: Update document indexing to use shared tokenization
```rust
// In the add_document method, update token processing
pub fn add_document(&mut self, doc: BM25Document) -> Result<()> {
    let doc_id = doc.id.clone();
    let doc_length = doc.tokens.len();
    
    println!("🔍 INDEX DEBUG: Adding document: {} with {} tokens", doc_id, doc_length);
    
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
        // Use consistent tokenization: normalize term to lowercase
        let term = token.text.to_lowercase();
        
        println!("🔍 INDEX DEBUG: Processing token '{}' -> '{}'", token.text, term);
        
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
        
        println!("🔍 INDEX DEBUG: Term '{}' appears {} times in positions: {:?}", 
                 term, freq, positions.iter().take(3).collect::<Vec<_>>());
        
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
    
    println!("🔍 INDEX DEBUG: Document indexed. Total docs: {}, total terms: {}", 
             self.total_docs, self.term_frequencies.len());
    
    Ok(())
}
```

### Step 3: Update search to use consistent tokenization
```rust
// Update the search method to use the shared tokenization
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
    
    // Rest of existing search logic...
    // Find candidate documents, calculate scores, etc.
    
    Ok(Vec::new()) // Placeholder - implement full search in next task
}
```

### Step 4: Update UnifiedSearcher tokenization to match
```rust
// In src/search/unified.rs, update convert_file_to_bm25_document to use consistent tokenization
async fn convert_file_to_bm25_document(&self, file_path: &Path) -> Result<BM25Document, SearchError> {
    println!("🔍 CONVERSION DEBUG: Converting file: {:?}", file_path);
    
    let content = fs::read_to_string(file_path).await
        .map_err(|e| SearchError::FileReadError(file_path.to_path_buf(), e))?;
    
    println!("🔍 CONVERSION DEBUG: File content length: {}", content.len());
    
    // Use the same tokenization logic as BM25Engine
    let token_strings: Vec<String> = content
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
    
    // Convert to Token structures
    let tokens: Vec<Token> = token_strings
        .into_iter()
        .enumerate()
        .map(|(pos, text)| Token {
            text,
            position: pos,
            importance_weight: 1.0,
        })
        .collect();
    
    println!("🔍 CONVERSION DEBUG: Generated {} tokens", tokens.len());
    
    // Rest of document creation...
    let doc = BM25Document {
        id: format!("file_{}", file_path.file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .replace(".", "_")),
        file_path: file_path.to_string_lossy().to_string(),
        chunk_index: 0,
        tokens,
        start_line: 0,
        end_line: content.lines().count(),
        language: None, // Simplified for now
    };
    
    Ok(doc)
}
```

### Step 5: Test tokenization consistency
```bash
cd C:\code\embed
cargo test test_bm25_exact_terms -- --nocapture
```

## Terminal Commands
```bash
cd C:\code\embed
cargo check --features core
cargo test test_bm25_exact_terms -- --nocapture
cargo test test_bm25_known_terms -- --nocapture
```

## Expected Results
- Query terms should now match indexed terms
- Debug output should show "✅ Term found" for query terms
- Search should find candidate documents

## Critical Success Indicators
- Query "hello" should find term "hello" in index
- Debug shows terms_found > 0
- Candidate documents are identified

## Next Task
task_011 - Implement complete BM25 search logic