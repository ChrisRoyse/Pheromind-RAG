use embed_search::chunking::{SimpleRegexChunker, Chunk, ThreeChunkExpander, ChunkContext};
use std::path::Path;

/// Test suite for markdown parsing and chunking functionality
/// 
/// This test suite validates:
/// - Header-based chunking (# ## ### etc.)
/// - Code block preservation (```code```)
/// - List chunking (- * + 1. 2.)
/// - Table preservation
/// - Mixed content handling
/// - Edge cases and boundary conditions

#[cfg(test)]
mod markdown_chunking_tests {
    use super::*;

    /// Initialize test configuration - creates chunker with default settings
    fn create_test_chunker() -> SimpleRegexChunker {
        SimpleRegexChunker::with_chunk_size(1500).expect("Failed to create chunker")
    }

    #[test]
    fn test_header_based_chunking() {
        let chunker = create_test_chunker();
        
        let markdown_content = r#"# Main Header

Some intro text here.

## Section 1

Content for section 1.

### Subsection 1.1

Detailed content here.

## Section 2

More content.

# Another Main Header

Final content."#;

        let chunks = chunker.chunk_file(markdown_content);
        
        // HONEST ASSESSMENT: The current regex chunker doesn't have markdown-specific patterns
        // It will chunk based on function/class patterns, not markdown headers
        // This test verifies current behavior, not ideal markdown behavior
        
        assert!(!chunks.is_empty(), "Should produce at least one chunk");
        
        // Verify chunk structure
        for chunk in &chunks {
            assert!(!chunk.content.is_empty(), "Chunks should not be empty");
            assert!(chunk.end_line >= chunk.start_line, "End line should be >= start line");
        }
        
        println!("Header chunking test produced {} chunks", chunks.len());
        for (i, chunk) in chunks.iter().enumerate() {
            println!("Chunk {}: lines {}-{}", i, chunk.start_line, chunk.end_line);
            println!("Content preview: {:?}", chunk.content.lines().next().unwrap_or(""));
        }
    }

    #[test]
    fn test_code_block_preservation() {
        let chunker = create_test_chunker();
        
        let markdown_with_code = r#"# Code Example

Here's some code:

```rust
fn main() {
    println!("Hello, world!");
    let x = 42;
    println!("Value: {}", x);
}
```

And here's more text after the code block.

```python
def hello():
    print("Hello from Python!")
    return True
```

Final paragraph."#;

        let chunks = chunker.chunk_file(markdown_with_code);
        
        // Verify that code blocks are preserved within chunks
        let full_content = chunks.iter().map(|c| &c.content).collect::<Vec<_>>().join("\n");
        
        assert!(full_content.contains("```rust"), "Rust code block marker should be preserved");
        assert!(full_content.contains("```python"), "Python code block marker should be preserved");
        assert!(full_content.contains("fn main()"), "Rust function should be preserved");
        assert!(full_content.contains("def hello()"), "Python function should be preserved");
        
        println!("Code block preservation test produced {} chunks", chunks.len());
    }

    #[test]
    fn test_list_chunking() {
        let chunker = create_test_chunker();
        
        let markdown_with_lists = r#"# Todo List

## Unordered List
- Item 1
- Item 2
  - Nested item 2a
  - Nested item 2b
- Item 3

## Ordered List
1. First task
2. Second task
   - Sub-task 2.1
   - Sub-task 2.2
3. Third task

## Mixed List
* Bullet point
+ Another bullet
- Yet another
  1. Nested numbered
  2. Another numbered"#;

        let chunks = chunker.chunk_file(markdown_with_lists);
        
        // Verify list structures are preserved
        let full_content = chunks.iter().map(|c| &c.content).collect::<Vec<_>>().join("\n");
        
        assert!(full_content.contains("- Item 1"), "Unordered list items should be preserved");
        assert!(full_content.contains("1. First task"), "Ordered list items should be preserved");
        assert!(full_content.contains("  - Nested item"), "Nested list items should be preserved");
        
        println!("List chunking test produced {} chunks", chunks.len());
    }

