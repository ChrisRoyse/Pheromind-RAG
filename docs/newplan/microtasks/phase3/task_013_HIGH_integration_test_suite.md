# Task 3.013: Create Comprehensive Integration Test Suite

**Time Estimate**: 10 minutes
**Priority**: HIGH
**Dependencies**: task_012
**File(s) to Modify**: `tests/tantivy_integration_tests.rs` (new file)

## Objective
Create a complete integration test suite that validates all Tantivy functionality end-to-end.

## Success Criteria
- [ ] All major Tantivy features tested
- [ ] Real-world scenarios covered
- [ ] Performance regression detection
- [ ] Error conditions tested
- [ ] All tests pass consistently

## Instructions

### Step 1: Create main integration test file
```rust
// tests/tantivy_integration_tests.rs
use std::fs;
use std::path::Path;
use tempfile::tempdir;
use embed::search::tantivy_search::*;

#[test]
fn test_complete_tantivy_workflow() {
    let dir = tempdir().unwrap();
    let mut tantivy = TantivySearch::new_with_error_context(dir.path()).unwrap();
    
    // Phase 1: Index a variety of documents
    let documents = vec![
        Document {
            content: "Rust is a systems programming language".to_string(),
            path: "rust_intro.txt".to_string(),
            chunk_index: 0,
            start_line: 1,
            end_line: 1,
        },
        Document {
            content: "fn main() { println!(\"Hello, world!\"); }".to_string(),
            path: "hello.rs".to_string(),
            chunk_index: 0,
            start_line: 1,
            end_line: 1,
        },
        Document {
            content: "use std::collections::HashMap; // Import HashMap".to_string(),
            path: "imports.rs".to_string(),
            chunk_index: 0,
            start_line: 1,
            end_line: 1,
        },
        Document {
            content: "The quick brown fox jumps over the lazy dog".to_string(),
            path: "pangram.txt".to_string(),
            chunk_index: 0,
            start_line: 1,
            end_line: 1,
        },
    ];
    
    for doc in documents {
        tantivy.add_document_monitored(doc).unwrap();
    }
    tantivy.commit().unwrap();
    
    // Phase 2: Test various search queries
    let test_queries = vec![
        ("Rust", 1),
        ("programming", 1),
        ("println", 1),
        ("HashMap", 1),
        ("fox", 1),
        ("nonexistent", 0),
    ];
    
    for (query, expected_min) in test_queries {
        let results = tantivy.search_monitored(query, 10).unwrap();
        assert!(results.len() >= expected_min, 
            "Query '{}' should return at least {} results, got {}", 
            query, expected_min, results.len());
    }
    
    // Phase 3: Test fuzzy search
    let fuzzy_results = tantivy.search_fuzzy("Rustt", 1).unwrap();  // Typo in "Rust"
    assert!(!fuzzy_results.is_empty(), "Fuzzy search should find results for 'Rustt'");
    
    // Phase 4: Test performance
    tantivy.metrics.print_summary();
    assert!(tantivy.metrics.avg_search_time_ms() < 50.0, "Search performance regression");
    
    // Phase 5: Test error recovery
    let health_alerts = tantivy.check_performance_health();
    if !health_alerts.is_empty() {
        println!("Performance alerts: {:?}", health_alerts);
    }
}
```

### Step 2: Test index persistence
```rust
#[test]
fn test_index_persistence_and_reload() {
    let dir = tempdir().unwrap();
    let index_path = dir.path().to_path_buf();
    
    // Create and populate index
    {
        let mut tantivy = TantivySearch::new_with_error_context(&index_path).unwrap();
        
        let doc = Document {
            content: "Persistent test document".to_string(),
            path: "persistent.txt".to_string(),
            chunk_index: 0,
            start_line: 1,
            end_line: 1,
        };
        
        tantivy.add_document(doc).unwrap();
        tantivy.commit().unwrap();
        
        // Verify document is findable
        let results = tantivy.search("Persistent", 10).unwrap();
        assert_eq!(results.len(), 1);
    } // Drop tantivy instance
    
    // Reload index and verify persistence
    {
        let tantivy = TantivySearch::open_existing(&index_path).unwrap();
        
        let results = tantivy.search("Persistent", 10).unwrap();
        assert_eq!(results.len(), 1, "Document should persist after reload");
        assert!(results[0].content.contains("Persistent"));
    }
}
```

