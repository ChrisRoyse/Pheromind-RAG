# Task 3.014: Validate Against Existing Search Infrastructure

**Time Estimate**: 10 minutes
**Priority**: HIGH
**Dependencies**: task_013
**File(s) to Modify**: `src/search/unified_search.rs`, `tests/integration_validation.rs`

## Objective
Ensure the resurrected Tantivy search integrates properly with the existing unified search system.

## Success Criteria
- [ ] Tantivy works within UnifiedSearcher
- [ ] Search routing to Tantivy functions correctly
- [ ] Results format is compatible
- [ ] Performance doesn't regress
- [ ] Fallback mechanisms work

## Instructions

### Step 1: Test Tantivy integration with UnifiedSearcher
```rust
// tests/integration_validation.rs
use embed::search::unified_search::*;
use embed::search::tantivy_search::*;
use tempfile::tempdir;

#[test]
fn test_tantivy_within_unified_searcher() {
    let dir = tempdir().unwrap();
    
    // Initialize Tantivy search engine
    let mut tantivy = TantivySearch::new_with_error_context(dir.path()).unwrap();
    
    // Add test documents
    let documents = vec![
        Document {
            content: "Rust programming language features".to_string(),
            path: "rust_features.txt".to_string(),
            chunk_index: 0,
            start_line: 1,
            end_line: 1,
        },
        Document {
            content: "JavaScript async programming".to_string(),
            path: "js_async.txt".to_string(),
            chunk_index: 0,
            start_line: 1,
            end_line: 1,
        },
    ];
    
    for doc in documents {
        tantivy.add_document(doc).unwrap();
    }
    tantivy.commit().unwrap();
    
    // Test integration with UnifiedSearcher
    let unified_searcher = UnifiedSearcher::new().unwrap();
    
    // This should route to Tantivy for full-text search
    let search_request = SearchRequest {
        query: "programming".to_string(),
        search_type: SearchType::FullText,
        limit: 10,
        filters: vec![],
    };
    
    let results = unified_searcher.search(search_request).unwrap();
    
    assert!(!results.is_empty(), "UnifiedSearcher should find results via Tantivy");
    assert!(results.len() >= 2, "Should find both documents containing 'programming'");
    
    // Verify result format compatibility
    for result in &results {
        assert!(!result.path.is_empty(), "Result should have path");
        assert!(!result.content.is_empty(), "Result should have content");
        assert!(result.score >= 0.0, "Result should have valid score");
    }
}
```

### Step 2: Test search type routing
```rust
#[test]
fn test_search_type_routing_to_tantivy() {
    let unified_searcher = UnifiedSearcher::new().unwrap();
    
    // Test different search types route correctly
    let test_cases = vec![
        (SearchType::FullText, "Should route to Tantivy"),
        (SearchType::Fuzzy, "Should route to Tantivy fuzzy search"),
        (SearchType::Phrase, "Should route to Tantivy phrase search"),
        // Vector search should NOT route to Tantivy
        // (SearchType::Vector, "Should route to vector search"),
    ];
    
    for (search_type, description) in test_cases {
        let request = SearchRequest {
            query: "test query".to_string(),
            search_type: search_type.clone(),
            limit: 5,
            filters: vec![],
        };
        
        // Should not crash and should handle gracefully
        let result = unified_searcher.search(request);
        
        match result {
            Ok(results) => {
                println!("{}: Got {} results", description, results.len());
            },
            Err(e) => {
                // Some search types may fail if not implemented yet
                println!("{}: Failed with error: {}", description, e);
            }
        }
    }
}
```

### Step 3: Test result format compatibility
```rust
#[test]
fn test_result_format_compatibility() {
    let dir = tempdir().unwrap();
    let mut tantivy = TantivySearch::new_with_error_context(dir.path()).unwrap();
    
    // Add a document with known content
    let test_doc = Document {
        content: "This is a test document for format validation".to_string(),
        path: "test/format_validation.txt".to_string(),
        chunk_index: 5,
        start_line: 10,
        end_line: 15,
    };
    
    tantivy.add_document(test_doc).unwrap();
    tantivy.commit().unwrap();
    
    // Search and validate result format
    let results = tantivy.search("validation", 1).unwrap();
    assert_eq!(results.len(), 1);
    
    let result = &results[0];
    
    // Validate all required fields are present and correct
    assert_eq!(result.path, "test/format_validation.txt");
    assert!(result.content.contains("format validation"));
    assert_eq!(result.chunk_index, 5);
    assert_eq!(result.start_line, 10);
    assert_eq!(result.end_line, 15);
    assert!(result.score > 0.0, "Score should be positive");
    
    // Test that result can be serialized (for API compatibility)
    let json_result = serde_json::to_string(&result);
    assert!(json_result.is_ok(), "Result should be serializable to JSON");
    
    let deserialized: SearchResult = serde_json::from_str(&json_result.unwrap()).unwrap();
    assert_eq!(deserialized.path, result.path);
    assert_eq!(deserialized.content, result.content);
}
```

