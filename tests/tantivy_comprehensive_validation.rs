/// Comprehensive validation of TantivySearcher functionality
/// This test provides a complete assessment of what works and what doesn't

use std::path::Path;
use tempfile::TempDir;
use std::fs;

#[cfg(feature = "tantivy")]
use embed_search::search::tantivy_search::TantivySearcher;

#[cfg(feature = "tantivy")]
#[tokio::test]
async fn test_tantivy_comprehensive_validation() {
    println!("🎯 COMPREHENSIVE TANTIVY SEARCH VALIDATION");
    println!("===========================================");
    
    // Create comprehensive test content
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    
    // Test files with different scenarios
    let test_files = vec![
        ("rust_code.rs", r#"
use std::collections::HashMap;

fn calculate_sum(a: i32, b: i32) -> i32 {
    a + b
}

fn authenticate_user(username: &str, password: &str) -> bool {
    verify_credentials(username, password)
}

struct DatabaseConnection {
    connection_string: String,
    timeout_seconds: u64,
}

impl DatabaseConnection {
    fn new(connection_string: String) -> Self {
        Self {
            connection_string,
            timeout_seconds: 30,
        }
    }
    
    fn execute_query(&self, sql: &str) -> Result<Vec<String>, String> {
        // Mock implementation
        Ok(vec!["result".to_string()])
    }
}

enum PaymentStatus {
    Pending,
    Completed,
    Failed,
}

trait Processable {
    fn process(&self) -> Result<(), String>;
}
"#),
        ("config.json", r#"{
    "database": {
        "host": "localhost",
        "port": 5432,
        "name": "app_database",
        "connection_pool_size": 20
    },
    "authentication": {
        "jwt_secret": "secret-key",
        "session_timeout": 3600
    },
    "logging": {
        "level": "info",
        "file_path": "/var/log/app.log"
    }
}
"#),
        ("documentation.md", r#"# API Documentation

## Authentication Endpoints

### POST /api/auth/login
Authenticate user credentials and return a JWT token.

### GET /api/auth/logout  
Logout the current user session.

## Database Operations

### GET /api/data/query
Execute a database query and return results.

### POST /api/data/insert
Insert new data into the database.

## Payment Processing

Handle payment transactions securely.

### POST /api/payments/process
Process a payment transaction.

### GET /api/payments/status
Check payment status.
"#),
    ];
    
    // Write test files
    for (filename, content) in &test_files {
        let file_path = temp_dir.path().join(filename);
        fs::write(&file_path, content).expect("Failed to write test file");
    }
    
    // Create and index
    let mut searcher = TantivySearcher::new().await.expect("Failed to create TantivySearcher");
    searcher.index_directory(temp_dir.path()).await.expect("Failed to index directory");
    println!("✅ Indexed {} test files", test_files.len());
    
    // Test Categories
    let mut test_results = TestResults::new();
    
    // 1. EXACT SEARCH TESTS
    println!("\n📋 1. EXACT SEARCH TESTS");
    println!("========================");
    
    let exact_tests = vec![
        ("calculate_sum", "Should find Rust function name"),
        ("authenticate_user", "Should find Rust function name"), 
        ("DatabaseConnection", "Should find Rust struct name"),
        ("execute_query", "Should find method name"),
        ("PaymentStatus", "Should find enum name"),
        ("Processable", "Should find trait name"),
        ("database", "Should find JSON key"),
        ("authentication", "Should find JSON key"),
        ("connection_pool_size", "Should find nested JSON key"),
        ("Authentication Endpoints", "Should find markdown header"),
        ("payment transactions", "Should find markdown text"),
        ("JWT token", "Should find technical terms"),
    ];
    
    for (query, _description) in &exact_tests {
        let results = searcher.search(query).await.expect("Search failed");
        let success = !results.is_empty();
        test_results.record("exact_search", success);
        
        println!("  '{}': {} matches - {}", 
                 query, results.len(), 
                 if success { "✅" } else { "❌" });
        
        if success {
            let sample = &results[0];
            let file_name = Path::new(&sample.file_path).file_name().unwrap().to_str().unwrap();
            println!("    Found in: {} (line {}): '{}'", 
                     file_name, sample.line_number, sample.content.trim());
        }
    }
    
    // 2. FUZZY SEARCH TESTS (LIMITED TO DISTANCE 1-2)
    println!("\n📋 2. FUZZY SEARCH TESTS");
    println!("========================");
    
    let fuzzy_tests = vec![
        // Single character errors (distance 1)
        ("calculat_sum", "calculate_sum", 1, "Missing character"),
        ("authenticat_user", "authenticate_user", 1, "Missing character"),
        ("Databse", "Database", 1, "Missing character"),
        ("execute_qery", "execute_query", 1, "Missing character"),
        
        // Substitution errors (distance 1)
        ("calculxte_sum", "calculate_sum", 1, "Character substitution"),
        ("authentixate_user", "authenticate_user", 1, "Character substitution"),
        ("DatabaseXonnection", "DatabaseConnection", 1, "Character substitution"),
        
        // Transposition errors (distance 2)
        ("calcualte_sum", "calculate_sum", 2, "Character transposition"),
        ("autehnticate_user", "authenticate_user", 2, "Character transposition"),
        ("DatabsaeConnection", "DatabaseConnection", 2, "Character transposition"),
    ];
    
    for (typo, correct, distance, error_type) in &fuzzy_tests {
        let results = searcher.search_fuzzy(typo, *distance).await.expect("Fuzzy search failed");
        let success = results.iter().any(|r| r.content.to_lowercase().contains(&correct.to_lowercase()));
        test_results.record("fuzzy_search", success);
        
        println!("  '{}' → '{}' ({}): {} matches - {}", 
                 typo, correct, error_type, results.len(),
                 if success { "✅" } else { "❌" });
        
        if success {
            for result in &results {
                if result.content.to_lowercase().contains(&correct.to_lowercase()) {
                    let file_name = Path::new(&result.file_path).file_name().unwrap().to_str().unwrap();
                    println!("    Found in: {} (line {}): '{}'", 
                             file_name, result.line_number, result.content.trim());
                    break;
                }
            }
        }
    }
    
    // 3. MULTI-FILE SEARCH TESTS
    println!("\n📋 3. MULTI-FILE SEARCH TESTS");
    println!("==============================");
    
    let multi_file_tests = vec![
        ("authentication", "Should find in multiple files"),
        ("database", "Should find in multiple files"), 
        ("process", "Should find in multiple files"),
        ("timeout", "Should find in multiple files"),
    ];
    
    for (query, _description) in &multi_file_tests {
        let results = searcher.search(query).await.expect("Search failed");
        let file_count = results.iter()
            .map(|r| Path::new(&r.file_path).file_name().unwrap().to_str().unwrap())
            .collect::<std::collections::HashSet<_>>()
            .len();
        
        let success = file_count >= 2;
        test_results.record("multi_file_search", success);
        
        println!("  '{}': {} matches across {} files - {}", 
                 query, results.len(), file_count,
                 if success { "✅" } else { "❌" });
        
        if success {
            let files: std::collections::HashSet<_> = results.iter()
                .map(|r| Path::new(&r.file_path).file_name().unwrap().to_str().unwrap())
                .collect();
            println!("    Files: {:?}", files);
        }
    }
    
    // 4. PERFORMANCE TESTS
    println!("\n📋 4. PERFORMANCE TESTS");
    println!("========================");
    
    let performance_queries = vec![
        "calculate", "auth", "database", "payment", "process", 
        "connection", "query", "status", "user", "api"
    ];
    
    let mut search_times = Vec::new();
    for query in &performance_queries {
        let start = std::time::Instant::now();
        let _results = searcher.search(query).await.expect("Search failed");
        let elapsed = start.elapsed();
        search_times.push(elapsed.as_millis());
    }
    
    let avg_time = search_times.iter().sum::<u128>() / search_times.len() as u128;
    let max_time = *search_times.iter().max().unwrap();
    
    let performance_good = avg_time < 50 && max_time < 100;
    test_results.record("performance", performance_good);
    
    println!("  Average search time: {}ms", avg_time);
    println!("  Maximum search time: {}ms", max_time);
    println!("  Performance: {}", if performance_good { "✅ Good" } else { "❌ Slow" });
    
    // 5. EDGE CASE TESTS
    println!("\n📋 5. EDGE CASE TESTS");
    println!("=====================");
    
    // Empty query
    let empty_results = searcher.search("").await.expect("Empty search failed");
    let empty_test = empty_results.is_empty();
    test_results.record("edge_cases", empty_test);
    println!("  Empty query: {} results - {}", 
             empty_results.len(), if empty_test { "✅" } else { "❌" });
    
    // Non-existent term
    let nonexistent_results = searcher.search("xyznotfoundanywhere123").await.expect("Search failed");
    let nonexistent_test = nonexistent_results.is_empty();
    test_results.record("edge_cases", nonexistent_test);
    println!("  Non-existent term: {} results - {}", 
             nonexistent_results.len(), if nonexistent_test { "✅" } else { "❌" });
    
    // Special characters
    let special_results = searcher.search("connection_string").await.expect("Search failed");
    let special_test = !special_results.is_empty();
    test_results.record("edge_cases", special_test);
    println!("  Underscore terms: {} results - {}", 
             special_results.len(), if special_test { "✅" } else { "❌" });
    
    // Case insensitive
    let lower_results = searcher.search("databaseconnection").await.expect("Search failed");
    let upper_results = searcher.search("DATABASECONNECTION").await.expect("Search failed");
    let case_test = !lower_results.is_empty() || !upper_results.is_empty();
    test_results.record("edge_cases", case_test);
    println!("  Case variations: lower={}, upper={} - {}", 
             lower_results.len(), upper_results.len(), if case_test { "✅" } else { "❌" });
    
    // 6. INDEX STATISTICS
    println!("\n📋 6. INDEX STATISTICS");
    println!("======================");
    
    let stats = searcher.get_index_stats().expect("Failed to get stats");
    println!("  {}", stats);
    
    let stats_test = stats.num_documents > 0;
    test_results.record("index_health", stats_test);
    
    // FINAL SUMMARY
    println!("\n🎯 COMPREHENSIVE VALIDATION SUMMARY");
    println!("====================================");
    
    test_results.print_summary();
    
    // Overall assessment
    let overall_score = test_results.overall_score();
    println!("\n📊 OVERALL TANTIVY SEARCH ASSESSMENT");
    println!("Overall Score: {:.1}%", overall_score);
    
    if overall_score >= 90.0 {
        println!("🎉 EXCELLENT: TantivySearcher is working very well");
    } else if overall_score >= 75.0 {
        println!("✅ GOOD: TantivySearcher is working well with minor issues");
    } else if overall_score >= 60.0 {
        println!("⚠️  ADEQUATE: TantivySearcher works but has significant issues");
    } else {
        println!("❌ POOR: TantivySearcher has major functionality problems");
    }
    
    // Specific recommendations
    println!("\n🔧 RECOMMENDATIONS:");
    if test_results.category_score("exact_search") < 90.0 {
        println!("  - Fix exact search accuracy issues");
    }
    if test_results.category_score("fuzzy_search") < 75.0 {
        println!("  - Improve fuzzy search, especially case sensitivity");
    }
    if test_results.category_score("multi_file_search") < 80.0 {
        println!("  - Verify cross-file search capabilities");
    }
    if test_results.category_score("performance") < 90.0 {
        println!("  - Optimize search performance");
    }
    if test_results.category_score("edge_cases") < 85.0 {
        println!("  - Handle edge cases more robustly");
    }
    
    println!("\n✅ Validation Complete - TantivySearcher assessment finished");
}

struct TestResults {
    categories: std::collections::HashMap<String, Vec<bool>>,
}

impl TestResults {
    fn new() -> Self {
        Self {
            categories: std::collections::HashMap::new(),
        }
    }
    
    fn record(&mut self, category: &str, success: bool) {
        self.categories.entry(category.to_string()).or_insert_with(Vec::new).push(success);
    }
    
    fn category_score(&self, category: &str) -> f64 {
        if let Some(results) = self.categories.get(category) {
            let passed = results.iter().filter(|&&x| x).count();
            (passed as f64 / results.len() as f64) * 100.0
        } else {
            0.0
        }
    }
    
    fn overall_score(&self) -> f64 {
        let mut total_tests = 0;
        let mut total_passed = 0;
        
        for results in self.categories.values() {
            total_tests += results.len();
            total_passed += results.iter().filter(|&&x| x).count();
        }
        
        if total_tests == 0 {
            0.0
        } else {
            (total_passed as f64 / total_tests as f64) * 100.0
        }
    }
    
    fn print_summary(&self) {
        for (category, results) in &self.categories {
            let passed = results.iter().filter(|&&x| x).count();
            let total = results.len();
            let score = (passed as f64 / total as f64) * 100.0;
            
            println!("  {}: {}/{} passed ({:.1}%)", category, passed, total, score);
        }
    }
}

#[cfg(not(feature = "tantivy"))]
#[test]
fn test_tantivy_comprehensive_disabled() {
    println!("⚠️ Tantivy feature is not enabled - skipping comprehensive validation");
}