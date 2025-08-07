# Task 2.008: Test BM25 Search with Known Terms

**Time Estimate**: 10 minutes
**Priority**: HIGH
**Dependencies**: task_007
**File(s) to Modify**: `tests/bm25_integration_tests.rs`

## Objective
Create focused tests that search for specific terms we know exist in the test files to verify the search pipeline works end-to-end.

## Success Criteria
- [ ] Searches for known indexed terms return results
- [ ] Term matching is case-insensitive
- [ ] Multi-word queries work correctly
- [ ] Results contain expected files

## Instructions

### Step 1: Create minimal known-term test
```rust
// Add this test to tests/bm25_integration_tests.rs
#[tokio::test]
async fn test_bm25_known_terms() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let project_path = temp_dir.path().to_path_buf();
    let db_path = project_path.join(".embed_db");
    
    // Create a minimal test file with very specific terms
    let known_content = r#"
function authenticate_user(username, password) {
    const database_connection = getConnection();
    const user_data = database_connection.query("SELECT * FROM users");
    return validate_credentials(user_data, password);
}

class DataProcessor {
    process_pipeline(data) {
        return this.transform_data(data);
    }
}
"#;
    
    fs::write(project_path.join("known_test.js"), known_content).await?;
    println!("🔍 KNOWN TERMS: Created test file with known content");
    
    // Initialize searcher
    let searcher = UnifiedSearcher::new_with_backend(
        project_path.clone(),
        db_path,
        SearchBackend::Tantivy
    ).await?;
    
    // Index the file
    println!("🔍 KNOWN TERMS: Indexing known test file");
    searcher.index_file(&project_path.join("known_test.js")).await?;
    
    // Verify index
    searcher.verify_bm25_index().await?;
    
    // Test individual terms we know exist
    let test_terms = vec![
        "authenticate",
        "database",
        "connection", 
        "user",
        "password",
        "process",
        "pipeline",
        "data",
    ];
    
    for term in test_terms {
        println!("🔍 KNOWN TERMS: Testing term: '{}'", term);
        let results = searcher.search(term).await?;
        println!("🔍 KNOWN TERMS: Term '{}' returned {} results", term, results.len());
        
        if results.is_empty() {
            println!("❌ KNOWN TERMS: Term '{}' should have results but returned none!", term);
        } else {
            println!("✅ KNOWN TERMS: Term '{}' found successfully", term);
            for (i, result) in results.iter().enumerate() {
                println!("   Result {}: file={}, score={:?}", i, result.file, result.score);
            }
        }
    }
    
    // Test multi-word queries
    let multi_word_queries = vec![
        "authenticate user",
        "database connection",
        "process pipeline",
        "transform data",
    ];
    
    for query in multi_word_queries {
        println!("🔍 KNOWN TERMS: Testing multi-word query: '{}'", query);
        let results = searcher.search(query).await?;
        println!("🔍 KNOWN TERMS: Query '{}' returned {} results", query, results.len());
        
        if results.is_empty() {
            println!("❌ KNOWN TERMS: Query '{}' should have results but returned none!", query);
        } else {
            println!("✅ KNOWN TERMS: Query '{}' found successfully", query);
        }
    }
    
    Ok(())
}
```

### Step 2: Test case sensitivity
```rust
// Add case sensitivity test
#[tokio::test] 
async fn test_bm25_case_sensitivity() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let project_path = temp_dir.path().to_path_buf();
    let db_path = project_path.join(".embed_db");
    
    // Content with mixed case
    let content = "DatabaseConnection class handles CONNECTION pooling";
    fs::write(project_path.join("case_test.js"), content).await?;
    
    let searcher = UnifiedSearcher::new_with_backend(
        project_path.clone(),
        db_path,
        SearchBackend::Tantivy
    ).await?;
    
    searcher.index_file(&project_path.join("case_test.js")).await?;
    
    // Test different cases of same term
    let case_variants = vec![
        "database",
        "Database", 
        "DATABASE",
        "connection",
        "Connection",
        "CONNECTION",
    ];
    
    for variant in case_variants {
        let results = searcher.search(variant).await?;
        println!("🔍 CASE TEST: '{}' returned {} results", variant, results.len());
        assert!(!results.is_empty(), "Case variant '{}' should return results", variant);
    }
    
    Ok(())
}
```

### Step 3: Test exact term matching
```rust
// Test that we can find exact terms
#[tokio::test]
async fn test_bm25_exact_terms() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let project_path = temp_dir.path().to_path_buf();
    let db_path = project_path.join(".embed_db");
    
    // Very simple content
    let content = "hello world test document";
    fs::write(project_path.join("exact_test.txt"), content).await?;
    
    let searcher = UnifiedSearcher::new_with_backend(
        project_path.clone(),
        db_path,
        SearchBackend::Tantivy
    ).await?;
    
    searcher.index_file(&project_path.join("exact_test.txt")).await?;
    
    // Search for each word
    let words = vec!["hello", "world", "test", "document"];
    
    for word in words {
        let results = searcher.search(word).await?;
        assert!(!results.is_empty(), "Word '{}' should be found", word);
        println!("✅ Found '{}' in {} results", word, results.len());
    }
    
    Ok(())
}
```

### Step 4: Run known terms tests
```bash
cd C:\code\embed
cargo test test_bm25_known_terms -- --nocapture
cargo test test_bm25_case_sensitivity -- --nocapture
cargo test test_bm25_exact_terms -- --nocapture
```

## Terminal Commands
```bash
cd C:\code\embed
cargo test test_bm25_known_terms -- --nocapture
cargo test test_bm25_case_sensitivity -- --nocapture
cargo test test_bm25_exact_terms -- --nocapture
```

## Expected Results
- All known terms should be found
- Case insensitive search should work
- Simple exact terms should return results

## Troubleshooting
If any tests fail:
- Check tokenization is working (previous task)
- Verify index has content (stats show > 0)
- Debug search query processing
- Check BM25 scoring logic

## Next Task
task_009 - Debug BM25 search query processing