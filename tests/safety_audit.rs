// Safety Audit Test Infrastructure - Phase 1: Foundation & Safety
// This test file implements comprehensive safety validation following TDD principles
// All tests should FAIL initially, proving they detect real issues

use std::fs;
use std::path::{Path, PathBuf};
use regex::Regex;
use walkdir::WalkDir;

#[derive(Debug)]
struct DirectoryTraversalError {
    source: walkdir::Error,
}

impl std::fmt::Display for DirectoryTraversalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Directory traversal failed: {}", self.source)
    }
}

impl std::error::Error for DirectoryTraversalError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}

#[derive(Debug, Clone)]
struct UnsafeImpl {
    type_name: String,
    file: PathBuf,
    line: usize,
    content: String,
    has_justification: bool,
}

#[derive(Debug, Clone)]
struct UnwrapCall {
    file: PathBuf,
    line: usize,
    context: String,
}

/// Audit all unsafe Send/Sync implementations in the codebase
fn audit_unsafe_implementations() -> Vec<UnsafeImpl> {
    let mut unsafe_impls = Vec::new();
    let unsafe_pattern = Regex::new(r"unsafe\s+impl\s+(Send|Sync)").unwrap();
    let safety_comment_pattern = Regex::new(r"(?i)safety:|safe\s+because").unwrap();
    
    let entries: Result<Vec<_>, _> = WalkDir::new("src")
        .into_iter()
        .collect();
    let entries = entries
        .map_err(|e| DirectoryTraversalError { source: e })
        .expect("Safety audit must traverse all directories");
    
    for entry in entries
        .into_iter()
        .filter(|e| {
            e.path().extension()
                .map_or(false, |ext| ext == "rs")
        })
    {
        let content = fs::read_to_string(entry.path())
            .unwrap_or_else(|e| panic!("SAFETY AUDIT FAILURE: Cannot read file {:?} for safety analysis: {}. Safety audit must access all source files to be valid.", entry.path(), e));
        
        for (line_num, line) in content.lines().enumerate() {
            if unsafe_pattern.is_match(line) {
                // Check for safety justification in surrounding lines
                let start = line_num.saturating_sub(5);
                let end = (line_num + 5).min(content.lines().count());
                let context: String = content.lines()
                    .skip(start)
                    .take(end - start)
                    .collect::<Vec<_>>()
                    .join("\n");
                
                let has_justification = safety_comment_pattern.is_match(&context);
                
                unsafe_impls.push(UnsafeImpl {
                    type_name: extract_type_name(line),
                    file: entry.path().to_path_buf(),
                    line: line_num + 1,
                    content: line.to_string(),
                    has_justification,
                });
            }
        }
    }
    
    unsafe_impls
}

/// Scan for unwrap() calls in production code
fn scan_for_unwrap_calls() -> Vec<UnwrapCall> {
    let mut unwrap_calls = Vec::new();
    let unwrap_pattern = Regex::new(r"\.unwrap\(\)").unwrap();
    
    let entries: Result<Vec<_>, _> = WalkDir::new("src")
        .into_iter()
        .collect();
    let entries = entries
        .map_err(|e| DirectoryTraversalError { source: e })
        .expect("Safety audit must traverse all directories");
    
    for entry in entries
        .into_iter()
        .filter(|e| {
            e.path().extension()
                .map_or(false, |ext| ext == "rs")
        })
    {
        let content = fs::read_to_string(entry.path())
            .unwrap_or_else(|e| panic!("SAFETY AUDIT FAILURE: Cannot read file {:?} for safety analysis: {}. Safety audit must access all source files to be valid.", entry.path(), e));
        
        for (line_num, line) in content.lines().enumerate() {
            if unwrap_pattern.is_match(line) {
                unwrap_calls.push(UnwrapCall {
                    file: entry.path().to_path_buf(),
                    line: line_num + 1,
                    context: line.trim().to_string(),
                });
            }
        }
    }
    
    unwrap_calls
}

/// Scan for expect() calls in production code
fn scan_for_expect_calls() -> Vec<UnwrapCall> {
    let mut expect_calls = Vec::new();
    let expect_pattern = Regex::new(r"\.expect\(").unwrap();
    
    let entries: Result<Vec<_>, _> = WalkDir::new("src")
        .into_iter()
        .collect();
    let entries = entries
        .map_err(|e| DirectoryTraversalError { source: e })
        .expect("Safety audit must traverse all directories");
    
    for entry in entries
        .into_iter()
        .filter(|e| {
            e.path().extension()
                .map_or(false, |ext| ext == "rs")
        })
    {
        let content = fs::read_to_string(entry.path())
            .unwrap_or_else(|e| panic!("SAFETY AUDIT FAILURE: Cannot read file {:?} for safety analysis: {}. Safety audit must access all source files to be valid.", entry.path(), e));
        
        for (line_num, line) in content.lines().enumerate() {
            if expect_pattern.is_match(line) {
                expect_calls.push(UnwrapCall {
                    file: entry.path().to_path_buf(),
                    line: line_num + 1,
                    context: line.trim().to_string(),
                });
            }
        }
    }
    
    expect_calls
}

