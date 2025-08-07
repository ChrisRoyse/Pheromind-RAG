# Task 2.007: Debug Tokenization Pipeline

**Time Estimate**: 10 minutes
**Priority**: HIGH
**Dependencies**: task_006
**File(s) to Modify**: `src/search/unified.rs`

## Objective
Debug the file-to-tokens conversion to ensure proper tokenization is happening and tokens are not being filtered out.

## Success Criteria
- [ ] File content is read correctly
- [ ] Tokenization produces reasonable token counts
- [ ] Token filtering is not too aggressive
- [ ] Token positions and weights are set correctly

## Instructions

### Step 1: Add comprehensive tokenization debugging
```rust
// In convert_file_to_bm25_document method, add detailed logging
async fn convert_file_to_bm25_document(&self, file_path: &Path) -> Result<BM25Document, SearchError> {
    println!("🔍 TOKENIZE DEBUG: Converting file: {:?}", file_path);
    
    // Read file content
    let content = fs::read_to_string(file_path).await
        .map_err(|e| SearchError::FileReadError(file_path.to_path_buf(), e))?;
    
    println!("🔍 TOKENIZE DEBUG: Raw content length: {}", content.len());
    println!("🔍 TOKENIZE DEBUG: Raw content preview: {}", 
             content.chars().take(100).collect::<String>());
    
    // Split by whitespace first
    let raw_words: Vec<&str> = content.split_whitespace().collect();
    println!("🔍 TOKENIZE DEBUG: Raw words count: {}", raw_words.len());
    
    // Apply filtering and cleaning
    let tokens: Vec<Token> = raw_words
        .iter()
        .enumerate()
        .filter_map(|(pos, word)| {
            let original_word = word.to_string();
            
            // Clean word - keep alphanumeric and underscore
            let clean_word = word.chars()
                .filter(|c| c.is_alphanumeric() || *c == '_')
                .collect::<String>()
                .to_lowercase();
            
            println!("🔍 TOKENIZE DEBUG: '{}' -> '{}'", original_word, clean_word);
            
            // Filter conditions
            if clean_word.is_empty() {
                println!("🔍 TOKENIZE DEBUG: Rejected - empty after cleaning");
                None
            } else if clean_word.len() <= 1 {
                println!("🔍 TOKENIZE DEBUG: Rejected - too short: '{}'", clean_word);
                None
            } else if clean_word.chars().all(|c| c.is_numeric()) {
                println!("🔍 TOKENIZE DEBUG: Rejected - pure number: '{}'", clean_word);
                None
            } else {
                println!("🔍 TOKENIZE DEBUG: Accepted: '{}'", clean_word);
                Some(Token {
                    text: clean_word,
                    position: pos,
                    importance_weight: 1.0,
                })
            }
        })
        .collect();
    
    println!("🔍 TOKENIZE DEBUG: Final token count: {} (from {} raw words)", 
             tokens.len(), raw_words.len());
    
    if !tokens.is_empty() {
        println!("🔍 TOKENIZE DEBUG: Final tokens sample: {:?}", 
                 tokens.iter().take(10).map(|t| &t.text).collect::<Vec<_>>());
    } else {
        println!("⚠️ TOKENIZE WARNING: No tokens generated! This will cause empty index.");
    }
    
    // Rest of document creation...
    let language = file_path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| match ext {
            "rs" => "rust",
            "js" | "jsx" => "javascript",
            "py" => "python",
            _ => "text",
        })
        .map(String::from);
    
    let doc_id = format!("file_{}", file_path.file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .replace(".", "_"));
    
    println!("🔍 TOKENIZE DEBUG: Created document ID: {}", doc_id);
    
    let doc = BM25Document {
        id: doc_id,
        file_path: file_path.to_string_lossy().to_string(),
        chunk_index: 0,
        tokens,
        start_line: 0,
        end_line: content.lines().count(),
        language,
    };
    
    Ok(doc)
}
```

### Step 2: Test tokenization with simple content
```rust
// Add a simple tokenization test in the test file
#[tokio::test]
async fn test_bm25_tokenization_debug() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let project_path = temp_dir.path().to_path_buf();
    let db_path = project_path.join(".embed_db");
    
    // Create a simple test file with known content
    let simple_content = r#"
pub fn test_function(input: &str) -> String {
    let result = process_input(input);
    return result;
}
"#;
    
    fs::write(project_path.join("simple.rs"), simple_content).await?;
    
    // Initialize searcher
    let searcher = UnifiedSearcher::new_with_backend(
        project_path.clone(),
        db_path,
        SearchBackend::Tantivy
    ).await?;
    
    // Index the simple file
    searcher.index_file(&project_path.join("simple.rs")).await?;
    
    // Verify index
    searcher.verify_bm25_index().await?;
    
    // Search for known terms
    let results = searcher.search("test function").await?;
    println!("🔍 SIMPLE TEST: Found {} results for 'test function'", results.len());
    
    let results = searcher.search("process input").await?;
    println!("🔍 SIMPLE TEST: Found {} results for 'process input'", results.len());
    
    Ok(())
}
```

### Step 3: Run tokenization debug test
```bash
cd C:\code\embed
cargo test test_bm25_tokenization_debug -- --nocapture
```

### Step 4: Adjust tokenization if needed
Based on debug output, adjust the tokenization logic:
- If too many tokens rejected: relax filtering
- If no tokens generated: check file reading
- If tokens look wrong: adjust cleaning logic

## Terminal Commands
```bash
cd C:\code\embed
cargo test test_bm25_tokenization_debug -- --nocapture
cargo test test_bm25_basic_search -- --nocapture
```

## Expected Issues
- Tokenization may be too aggressive (filtering out valid words)
- File reading may fail for certain file types
- Token cleaning may remove important technical terms
- Position tracking may be incorrect

## Troubleshooting
- Try with simple content first
- Check if specific file types cause issues
- Verify character filtering rules
- Test with known technical terms

## Next Task
task_008 - Test BM25 search with known terms