# Task 3.010: Test with Real File Content

**Time Estimate**: 10 minutes
**Priority**: HIGH
**Dependencies**: task_009
**File(s) to Modify**: `src/search/tantivy_search.rs`

## Objective
Verify Tantivy works correctly with real codebase files, handling various content types and sizes.

## Success Criteria
- [ ] Can index actual Rust source files
- [ ] Search finds relevant code snippets
- [ ] Handles different file types properly
- [ ] Performance is acceptable with real content
- [ ] No crashes with unusual content

## Instructions

### Step 1: Create real file test
```rust
#[test]
fn test_tantivy_with_real_codebase() {
    use walkdir::WalkDir;
    use std::ffi::OsStr;
    
    let dir = tempdir().unwrap();
    let mut tantivy = TantivySearch::new_optimized(dir.path()).unwrap();
    
    let mut file_count = 0;
    let mut total_size = 0;
    
    // Index real Rust files from src directory
    for entry in WalkDir::new("src").into_iter().filter_map(|e| e.ok()) {
        if entry.path().extension() == Some(OsStr::new("rs")) {
            if let Ok(content) = fs::read_to_string(entry.path()) {
                total_size += content.len();
                
                let doc = Document {
                    content: content.clone(),
                    path: entry.path().to_string_lossy().to_string(),
                    chunk_index: 0,
                    start_line: 1,
                    end_line: content.lines().count() as u64,
                };
                
                tantivy.add_document(doc).unwrap();
                file_count += 1;
                
                if file_count >= 20 {  // Limit for test speed
                    break;
                }
            }
        }
    }
    
    tantivy.commit().unwrap();
    
    println!("Indexed {} files, total size: {} bytes", file_count, total_size);
    assert!(file_count > 0, "No files were indexed");
    
    // Test Rust-specific searches
    let impl_results = tantivy.search("impl", 10).unwrap();
    assert!(!impl_results.is_empty(), "Should find 'impl' in Rust code");
    
    let struct_results = tantivy.search("struct", 5).unwrap();
    println!("Found {} structs", struct_results.len());
    
    let fn_results = tantivy.search("fn", 5).unwrap();
    println!("Found {} functions", fn_results.len());
}
```

### Step 2: Test with various content types
```rust
#[test]
fn test_content_edge_cases() {
    let dir = tempdir().unwrap();
    let mut tantivy = TantivySearch::new_optimized(dir.path()).unwrap();
    
    // Test different content types
    let test_contents = vec![
        ("empty.txt", ""),  // Empty file
        ("unicode.txt", "Hello 世界 🌍 émojis"),  // Unicode
        ("large.txt", "word ".repeat(10000)),  // Large content
        ("special.txt", "!@#$%^&*(){}[]|\\:;\"'<>,.?/~`"),  // Special chars
        ("code.rs", "fn main() { println!(\"Hello, world!\"); }"),  // Code
        ("json.json", r#"{"key": "value", "number": 42}"#),  // JSON
    ];
    
    for (path, content) in test_contents {
        let doc = Document {
            content: content.to_string(),
            path: path.to_string(),
            chunk_index: 0,
            start_line: 1,
            end_line: content.lines().count() as u64,
        };
        
        // Should not panic on any content type
        tantivy.add_document(doc).unwrap();
    }
    
    tantivy.commit().unwrap();
    
    // Test searches work with different content
    let hello_results = tantivy.search("Hello", 5).unwrap();
    assert!(!hello_results.is_empty());
    
    let unicode_results = tantivy.search("世界", 5).unwrap();
    assert!(!unicode_results.is_empty());
    
    let code_results = tantivy.search("println", 5).unwrap();
    assert!(!code_results.is_empty());
}
```

### Step 3: Test fuzzy search with real content
```rust
#[test]
fn test_fuzzy_with_real_content() {
    let dir = tempdir().unwrap();
    let mut tantivy = TantivySearch::new_optimized(dir.path()).unwrap();
    
    // Add real Rust code with common patterns
    let rust_code = r#"
        pub struct SearchEngine {
            index: Index,
        }
        
        impl SearchEngine {
            pub fn new() -> Self {
                SearchEngine {
                    index: Index::new(),
                }
            }
        }
    "#;
    
    let doc = Document {
        content: rust_code.to_string(),
        path: "search_engine.rs".to_string(),
        chunk_index: 0,
        start_line: 1,
        end_line: rust_code.lines().count() as u64,
    };
    
    tantivy.add_document(doc).unwrap();
    tantivy.commit().unwrap();
    
    // Test fuzzy search with common typos
    let results = tantivy.search_fuzzy("SearchEnigne", 2).unwrap();  // Typo in "Engine"
    assert!(!results.is_empty(), "Should find SearchEngine despite typo");
    
    let results2 = tantivy.search_fuzzy("implm", 1).unwrap();  // Typo in "impl"
    assert!(!results2.is_empty(), "Should find impl despite typo");
}
```

### Step 4: Add required imports
```rust
use walkdir::WalkDir;
use std::ffi::OsStr;
use std::fs;
```

## Terminal Commands
```bash
cd C:\code\embed
cargo test --features tantivy test_tantivy_with_real_codebase -v
cargo test --features tantivy test_content_edge_cases -v
cargo test --features tantivy test_fuzzy_with_real_content -v
```

## Troubleshooting
- If walkdir is missing, add to Cargo.toml: `walkdir = "2"`
- If no files indexed, check src directory exists
- If unicode tests fail, check terminal encoding
- If performance is poor, reduce test file count

## Next Task
task_011 - Add comprehensive error handling