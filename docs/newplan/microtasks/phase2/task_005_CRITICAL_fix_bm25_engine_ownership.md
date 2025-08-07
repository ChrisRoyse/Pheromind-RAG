# Task 2.005: Fix BM25 Engine Ownership Issues

**Time Estimate**: 10 minutes
**Priority**: CRITICAL
**Dependencies**: task_004
**File(s) to Modify**: `src/search/unified.rs`

## Objective
Fix Rust ownership and mutability issues with BM25Engine in UnifiedSearcher to allow both indexing and searching.

## Success Criteria
- [ ] BM25Engine can be mutably borrowed for indexing
- [ ] BM25Engine can be immutably borrowed for searching
- [ ] No borrowing conflicts during simultaneous operations
- [ ] Thread safety is maintained

## Instructions

### Step 1: Fix UnifiedSearcher BM25 field definition
```rust
// In src/search/unified.rs, update the struct definition
use std::sync::{Arc, Mutex};

pub struct UnifiedSearcher {
    project_path: PathBuf,
    db_path: PathBuf,
    backend: SearchBackend,
    // Use Arc<Mutex<>> for thread-safe mutable access
    bm25_engine: Option<Arc<Mutex<BM25Engine>>>,
    // other fields...
}
```

### Step 2: Update BM25 initialization
```rust
// In the constructor, wrap BM25Engine in Arc<Mutex<>>
impl UnifiedSearcher {
    pub async fn new_with_backend(
        project_path: PathBuf,
        db_path: PathBuf,
        backend: SearchBackend,
    ) -> Result<Self, SearchError> {
        println!("🔍 UNIFIED DEBUG: Initializing with backend: {:?}", backend);
        
        let bm25_engine = if backend == SearchBackend::Tantivy {
            println!("🔍 UNIFIED DEBUG: Initializing BM25 engine");
            let bm25 = BM25Engine::new();
            Some(Arc::new(Mutex::new(bm25)))
        } else {
            None
        };
        
        Ok(Self {
            project_path,
            db_path,
            backend,
            bm25_engine,
            // other fields...
        })
    }
}
```

### Step 3: Fix indexing with proper locking
```rust
// Update index_file method to handle Arc<Mutex<>>
impl UnifiedSearcher {
    pub async fn index_file(&self, file_path: &Path) -> Result<(), SearchError> {
        println!("🔍 UNIFIED DEBUG: Indexing file: {:?}", file_path);
        
        if let Some(ref bm25_arc) = self.bm25_engine {
            println!("🔍 UNIFIED DEBUG: Converting file to BM25Document");
            let doc = self.convert_file_to_bm25_document(file_path).await?;
            
            println!("🔍 UNIFIED DEBUG: Acquiring BM25 engine lock for indexing");
            let mut bm25_engine = bm25_arc.lock()
                .map_err(|_| SearchError::IndexingError("BM25 engine lock poisoned".to_string()))?;
            
            println!("🔍 UNIFIED DEBUG: Adding document to BM25 index");
            bm25_engine.add_document(doc)
                .map_err(|e| SearchError::IndexingError(format!("BM25 indexing failed: {}", e)))?;
            
            println!("🔍 UNIFIED DEBUG: Document added to BM25 index successfully");
        } else {
            println!("🔍 UNIFIED DEBUG: BM25 engine not available for indexing");
        }
        
        Ok(())
    }
}
```

### Step 4: Fix searching with proper locking
```rust
// Update search method to handle Arc<Mutex<>>
impl UnifiedSearcher {
    pub async fn search(&self, query: &str) -> Result<Vec<SearchResult>, SearchError> {
        println!("🔍 UNIFIED DEBUG: Search query received: '{}'", query);
        
        let mut all_results = Vec::new();
        
        // Try BM25 search if engine is available
        if let Some(ref bm25_arc) = self.bm25_engine {
            println!("🔍 UNIFIED DEBUG: Acquiring BM25 engine lock for searching");
            
            let bm25_engine = bm25_arc.lock()
                .map_err(|_| SearchError::SearchExecutionError("BM25 engine lock poisoned".to_string()))?;
            
            println!("🔍 UNIFIED DEBUG: Attempting BM25 search");
            match bm25_engine.search(query, 50) {
                Ok(bm25_results) => {
                    println!("🔍 UNIFIED DEBUG: BM25 returned {} results", bm25_results.len());
                    
                    // Drop the lock before async operations
                    drop(bm25_engine);
                    
                    // Convert BM25Match to SearchResult
                    for bm25_match in bm25_results {
                        let search_result = self.convert_bm25_to_search_result(bm25_match).await?;
                        all_results.push(search_result);
                    }
                    
                    println!("🔍 UNIFIED DEBUG: Converted {} BM25 results to SearchResult", all_results.len());
                }
                Err(e) => {
                    println!("🔍 UNIFIED DEBUG: BM25 search failed: {}", e);
                }
            }
        }
        
        println!("🔍 UNIFIED DEBUG: Returning {} total results", all_results.len());
        Ok(all_results)
    }
}
```

### Step 5: Add missing SearchError variant
```rust
// Add to SearchError enum if missing
#[derive(Debug)]
pub enum SearchError {
    // existing variants...
    SearchExecutionError(String),
}
```

### Step 6: Test the ownership fix
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
- Async operations with Mutex may need special handling
- Lock contention during simultaneous index/search operations
- Potential deadlocks if locks are not dropped properly

## Troubleshooting
- Always drop locks before async operations
- Use `tokio::sync::Mutex` instead of `std::sync::Mutex` for async contexts
- Consider `RwLock` if read-heavy workload

## Next Task
task_006 - Verify document indexing is working