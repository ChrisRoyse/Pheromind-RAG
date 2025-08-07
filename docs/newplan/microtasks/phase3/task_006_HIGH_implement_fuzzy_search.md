# Task 3.006: Implement Fuzzy Search Capability

**Time Estimate**: 10 minutes
**Priority**: HIGH
**Dependencies**: task_005
**File(s) to Modify**: `src/search/tantivy_search.rs`

## Objective
Add fuzzy search functionality to handle typos and approximate matches using Levenshtein distance.

## Success Criteria
- [ ] FuzzyTermQuery is implemented correctly
- [ ] Can find documents with typos
- [ ] Distance parameter controls fuzziness
- [ ] Performance is acceptable
- [ ] Results include relevance scoring

## Instructions

### Step 1: Add fuzzy search method
```rust
use tantivy::query::FuzzyTermQuery;
use tantivy::Term;

impl TantivySearch {
    pub fn search_fuzzy(&self, query: &str, max_distance: u8) -> Result<Vec<SearchResult>, TantivyError> {
        let reader = self.index.reader()?;
        let searcher = reader.searcher();
        
        // Build fuzzy query
        let fuzzy_query = FuzzyTermQuery::new(
            Term::from_field_text(self.body_field, query),
            max_distance,  // Levenshtein distance
            true,  // With transpositions
        );
        
        // Execute search
        let top_docs = searcher.search(&fuzzy_query, &TopDocs::with_limit(100))?;
        
        // Convert to results
        let mut results = Vec::new();
        for (score, doc_address) in top_docs {
            let doc = searcher.doc(doc_address)?;
            let mut result = self.doc_to_result(doc)?;
            result.score = score;
            results.push(result);
        }
        
        Ok(results)
    }
}
```

### Step 2: Add required imports
```rust
use tantivy::collector::TopDocs;
use tantivy::query::FuzzyTermQuery;
use tantivy::Term;
```

### Step 3: Create fuzzy search test
```rust
#[test]
fn test_fuzzy_search() {
    let dir = tempdir().unwrap();
    let mut tantivy = TantivySearch::new(dir.path()).unwrap();
    
    // Add documents with typos
    let docs = vec![
        "The quick brown fox",
        "The quikc brown fox",  // Typo in 'quick'
        "The quick browm fox",  // Typo in 'brown'
        "The slow red cat",     // Different content
    ];
    
    for (i, content) in docs.iter().enumerate() {
        let doc = Document {
            content: content.to_string(),
            path: format!("file{}.txt", i),
            chunk_index: 0,
            start_line: 1,
            end_line: 1,
        };
        tantivy.add_document(doc).unwrap();
    }
    tantivy.commit().unwrap();
    
    // Search with correct spelling - should find typos too
    let results = tantivy.search_fuzzy("quick", 1).unwrap();
    assert!(results.len() >= 2); // Should find both correct and "quikc"
    
    // Test with distance 0 (exact match only)
    let exact_results = tantivy.search_fuzzy("quick", 0).unwrap();
    assert_eq!(exact_results.len(), 1); // Only exact match
}
```

## Terminal Commands
```bash
cd C:\code\embed
cargo check --features tantivy
cargo test --features tantivy test_fuzzy_search -v
```

## Troubleshooting
- If FuzzyTermQuery import fails, check Tantivy v0.24 docs
- If no fuzzy results, verify max_distance > 0
- Check that Term::from_field_text uses correct field

## Next Task
task_007 - Check existing index compatibility