# Task 2.004: Fix BM25 Search Routing in UnifiedSearcher

**Time Estimate**: 10 minutes
**Priority**: HIGH
**Dependencies**: task_003
**File(s) to Modify**: `src/search/unified.rs`

## Objective
Ensure that search queries are properly routed to the BM25 engine and results are converted back to SearchResult format.

## Success Criteria
- [ ] Search queries reach BM25 engine
- [ ] BM25 results are converted to SearchResult format
- [ ] Search returns non-empty results for indexed content
- [ ] Result ranking is preserved

## Instructions

### Step 1: Fix search routing to BM25
```rust
// In src/search/unified.rs, update the search method
use crate::search::bm25::BM25Match;

impl UnifiedSearcher {
    pub async fn search(&self, query: &str) -> Result<Vec<SearchResult>, SearchError> {
        println!("🔍 UNIFIED DEBUG: Search query received: '{}'", query);
        println!("🔍 UNIFIED DEBUG: Backend configuration: {:?}", self.backend);
        
        let mut all_results = Vec::new();
        
        // Try BM25 search if engine is available
        if let Some(ref bm25_engine) = self.bm25_engine {
            println!("🔍 UNIFIED DEBUG: Attempting BM25 search");
            
            match bm25_engine.search(query, 50) {
                Ok(bm25_results) => {
                    println!("🔍 UNIFIED DEBUG: BM25 returned {} results", bm25_results.len());
                    
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
        } else {
            println!("🔍 UNIFIED DEBUG: BM25 engine not available");
        }
        
        // Fall back to other search methods if no BM25 results
        if all_results.is_empty() {
            println!("🔍 UNIFIED DEBUG: Falling back to other search methods");
            // Existing search implementation
        }
        
        println!("🔍 UNIFIED DEBUG: Returning {} total results", all_results.len());
        Ok(all_results)
    }
}
```

### Step 2: Implement BM25Match to SearchResult conversion
```rust
// Add this method to UnifiedSearcher
use crate::search::{SearchResult, MatchType};

impl UnifiedSearcher {
    async fn convert_bm25_to_search_result(&self, bm25_match: BM25Match) -> Result<SearchResult, SearchError> {
        println!("🔍 CONVERSION DEBUG: Converting BM25Match for doc: {}", bm25_match.doc_id);
        
        // Extract file path from BM25Document ID or use a lookup
        let file_path = self.get_file_path_from_doc_id(&bm25_match.doc_id)?;
        
        // Read a snippet of the file for context
        let content = tokio::fs::read_to_string(&file_path).await
            .map_err(|e| SearchError::FileReadError(file_path.clone(), e))?;
        
        let snippet = content.lines().take(3).collect::<Vec<_>>().join("\n");
        
        Ok(SearchResult {
            file: file_path.to_string_lossy().to_string(),
            line: 1, // BM25 doesn't provide line-specific matches
            column: 1,
            snippet,
            match_type: MatchType::Statistical,
            score: Some(bm25_match.score),
            matched_term: bm25_match.matched_terms.join(" "),
            context_before: String::new(),
            context_after: String::new(),
        })
    }
    
    fn get_file_path_from_doc_id(&self, doc_id: &str) -> Result<PathBuf, SearchError> {
        // Simple implementation: doc_id format is "file_filename_ext"
        let filename = doc_id.strip_prefix("file_")
            .unwrap_or(doc_id)
            .replace("_", ".");
        
        let file_path = self.project_path.join(&filename);
        
        if file_path.exists() {
            Ok(file_path)
        } else {
            Err(SearchError::FileNotFound(file_path))
        }
    }
}
```

### Step 3: Add required SearchError variants if missing
```rust
// Check if these variants exist in SearchError enum, add if missing:
#[derive(Debug)]
pub enum SearchError {
    // existing variants...
    FileNotFound(PathBuf),
    FileReadError(PathBuf, std::io::Error),
    IndexingError(String),
}
```

### Step 4: Test the search routing
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
- BM25Match import errors
- SearchResult field mismatches
- File path resolution failures
- Missing SearchError variants

## Troubleshooting
- Verify all imports are correct
- Check SearchResult struct definition
- Test file path resolution with simple cases
- Add debug logging for each conversion step

## Next Task
task_005 - Fix BM25 engine ownership issues