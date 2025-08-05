use embed_search::chunking::{SimpleRegexChunker, ThreeChunkExpander, ChunkContext};
use std::fs;
use std::path::Path;
use std::time::Instant;

/// Comprehensive stress testing for ThreeChunkExpander against vectortest files
struct StressTestResults {
    files_tested: usize,
    total_contexts: usize,
    errors: Vec<String>,
    performance_issues: Vec<String>,
}

impl StressTestResults {
    fn new() -> Self {
        Self {
            files_tested: 0,
            total_contexts: 0,
            errors: Vec::new(),
            performance_issues: Vec::new(),
        }
    }
    
    fn add_error(&mut self, error: String) {
        self.errors.push(error);
    }
    
    fn add_performance_issue(&mut self, issue: String) {
        self.performance_issues.push(issue);
    }
    
    fn is_success(&self) -> bool {
        self.errors.is_empty() && self.performance_issues.is_empty()
    }
    
    fn summary(&self) -> String {
        format!(
            "Tested {} files, {} contexts. Errors: {}, Performance issues: {}",
            self.files_tested, self.total_contexts, 
            self.errors.len(), self.performance_issues.len()
        )
    }
}

/// Test a single file comprehensively
fn stress_test_file(file_path: &Path) -> Result<StressTestResults, String> {
    let mut results = StressTestResults::new();
    results.files_tested = 1;
    
    let content = fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read {:?}: {}", file_path, e))?;
    
    let chunker = SimpleRegexChunker::new();
    let chunks = chunker.chunk_file(&content);
    
    if chunks.is_empty() {
        return Ok(results); // Empty file is valid
    }
    
    // Test every possible chunk as target
    for target_index in 0..chunks.len() {
        results.total_contexts += 1;
        
        // Measure expansion performance
        let start = Instant::now();
        let context = match ThreeChunkExpander::expand(&chunks, target_index) {
            Ok(ctx) => ctx,
            Err(e) => {
                results.add_error(format!("Expansion failed for {:?} index {}: {}", 
                                        file_path, target_index, e));
                continue;
            }
        };
        let expansion_time = start.elapsed();
        
        // Performance check: should be < 1ms
        if expansion_time.as_millis() > 1 {
            results.add_performance_issue(format!(
                "Slow expansion for {:?} index {}: {}ms", 
                file_path, target_index, expansion_time.as_millis()
            ));
        }
        
        // Validate context integrity
        if let Err(e) = validate_context_integrity(&context, &chunks, target_index) {
            results.add_error(format!("Context integrity failed for {:?} index {}: {}", 
                                    file_path, target_index, e));
        }
        
        // Test display formatting (should not panic)
        let _display = context.format_for_display();
        let _summary = context.format_summary();
        let _content = context.get_full_content();
    }
    
    Ok(results)
}

/// Validate that a context is internally consistent
fn validate_context_integrity(context: &ChunkContext, original_chunks: &[embed_search::chunking::Chunk], target_index: usize) -> Result<(), String> {
    // Verify target index matches
    if context.target_index != target_index {
        return Err(format!("Target index mismatch: expected {}, got {}", 
                          target_index, context.target_index));
    }
    
    // Verify target chunk matches original
    if context.target != original_chunks[target_index] {
        return Err("Target chunk doesn't match original".to_string());
    }
    
    // Verify above chunk (if present)
    if let Some(above) = &context.above {
        if target_index == 0 {
            return Err("Above chunk present but target is first chunk".to_string());
        }
        if above != &original_chunks[target_index - 1] {
            return Err("Above chunk doesn't match expected chunk".to_string());
        }
    } else if target_index > 0 {
        return Err("Above chunk missing but target is not first chunk".to_string());
    }
    
    // Verify below chunk (if present)
    if let Some(below) = &context.below {
        if target_index >= original_chunks.len() - 1 {
            return Err("Below chunk present but target is last chunk".to_string());
        }
        if below != &original_chunks[target_index + 1] {
            return Err("Below chunk doesn't match expected chunk".to_string());
        }
    } else if target_index < original_chunks.len() - 1 {
        return Err("Below chunk missing but target is not last chunk".to_string());
    }
    
    // Verify line ranges are consistent
    let (start, end) = ThreeChunkExpander::get_line_range(context);
    let expected_start = context.above.as_ref()
        .map(|c| c.start_line)
        .unwrap_or(context.target.start_line);
    let expected_end = context.below.as_ref()
        .map(|c| c.end_line)
        .unwrap_or(context.target.end_line);
    
    if start != expected_start || end != expected_end {
        return Err(format!("Line range mismatch: got ({}, {}), expected ({}, {})", 
                          start, end, expected_start, expected_end));
    }
    
    Ok(())
}

