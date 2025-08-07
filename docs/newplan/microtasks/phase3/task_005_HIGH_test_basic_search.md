# Task 3.005: Test Basic Search Functionality

**Time Estimate**: 10 minutes
**Priority**: HIGH
**Dependencies**: task_004
**File(s) to Modify**: `src/search/tantivy_search.rs`

## Objective
Verify that basic search operations work correctly with indexed documents.

## Success Criteria
- [ ] Can search indexed documents
- [ ] Search returns correct results
- [ ] Result structure is properly formatted
- [ ] Search performance is reasonable
- [ ] No panics during search

## Instructions

### Step 1: Create search test
```rust
#[test]
fn test_basic_search() {
    let dir = tempdir().unwrap();
    let mut tantivy = TantivySearch::new(dir.path()).unwrap();
    
    // Add test documents
    let docs = vec![
        Document {
            content: "The quick brown fox jumps".to_string(),
            path: "file1.txt".to_string(),
            chunk_index: 0,
            start_line: 1,
            end_line: 1,
        },
        Document {
            content: "The lazy dog sleeps".to_string(),
            path: "file2.txt".to_string(),
            chunk_index: 0,
            start_line: 1,
            end_line: 1,
        },
    ];
    
    // Index documents
    for doc in docs {
        tantivy.add_document(doc).unwrap();
    }
    tantivy.commit().unwrap();
    
    // Test search
    let results = tantivy.search("quick", 10).unwrap();
    assert_eq!(results.len(), 1);
    assert!(results[0].content.contains("quick"));
    
    let results2 = tantivy.search("dog", 10).unwrap();
    assert_eq!(results2.len(), 1);
    assert!(results2[0].content.contains("lazy dog"));
}
```

### Step 2: Test empty search
```rust
#[test]
fn test_search_no_results() {
    let dir = tempdir().unwrap();
    let mut tantivy = TantivySearch::new(dir.path()).unwrap();
    
    // Add one document
    let doc = Document {
        content: "Hello world".to_string(),
        path: "test.txt".to_string(),
        chunk_index: 0,
        start_line: 1,
        end_line: 1,
    };
    tantivy.add_document(doc).unwrap();
    tantivy.commit().unwrap();
    
    // Search for non-existent term
    let results = tantivy.search("nonexistent", 10).unwrap();
    assert_eq!(results.len(), 0);
}
```

### Step 3: Run tests
```bash
cargo test --features tantivy test_basic_search
```

## Terminal Commands
```bash
cd C:\code\embed
cargo test --features tantivy test_basic_search -v
cargo test --features tantivy test_search_no_results -v
```

## Troubleshooting
- If search returns no results, check tokenization
- If results format is wrong, verify SearchResult struct
- Check that commit() is called before searching

## Next Task
task_006 - Implement fuzzy search capability