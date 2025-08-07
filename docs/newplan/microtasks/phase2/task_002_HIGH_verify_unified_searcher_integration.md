# Task 2.002: Verify UnifiedSearcher Integration with BM25

**Time Estimate**: 10 minutes
**Priority**: HIGH
**Dependencies**: task_001
**File(s) to Modify**: `src/search/unified.rs`

## Objective
Verify that UnifiedSearcher is correctly routing search requests to the BM25 engine when configured with Tantivy backend.

## Success Criteria
- [ ] UnifiedSearcher instantiates BM25Engine correctly
- [ ] Search queries route to BM25 when appropriate
- [ ] BM25 engine is properly initialized
- [ ] Backend selection logic works correctly

## Instructions

### Step 1: Add debug logging to UnifiedSearcher
```rust
// In src/search/unified.rs, find the search method and add logging
pub async fn search(&self, query: &str) -> Result<Vec<SearchResult>, SearchError> {
    println!("🔍 UNIFIED DEBUG: Search query received: '{}'", query);
    println!("🔍 UNIFIED DEBUG: Backend configuration: {:?}", self.backend);
    
    match &self.backend {
        SearchBackend::Tantivy => {
            println!("🔍 UNIFIED DEBUG: Using Tantivy backend (should include BM25)");
            // Check if BM25 is enabled
            if let Some(ref bm25) = self.bm25_engine {
                println!("🔍 UNIFIED DEBUG: BM25 engine is available");
                // Existing BM25 search logic
            } else {
                println!("🔍 UNIFIED DEBUG: BM25 engine is NOT available!");
                // Fall back to other search methods
            }
        }
        _ => {
            println!("🔍 UNIFIED DEBUG: Using non-Tantivy backend");
        }
    }
    
    // Rest of existing search implementation
}
```

### Step 2: Check BM25 engine initialization
```rust
// In the UnifiedSearcher constructor, add logging
pub async fn new_with_backend(
    project_path: PathBuf,
    db_path: PathBuf,
    backend: SearchBackend,
) -> Result<Self, SearchError> {
    println!("🔍 UNIFIED DEBUG: Initializing with backend: {:?}", backend);
    
    let mut searcher = Self {
        project_path,
        db_path,
        backend,
        bm25_engine: None,
        // other fields
    };
    
    // Initialize BM25 if using Tantivy
    if backend == SearchBackend::Tantivy {
        println!("🔍 UNIFIED DEBUG: Initializing BM25 engine");
        let bm25 = BM25Engine::new();
        searcher.bm25_engine = Some(bm25);
        println!("🔍 UNIFIED DEBUG: BM25 engine initialized successfully");
    }
    
    Ok(searcher)
}
```

### Step 3: Verify index_file integration
```rust
// In the index_file method, add BM25-specific logging
pub async fn index_file(&self, file_path: &Path) -> Result<(), SearchError> {
    println!("🔍 UNIFIED DEBUG: Indexing file: {:?}", file_path);
    
    if let Some(ref mut bm25) = self.bm25_engine {
        println!("🔍 UNIFIED DEBUG: Adding file to BM25 index");
        // Convert file to BM25Document and add to index
        let doc = self.convert_file_to_bm25_document(file_path).await?;
        bm25.add_document(doc)?;
        println!("🔍 UNIFIED DEBUG: File added to BM25 index successfully");
    } else {
        println!("🔍 UNIFIED DEBUG: BM25 engine not available for indexing");
    }
    
    // Rest of existing indexing logic
}
```

### Step 4: Run test again to verify integration
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

## Expected Issues
- BM25 engine may not be initialized in UnifiedSearcher
- Backend routing may not include BM25
- File conversion to BM25Document may be missing
- Search may not route to BM25 engine

## Next Task
task_003 - Check file-to-BM25Document conversion