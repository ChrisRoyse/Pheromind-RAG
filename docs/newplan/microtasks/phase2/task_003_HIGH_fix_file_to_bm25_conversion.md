# Task 2.003: Fix File-to-BM25Document Conversion

**Time Estimate**: 10 minutes
**Priority**: HIGH
**Dependencies**: task_002
**File(s) to Modify**: `src/search/unified.rs`

## Objective
Implement or fix the conversion from file content to BM25Document structure that BM25Engine expects.

## Success Criteria
- [ ] Files are properly tokenized for BM25
- [ ] BM25Document structure is correctly populated
- [ ] Token positions and weights are set
- [ ] Conversion handles different file types

## Instructions

### Step 1: Implement file-to-BM25Document conversion
```rust
// In src/search/unified.rs, add this method
use crate::search::bm25::{BM25Document, Token};
use tokio::fs;

impl UnifiedSearcher {
    async fn convert_file_to_bm25_document(&self, file_path: &Path) -> Result<BM25Document, SearchError> {
        println!("🔍 CONVERSION DEBUG: Converting file: {:?}", file_path);
        
        // Read file content
        let content = fs::read_to_string(file_path).await
            .map_err(|e| SearchError::FileReadError(file_path.to_path_buf(), e))?;
        
        println!("🔍 CONVERSION DEBUG: File content length: {}", content.len());
        
        // Simple tokenization (whitespace + alphanumeric)
        let tokens: Vec<Token> = content
            .split_whitespace()
            .enumerate()
            .filter_map(|(pos, word)| {
                let clean_word = word.chars()
                    .filter(|c| c.is_alphanumeric() || *c == '_')
                    .collect::<String>()
                    .to_lowercase();
                
                if !clean_word.is_empty() && clean_word.len() > 1 {
                    Some(Token {
                        text: clean_word,
                        position: pos,
                        importance_weight: 1.0,
                    })
                } else {
                    None
                }
            })
            .collect();
        
        println!("🔍 CONVERSION DEBUG: Generated {} tokens", tokens.len());
        if tokens.len() > 0 {
            println!("🔍 CONVERSION DEBUG: First 5 tokens: {:?}", 
                     tokens.iter().take(5).map(|t| &t.text).collect::<Vec<_>>());
        }
        
        // Determine file language
        let language = file_path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| match ext {
                "rs" => "rust",
                "js" | "jsx" => "javascript",
                "py" => "python",
                "cpp" | "cc" => "cpp",
                "java" => "java",
                _ => "text",
            })
            .map(String::from);
        
        let doc = BM25Document {
            id: format!("file_{}", file_path.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .replace(".", "_")),
            file_path: file_path.to_string_lossy().to_string(),
            chunk_index: 0,
            tokens,
            start_line: 0,
            end_line: content.lines().count(),
            language,
        };
        
        println!("🔍 CONVERSION DEBUG: Created BM25Document with ID: {}", doc.id);
        Ok(doc)
    }
}
```

### Step 2: Update index_file to use conversion
```rust
// In the index_file method, update BM25 integration
pub async fn index_file(&self, file_path: &Path) -> Result<(), SearchError> {
    println!("🔍 UNIFIED DEBUG: Indexing file: {:?}", file_path);
    
    if let Some(ref mut bm25) = self.bm25_engine {
        println!("🔍 UNIFIED DEBUG: Converting file to BM25Document");
        let doc = self.convert_file_to_bm25_document(file_path).await?;
        
        println!("🔍 UNIFIED DEBUG: Adding document to BM25 index");
        bm25.add_document(doc)
            .map_err(|e| SearchError::IndexingError(format!("BM25 indexing failed: {}", e)))?;
        
        println!("🔍 UNIFIED DEBUG: Document added to BM25 index successfully");
    } else {
        println!("🔍 UNIFIED DEBUG: BM25 engine not available for indexing");
    }
    
    // Continue with other indexing methods
    Ok(())
}
```

### Step 3: Add necessary imports
```rust
// At the top of src/search/unified.rs
use crate::search::bm25::{BM25Engine, BM25Document, Token};
use tokio::fs;
```

### Step 4: Test the conversion
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
- Import errors for BM25 types
- SearchError may need new variants
- File reading may fail
- Tokenization may be too aggressive or too lenient

## Troubleshooting
- Check if BM25 module is properly exported
- Verify SearchError enum has required variants
- Ensure file paths are accessible
- Test tokenization with simple content first

## Next Task
task_004 - Fix BM25 search routing in UnifiedSearcher