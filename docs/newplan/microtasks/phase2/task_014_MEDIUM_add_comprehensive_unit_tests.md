# Task 2.014: Add Comprehensive BM25 Unit Tests

**Time Estimate**: 10 minutes
**Priority**: MEDIUM
**Dependencies**: task_013
**File(s) to Modify**: `src/search/bm25.rs`

## Objective
Add comprehensive unit tests directly in the BM25 module to cover all core functionality and edge cases.

## Success Criteria
- [ ] Tokenization is thoroughly tested
- [ ] Index building is tested
- [ ] Search functionality is tested
- [ ] Edge cases are covered
- [ ] All tests pass consistently

## Instructions

### Step 1: Add tokenization tests
```rust
// In src/search/bm25.rs, in the tests module, add comprehensive tests
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_tokenization() {
        let engine = BM25Engine::new();
        
        // Test basic tokenization
        let tokens = engine.tokenize_text("hello world test");
        assert_eq!(tokens, vec!["hello", "world", "test"]);
        
        // Test case normalization
        let tokens = engine.tokenize_text("Hello WORLD Test");
        assert_eq!(tokens, vec!["hello", "world", "test"]);
        
        // Test special characters
        let tokens = engine.tokenize_text("hello_world! test@example.com");
        assert_eq!(tokens, vec!["hello_world", "test", "example", "com"]);
        
        // Test filtering short words
        let tokens = engine.tokenize_text("a to be or not i");
        assert!(tokens.is_empty()); // All words too short
        
        // Test filtering numbers
        let tokens = engine.tokenize_text("hello 123 world 456");
        assert_eq!(tokens, vec!["hello", "world"]);
        
        // Test empty input
        let tokens = engine.tokenize_text("");
        assert!(tokens.is_empty());
        
        // Test whitespace-only input
        let tokens = engine.tokenize_text("   \t\n  ");
        assert!(tokens.is_empty());
    }
    
    #[test]
    fn test_document_indexing() {
        let mut engine = BM25Engine::new();
        
        // Create test document
        let doc = BM25Document {
            id: "test_doc".to_string(),
            file_path: "test.rs".to_string(),
            chunk_index: 0,
            tokens: vec![
                Token { text: "hello".to_string(), position: 0, importance_weight: 1.0 },
                Token { text: "world".to_string(), position: 1, importance_weight: 1.0 },
                Token { text: "hello".to_string(), position: 2, importance_weight: 1.0 },
            ],
            start_line: 0,
            end_line: 10,
            language: Some("rust".to_string()),
        };
        
        // Index the document
        engine.add_document(doc).unwrap();
        
        // Verify index state
        let stats = engine.get_stats();
        assert_eq!(stats.total_documents, 1);
        assert_eq!(stats.total_terms, 2); // "hello" and "world"
        assert_eq!(stats.avg_document_length, 3.0);
        
        // Verify inverted index
        assert!(engine.inverted_index.contains_key("hello"));
        assert!(engine.inverted_index.contains_key("world"));
        
        // Verify term frequencies
        let hello_docs = &engine.inverted_index["hello"];
        assert_eq!(hello_docs.len(), 1);
        assert_eq!(hello_docs[0].doc_id, "test_doc");
        assert_eq!(hello_docs[0].term_frequency, 2); // "hello" appears twice
        
        let world_docs = &engine.inverted_index["world"];
        assert_eq!(world_docs.len(), 1);
        assert_eq!(world_docs[0].doc_id, "test_doc");
        assert_eq!(world_docs[0].term_frequency, 1); // "world" appears once
    }
    
    #[test]
    fn test_idf_calculation() {
        let mut engine = BM25Engine::new();
        
        // Add multiple documents to test IDF
        for i in 0..10 {
            let mut tokens = vec![
                Token { text: "common".to_string(), position: 0, importance_weight: 1.0 },
            ];
            
            // "rare" appears in only 2 out of 10 documents
            if i < 2 {
                tokens.push(Token { text: "rare".to_string(), position: 1, importance_weight: 1.0 });
            }
            
            let doc = BM25Document {
                id: format!("doc_{}", i),
                file_path: format!("test_{}.rs", i),
                chunk_index: 0,
                tokens,
                start_line: 0,
                end_line: 10,
                language: Some("rust".to_string()),
            };
            
            engine.add_document(doc).unwrap();
        }
        
        // Calculate IDF values
        let idf_common = engine.calculate_idf("common");
        let idf_rare = engine.calculate_idf("rare");
        let idf_nonexistent = engine.calculate_idf("nonexistent");
        
        // Rare terms should have higher IDF than common terms
        assert!(idf_rare > idf_common, "Rare term should have higher IDF");
        
        // Non-existent terms should have highest IDF
        assert!(idf_nonexistent > idf_rare, "Non-existent term should have highest IDF");
        
        // Common term IDF should be relatively low (appears in all documents)
        assert!(idf_common < 0.0, "Very common term should have negative IDF");
    }
    
    #[test]
    fn test_bm25_scoring() {
        let mut engine = BM25Engine::new();
        
        // Document 1: "hello world hello"
        let doc1 = BM25Document {
            id: "doc1".to_string(),
            file_path: "test1.rs".to_string(),
            chunk_index: 0,
            tokens: vec![
                Token { text: "hello".to_string(), position: 0, importance_weight: 1.0 },
                Token { text: "world".to_string(), position: 1, importance_weight: 1.0 },
                Token { text: "hello".to_string(), position: 2, importance_weight: 1.0 },
            ],
            start_line: 0,
            end_line: 10,
            language: Some("rust".to_string()),
        };
        
        // Document 2: "hello test"
        let doc2 = BM25Document {
            id: "doc2".to_string(),
            file_path: "test2.rs".to_string(),
            chunk_index: 0,
            tokens: vec![
                Token { text: "hello".to_string(), position: 0, importance_weight: 1.0 },
                Token { text: "test".to_string(), position: 1, importance_weight: 1.0 },
            ],
            start_line: 0,
            end_line: 10,
            language: Some("rust".to_string()),
        };
        
        engine.add_document(doc1).unwrap();
        engine.add_document(doc2).unwrap();
        
        // Test scoring for query "hello"
        let score1 = engine.calculate_bm25_score(&["hello".to_string()], "doc1").unwrap();
        let score2 = engine.calculate_bm25_score(&["hello".to_string()], "doc2").unwrap();
        
        // doc1 should score higher because "hello" appears twice
        assert!(score1 > score2, "Document with higher term frequency should score higher");
        
        // Both scores should be positive
        assert!(score1 > 0.0, "BM25 score should be positive");
        assert!(score2 > 0.0, "BM25 score should be positive");
        
        // Test multi-term query
        let multi_score = engine.calculate_bm25_score(&["hello".to_string(), "world".to_string()], "doc1").unwrap();
        assert!(multi_score > score1, "Multi-term query should score higher when all terms match");
    }
    
    #[test]
    fn test_search_functionality() {
        let mut engine = BM25Engine::new();
        
        // Add test documents
        let docs = vec![
            ("doc1", vec!["rust", "programming", "language"]),
            ("doc2", vec!["rust", "systems", "programming"]),
            ("doc3", vec!["python", "programming", "scripting"]),
            ("doc4", vec!["javascript", "web", "development"]),
        ];
        
        for (doc_id, terms) in docs {
            let tokens: Vec<Token> = terms.into_iter().enumerate()
                .map(|(pos, term)| Token {
                    text: term.to_string(),
                    position: pos,
                    importance_weight: 1.0,
                })
                .collect();
            
            let doc = BM25Document {
                id: doc_id.to_string(),
                file_path: format!("{}.rs", doc_id),
                chunk_index: 0,
                tokens,
                start_line: 0,
                end_line: 10,
                language: Some("rust".to_string()),
            };
            
            engine.add_document(doc).unwrap();
        }
        
        // Test single-term search
        let results = engine.search("rust", 10).unwrap();
        assert_eq!(results.len(), 2); // doc1 and doc2 contain "rust"
        
        // Verify result order (should be sorted by score)
        assert!(results[0].score >= results[1].score);
        
        // Test multi-term search
        let results = engine.search("rust programming", 10).unwrap();
        assert_eq!(results.len(), 2); // doc1 and doc2 contain both terms
        
        // Test search for non-existent term
        let results = engine.search("nonexistent", 10).unwrap();
        assert_eq!(results.len(), 0);
        
        // Test empty query
        assert!(engine.search("", 10).is_err());
        
        // Test limit functionality
        let results = engine.search("programming", 1).unwrap();
        assert_eq!(results.len(), 1); // Should limit to 1 result
    }
    
    #[test]
    fn test_edge_cases() {
        let mut engine = BM25Engine::new();
        
        // Test empty document
        let empty_doc = BM25Document {
            id: "empty".to_string(),
            file_path: "empty.rs".to_string(),
            chunk_index: 0,
            tokens: vec![],
            start_line: 0,
            end_line: 0,
            language: Some("rust".to_string()),
        };
        
        engine.add_document(empty_doc).unwrap();
        
        // Search should not crash with empty documents
        let results = engine.search("test", 10).unwrap();
        assert_eq!(results.len(), 0);
        
        // Test document with single character tokens (should be filtered)
        let single_char_doc = BM25Document {
            id: "single".to_string(),
            file_path: "single.rs".to_string(),
            chunk_index: 0,
            tokens: vec![
                Token { text: "a".to_string(), position: 0, importance_weight: 1.0 },
                Token { text: "i".to_string(), position: 1, importance_weight: 1.0 },
                Token { text: "valid_token".to_string(), position: 2, importance_weight: 1.0 },
            ],
            start_line: 0,
            end_line: 10,
            language: Some("rust".to_string()),
        };
        
        engine.add_document(single_char_doc).unwrap();
        
        // Only valid_token should be searchable
        let results = engine.search("valid_token", 10).unwrap();
        assert_eq!(results.len(), 1);
        
        let results = engine.search("a", 10).unwrap();
        assert_eq!(results.len(), 0); // Single character should not match
    }
}
```

### Step 2: Test all new unit tests
```bash
cd C:\code\embed
cargo test test_tokenization -- --nocapture
cargo test test_document_indexing -- --nocapture
cargo test test_idf_calculation -- --nocapture
cargo test test_bm25_scoring -- --nocapture
cargo test test_search_functionality -- --nocapture
cargo test test_edge_cases -- --nocapture
```

### Step 3: Run all BM25 tests together
```bash
cd C:\code\embed
cargo test bm25 -- --nocapture
```

## Terminal Commands
```bash
cd C:\code\embed
cargo test bm25 -- --nocapture
cargo test -p embed_search --lib search::bm25::tests
```

## Expected Results
- All unit tests pass
- Tests cover tokenization, indexing, IDF, scoring, search, and edge cases
- Tests provide good coverage of BM25 functionality

## Success Indicators
- No test failures
- Tests run quickly (<1 second total)
- Tests catch potential regressions

## Next Task
task_015 - Verify Phase 2 completion and success criteria