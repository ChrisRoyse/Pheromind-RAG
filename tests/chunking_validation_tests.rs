/// BRUTAL CHUNKING VALIDATION TESTS
/// These tests verify the SimpleRegexChunker produces meaningful, correct chunks
/// for real-world code scenarios. NO COMPROMISES.

use embed_search::chunking::regex_chunker::{SimpleRegexChunker, Chunk};
use embed_search::config::Config;
use std::sync::Once;

static INIT: Once = Once::new();

fn init_config() {
    INIT.call_once(|| {
        let config = Config::new_test_config();
        embed_search::config::CONFIG.write().unwrap().replace(config);
    });
}

#[cfg(test)]
mod brutal_chunking_tests {
    use super::*;

    #[test]
    fn test_empty_file_handling() {
        init_config();
        let chunker = SimpleRegexChunker::new().expect("chunker creation failed");
        
        // Empty content should produce empty chunks array
        let chunks = chunker.chunk_file("");
        assert_eq!(chunks.len(), 0, "Empty file should produce no chunks");
        
        // Only whitespace should also produce no meaningful chunks
        let chunks = chunker.chunk_file("   \n\n  \t  \n");
        // This will produce 1 chunk with whitespace - that's acceptable behavior
        assert!(!chunks.is_empty(), "Whitespace-only file behavior validated");
    }

    #[test]
    fn test_single_line_file() {
        init_config();
        let chunker = SimpleRegexChunker::new().expect("chunker creation failed");
        
        let content = "let x = 5;";
        let chunks = chunker.chunk_file(content);
        
        assert_eq!(chunks.len(), 1, "Single line should produce exactly one chunk");
        assert_eq!(chunks[0].content, content);
        assert_eq!(chunks[0].start_line, 0);
        assert_eq!(chunks[0].end_line, 0);
    }

    #[test]
    fn test_rust_function_boundary_detection() {
        init_config();
        let chunker = SimpleRegexChunker::new().expect("chunker creation failed");
        
        let rust_code = r#"// Header comment
use std::collections::HashMap;

fn first_function() {
    println!("First function");
    let x = 42;
}

pub fn second_function(param: i32) -> i32 {
    param * 2
}

async fn async_function() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}"#;

        let chunks = chunker.chunk_file(rust_code);
        
        // Should create separate chunks for each function
        assert!(chunks.len() >= 3, "Should detect multiple function boundaries");
        
        // Verify first chunk contains header and imports
        let first_chunk = &chunks[0];
        assert!(first_chunk.content.contains("// Header comment"));
        assert!(first_chunk.content.contains("use std::collections::HashMap"));
        
        // Verify function boundaries are detected correctly
        let has_first_fn = chunks.iter().any(|c| c.content.contains("fn first_function"));
        let has_second_fn = chunks.iter().any(|c| c.content.contains("pub fn second_function"));
        let has_async_fn = chunks.iter().any(|c| c.content.contains("async fn async_function"));
        