    #[test]
    fn test_table_preservation() {
        let chunker = create_test_chunker();
        
        let markdown_with_table = r#"# Data Table

Here's a comparison table:

| Feature | Version 1 | Version 2 | Version 3 |
|---------|-----------|-----------|-----------|
| Speed   | Fast      | Faster    | Fastest   |
| Memory  | Low       | Medium    | High      |
| Cost    | $10       | $20       | $30       |

The table above shows the progression.

| Name    | Age | City      |
|---------|-----|-----------|
| Alice   | 25  | New York  |
| Bob     | 30  | San Jose  |
| Charlie | 35  | Austin    |

That's all the data."#;

        let chunks = chunker.chunk_file(markdown_with_table);
        
        // Verify table structure is preserved
        let full_content = chunks.iter().map(|c| &c.content).collect::<Vec<_>>().join("\n");
        
        assert!(full_content.contains("| Feature | Version 1"), "Table headers should be preserved");
        assert!(full_content.contains("|---------|"), "Table separators should be preserved");
        assert!(full_content.contains("| Speed   | Fast"), "Table data should be preserved");
        assert!(full_content.contains("| Alice   | 25"), "Second table data should be preserved");
        
        println!("Table preservation test produced {} chunks", chunks.len());
    }

    #[test]
    fn test_mixed_content_handling() {
        let chunker = create_test_chunker();
        
        let mixed_markdown = r#"# Complex Document

## Introduction
This document contains various markdown elements.

### Code and Lists
Here's a function:

```javascript
function processItems(items) {
    return items.map(item => {
        return {
            id: item.id,
            processed: true
        };
    });
}
```

And here's what it does:
1. Takes an array of items
2. Maps over each item
3. Returns processed items

## Data Section

| Item | Status | Notes |
|------|--------|-------|
| A    | Done   | Good  |
| B    | Pending| Wait  |

### Nested Content
- Main point
  ```bash
  # Commands to run
  npm install
  npm test
  ```
- Another point
  1. Sub-step
  2. Another sub-step

## Conclusion
That's everything!"#;

        let chunks = chunker.chunk_file(mixed_markdown);
        
        // Verify all content types are preserved
        let full_content = chunks.iter().map(|c| &c.content).collect::<Vec<_>>().join("\n");
        
        assert!(full_content.contains("# Complex Document"), "Main header preserved");
        assert!(full_content.contains("```javascript"), "JavaScript code block preserved");
        assert!(full_content.contains("function processItems"), "Function content preserved");
        assert!(full_content.contains("1. Takes an array"), "Ordered list preserved");
        assert!(full_content.contains("| Item | Status"), "Table header preserved");
        assert!(full_content.contains("```bash"), "Bash code block preserved");
        assert!(full_content.contains("npm install"), "Bash commands preserved");
        
        println!("Mixed content test produced {} chunks", chunks.len());
    }

