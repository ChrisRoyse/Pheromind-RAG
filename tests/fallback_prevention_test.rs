// Fallback Prevention Test - Ensures no fallback patterns are introduced
// This test enforces the "Truth Above All" principle by detecting fallback patterns

use std::fs;
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

#[test]
fn test_no_map_or_fallbacks() {
    // Detect .map_or patterns that might be fallbacks
    let map_or_pattern = Regex::new(r"\.map_or\(").unwrap();
    let mut violations = Vec::new();

    let entries: Result<Vec<_>, _> = WalkDir::new("src")
        .into_iter()
        .collect();
    let entries = entries
        .map_err(|e| DirectoryTraversalError { source: e })
        .expect("Fallback prevention test must traverse all directories");
    
    for entry in entries
        .into_iter()
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "rs"))
    {
        let content = fs::read_to_string(entry.path())
            .expect("Fallback prevention test must read all files");

        for (line_num, line) in content.lines().enumerate() {
            if map_or_pattern.is_match(line) {
                // Allow only specific legitimate patterns
                let is_legitimate = line.contains("extension().map_or(false,") ||  // File extension checks
                                  line.contains("FIXED:") ||                        // Fixed pattern comments
                                  line.contains("// Legitimate:");                   // Explicitly marked

                if !is_legitimate {
                    violations.push(format!("{}:{} - {}", entry.path().display(), line_num + 1, line.trim()));
                }
            }
        }
    }

    if !violations.is_empty() {
        panic!("Found {} .map_or patterns that may be fallbacks:\n{}", 
               violations.len(), 
               violations.join("\n"));
    }
}

#[test]
fn test_no_unwrap_or_fallbacks() {
    // Detect .unwrap_or patterns that provide defaults instead of proper error handling
    let unwrap_or_pattern = Regex::new(r"\.unwrap_or\(").unwrap();
    let mut violations = Vec::new();

    let entries: Result<Vec<_>, _> = WalkDir::new("src")
        .into_iter()
        .collect();
    let entries = entries
        .map_err(|e| DirectoryTraversalError { source: e })
        .expect("Fallback prevention test must traverse all directories");
    
    for entry in entries
        .into_iter()
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "rs"))
    {
        let content = fs::read_to_string(entry.path())
            .expect("Fallback prevention test must read all files");

        for (line_num, line) in content.lines().enumerate() {
            if unwrap_or_pattern.is_match(line) {
                // All .unwrap_or patterns are considered fallbacks in this codebase
                violations.push(format!("{}:{} - {}", entry.path().display(), line_num + 1, line.trim()));
            }
        }
    }

    if !violations.is_empty() {
        panic!("Found {} .unwrap_or patterns (fallbacks not allowed):\n{}", 
               violations.len(), 
               violations.join("\n"));
    }
}

#[test]
fn test_no_silent_error_ignoring() {
    // Detect patterns that ignore errors silently
    let let_underscore_pattern = Regex::new(r"let\s+_\s*=").unwrap();
    let mut violations = Vec::new();

    let entries: Result<Vec<_>, _> = WalkDir::new("src")
        .into_iter()
        .collect();
    let entries = entries
        .map_err(|e| DirectoryTraversalError { source: e })
        .expect("Fallback prevention test must traverse all directories");
    
    for entry in entries
        .into_iter()
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "rs"))
    {
        let content = fs::read_to_string(entry.path())
            .expect("Fallback prevention test must read all files");

        for (line_num, line) in content.lines().enumerate() {
            if let_underscore_pattern.is_match(line) {
                // Allow specific legitimate patterns
                let is_legitimate = line.contains("let _ = writeln!") ||     // Print operations
                                  line.contains("let _ = write!") ||       // Print operations  
                                  line.contains("// Legitimate:") ||       // Explicitly marked
                                  line.contains("FIXED:") ||               // Fixed pattern comments
                                  line.contains("let _ = env::set_var") ||  // Environment setup in tests
                                  line.contains("let _ = std::panic::take_hook");  // Panic handler setup

                if !is_legitimate {
                    violations.push(format!("{}:{} - {}", entry.path().display(), line_num + 1, line.trim()));
                }
            }
        }
    }

    if !violations.is_empty() {
        panic!("Found {} potentially problematic 'let _ =' patterns:\n{}", 
               violations.len(), 
               violations.join("\n"));
    }
}

