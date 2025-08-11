// Test markdown chunker integration with the indexer
use anyhow::Result;
use embed_search::{Config, IncrementalIndexer};
use tempfile::tempdir;
use std::fs;

#[test]
fn test_markdown_chunker_integration() -> Result<()> {
    // Create test configuration
    let config = Config::default();
    let indexer = IncrementalIndexer::new(config.indexing)?;
    
    // Create temporary directory
    let temp_dir = tempdir()?;
    
    // Create a markdown file with various structures
    let markdown_content = r#"# Main Header

This is some introductory text.

## Section 1

Some content in section 1.

### Subsection 1.1

More detailed content here.

```rust
fn example() {
    println!("Hello world");
}
```

## Section 2

- List item 1
- List item 2
  - Nested item
  
### Task List

- [x] Completed task
- [ ] Incomplete task

| Column 1 | Column 2 |
|----------|----------|
| Data 1   | Data 2   |

> This is a blockquote
> with multiple lines

---

Final section with more content.
"#;

    let md_file = temp_dir.path().join("test.md");
    fs::write(&md_file, markdown_content)?;
    
    // Test chunking
    let chunks = indexer.create_chunks(markdown_content, &md_file)?;
    
    // Verify chunks were created
    assert!(!chunks.is_empty(), "Should create chunks from markdown content");
    
    // Verify markdown-specific chunking behavior
    // The markdown chunker should create chunks at header boundaries
    let header_chunks: Vec<_> = chunks.iter()
        .filter(|chunk| chunk.content.starts_with('#'))
        .collect();
    
    assert!(!header_chunks.is_empty(), "Should create chunks at header boundaries");
    
    println!("Created {} chunks from markdown content", chunks.len());
    for (i, chunk) in chunks.iter().enumerate() {
        println!("Chunk {}: lines {}-{}, length: {}", 
                i, chunk.start_line, chunk.end_line, chunk.content.len());
    }
    
    Ok(())
}

#[test]
fn test_regular_file_chunking() -> Result<()> {
    // Test that non-markdown files still use the regex chunker
    let config = Config::default();
    let indexer = IncrementalIndexer::new(config.indexing)?;
    
    let temp_dir = tempdir()?;
    
    let rust_content = r#"
fn first_function() {
    println!("First function");
}

fn second_function() {
    println!("Second function");
}

struct MyStruct {
    field: i32,
}
"#;

    let rust_file = temp_dir.path().join("test.rs");
    fs::write(&rust_file, rust_content)?;
    
    let chunks = indexer.create_chunks(rust_content, &rust_file)?;
    
    assert!(!chunks.is_empty(), "Should create chunks from Rust content");
    
    println!("Created {} chunks from Rust content", chunks.len());
    
    Ok(())
}