/// Scan for panic!() calls in production code
fn scan_for_panic_calls() -> Vec<UnwrapCall> {
    let mut panic_calls = Vec::new();
    let panic_pattern = Regex::new(r"panic!\(").unwrap();
    
    let entries: Result<Vec<_>, _> = WalkDir::new("src")
        .into_iter()
        .collect();
    let entries = entries
        .map_err(|e| DirectoryTraversalError { source: e })
        .expect("Safety audit must traverse all directories");
    
    for entry in entries
        .into_iter()
        .filter(|e| {
            e.path().extension()
                .map_or(false, |ext| ext == "rs")
        })
    {
        let content = fs::read_to_string(entry.path())
            .unwrap_or_else(|e| panic!("SAFETY AUDIT FAILURE: Cannot read file {:?} for safety analysis: {}. Safety audit must access all source files to be valid.", entry.path(), e));
        
        for (line_num, line) in content.lines().enumerate() {
            if panic_pattern.is_match(line) {
                panic_calls.push(UnwrapCall {
                    file: entry.path().to_path_buf(),
                    line: line_num + 1,
                    context: line.trim().to_string(),
                });
            }
        }
    }
    
    panic_calls
}

fn extract_type_name(line: &str) -> String {
    // Extract the type name from "unsafe impl Send for TypeName"
    if let Some(pos) = line.find(" for ") {
        let after_for = &line[pos + 5..];
        match after_for.split_whitespace().next() {
            Some(type_name) => type_name.trim_end_matches('{').to_string(),
            None => {
                eprintln!("Warning: Could not extract type name from line: {}", line);
                "UnparseableType".to_string()
            }
        }
    } else {
        eprintln!("Warning: Unexpected unsafe impl format in line: {}", line);
        "UnparseableImplementation".to_string()
    }
}

// ==================== TESTS ====================
// These tests should FAIL initially (RED phase of TDD)
// They will pass only after implementing the fixes

#[test]
fn test_no_unsafe_send_sync_without_justification() {
    let unsafe_impls = audit_unsafe_implementations();
    
    let unjustified: Vec<_> = unsafe_impls
        .iter()
        .filter(|impl_| !impl_.has_justification)
        .collect();
    
    if !unjustified.is_empty() {
        eprintln!("\n❌ Found {} unsafe Send/Sync implementations without safety justification:", 
            unjustified.len());
        for unsafe_impl in &unjustified {
            eprintln!("  • {} at {}:{}", 
                unsafe_impl.type_name, 
                unsafe_impl.file.display(), 
                unsafe_impl.line);
            eprintln!("    {}", unsafe_impl.content);
        }
    }
    
    assert!(
        unjustified.is_empty(),
        "Found {} unsafe Send/Sync implementations without safety justification",
        unjustified.len()
    );
}

#[test]
fn test_no_production_unwrap_calls() {
    let unwrap_calls = scan_for_unwrap_calls();
    
    if !unwrap_calls.is_empty() {
        eprintln!("\n❌ Found {} unwrap() calls in production code:", unwrap_calls.len());
        
        // Group by file for better readability
        let mut by_file: std::collections::HashMap<PathBuf, Vec<&UnwrapCall>> = 
            std::collections::HashMap::new();
        
        for call in &unwrap_calls {
            by_file.entry(call.file.clone()).or_default().push(call);
        }
        
        for (file, calls) in by_file.iter() {
            eprintln!("\n  📁 {} ({} calls):", file.display(), calls.len());
            for call in calls.iter().take(5) {  // Show first 5 per file
                eprintln!("    Line {}: {}", call.line, call.context);
            }
            if calls.len() > 5 {
                eprintln!("    ... and {} more", calls.len() - 5);
            }
        }
    }
    
    assert!(
        unwrap_calls.is_empty(),
        "Found {} unwrap() calls in production code. Use proper error handling instead.",
        unwrap_calls.len()
    );
}

#[test]
fn test_no_production_expect_calls() {
    let expect_calls = scan_for_expect_calls();
    
    // Allow expect() only in initialization code with proper messages
    let problematic: Vec<_> = expect_calls
        .iter()
        .filter(|call| {
            // Check if expect has a descriptive message
            !call.context.contains("\"") || call.context.contains("\"\"")
        })
        .collect();
    
    if !problematic.is_empty() {
        eprintln!("\n❌ Found {} expect() calls without descriptive messages:", 
            problematic.len());
        for call in problematic.iter().take(10) {
            eprintln!("  • {}:{} - {}", 
                call.file.display(), 
                call.line, 
                call.context);
        }
    }
    
    assert!(
        problematic.is_empty(),
        "Found {} expect() calls without descriptive error messages",
        problematic.len()
    );
}