#[test]
fn test_no_catch_all_match_arms() {
    // Detect match arms with _ => that might hide important cases
    let catch_all_pattern = Regex::new(r"_\s*=>\s*\{\s*\}").unwrap();
    let mut violations = Vec::new();

    let entries: Result<Vec<_>, _> = WalkDir::new("src")
        .into_iter()
        .collect();
    let entries = entries
        .map_err(|e| DirectoryTraversalError { source: e })
        .expect("Fallback prevention test must traverse all directories");
    
    for entry in entries
        .into_iter()
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "rs"))
    {
        let content = fs::read_to_string(entry.path())
            .expect("Fallback prevention test must read all files");

        for (line_num, line) in content.lines().enumerate() {
            if catch_all_pattern.is_match(line) {
                // Allow specific legitimate patterns
                let is_legitimate = line.contains("// Ignore other statuses") ||    // Git status filtering
                                  line.contains("// Legitimate:") ||                // Explicitly marked
                                  line.contains("FIXED:");                          // Fixed pattern comments

                if !is_legitimate {
                    violations.push(format!("{}:{} - {}", entry.path().display(), line_num + 1, line.trim()));
                }
            }
        }
    }

    if !violations.is_empty() {
        panic!("Found {} empty catch-all match arms that may hide important cases:\n{}", 
               violations.len(), 
               violations.join("\n"));
    }
}

#[test]
fn test_ok_or_else_legitimacy() {
    // Verify all .ok_or_else patterns have proper error messages
    let ok_or_else_pattern = Regex::new(r"\.ok_or_else\(").unwrap();
    let mut violations = Vec::new();

    let entries: Result<Vec<_>, _> = WalkDir::new("src")
        .into_iter()
        .collect();
    let entries = entries
        .map_err(|e| DirectoryTraversalError { source: e })
        .expect("Fallback prevention test must traverse all directories");
    
    for entry in entries
        .into_iter()
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "rs"))
    {
        let content = fs::read_to_string(entry.path())
            .expect("Fallback prevention test must read all files");

        for (line_num, line) in content.lines().enumerate() {
            if ok_or_else_pattern.is_match(line) {
                // Must have error message or proper error construction
                let has_proper_error = line.contains("Error::") ||
                                     line.contains("anyhow!") ||
                                     line.contains("format!") ||
                                     line.contains("SearchError::");

                if !has_proper_error {
                    violations.push(format!("{}:{} - {}", entry.path().display(), line_num + 1, line.trim()));
                }
            }
        }
    }

    if !violations.is_empty() {
        panic!("Found {} .ok_or_else patterns without proper error construction:\n{}", 
               violations.len(), 
               violations.join("\n"));
    }
}

#[test]
fn test_comprehensive_fallback_audit() {
    println!("🔍 Running Comprehensive Fallback Prevention Audit...\n");

    // Run all fallback detection tests
    let test_results = vec![
        ("No .map_or fallbacks", std::panic::catch_unwind(|| test_no_map_or_fallbacks())),
        ("No .unwrap_or fallbacks", std::panic::catch_unwind(|| test_no_unwrap_or_fallbacks())),
        ("No silent error ignoring", std::panic::catch_unwind(|| test_no_silent_error_ignoring())),
        ("No catch-all match arms", std::panic::catch_unwind(|| test_no_catch_all_match_arms())),
        ("Legitimate .ok_or_else usage", std::panic::catch_unwind(|| test_ok_or_else_legitimacy())),
    ];

    let mut failures = Vec::new();
    let mut passes = 0;

    for (name, result) in test_results {
        match result {
            Ok(_) => {
                println!("✅ {}", name);
                passes += 1;
            },
            Err(_) => {
                println!("❌ {}", name);
                failures.push(name);
            }
        }
    }

    println!("\n📊 Fallback Prevention Audit Results:");
    println!("  Passed: {} / {}", passes, passes + failures.len());
    
    if !failures.is_empty() {
        println!("  Failed: {}", failures.len());
        panic!("Fallback prevention audit failed. {} tests failed: {}", 
               failures.len(), 
               failures.join(", "));
    }

    println!("✅ All fallback prevention checks PASSED - codebase is fallback-free!");
}