#[test]
fn test_python_files() {
    let python_files = vec!["vectortest/auth_service.py"];
    
    for file_path in python_files {
        let path = Path::new(file_path);
        if path.exists() {
            match stress_test_file(path) {
                Ok(results) => {
                    if !results.is_success() {
                        panic!("Python file {} failed: {}", file_path, results.summary());
                    }
                    println!("✅ Python file {} passed: {}", file_path, results.summary());
                }
                Err(e) => panic!("Python file {} error: {}", file_path, e),
            }
        }
    }
}

#[test]
fn test_javascript_files() {
    let js_files = vec!["vectortest/user_controller.js"];
    
    for file_path in js_files {
        let path = Path::new(file_path);
        if path.exists() {
            match stress_test_file(path) {
                Ok(results) => {
                    if !results.is_success() {
                        panic!("JavaScript file {} failed: {}", file_path, results.summary());
                    }
                    println!("✅ JavaScript file {} passed: {}", file_path, results.summary());
                }
                Err(e) => panic!("JavaScript file {} error: {}", file_path, e),
            }
        }
    }
}

#[test]
fn test_typescript_files() {
    let ts_files = vec!["vectortest/payment_gateway.ts"];
    
    for file_path in ts_files {
        let path = Path::new(file_path);
        if path.exists() {
            match stress_test_file(path) {
                Ok(results) => {
                    if !results.is_success() {
                        panic!("TypeScript file {} failed: {}", file_path, results.summary());
                    }
                    println!("✅ TypeScript file {} passed: {}", file_path, results.summary());
                }
                Err(e) => panic!("TypeScript file {} error: {}", file_path, e),
            }
        }
    }
}

#[test]
fn test_java_files() {
    let java_files = vec!["vectortest/OrderService.java"];
    
    for file_path in java_files {
        let path = Path::new(file_path);
        if path.exists() {
            match stress_test_file(path) {
                Ok(results) => {
                    if !results.is_success() {
                        panic!("Java file {} failed: {}", file_path, results.summary());
                    }
                    println!("✅ Java file {} passed: {}", file_path, results.summary());
                }
                Err(e) => panic!("Java file {} error: {}", file_path, e),
            }
        }
    }
}

#[test]
fn test_go_files() {
    let go_files = vec!["vectortest/analytics_dashboard.go"];
    
    for file_path in go_files {
        let path = Path::new(file_path);
        if path.exists() {
            match stress_test_file(path) {
                Ok(results) => {
                    if !results.is_success() {
                        panic!("Go file {} failed: {}", file_path, results.summary());
                    }
                    println!("✅ Go file {} passed: {}", file_path, results.summary());
                }
                Err(e) => panic!("Go file {} error: {}", file_path, e),
            }
        }
    }
}

#[test]
fn test_rust_files() {
    let rust_files = vec!["vectortest/memory_cache.rs"];
    
    for file_path in rust_files {
        let path = Path::new(file_path);
        if path.exists() {
            match stress_test_file(path) {
                Ok(results) => {
                    if !results.is_success() {
                        panic!("Rust file {} failed: {}", file_path, results.summary());
                    }
                    println!("✅ Rust file {} passed: {}", file_path, results.summary());
                }
                Err(e) => panic!("Rust file {} error: {}", file_path, e),
            }
        }
    }
}

#[test]
fn test_ruby_files() {
    let ruby_files = vec!["vectortest/product_catalog.rb"];
    
    for file_path in ruby_files {
        let path = Path::new(file_path);
        if path.exists() {
            match stress_test_file(path) {
                Ok(results) => {
                    if !results.is_success() {
                        panic!("Ruby file {} failed: {}", file_path, results.summary());
                    }
                    println!("✅ Ruby file {} passed: {}", file_path, results.summary());
                }
                Err(e) => panic!("Ruby file {} error: {}", file_path, e),
            }
        }
    }
}

#[test]
fn test_cpp_files() {
    let cpp_files = vec!["vectortest/websocket_server.cpp"];
    
    for file_path in cpp_files {
        let path = Path::new(file_path);
        if path.exists() {
            match stress_test_file(path) {
                Ok(results) => {
                    if !results.is_success() {
                        panic!("C++ file {} failed: {}", file_path, results.summary());
                    }
                    println!("✅ C++ file {} passed: {}", file_path, results.summary());
                }
                Err(e) => panic!("C++ file {} error: {}", file_path, e),
            }
        }
    }
}

#[test]
fn test_csharp_files() {
    let csharp_files = vec!["vectortest/DataProcessor.cs"];
    
    for file_path in csharp_files {
        let path = Path::new(file_path);
        if path.exists() {
            match stress_test_file(path) {
                Ok(results) => {
                    if !results.is_success() {
                        panic!("C# file {} failed: {}", file_path, results.summary());
                    }
                    println!("✅ C# file {} passed: {}", file_path, results.summary());
                }
                Err(e) => panic!("C# file {} error: {}", file_path, e),
            }
        }
    }
}