#[test]
fn test_no_production_panic_calls() {
    let panic_calls = scan_for_panic_calls();
    
    if !panic_calls.is_empty() {
        eprintln!("\n❌ Found {} panic!() calls in production code:", panic_calls.len());
        for call in &panic_calls {
            eprintln!("  • {}:{} - {}", 
                call.file.display(), 
                call.line, 
                call.context);
        }
    }
    
    assert!(
        panic_calls.is_empty(),
        "Found {} panic!() calls in production code. Use Result types instead.",
        panic_calls.len()
    );
}

#[test]
fn test_thread_safety_patterns() {
    // Check that storage implementations use proper concurrency primitives
    let storage_files = vec![
        "src/storage/simple_vectordb.rs",
        "src/storage/lancedb_storage.rs",
        "src/storage/lancedb.rs",
    ];
    
    for file in storage_files {
        let path = Path::new(file);
        if path.exists() {
            let content = fs::read_to_string(path).unwrap();
            
            // Should use Arc<RwLock> or Arc<Mutex> patterns
            let has_arc = content.contains("Arc<");
            let has_lock = content.contains("RwLock") || content.contains("Mutex");
            
            assert!(
                has_arc && has_lock,
                "File {} should use Arc<RwLock> or Arc<Mutex> for thread safety",
                file
            );
        }
    }
}

#[test]
fn test_bounded_caches() {
    // Check that cache implementations have size limits
    let cache_pattern = Regex::new(r"(HashMap|BTreeMap|Vec)<.*>").unwrap();
    let bounded_pattern = Regex::new(r"(LruCache|BoundedCache|max_size|capacity)").unwrap();
    
    let entries: Result<Vec<_>, _> = WalkDir::new("src")
        .into_iter()
        .collect();
    let entries = entries
        .map_err(|e| DirectoryTraversalError { source: e })
        .expect("Safety audit must traverse all directories");
    
    for entry in entries
        .into_iter()
        .filter(|e| e.path().to_string_lossy().contains("cache"))
        .filter(|e| {
            e.path().extension()
                .map_or(false, |ext| ext == "rs")
        })
    {
        let content = fs::read_to_string(entry.path())
            .unwrap_or_else(|e| panic!("SAFETY AUDIT FAILURE: Cannot read file {:?} for safety analysis: {}. Safety audit must access all source files to be valid.", entry.path(), e));
        
        if cache_pattern.is_match(&content) {
            assert!(
                bounded_pattern.is_match(&content),
                "Cache in {} should have bounded size (use LruCache or similar)",
                entry.path().display()
            );
        }
    }
}

// ==================== COMPREHENSIVE VALIDATION ====================

#[test]
fn comprehensive_safety_validation() {
    eprintln!("\n🔍 Running Comprehensive Safety Validation for Phase 1...\n");
    
    let mut all_passed = true;
    
    // Memory Safety
    eprintln!("📊 Memory Safety Validation:");
    let unsafe_impls = audit_unsafe_implementations();
    let unjustified = unsafe_impls.iter().filter(|i| !i.has_justification).count();
    if unjustified > 0 {
        eprintln!("  ❌ {} unsafe implementations without justification", unjustified);
        all_passed = false;
    } else {
        eprintln!("  ✅ All unsafe implementations properly justified");
    }
    
    // Error Handling
    eprintln!("\n📊 Error Handling Validation:");
    let unwrap_count = scan_for_unwrap_calls().len();
    let expect_count = scan_for_expect_calls().len();
    let panic_count = scan_for_panic_calls().len();
    
    if unwrap_count > 0 {
        eprintln!("  ❌ {} unwrap() calls found", unwrap_count);
        all_passed = false;
    } else {
        eprintln!("  ✅ No unwrap() calls in production code");
    }
    
    if expect_count > 0 {
        eprintln!("  ⚠️ {} expect() calls found (review needed)", expect_count);
    }
    
    if panic_count > 0 {
        eprintln!("  ❌ {} panic!() calls found", panic_count);
        all_passed = false;
    } else {
        eprintln!("  ✅ No panic!() calls in production code");
    }
    
    eprintln!("\n📊 Summary:");
    if all_passed {
        eprintln!("  ✅ All safety validations PASSED - Phase 1 complete!");
    } else {
        eprintln!("  ❌ Safety validations FAILED - fixes required");
    }
    
    assert!(all_passed, "Phase 1 safety validation failed. See details above.");
}