// Simple test for markdown chunking functionality without complex dependencies

#[cfg(test)]
mod tests {
    use std::path::Path;
    use embed_search::chunking::{SimpleRegexChunker, MarkdownRegexChunker};

    #[test]
    fn test_markdown_chunker_exists() {
        // Test that we can create the markdown chunker
        let result = MarkdownRegexChunker::new();
        assert!(result.is_ok(), "Should be able to create MarkdownRegexChunker");
        
        let chunker = result.unwrap();
        
        // Test basic markdown content
        let content = r#"# Header 1
Some content here.

## Header 2
More content.

```rust
fn test() {}
```
"#;
        
        let chunks = chunker.chunk_markdown(content);
        assert!(!chunks.is_empty(), "Should produce chunks from markdown content");
        assert!(chunks.len() >= 2, "Should create multiple chunks from markdown with headers");
        
        println!("Successfully created {} markdown chunks", chunks.len());
    }
    
    #[test]
    fn test_regex_chunker_still_works() {
        // Test that the regular regex chunker still works
        let result = SimpleRegexChunker::new();
        assert!(result.is_ok(), "Should be able to create SimpleRegexChunker");
        
        let chunker = result.unwrap();
        
        let content = r#"fn first() {
    println!("First");
}

fn second() {
    println!("Second");
}
"#;
        
        let chunks = chunker.chunk_file(content);
        assert!(!chunks.is_empty(), "Should produce chunks from code content");
        
        println!("Successfully created {} code chunks", chunks.len());
    }
    
    #[test]
    fn test_file_extension_detection() {
        // Test that we can detect file extensions properly
        let md_path = Path::new("test.md");
        assert_eq!(md_path.extension().and_then(|e| e.to_str()), Some("md"));
        
        let rs_path = Path::new("test.rs");
        assert_eq!(rs_path.extension().and_then(|e| e.to_str()), Some("rs"));
        
        println!("File extension detection works correctly");
    }
}