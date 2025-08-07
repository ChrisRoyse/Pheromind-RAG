# Phase 2: Core Search Repair - Make BM25 Actually Work

**Duration**: 1 day  
**Goal**: Fix BM25 search to return actual results  
**Success Metric**: `test_bm25_basic` passes

## The Core Problem

BM25 search returns 0 results when it should return 2. This is a logic bug, not a compilation issue.

## Task 2.1: Debug the Indexing Pipeline (2 hours)

### Investigation Steps

```rust
// File: src/search/bm25.rs
// Add debug output to understand what's happening

impl BM25Engine {
    pub fn add_document(&mut self, doc_id: String, content: String) -> Result<()> {
        println!("DEBUG: Adding document: {}", doc_id);
        println!("DEBUG: Content length: {}", content.len());
        
        let processed = self.preprocess_text(&content);
        println!("DEBUG: Processed tokens: {:?}", processed.tokens);
        
        // Existing code...
        
        println!("DEBUG: Document added to index");
        println!("DEBUG: Total documents: {}", self.documents.len());
        Ok(())
    }
}
```

### Run the failing test with debug output:
```bash
RUST_LOG=debug cargo test test_bm25_basic -- --nocapture
```

### Likely Issues to Check

1. **Documents not being stored**
   ```rust
   // Check if documents HashMap is actually populated
   assert!(!self.documents.is_empty(), "No documents in index!");
   ```

2. **Term frequencies not calculated**
   ```rust
   // Check if term_doc_freqs is populated
   assert!(!self.term_doc_freqs.is_empty(), "No term frequencies!");
   ```

3. **Preprocessing removing all tokens**
   ```rust
   // Check if preprocessing is too aggressive
   let tokens = self.preprocess_text("test document");
   assert!(!tokens.is_empty(), "Preprocessing removed all tokens!");
   ```

## Task 2.2: Fix the Inverted Index (2 hours)

### The Likely Bug

The inverted index is probably not being built correctly.

```rust
// File: src/search/bm25.rs
// Check the index building logic

pub fn build_index(&mut self) -> Result<()> {
    // Make sure this is actually called
    println!("DEBUG: Building inverted index");
    
    for (doc_id, doc) in &self.documents {
        for term in &doc.tokens {
            // This might be the problem - not updating the index
            self.inverted_index
                .entry(term.clone())
                .or_insert_with(HashSet::new())
                .insert(doc_id.clone());
                
            // Also update term document frequencies
            *self.term_doc_freqs.entry(term.clone()).or_insert(0) += 1;
        }
    }
    
    println!("DEBUG: Inverted index size: {}", self.inverted_index.len());
    Ok(())
}
```

### Verify the Fix

```rust
#[test]
fn test_inverted_index_building() {
    let mut engine = BM25Engine::new(1.2, 0.75);
    engine.add_document("doc1".to_string(), "hello world".to_string()).unwrap();
    engine.build_index().unwrap();
    
    // Index should contain terms
    assert!(engine.inverted_index.contains_key("hello"));
    assert!(engine.inverted_index.contains_key("world"));
    
    // Terms should map to documents
    assert!(engine.inverted_index["hello"].contains("doc1"));
}
```

## Task 2.3: Fix the Search Logic (2 hours)

### Debug the Search Function

