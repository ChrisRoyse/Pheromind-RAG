# Task 2.006: Verify Document Indexing is Working

**Time Estimate**: 10 minutes
**Priority**: HIGH
**Dependencies**: task_005
**File(s) to Modify**: `src/search/unified.rs`

## Objective
Add comprehensive logging to verify that documents are actually being indexed in the BM25 engine.

## Success Criteria
- [ ] Documents are successfully added to BM25 index
- [ ] Token counts are reasonable
- [ ] Inverted index is populated
- [ ] Index statistics show expected values

## Instructions

### Step 1: Add BM25 index verification logging
```rust
// In src/search/unified.rs, add index verification method
impl UnifiedSearcher {
    pub async fn verify_bm25_index(&self) -> Result<(), SearchError> {
        if let Some(ref bm25_arc) = self.bm25_engine {
            let bm25_engine = bm25_arc.lock()
                .map_err(|_| SearchError::SearchExecutionError("BM25 engine lock poisoned".to_string()))?;
            
            let stats = bm25_engine.get_stats();
            println!("🔍 INDEX STATS: Total documents: {}", stats.total_documents);
            println!("🔍 INDEX STATS: Total terms: {}", stats.total_terms);
            println!("🔍 INDEX STATS: Average document length: {:.2}", stats.avg_document_length);
            println!("🔍 INDEX STATS: K1 parameter: {}", stats.k1);
            println!("🔍 INDEX STATS: B parameter: {}", stats.b);
            
            // Verify index is not empty
            if stats.total_documents == 0 {
                println!("⚠️ INDEX WARNING: No documents in BM25 index!");
                return Err(SearchError::IndexingError("BM25 index is empty".to_string()));
            }
            
            if stats.total_terms == 0 {
                println!("⚠️ INDEX WARNING: No terms in BM25 index!");
                return Err(SearchError::IndexingError("BM25 index has no terms".to_string()));
            }
            
            println!("✅ INDEX VERIFICATION: BM25 index appears healthy");
        } else {
            println!("⚠️ INDEX WARNING: BM25 engine not available for verification");
        }
        
        Ok(())
    }
}
```

### Step 2: Update test to verify indexing
```rust
// In tests/bm25_integration_tests.rs, modify the test
#[tokio::test]
async fn test_bm25_basic_search() -> Result<()> {
    println!("🔍 DEBUG: Starting BM25 basic search test");
    
    let temp_dir = TempDir::new()?;
    let project_path = temp_dir.path().to_path_buf();
    let db_path = project_path.join(".embed_db");
    
    // Create test codebase
    create_test_codebase(&project_path).await?;
    
    // Initialize searcher
    let searcher = UnifiedSearcher::new_with_backend(
        project_path.clone(),
        db_path,
        SearchBackend::Tantivy
    ).await?;
    
    // Index all files
    let mut file_count = 0;
    for entry in std::fs::read_dir(&project_path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            println!("🔍 DEBUG: Indexing file: {:?}", path);
            searcher.index_file(&path).await?;
            file_count += 1;
        }
    }
    println!("🔍 DEBUG: Total files indexed: {}", file_count);
    
    // VERIFY INDEX AFTER INDEXING
    println!("🔍 DEBUG: Verifying BM25 index contents");
    searcher.verify_bm25_index().await?;
    
    // Now test search
    println!("🔍 DEBUG: Starting search for 'database connection'");
    let results = searcher.search("database connection").await?;
    println!("🔍 DEBUG: Search completed. Results count: {}", results.len());
    
    assert!(!results.is_empty(), "Should find results for 'database connection'");
    
    Ok(())
}
```

### Step 3: Add detailed document indexing logs
```rust
// In the index_file method, add more detailed logging
pub async fn index_file(&self, file_path: &Path) -> Result<(), SearchError> {
    println!("🔍 UNIFIED DEBUG: Indexing file: {:?}", file_path);
    
    if let Some(ref bm25_arc) = self.bm25_engine {
        println!("🔍 UNIFIED DEBUG: Converting file to BM25Document");
        let doc = self.convert_file_to_bm25_document(file_path).await?;
        
        println!("🔍 UNIFIED DEBUG: Document created - ID: {}, tokens: {}", 
                 doc.id, doc.tokens.len());
        
        if doc.tokens.is_empty() {
            println!("⚠️ WARNING: Document has no tokens! File: {:?}", file_path);
        } else {
            println!("🔍 UNIFIED DEBUG: First 5 tokens: {:?}", 
                     doc.tokens.iter().take(5).map(|t| &t.text).collect::<Vec<_>>());
        }
        
        let mut bm25_engine = bm25_arc.lock()
            .map_err(|_| SearchError::IndexingError("BM25 engine lock poisoned".to_string()))?;
        
        println!("🔍 UNIFIED DEBUG: Adding document to BM25 index");
        bm25_engine.add_document(doc)
            .map_err(|e| SearchError::IndexingError(format!("BM25 indexing failed: {}", e)))?;
        
        println!("🔍 UNIFIED DEBUG: Document added successfully");
    }
    
    Ok(())
}
```

### Step 4: Test document indexing verification
```bash
cd C:\code\embed
cargo test test_bm25_basic_search -- --nocapture
```

## Terminal Commands
```bash
cd C:\code\embed
cargo check --features core
cargo test test_bm25_basic_search -- --nocapture
```

## Expected Findings
- Index should show 5 documents after indexing test files
- Each document should have reasonable token counts (50-200 tokens)
- If any counts are 0, that indicates the problem area

## Troubleshooting
- If total_documents = 0: Documents aren't being added to index
- If total_terms = 0: Tokenization is failing
- If avg_document_length = 0: Token generation is broken

## Next Task
task_007 - Debug tokenization pipeline