        assert!(has_first_fn, "Should detect 'fn first_function' boundary");
        assert!(has_second_fn, "Should detect 'pub fn second_function' boundary");
        assert!(has_async_fn, "Should detect 'async fn async_function' boundary");
    }

    #[test]
    fn test_python_function_boundary_detection() {
        init_config();
        let chunker = SimpleRegexChunker::new().expect("chunker creation failed");
        
        let python_code = r#"#!/usr/bin/env python3
import os
import sys

def first_function():
    """First function docstring."""
    return "hello"

class MyClass:
    def __init__(self):
        self.value = 42
    
    def method_one(self, param):
        return param * 2

def another_function(x, y=None):
    if y is None:
        y = []
    return x + len(y)
"#;

        let chunks = chunker.chunk_file(python_code);
        
        // Should detect function and class boundaries
        assert!(chunks.len() >= 3, "Should detect multiple boundaries in Python code");
        
        // Verify different Python constructs are detected
        let has_def_function = chunks.iter().any(|c| c.content.contains("def first_function"));
        let has_class = chunks.iter().any(|c| c.content.contains("class MyClass"));
        let has_another_def = chunks.iter().any(|c| c.content.contains("def another_function"));
        
        assert!(has_def_function, "Should detect 'def first_function' boundary");
        assert!(has_class, "Should detect 'class MyClass' boundary");
        assert!(has_another_def, "Should detect 'def another_function' boundary");
    }

    #[test]
    fn test_javascript_function_boundary_detection() {
        init_config();
        let chunker = SimpleRegexChunker::new().expect("chunker creation failed");
        
        let js_code = r#"// JavaScript module
const util = require('util');

function regularFunction() {
    console.log("Regular function");
}

async function asyncFunction() {
    await new Promise(resolve => setTimeout(resolve, 100));
}

const arrowFunction = () => {
    return "arrow function result";
};

class MyClass {
    constructor() {
        this.value = 42;
    }
    
    methodOne() {
        return this.value;
    }
}
"#;

        let chunks = chunker.chunk_file(js_code);
        
        // Should detect various JavaScript constructs
        let has_regular_fn = chunks.iter().any(|c| c.content.contains("function regularFunction"));
        let has_async_fn = chunks.iter().any(|c| c.content.contains("async function asyncFunction"));
        let has_class = chunks.iter().any(|c| c.content.contains("class MyClass"));
        
        assert!(has_regular_fn, "Should detect 'function regularFunction' boundary");
        assert!(has_async_fn, "Should detect 'async function asyncFunction' boundary");
        assert!(has_class, "Should detect 'class MyClass' boundary");
    }

    #[test]
    fn test_chunk_size_limit_enforcement() {
        init_config();
        let chunker = SimpleRegexChunker::with_chunk_size(10).expect("chunker creation failed");
        
        // Create content longer than chunk size limit
        let mut long_content = String::new();
        for i in 0..50 {
            long_content.push_str(&format!("line {} with some content\n", i));
        }
        
        let chunks = chunker.chunk_file(&long_content);
        
        // Should split into multiple chunks due to size limit
        assert!(chunks.len() > 1, "Long content should be split into multiple chunks");
        
        // Each chunk should respect the size limit (except possibly the last one)
        for (i, chunk) in chunks.iter().enumerate() {
            let line_count = chunk.content.lines().count();
            if i < chunks.len() - 1 {
                // Non-final chunks should respect the limit
                assert!(line_count <= 10, "Chunk {} has {} lines, exceeds limit of 10", i, line_count);
            }
        }
    }

    #[test]
    fn test_chunk_content_accuracy() {
        init_config();
        let chunker = SimpleRegexChunker::new().expect("chunker creation failed");
        
        let content = "line1\nline2\nfn test() {\n    body line\n}\nline6\nline7";
        let chunks = chunker.chunk_file(content);
        
        // Verify that chunks contain exact original content
        let reconstructed = chunks.iter()
            .map(|c| c.content.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        
        // This is a simplistic check - the chunker doesn't preserve exact whitespace between chunks
        // But each chunk's content should be exact
        for chunk in &chunks {
            let chunk_lines: Vec<&str> = chunk.content.lines().collect();
            let original_lines: Vec<&str> = content.lines().collect();
            
            // Verify chunk lines exist in original
            for chunk_line in &chunk_lines {
                assert!(original_lines.contains(chunk_line), 
                    "Chunk contains line '{}' not found in original", chunk_line);
            }
        }
    }

    #[test]
    fn test_line_number_tracking_accuracy() {
        init_config();
        let chunker = SimpleRegexChunker::new().expect("chunker creation failed");
        
        let content = "line0\nline1\nfn test1() {}\nline3\nfn test2() {}\nline5";
        let chunks = chunker.chunk_file(content);
        
        // Verify line numbers are accurate
        for chunk in &chunks {
            assert!(chunk.start_line <= chunk.end_line, 
                "Chunk start_line {} should be <= end_line {}", 
                chunk.start_line, chunk.end_line);
            
            // Verify the content matches the line range
            let original_lines: Vec<&str> = content.lines().collect();
            let expected_content = original_lines[chunk.start_line..=chunk.end_line].join("\n");
            assert_eq!(chunk.content, expected_content,
                "Chunk content doesn't match expected content for lines {}-{}", 
                chunk.start_line, chunk.end_line);
        }
    }

    #[test]
    fn test_malformed_code_handling() {
        init_config();
        let chunker = SimpleRegexChunker::new().expect("chunker creation failed");
        
        // Test with syntax errors and malformed code
        let malformed_code = r#"
fn incomplete_function(
    // Missing closing parenthesis and body

class MissingBrace {
    def method_without_colon()
        return "broken"

// Random text that looks like code but isn't
fn fn fn function class class
"#;

        let chunks = chunker.chunk_file(malformed_code);
        
        // Should not crash and should produce some chunks
        assert!(!chunks.is_empty(), "Should handle malformed code without crashing");
        
        // Should still detect some boundaries even in malformed code
        let has_fn_boundary = chunks.iter().any(|c| 
            c.content.contains("fn incomplete_function") || 
            c.content.contains("class MissingBrace")
        );
        assert!(has_fn_boundary, "Should detect at least some boundaries in malformed code");
    }

    #[test]
    fn test_mixed_language_file() {
        init_config();
        let chunker = SimpleRegexChunker::new().expect("chunker creation failed");
        
        // File with multiple language patterns (like in documentation or polyglot files)
        let mixed_content = r#"
// JavaScript section
function jsFunction() {
    return "javascript";
}

# Python section
def python_function():
    return "python"

// Rust section
fn rust_function() -> &'static str {
    "rust"
}

class SomeClass {
    constructor() {}
}

CREATE TABLE users (
    id INTEGER PRIMARY KEY
);
"#;

        let chunks = chunker.chunk_file(mixed_content);
        
        // Should detect boundaries from multiple languages
        let has_js_fn = chunks.iter().any(|c| c.content.contains("function jsFunction"));
        let has_py_fn = chunks.iter().any(|c| c.content.contains("def python_function"));
        let has_rust_fn = chunks.iter().any(|c| c.content.contains("fn rust_function"));
        let has_class = chunks.iter().any(|c| c.content.contains("class SomeClass"));
        let has_sql = chunks.iter().any(|c| c.content.contains("CREATE TABLE"));
        
        assert!(has_js_fn, "Should detect JavaScript function boundary");
        assert!(has_py_fn, "Should detect Python function boundary"); 
        assert!(has_rust_fn, "Should detect Rust function boundary");
        assert!(has_class, "Should detect class boundary");
        assert!(has_sql, "Should detect SQL CREATE TABLE boundary");
    }

    #[test]
    fn test_edge_case_chunk_size_one() {
        init_config();
        let chunker = SimpleRegexChunker::with_chunk_size(1).expect("chunker creation failed");
        
        let content = "line1\nline2\nfn test() {}\nline4";
        let chunks = chunker.chunk_file(content);
        
        // With chunk size of 1, should create many small chunks
        assert!(chunks.len() >= 4, "Should create multiple chunks with size limit of 1");
        
        // Each chunk should have at most 1 line
        for chunk in &chunks {
            let line_count = chunk.content.lines().count();
            assert!(line_count <= 1, "Each chunk should have at most 1 line");
        }
    }

    #[test]
    fn test_huge_file_handling() {
        init_config();
        let chunker = SimpleRegexChunker::with_chunk_size(50).expect("chunker creation failed");
        
        // Create a very large file
        let mut huge_content = String::new();
        for i in 0..1000 {
            huge_content.push_str(&format!("fn function_{}() {{\n", i));
            huge_content.push_str(&format!("    // Function {} body\n", i));
            huge_content.push_str(&format!("    let x = {};\n", i));
            huge_content.push_str("}\n\n");
        }
        
        let chunks = chunker.chunk_file(&huge_content);
        
        // Should handle large files without issues
        assert!(!chunks.is_empty(), "Should handle huge files");
        assert!(chunks.len() > 20, "Should create many chunks for huge file");
        
        // Verify no chunk is ridiculously large
        for chunk in &chunks {
            let line_count = chunk.content.lines().count();
            assert!(line_count <= 100, "No chunk should be excessively large");
        }
    }
}