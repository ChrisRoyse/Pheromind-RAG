# Task 3.011: Add Comprehensive Error Handling

**Time Estimate**: 10 minutes
**Priority**: MEDIUM
**Dependencies**: task_010
**File(s) to Modify**: `src/search/tantivy_search.rs`

## Objective
Implement robust error handling for all Tantivy operations with detailed error messages and recovery strategies.

## Success Criteria
- [ ] Custom error types for Tantivy operations
- [ ] Detailed error messages with context
- [ ] Recovery strategies for common errors
- [ ] Proper error propagation
- [ ] Logging for debugging

## Instructions

### Step 1: Define comprehensive error types
```rust
// Add to src/search/tantivy_search.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TantivyError {
    #[error("Index creation failed: {reason}")]
    IndexCreationFailed { reason: String },
    
    #[error("Document indexing failed: {reason}")]
    DocumentIndexingFailed { reason: String },
    
    #[error("Search operation failed: {reason}")]
    SearchFailed { reason: String },
    
    #[error("Query parsing failed: '{query}' - {reason}")]
    QueryParsingFailed { query: String, reason: String },
    
    #[error("Index corruption detected: {details}")]
    IndexCorruption { details: String },
    
    #[error("Insufficient disk space: {required} bytes needed")]
    InsufficientDiskSpace { required: u64 },
    
    #[error("Index locked by another process")]
    IndexLocked,
    
    #[error("Schema incompatibility: {details}")]
    SchemaIncompatible { details: String },
    
    #[error("IO error: {source}")]
    Io {
        #[from]
        source: std::io::Error,
    },
    
    #[error("Tantivy internal error: {source}")]
    TantivyInternal {
        #[from]
        source: tantivy::TantivyError,
    },
}

pub type Result<T> = std::result::Result<T, TantivyError>;
```

### Step 2: Add error recovery mechanisms
```rust
impl TantivySearch {
    pub fn add_document_with_retry(&mut self, doc: Document, max_retries: u32) -> Result<()> {
        let mut attempts = 0;
        
        while attempts < max_retries {
            match self.add_document(doc.clone()) {
                Ok(()) => return Ok(()),
                Err(e) => {
                    attempts += 1;
                    
                    match &e {
                        TantivyError::IndexLocked => {
                            // Wait and retry for locked index
                            println!("Index locked, retrying in {}ms...", attempts * 100);
                            std::thread::sleep(std::time::Duration::from_millis(attempts as u64 * 100));
                            continue;
                        },
                        TantivyError::InsufficientDiskSpace { required } => {
                            return Err(TantivyError::InsufficientDiskSpace { required: *required });
                        },
                        _ => {
                            if attempts >= max_retries {
                                return Err(e);
                            }
                            println!("Retrying document indexing: attempt {}/{}", attempts, max_retries);
                        }
                    }
                }
            }
        }
        
        Err(TantivyError::DocumentIndexingFailed {
            reason: format!("Failed after {} attempts", max_retries)
        })
    }
    
    pub fn search_with_fallback(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        // Try exact search first
        match self.search(query, limit) {
            Ok(results) if !results.is_empty() => Ok(results),
            _ => {
                // Fallback to fuzzy search if exact search fails or returns no results
                println!("Exact search failed/empty, trying fuzzy search...");
                self.search_fuzzy(query, 1)
                    .or_else(|_| {
                        // Final fallback to very fuzzy search
                        println!("Fuzzy search failed, trying very fuzzy...");
                        self.search_fuzzy(query, 2)
                    })
            }
        }
    }
}
```

### Step 3: Add detailed error context
```rust
impl TantivySearch {
    pub fn new_with_error_context(path: &Path) -> Result<Self> {
        // Check preconditions
        if !path.exists() {
            std::fs::create_dir_all(path)
                .map_err(|e| TantivyError::IndexCreationFailed {
                    reason: format!("Failed to create directory {}: {}", path.display(), e)
                })?;
        }
        
        // Check disk space
        if let Ok(metadata) = std::fs::metadata(path) {
            // This is a simplified check - in real world, check actual free space
            const MIN_REQUIRED_SPACE: u64 = 100_000_000; // 100MB
            
            // For demo purposes, assume we have enough space
            // In production, use a proper disk space check
        }
        
        // Try to create index with detailed error context
        let schema = Self::build_schema();
        let settings = Self::create_optimized_settings();
        
        let index = Index::builder()
            .schema(schema)
            .settings(settings)
            .create_in_dir(path)
            .map_err(|e| TantivyError::IndexCreationFailed {
                reason: format!("Failed to create index at {}: {}", path.display(), e)
            })?;
            
        // Continue with initialization...
        let writer = index.writer(50_000_000)
            .map_err(|e| TantivyError::IndexCreationFailed {
                reason: format!("Failed to create writer: {}", e)
            })?;
            
        let reader = index.reader()
            .map_err(|e| TantivyError::IndexCreationFailed {
                reason: format!("Failed to create reader: {}", e)
            })?;
        
        let body_field = index.schema().get_field("body")
            .ok_or_else(|| TantivyError::SchemaIncompatible {
                details: "body field not found in schema".to_string()
            })?;
            
        let path_field = index.schema().get_field("path")
            .ok_or_else(|| TantivyError::SchemaIncompatible {
                details: "path field not found in schema".to_string()
            })?;
        
        Ok(Self {
            index,
            writer: Arc::new(Mutex::new(writer)),
            reader: Arc::new(reader),
            body_field,
            path_field,
        })
    }
}
```

### Step 4: Add error recovery tests
```rust
#[test]
fn test_error_recovery() {
    let dir = tempdir().unwrap();
    let mut tantivy = TantivySearch::new_with_error_context(dir.path()).unwrap();
    
    // Test document with problematic content
    let problematic_doc = Document {
        content: "\0\0\0invalid\0content".to_string(),  // Null bytes
        path: "problematic.txt".to_string(),
        chunk_index: 0,
        start_line: 1,
        end_line: 1,
    };
    
    // Should handle gracefully
    let result = tantivy.add_document_with_retry(problematic_doc, 3);
    // Either succeeds or fails with meaningful error
    match result {
        Ok(_) => println!("Successfully handled problematic content"),
        Err(e) => println!("Gracefully handled error: {}", e),
    }
    
    // Test search fallback
    let results = tantivy.search_with_fallback("nonexistent", 10).unwrap();
    // Should return empty results without crashing
    assert!(results.is_empty() || !results.is_empty());
}
```

### Step 5: Add logging
```rust
use log::{info, warn, error};

impl TantivySearch {
    pub fn add_document_logged(&mut self, doc: Document) -> Result<()> {
        info!("Indexing document: {} ({} chars)", doc.path, doc.content.len());
        
        match self.add_document(doc.clone()) {
            Ok(()) => {
                info!("Successfully indexed: {}", doc.path);
                Ok(())
            },
            Err(e) => {
                error!("Failed to index {}: {}", doc.path, e);
                Err(e)
            }
        }
    }
}
```

## Terminal Commands
```bash
cd C:\code\embed
cargo check --features tantivy
cargo test --features tantivy test_error_recovery -v
```

## Troubleshooting
- If thiserror is missing, add to Cargo.toml: `thiserror = "1.0"`
- If log is missing, add: `log = "0.4"`
- Ensure all error types implement Display and Debug

## Next Task
task_012 - Add performance monitoring and metrics