```rust
// File: src/search/bm25.rs
pub fn search(&self, query: &str, top_k: usize) -> Vec<BM25Match> {
    println!("DEBUG: Searching for: {}", query);
    
    let query_terms = self.preprocess_text(query);
    println!("DEBUG: Query terms: {:?}", query_terms);
    
    let mut scores = HashMap::new();
    
    for term in query_terms {
        println!("DEBUG: Processing term: {}", term);
        
        if let Some(doc_ids) = self.inverted_index.get(&term) {
            println!("DEBUG: Found {} documents for term", doc_ids.len());
            
            for doc_id in doc_ids {
                let score = self.calculate_bm25_score(&term, doc_id);
                println!("DEBUG: Score for {} in {}: {}", term, doc_id, score);
                
                *scores.entry(doc_id.clone()).or_insert(0.0) += score;
            }
        } else {
            println!("DEBUG: Term '{}' not in index!", term);
        }
    }
    
    println!("DEBUG: Total scored documents: {}", scores.len());
    
    // Convert to matches and sort
    let mut matches: Vec<BM25Match> = scores.into_iter()
        .map(|(doc_id, score)| BM25Match {
            document_id: doc_id,
            score,
            // Add other fields
        })
        .collect();
        
    matches.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
    matches.truncate(top_k);
    
    println!("DEBUG: Returning {} matches", matches.len());
    matches
}
```

## Task 2.4: Fix the Failing Test (1 hour)

### The Test That's Failing

```rust
#[test]
fn test_bm25_basic() {
    let mut engine = BM25Engine::new(1.2, 0.75);
    
    // Add test documents
    engine.add_document(
        "doc1".to_string(),
        "The quick brown fox jumps over the lazy dog".to_string()
    ).unwrap();
    
    engine.add_document(
        "doc2".to_string(),
        "The lazy dog sleeps all day".to_string()
    ).unwrap();
    
    // THIS IS PROBABLY MISSING - BUILD THE INDEX!
    engine.build_index().unwrap();
    
    // Search
    let results = engine.search("lazy dog", 10);
    
    // This assertion fails: expecting 2, getting 0
    assert_eq!(results.len(), 2);
}
```

### The Fix

Most likely the test is not calling `build_index()` after adding documents. The inverted index needs to be explicitly built.

## Task 2.5: Verify All BM25 Functionality (1 hour)

### Create Comprehensive Tests

```rust
#[cfg(test)]
mod comprehensive_tests {
    use super::*;
    
    #[test]
    fn test_empty_index() {
        let engine = BM25Engine::new(1.2, 0.75);
        let results = engine.search("test", 10);
        assert_eq!(results.len(), 0);
    }
    
    #[test]
    fn test_single_document() {
        let mut engine = BM25Engine::new(1.2, 0.75);
        engine.add_document("doc1".to_string(), "test content".to_string()).unwrap();
        engine.build_index().unwrap();
        
        let results = engine.search("test", 10);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].document_id, "doc1");
    }
    
    #[test]
    fn test_ranking_order() {
        let mut engine = BM25Engine::new(1.2, 0.75);
        engine.add_document("doc1".to_string(), "cat cat cat".to_string()).unwrap();
        engine.add_document("doc2".to_string(), "cat dog".to_string()).unwrap();
        engine.build_index().unwrap();
        
        let results = engine.search("cat", 10);
        assert_eq!(results[0].document_id, "doc1"); // Should rank higher
        assert!(results[0].score > results[1].score);
    }
}
```

### Run All Tests

```bash
cargo test --features core bm25
# ALL tests must pass
```

## Success Criteria

- [ ] `test_bm25_basic` passes
- [ ] `test_idf_calculation` still passes
- [ ] Documents are properly indexed
- [ ] Inverted index is populated
- [ ] Search returns correct number of results
- [ ] Results are properly ranked by score
- [ ] No debug println! statements left in code

## Common Pitfalls to Avoid

1. **Not calling build_index()** - Most common issue
2. **Preprocessing too aggressive** - Removing all tokens
3. **Case sensitivity issues** - Make sure lowercase is consistent
4. **Empty stop words** - Don't filter everything out
5. **HashMap not initialized** - Check all collections are created

## Performance Check

After fixing:
```bash
cargo bench --features core bm25
```

Expected performance:
- Indexing: <1ms per document
- Search: <1ms for 1000 documents

## Next Phase

Only proceed to Phase 3 (Tantivy Resurrection) after BM25 search works correctly.