### Step 3: Test large dataset handling
```rust
#[test]
fn test_large_dataset_performance() {
    let dir = tempdir().unwrap();
    let mut tantivy = TantivySearch::new_with_metrics(dir.path()).unwrap();
    
    let start_time = std::time::Instant::now();
    
    // Index 1000 documents
    for i in 0..1000 {
        let doc = Document {
            content: format!("Document number {} with unique content and keywords batch{}", 
                i, i / 100),
            path: format!("doc_{}.txt", i),
            chunk_index: 0,
            start_line: 1,
            end_line: 1,
        };
        
        tantivy.add_document_monitored(doc).unwrap();
        
        if i % 100 == 0 {
            tantivy.commit().unwrap();
            println!("Indexed {} documents...", i);
        }
    }
    tantivy.commit().unwrap();
    
    let indexing_time = start_time.elapsed();
    println!("Indexed 1000 documents in: {:?}", indexing_time);
    
    // Test search performance on large dataset
    let search_start = std::time::Instant::now();
    
    let results = tantivy.search_monitored("Document", 100).unwrap();
    assert!(results.len() > 100, "Should find many documents with 'Document'");
    
    let search_time = search_start.elapsed();
    println!("Searched 1000 documents in: {:?}", search_time);
    
    // Performance assertions
    assert!(indexing_time.as_secs() < 30, "Indexing 1000 docs should take <30s");
    assert!(search_time.as_millis() < 100, "Search should take <100ms");
    
    // Test fuzzy search on large dataset
    let fuzzy_results = tantivy.search_fuzzy("Documen", 1).unwrap();  // Missing 't'
    assert!(!fuzzy_results.is_empty(), "Fuzzy search should work on large dataset");
    
    tantivy.metrics.print_summary();
}
```

### Step 4: Test edge cases and error conditions
```rust
#[test]
fn test_edge_cases_and_error_conditions() {
    let dir = tempdir().unwrap();
    let mut tantivy = TantivySearch::new_with_error_context(dir.path()).unwrap();
    
    // Test empty documents
    let empty_doc = Document {
        content: "".to_string(),
        path: "empty.txt".to_string(),
        chunk_index: 0,
        start_line: 1,
        end_line: 1,
    };
    tantivy.add_document(empty_doc).unwrap();  // Should not crash
    
    // Test very large documents
    let large_content = "word ".repeat(10000);
    let large_doc = Document {
        content: large_content,
        path: "large.txt".to_string(),
        chunk_index: 0,
        start_line: 1,
        end_line: 10000,
    };
    tantivy.add_document(large_doc).unwrap();  // Should handle large docs
    
    // Test special characters
    let special_doc = Document {
        content: "!@#$%^&*(){}[]|\\:;\"'<>,.?/~`".to_string(),
        path: "special.txt".to_string(),
        chunk_index: 0,
        start_line: 1,
        end_line: 1,
    };
    tantivy.add_document(special_doc).unwrap();  // Should handle special chars
    
    // Test Unicode
    let unicode_doc = Document {
        content: "Hello 世界 🌍 émojis and unicode".to_string(),
        path: "unicode.txt".to_string(),
        chunk_index: 0,
        start_line: 1,
        end_line: 1,
    };
    tantivy.add_document(unicode_doc).unwrap();  // Should handle Unicode
    
    tantivy.commit().unwrap();
    
    // Test searches with special cases
    let empty_results = tantivy.search("", 10).unwrap_or_else(|_| vec![]);
    // Empty query should either work or fail gracefully
    
    let unicode_results = tantivy.search("世界", 10).unwrap();
    assert!(!unicode_results.is_empty(), "Should find Unicode content");
    
    // Test very long queries
    let long_query = "word ".repeat(1000);
    let _long_results = tantivy.search(&long_query, 10).unwrap_or_else(|_| vec![]);
    // Should handle long queries gracefully
}
```

### Step 5: Test concurrent access
```rust
#[test]
fn test_concurrent_search_operations() {
    use std::thread;
    use std::sync::Arc;
    
    let dir = tempdir().unwrap();
    let mut tantivy = TantivySearch::new_with_error_context(dir.path()).unwrap();
    
    // Index test documents
    for i in 0..100 {
        let doc = Document {
            content: format!("Concurrent test document {}", i),
            path: format!("concurrent_{}.txt", i),
            chunk_index: 0,
            start_line: 1,
            end_line: 1,
        };
        tantivy.add_document(doc).unwrap();
    }
    tantivy.commit().unwrap();
    
    let tantivy_arc = Arc::new(tantivy);
    let mut handles = vec![];
    
    // Spawn multiple threads to perform concurrent searches
    for thread_id in 0..5 {
        let tantivy_clone = Arc::clone(&tantivy_arc);
        
        let handle = thread::spawn(move || {
            for i in 0..20 {
                let query = format!("document {}", (thread_id * 20 + i) % 100);
                let results = tantivy_clone.search_monitored(&query, 10).unwrap();
                assert!(!results.is_empty(), "Thread {} should find results for query {}", 
                    thread_id, query);
            }
        });
        
        handles.push(handle);
    }
    
    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }
    
    println!("Concurrent search test completed successfully");
    tantivy_arc.metrics.print_summary();
}
```

## Terminal Commands
```bash
cd C:\code\embed
cargo test --features tantivy tantivy_integration_tests -v
cargo test --features tantivy test_complete_tantivy_workflow -v
cargo test --features tantivy test_large_dataset_performance -v
```

## Expected Test Results
- All integration tests should pass
- Performance should meet established targets
- No crashes or panics under any conditions
- Proper error handling for edge cases

## Troubleshooting
- If tests are too slow, reduce dataset sizes
- If concurrent tests fail, check thread safety
- If Unicode tests fail, check terminal encoding
- For memory issues, monitor test resource usage

## Next Task
task_014 - Validate against existing search infrastructure