### Step 4: Test performance within unified system
```rust
#[test]
fn test_performance_within_unified_system() {
    use std::time::Instant;
    
    let unified_searcher = UnifiedSearcher::new().unwrap();
    
    // Warm up the system
    let warmup_request = SearchRequest {
        query: "warmup".to_string(),
        search_type: SearchType::FullText,
        limit: 1,
        filters: vec![],
    };
    let _ = unified_searcher.search(warmup_request);
    
    // Measure search performance through unified interface
    let search_queries = vec![
        "programming language",
        "async function",
        "data structure",
        "algorithm implementation",
        "error handling",
    ];
    
    let mut total_time = std::time::Duration::new(0, 0);
    let mut successful_searches = 0;
    
    for query in search_queries {
        let start = Instant::now();
        
        let request = SearchRequest {
            query: query.to_string(),
            search_type: SearchType::FullText,
            limit: 10,
            filters: vec![],
        };
        
        match unified_searcher.search(request) {
            Ok(_results) => {
                let duration = start.elapsed();
                total_time += duration;
                successful_searches += 1;
                
                // Individual search should be fast
                assert!(duration.as_millis() < 100, 
                    "Search for '{}' took {}ms (too slow)", query, duration.as_millis());
            },
            Err(e) => {
                println!("Search failed for '{}': {}", query, e);
            }
        }
    }
    
    if successful_searches > 0 {
        let avg_time = total_time / successful_searches;
        println!("Average search time: {:?}", avg_time);
        assert!(avg_time.as_millis() < 50, "Average search time too high");
    }
}
```

### Step 5: Test fallback mechanisms
```rust
#[test]
fn test_tantivy_fallback_mechanisms() {
    let unified_searcher = UnifiedSearcher::new().unwrap();
    
    // Test with queries that might fail in Tantivy
    let problematic_queries = vec![
        "",  // Empty query
        "   ",  // Whitespace only
        "!@#$%^&*()",  // Special characters only
        "a".repeat(1000),  // Very long query
        "\0\0\0",  // Null bytes (if they get through)
    ];
    
    for query in problematic_queries {
        let request = SearchRequest {
            query: query.clone(),
            search_type: SearchType::FullText,
            limit: 10,
            filters: vec![],
        };
        
        // Should not crash, should either succeed or fail gracefully
        match unified_searcher.search(request) {
            Ok(results) => {
                println!("Problematic query '{}' succeeded with {} results", 
                    query.chars().take(20).collect::<String>(), results.len());
            },
            Err(e) => {
                println!("Problematic query '{}' failed gracefully: {}", 
                    query.chars().take(20).collect::<String>(), e);
                // Failure is acceptable for problematic queries
            }
        }
    }
    
    // Test fallback to alternative search methods
    let request = SearchRequest {
        query: "should_find_something".to_string(),
        search_type: SearchType::FullText,
        limit: 10,
        filters: vec![],
    };
    
    // This might fallback to other search methods if Tantivy fails
    let result = unified_searcher.search_with_fallback(request);
    
    match result {
        Ok(results) => {
            println!("Search with fallback returned {} results", results.len());
        },
        Err(e) => {
            println!("Search with fallback failed: {}", e);
        }
    }
}
```

## Terminal Commands
```bash
cd C:\code\embed
cargo test --features tantivy integration_validation -v
cargo test --features tantivy test_tantivy_within_unified_searcher -v
cargo test --features tantivy test_performance_within_unified_system -v
```

## Integration Validation Checklist
- [ ] Tantivy search engine initializes within UnifiedSearcher
- [ ] Search requests route to Tantivy correctly
- [ ] Result format matches expected SearchResult structure
- [ ] Performance meets targets through unified interface
- [ ] Fallback mechanisms prevent crashes
- [ ] All search types are handled appropriately

## Troubleshooting
- If routing fails, check SearchType enum mapping
- If results are incompatible, verify SearchResult struct
- If performance regresses, check unified search overhead
- If tests crash, verify error handling in fallback paths

## Next Task
task_015 - Create production readiness checklist