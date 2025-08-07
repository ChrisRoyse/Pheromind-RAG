/// Investigation into TantivySearcher fuzzy search functionality
/// This test specifically focuses on the fuzzy search problems

use tempfile::TempDir;
use std::fs;

#[cfg(feature = "tantivy")]
use embed_search::search::tantivy_search::TantivySearcher;

#[cfg(feature = "tantivy")]
#[tokio::test]
async fn test_tantivy_fuzzy_search_investigation() {
    println!("🔬 Investigating TantivySearcher Fuzzy Search Issues");
    
    // Create temporary directory and test file
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let test_file = temp_dir.path().join("fuzzy_test.rs");
    
    let test_content = r#"fn authenticate_user(username: &str, password: &str) -> bool {
    verify_credentials(username, password)
}

fn calculate_sum(a: i32, b: i32) -> i32 {
    a + b
}

struct DatabaseConnection {
    connection_string: String,
    timeout: u64,
}

fn process_payment(amount: f64, currency: &str) -> Result<(), String> {
    // Process payment logic
    Ok(())
}
"#;
    
    fs::write(&test_file, test_content).expect("Failed to write test file");
    println!("✅ Created test file with known content");
    
    // Create and index
    let mut searcher = TantivySearcher::new().await.expect("Failed to create TantivySearcher");
    searcher.index_directory(temp_dir.path()).await.expect("Failed to index directory");
    println!("✅ Indexed test file");
    
    // Test exact searches first (baseline)
    println!("\n🔍 BASELINE: Testing exact searches");
    let exact_terms = vec![
        "authenticate",
        "calculate", 
        "Database",
        "process_payment"
    ];
    
    for term in &exact_terms {
        let results = searcher.search(term).await.expect("Search failed");
        println!("  '{}': {} exact matches", term, results.len());
        if !results.is_empty() {
            println!("    -> Found: '{}'", results[0].content.trim());
        }
    }
    
    // Test fuzzy searches with different edit distances
    println!("\n🔬 FUZZY SEARCH INVESTIGATION:");
    
    let fuzzy_tests = vec![
        // Single character errors
        ("authenticat", "authenticate", 1),      // missing 'e'
        ("calculat", "calculate", 1),            // missing 'e'  
        ("Databas", "Database", 1),              // missing 'e'
        ("proces_payment", "process_payment", 1), // missing 's'
        
        // Transposition errors
        ("autehnticate", "authenticate", 2),     // swapped 'th'
        ("calcualte", "calculate", 2),           // swapped 'ul'
        ("Databsae", "Database", 2),             // swapped 'sa'
        
        // Substitution errors
        ("authxnticate", "authenticate", 1),     // 'x' instead of 'e'
        ("calcylate", "calculate", 1),           // 'y' instead of 'u'
        ("Databasx", "Database", 1),             // 'x' instead of 'e'
    ];
    
    let mut fuzzy_working = 0;
    let fuzzy_total = fuzzy_tests.len();
    
    for (typo_term, correct_term, max_distance) in &fuzzy_tests {
        println!("\n  Testing: '{}' -> '{}' (max distance: {})", typo_term, correct_term, max_distance);
        
        let fuzzy_results = searcher.search_fuzzy(typo_term, *max_distance).await.expect("Fuzzy search failed");
        println!("    Fuzzy results: {}", fuzzy_results.len());
        
        // Check if we found the intended term
        let found_correct = fuzzy_results.iter().any(|r| {
            r.content.to_lowercase().contains(&correct_term.to_lowercase())
        });
        
        if found_correct {
            println!("    ✅ SUCCESS: Found '{}' with fuzzy search", correct_term);
            fuzzy_working += 1;
            // Show what was found
            for result in &fuzzy_results {
                if result.content.to_lowercase().contains(&correct_term.to_lowercase()) {
                    println!("      -> '{}'", result.content.trim());
                    break;
                }
            }
        } else {
            println!("    ❌ FAILURE: Did not find '{}' with fuzzy search", correct_term);
            if !fuzzy_results.is_empty() {
                println!("      Found instead:");
                for result in &fuzzy_results {
                    println!("        - '{}'", result.content.trim());
                }
            }
        }
    }
    
    // Test with higher edit distances
    println!("\n🔬 HIGH EDIT DISTANCE TESTS:");
    let high_distance_tests = vec![
        ("authnticat", "authenticate", 3),  // multiple errors
        ("calcualt", "calculate", 2),       // multiple errors
        ("Databse", "Database", 2),         // multiple errors
    ];
    
    for (typo_term, correct_term, max_distance) in &high_distance_tests {
        println!("\n  Testing: '{}' -> '{}' (max distance: {})", typo_term, correct_term, max_distance);
        
        let fuzzy_results = searcher.search_fuzzy(typo_term, *max_distance).await.expect("Fuzzy search failed");
        println!("    Results: {}", fuzzy_results.len());
        
        let found_correct = fuzzy_results.iter().any(|r| {
            r.content.to_lowercase().contains(&correct_term.to_lowercase())
        });
        
        if found_correct {
            println!("    ✅ Found with high edit distance");
        } else {
            println!("    ❌ Not found with high edit distance");
        }
    }
    
    // Test edge cases
    println!("\n🔬 EDGE CASE TESTS:");
    
    // Empty query
    let empty_results = searcher.search_fuzzy("", 1).await.expect("Empty fuzzy search failed");
    println!("  Empty query: {} results (should be 0)", empty_results.len());
    
    // Very short query
    let short_results = searcher.search_fuzzy("a", 1).await.expect("Short fuzzy search failed");
    println!("  Single character 'a': {} results", short_results.len());
    
    // Non-existent with fuzzy
    let nonexistent_results = searcher.search_fuzzy("xyznotfound", 2).await.expect("Non-existent fuzzy search failed");
    println!("  Non-existent term: {} results (should be 0)", nonexistent_results.len());
    
    // Summary
    println!("\n📊 FUZZY SEARCH ANALYSIS SUMMARY");
    println!("=================================");
    println!("Fuzzy search success rate: {}/{} ({:.1}%)", 
             fuzzy_working, fuzzy_total, 
             (fuzzy_working as f64 / fuzzy_total as f64) * 100.0);
    
    if fuzzy_working == 0 {
        println!("🚨 CRITICAL: Fuzzy search is completely non-functional");
    } else if fuzzy_working < fuzzy_total / 2 {
        println!("⚠️  WARNING: Fuzzy search has major accuracy issues");
    } else {
        println!("✅ Fuzzy search is working reasonably well");
    }
    
    // Test if the issue is with the fuzzy query construction
    println!("\n🔍 DEBUGGING: Let's try different approaches");
    
    // Try manual term construction for debugging
    println!("Attempting direct fuzzy term construction...");
    let results = searcher.search_fuzzy("authenticate_user", 0).await.expect("Search failed");
    println!("  Exact fuzzy match with distance 0: {} results", results.len());
    
    let results = searcher.search_fuzzy("authenticate_user", 1).await.expect("Search failed");
    println!("  Exact fuzzy match with distance 1: {} results", results.len());
    
    // This is a strong indicator of whether fuzzy search fundamentally works
    assert!(fuzzy_working > 0 || fuzzy_total == 0, 
            "Fuzzy search should work for at least some basic cases");
}

#[cfg(not(feature = "tantivy"))]
#[test]
fn test_fuzzy_feature_disabled() {
    println!("⚠️ Tantivy feature is not enabled - skipping fuzzy tests");
}