#[test]
fn test_sql_files() {
    let sql_files = vec!["vectortest/database_migration.sql"];
    
    for file_path in sql_files {
        let path = Path::new(file_path);
        if path.exists() {
            match stress_test_file(path) {
                Ok(results) => {
                    if !results.is_success() {
                        panic!("SQL file {} failed: {}", file_path, results.summary());
                    }
                    println!("✅ SQL file {} passed: {}", file_path, results.summary());
                }
                Err(e) => panic!("SQL file {} error: {}", file_path, e),
            }
        }
    }
}

#[test]
fn test_markdown_documentation() {
    let md_files = vec![
        "vectortest/API_DOCUMENTATION.md",
        "vectortest/ARCHITECTURE_OVERVIEW.md", 
        "vectortest/CONTRIBUTING.md",
        "vectortest/DEPLOYMENT_GUIDE.md",
        "vectortest/TROUBLESHOOTING.md",
    ];
    
    for file_path in md_files {
        let path = Path::new(file_path);
        if path.exists() {
            match stress_test_file(path) {
                Ok(results) => {
                    if !results.is_success() {
                        panic!("Markdown file {} failed: {}", file_path, results.summary());
                    }
                    println!("✅ Markdown file {} passed: {}", file_path, results.summary());
                }
                Err(e) => panic!("Markdown file {} error: {}", file_path, e),
            }
        }
    }
}

#[test]
fn test_all_vectortest_files_comprehensive() {
    let vectortest_dir = Path::new("vectortest");
    if !vectortest_dir.exists() {
        println!("Skipping comprehensive test: vectortest directory not found");
        return;
    }
    
    let mut overall_results = StressTestResults::new();
    
    for entry in fs::read_dir(vectortest_dir).expect("Failed to read vectortest directory") {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();
        
        if path.is_file() {
            match stress_test_file(&path) {
                Ok(mut results) => {
                    overall_results.files_tested += results.files_tested;
                    overall_results.total_contexts += results.total_contexts;
                    overall_results.errors.append(&mut results.errors);
                    overall_results.performance_issues.append(&mut results.performance_issues);
                }
                Err(e) => {
                    overall_results.add_error(format!("File {:?}: {}", path, e));
                }
            }
        }
    }
    
    println!("📊 Comprehensive test results: {}", overall_results.summary());
    
    if !overall_results.errors.is_empty() {
        println!("❌ Errors found:");
        for error in &overall_results.errors {
            println!("  - {}", error);
        }
    }
    
    if !overall_results.performance_issues.is_empty() {
        println!("⚠️  Performance issues:");
        for issue in &overall_results.performance_issues {
            println!("  - {}", issue);
        }
    }
    
    if !overall_results.is_success() {
        panic!("Comprehensive stress test failed: {}", overall_results.summary());
    }
    
    println!("✅ All vectortest files passed three-chunk expansion stress test!");
}

#[test]
fn test_display_formatting() {
    // Test display formatting with a realistic example
    let chunker = SimpleRegexChunker::new();
    let content = r#"// Header comment
fn first_function() {
    println!("First");
}

fn second_function() {
    println!("Second");
}

fn third_function() {
    println!("Third");
}"#;
    
    let chunks = chunker.chunk_file(content);
    assert!(chunks.len() >= 3, "Need at least 3 chunks for display test");
    
    // Test middle chunk with full context
    let context = ThreeChunkExpander::expand(&chunks, 1).unwrap();
    
    let display = context.format_for_display();
    assert!(display.contains("Context Above"));
    assert!(display.contains("TARGET MATCH"));
    assert!(display.contains("Context Below"));
    assert!(display.contains("lines"));
    
    let summary = context.format_summary();
    assert!(summary.contains("Match at chunk 1"));
    assert!(summary.contains("full context"));
    
    let full_content = context.get_full_content();
    assert!(!full_content.is_empty());
    
    println!("Display format sample:\n{}", display);
    println!("Summary: {}", summary);
}

#[test]
fn test_edge_case_robustness() {
    let chunker = SimpleRegexChunker::new();
    
    // Test with various edge cases
    let edge_cases = vec![
        ("empty", ""),
        ("single_line", "fn test() {}"),
        ("no_boundaries", "line1\nline2\nline3"),
        ("many_small_chunks", "fn a(){}\nfn b(){}\nfn c(){}\nfn d(){}\nfn e(){}"),
    ];
    
    for (name, content) in edge_cases {
        let chunks = chunker.chunk_file(content);
        
        if chunks.is_empty() {
            continue; // Skip empty content
        }
        
        // Test expansion for every chunk
        for i in 0..chunks.len() {
            let context = ThreeChunkExpander::expand(&chunks, i)
                .unwrap_or_else(|e| panic!("Edge case '{}' chunk {} failed: {}", name, i, e));
            
            // Validate display doesn't panic
            let _display = context.format_for_display();
            let _summary = context.format_summary();
            let _content = context.get_full_content();
            
            // Validate line counts
            let line_count = ThreeChunkExpander::count_lines(&context);
            assert!(line_count > 0, "Edge case '{}' chunk {} has zero lines", name, i);
        }
        
        println!("✅ Edge case '{}' passed with {} chunks", name, chunks.len());
    }
}