use embed_search::chunking::{SimpleRegexChunker, LineValidator};
use std::path::Path;
use std::fs;

/// Test line tracking accuracy for a specific file
fn test_file_line_tracking(file_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let chunker = SimpleRegexChunker::new();
    let content = fs::read_to_string(file_path)?;
    let lines: Vec<&str> = content.lines().collect();
    let chunks = chunker.chunk_file(&content);
    
    // Validate coverage
    LineValidator::validate_coverage(&chunks, lines.len())?;
    
    // Validate content matches
    LineValidator::validate_content(&chunks, &lines)?;
    
    // Additional validation: verify we can reconstruct the file
    let mut reconstructed = String::new();
    for chunk in &chunks {
        if !reconstructed.is_empty() && chunk.start_line > 0 {
            reconstructed.push('\n');
        }
        reconstructed.push_str(&chunk.content);
    }
    
    // Compare line by line (to handle line ending differences)
    let reconstructed_lines: Vec<&str> = reconstructed.lines().collect();
    assert_eq!(reconstructed_lines.len(), lines.len(),
               "Reconstructed file has different line count for {:?}", file_path);
    
    Ok(())
}

#[test]
fn test_python_file_line_tracking() {
    let path = Path::new("vectortest/auth_service.py");
    if path.exists() {
        test_file_line_tracking(path).expect("Python file line tracking failed");
    }
}

#[test]
fn test_javascript_file_line_tracking() {
    let path = Path::new("vectortest/user_controller.js");
    if path.exists() {
        test_file_line_tracking(path).expect("JavaScript file line tracking failed");
    }
}

#[test]
fn test_typescript_file_line_tracking() {
    let path = Path::new("vectortest/payment_gateway.ts");
    if path.exists() {
        test_file_line_tracking(path).expect("TypeScript file line tracking failed");
    }
}

#[test]
fn test_java_file_line_tracking() {
    let path = Path::new("vectortest/OrderService.java");
    if path.exists() {
        test_file_line_tracking(path).expect("Java file line tracking failed");
    }
}

#[test]
fn test_go_file_line_tracking() {
    let path = Path::new("vectortest/analytics_dashboard.go");
    if path.exists() {
        test_file_line_tracking(path).expect("Go file line tracking failed");
    }
}

#[test]
fn test_rust_file_line_tracking() {
    let path = Path::new("vectortest/memory_cache.rs");
    if path.exists() {
        test_file_line_tracking(path).expect("Rust file line tracking failed");
    }
}

#[test]
fn test_ruby_file_line_tracking() {
    let path = Path::new("vectortest/product_catalog.rb");
    if path.exists() {
        test_file_line_tracking(path).expect("Ruby file line tracking failed");
    }
}

#[test]
fn test_cpp_file_line_tracking() {
    let path = Path::new("vectortest/websocket_server.cpp");
    if path.exists() {
        test_file_line_tracking(path).expect("C++ file line tracking failed");
    }
}

#[test]
fn test_csharp_file_line_tracking() {
    let path = Path::new("vectortest/DataProcessor.cs");
    if path.exists() {
        test_file_line_tracking(path).expect("C# file line tracking failed");
    }
}

#[test]
fn test_sql_file_line_tracking() {
    let path = Path::new("vectortest/database_migration.sql");
    if path.exists() {
        test_file_line_tracking(path).expect("SQL file line tracking failed");
    }
}

#[test]
fn test_markdown_file_line_tracking() {
    let paths = vec![
        "vectortest/API_DOCUMENTATION.md",
        "vectortest/ARCHITECTURE_OVERVIEW.md",
        "vectortest/CONTRIBUTING.md",
        "vectortest/DEPLOYMENT_GUIDE.md",
        "vectortest/TROUBLESHOOTING.md",
    ];
    
    for path_str in paths {
        let path = Path::new(path_str);
        if path.exists() {
            test_file_line_tracking(path)
                .unwrap_or_else(|e| panic!("Markdown file {:?} line tracking failed: {}", path, e));
        }
    }
}

#[test]
fn test_all_vectortest_files() {
    let vectortest_dir = Path::new("vectortest");
    if !vectortest_dir.exists() {
        println!("Skipping test: vectortest directory not found");
        return;
    }
    
    let mut tested_files = 0;
    let mut errors = Vec::new();
    
    for entry in fs::read_dir(vectortest_dir).expect("Failed to read vectortest directory") {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();
        
        if path.is_file() {
            tested_files += 1;
            if let Err(e) = test_file_line_tracking(&path) {
                errors.push((path.clone(), e));
            }
        }
    }
    
    if !errors.is_empty() {
        eprintln!("Line tracking errors found:");
        for (path, error) in &errors {
            eprintln!("  {:?}: {}", path, error);
        }
        panic!("{} files failed line tracking validation out of {}", errors.len(), tested_files);
    }
    
    println!("Successfully validated line tracking for {} files", tested_files);
}

#[test]
fn test_specific_line_retrieval() {
    let path = Path::new("vectortest/auth_service.py");
    if path.exists() {
        let chunker = SimpleRegexChunker::new();
        let content = fs::read_to_string(path).expect("Failed to read file");
        let lines: Vec<&str> = content.lines().collect();
        let chunks = chunker.chunk_file(&content);
        
        // Test retrieving specific lines
        let test_lines = vec![0, 10, 25, 50, 100, lines.len() - 1];
        
        for &line_num in &test_lines {
            if line_num < lines.len() {
                // Find which chunk contains this line
                let chunk = chunks.iter().find(|c| {
                    line_num >= c.start_line && line_num <= c.end_line
                }).expect(&format!("Line {} not found in any chunk", line_num));
                
                // Extract the specific line from the chunk
                let chunk_lines: Vec<&str> = chunk.content.lines().collect();
                let line_index_in_chunk = line_num - chunk.start_line;
                
                assert!(line_index_in_chunk < chunk_lines.len(),
                        "Line index {} out of bounds in chunk", line_index_in_chunk);
                
                assert_eq!(chunk_lines[line_index_in_chunk], lines[line_num],
                          "Line {} content mismatch", line_num);
            }
        }
    }
}

#[test]
fn test_chunk_boundaries_preserve_context() {
    let path = Path::new("vectortest/OrderService.java");
    if path.exists() {
        let chunker = SimpleRegexChunker::new();
        let chunks = chunker.chunk_file_from_path(path).expect("Failed to chunk file");
        
        // Verify that method signatures are kept with their bodies
        for chunk in &chunks {
            let lines: Vec<&str> = chunk.content.lines().collect();
            
            // If chunk starts with a method annotation, the method should be in the same chunk
            if lines.first().map_or(false, |l| l.trim().starts_with("@")) {
                let has_method = lines.iter().any(|l| {
                    l.contains("public") || l.contains("private") || l.contains("protected")
                });
                assert!(has_method, "Annotation without method in chunk: {:?}", lines[0]);
            }
        }
    }
}