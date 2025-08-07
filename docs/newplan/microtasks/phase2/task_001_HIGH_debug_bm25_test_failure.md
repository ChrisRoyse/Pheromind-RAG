# Task 2.001: Debug BM25 Test Failure - Root Cause Analysis

**Time Estimate**: 10 minutes
**Priority**: HIGH
**Dependencies**: None
**File(s) to Modify**: Add debug logging to `tests/bm25_integration_tests.rs`

## Objective
Add comprehensive debug logging to the failing BM25 test to understand why it returns 0 results instead of 2.

## Success Criteria
- [ ] Debug logs show what documents are being indexed
- [ ] Debug logs show query processing steps
- [ ] Debug logs reveal where the search pipeline breaks
- [ ] Root cause identified

## Instructions

### Step 1: Add debug logging to test
```rust
// In tests/bm25_integration_tests.rs, modify test_bm25_basic_search
#[tokio::test]
async fn test_bm25_basic_search() -> Result<()> {
    println!("🔍 DEBUG: Starting BM25 basic search test");
    
    let temp_dir = TempDir::new()?;
    let project_path = temp_dir.path().to_path_buf();
    let db_path = project_path.join(".embed_db");
    
    println!("🔍 DEBUG: Test directories created: {:?}", project_path);
    
    // Create test codebase
    create_test_codebase(&project_path).await?;
    println!("🔍 DEBUG: Test codebase created");
    
    // Initialize searcher with BM25 enabled
    let searcher = UnifiedSearcher::new_with_backend(
        project_path.clone(),
        db_path,
        SearchBackend::Tantivy
    ).await?;
    println!("🔍 DEBUG: UnifiedSearcher initialized with Tantivy backend");
    
    // Index all files with detailed logging
    let mut file_count = 0;
    for entry in std::fs::read_dir(&project_path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            println!("🔍 DEBUG: Indexing file: {:?}", path);
            searcher.index_file(&path).await?;
            file_count += 1;
            println!("🔍 DEBUG: File indexed successfully");
        }
    }
    println!("🔍 DEBUG: Total files indexed: {}", file_count);
    
    // Test search with debug info
    println!("🔍 DEBUG: Starting search for 'database connection'");
    let results = searcher.search("database connection").await?;
    println!("🔍 DEBUG: Search completed. Results count: {}", results.len());
    
    for (i, result) in results.iter().enumerate() {
        println!("🔍 DEBUG: Result {}: file={}, score={:?}, match_type={:?}", 
                 i, result.file, result.score, result.match_type);
    }
    
    assert!(!results.is_empty(), "Should find results for 'database connection'");
    
    Ok(())
}
```

### Step 2: Run test with debug output
```bash
cd C:\code\embed
RUST_LOG=debug cargo test test_bm25_basic_search -- --nocapture
```

### Step 3: Analyze output and document findings
Create a summary of what the debug output reveals about the failure.

## Terminal Commands
```bash
cd C:\code\embed
RUST_LOG=debug cargo test test_bm25_basic_search -- --nocapture 2>&1 | tee debug_output.log
```

## Expected Findings
- Files may not be getting indexed properly
- UnifiedSearcher may not be using BM25 backend
- Search query may not reach BM25 engine
- Document preprocessing may be failing

## Next Task
task_002 - Verify UnifiedSearcher integration with BM25