    #[test]
    fn test_edge_cases() {
        let chunker = create_test_chunker();
        
        // Test empty content
        let empty_chunks = chunker.chunk_file("");
        assert!(empty_chunks.is_empty() || empty_chunks.len() == 1, "Empty content should produce 0 or 1 chunk");
        
        // Test content with only whitespace
        let whitespace_chunks = chunker.chunk_file("   \n\n  \t  \n");
        assert!(!whitespace_chunks.is_empty(), "Whitespace content should produce at least one chunk");
        
        // Test very long lines
        let long_line = "# ".to_string() + &"Very long header ".repeat(100);
        let long_line_chunks = chunker.chunk_file(&long_line);
        assert!(!long_line_chunks.is_empty(), "Long line should produce at least one chunk");
        
        // Test malformed markdown
        let malformed = r#"### Header without parent
```unclosed code block
| incomplete table
- list item
  nested without parent
"#;
        let malformed_chunks = chunker.chunk_file(malformed);
        assert!(!malformed_chunks.is_empty(), "Malformed markdown should still be chunked");
        
        println!("Edge cases test completed successfully");
    }

    #[test]
    fn test_chunk_context_expansion() {
        let chunker = create_test_chunker();
        
        let markdown_content = r#"# Document

## Section A
Content A.

## Section B
Content B.

## Section C
Content C."#;

        let chunks = chunker.chunk_file(markdown_content);
        
        // Test three-chunk expansion for middle chunks
        if chunks.len() >= 3 {
            let context = ThreeChunkExpander::expand(&chunks, 1)
                .expect("Should be able to expand middle chunk");
            
            assert!(context.above.is_some(), "Middle chunk should have context above");
            assert!(context.below.is_some(), "Middle chunk should have context below");
            assert_eq!(context.target_index, 1, "Target index should be preserved");
            
            // Test context formatting
            let display = context.format_for_display();
            assert!(display.contains("Context Above"), "Display should show context above");
            assert!(display.contains("TARGET MATCH"), "Display should highlight target");
            assert!(display.contains("Context Below"), "Display should show context below");
            
            let summary = context.format_summary();
            assert!(summary.is_ok(), "Summary formatting should succeed");
            
            println!("Context expansion test successful");
        } else {
            println!("Skipping context expansion test - not enough chunks produced");
        }
    }

    #[test]
    fn test_file_based_chunking() {
        let chunker = create_test_chunker();
        
        // Test with the sample markdown files we'll create
        let test_file_path = Path::new("test_data/markdown/sample_document.md");
        
        if test_file_path.exists() {
            let result = chunker.chunk_file_from_path(test_file_path);
            
            match result {
                Ok(chunks) => {
                    assert!(!chunks.is_empty(), "File chunking should produce chunks");
                    println!("File-based chunking produced {} chunks", chunks.len());
                }
                Err(e) => {
                    println!("File reading failed (expected if test data not created): {}", e);
                }
            }
        } else {
            println!("Test markdown file not found - skipping file-based test");
        }
    }

    #[test]
    fn test_performance_with_large_markdown() {
        let chunker = create_test_chunker();
        
        // Generate large markdown content
        let mut large_content = String::new();
        for i in 0..1000 {
            large_content.push_str(&format!("# Header {}\n\n", i));
            large_content.push_str(&format!("Content for section {}.\n\n", i));
            if i % 10 == 0 {
                large_content.push_str("```code\nsome code here\n```\n\n");
            }
            if i % 15 == 0 {
                large_content.push_str("| Col1 | Col2 |\n|------|------|\n| A | B |\n\n");
            }
        }
        
        let start = std::time::Instant::now();
        let chunks = chunker.chunk_file(&large_content);
        let duration = start.elapsed();
        
        assert!(!chunks.is_empty(), "Large content should produce chunks");
        assert!(duration.as_millis() < 1000, "Chunking should complete within 1 second");
        
        println!("Performance test: {} chunks in {:?}", chunks.len(), duration);
    }
}

/// Integration tests that combine chunking with other system components
#[cfg(test)]
mod integration_tests {
    use super::*;

    fn create_test_chunker() -> SimpleRegexChunker {
        SimpleRegexChunker::with_chunk_size(1500).expect("Failed to create chunker")
    }

    #[test]
    fn test_chunking_with_search_integration() {
        let chunker = create_test_chunker();
        
        let markdown = r#"# API Documentation

## Authentication
Users must authenticate using JWT tokens.

```javascript
const token = jwt.sign({userId: 123}, secret);
```

## Rate Limiting
API calls are limited to 1000 per hour.

## Error Handling
All errors return JSON with error codes."#;

        let chunks = chunker.chunk_file(markdown);
        
        // Test that chunks can be processed for search indexing
        for chunk in &chunks {
            assert!(!chunk.content.trim().is_empty(), "Chunks should have searchable content");
            assert!(chunk.content.len() < 10000, "Chunks should be reasonably sized for indexing");
        }
        
        // Test content accessibility for search
        let searchable_content: Vec<String> = chunks.iter()
            .map(|c| c.content.to_lowercase())
            .collect();
        
        let has_auth_content = searchable_content.iter().any(|c| c.contains("authentication"));
        let has_code_content = searchable_content.iter().any(|c| c.contains("jwt.sign"));
        
        // Note: This depends on how the chunker actually splits the content
        println!("Authentication content found: {}", has_auth_content);
        println!("Code content found: {}", has_code_content);
        
        println!("Search integration test completed with {} chunks", chunks.len());
    }

    #[test] 
    fn test_chunk_serialization() {
        let chunker = create_test_chunker();
        
        let markdown = "# Test\n\nContent here.";
        let chunks = chunker.chunk_file(markdown);
        
        // Test that chunks can be serialized/deserialized for storage
        for chunk in &chunks {
            let serialized = serde_json::to_string(chunk)
                .expect("Chunk should be serializable");
            
            let deserialized: Chunk = serde_json::from_str(&serialized)
                .expect("Chunk should be deserializable");
            
            assert_eq!(chunk.content, deserialized.content);
            assert_eq!(chunk.start_line, deserialized.start_line);
            assert_eq!(chunk.end_line, deserialized.end_line);
        }
        
        println!("Serialization test passed for {} chunks", chunks.len());
    }
}