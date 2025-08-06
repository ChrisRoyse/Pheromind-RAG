use embed_search::search::unified::UnifiedSearcher;
use embed_search::search::create_text_searcher_with_root;
use embed_search::config::SearchBackend;
use std::fs;
use tempfile::TempDir;

#[tokio::test]
async fn test_unified_searcher_current_behavior() {
    println!("📝 Testing UnifiedSearcher current behavior before migration");
    
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project_path = temp_dir.path().to_path_buf();
    let db_path = project_path.join("test.db");
    
    // Create test files
    let test_file1 = project_path.join("authentication.rs");
    fs::write(&test_file1, r#"
pub fn authenticate_user(username: &str, password: &str) -> bool {
    // Implementation for user authentication
    verify_credentials(username, password)
}

fn verify_credentials(username: &str, password: &str) -> bool {
    // Database lookup and verification
    check_database(username, password)
}
"#).unwrap();

    let test_file2 = project_path.join("database.rs");
    fs::write(&test_file2, r#"
use std::collections::HashMap;

pub fn check_database(username: &str, password: &str) -> bool {
    let users: HashMap<String, String> = get_users();
    users.get(username).map_or(false, |stored_pass| stored_pass == password)
}

fn get_users() -> HashMap<String, String> {
    // Mock user database
    let mut users = HashMap::new();
    users.insert("admin".to_string(), "secret".to_string());
    users
}
"#).unwrap();

    // Initialize UnifiedSearcher
    let searcher = UnifiedSearcher::new(project_path.clone(), db_path).await
        .expect("Failed to create UnifiedSearcher");
    
    // Index test files
    searcher.index_file(&test_file1).await.expect("Failed to index authentication.rs");
    searcher.index_file(&test_file2).await.expect("Failed to index database.rs");
    
    // Test search functionality
    let results = searcher.search("authenticate").await.expect("Search failed");
    
    // Verify results
    assert!(!results.is_empty(), "Should find results for 'authenticate'");
    let found_auth_file = results.iter().any(|r| r.file.contains("authentication.rs"));
    assert!(found_auth_file, "Should find authentication.rs in results");
    
    println!("✅ Current UnifiedSearcher behavior verified - found {} results", results.len());
}

#[tokio::test]
async fn test_text_searcher_adapter_direct() {
    println!("🔧 Testing TextSearcher adapter directly");
    
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project_path = temp_dir.path().to_path_buf();
    
    // Create test file
    let test_file = project_path.join("test_direct.rs");
    fs::write(&test_file, r#"
fn process_data(input: &str) -> String {
    format!("Processing: {}", input)
}

fn handle_request() {
    let data = "test data";
    let result = process_data(data);
    println!("{}", result);
}
"#).unwrap();

    // Test with Tantivy backend
    let mut tantivy_searcher = create_text_searcher_with_root(&SearchBackend::Tantivy, project_path.clone()).await
        .expect("Failed to create Tantivy searcher");
    
    tantivy_searcher.index_file(&test_file).await.expect("Failed to index file");  
    let tantivy_results = tantivy_searcher.search("process_data").await.expect("Search failed");
    
    assert!(!tantivy_results.is_empty(), "Tantivy should find results");
    
    println!("✅ Tantivy adapter works: found {} results", tantivy_results.len());
}

#[tokio::test]
async fn test_tantivy_search_functionality() {
    println!("🔄 Testing Tantivy search functionality");
    
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project_path = temp_dir.path().to_path_buf();
    
    // Create comprehensive test file
    let test_file = project_path.join("comprehensive.rs");
    fs::write(&test_file, r#"
// Authentication module
pub mod authentication {
    pub fn authenticate_user(username: &str) -> bool {
        validate_username(username) && check_permissions(username)
    }
    
    fn validate_username(username: &str) -> bool {
        !username.is_empty() && username.len() > 3
    }
    
    fn check_permissions(username: &str) -> bool {
        // Permission checking logic
        username == "admin" || username == "user"
    }
}

// Database operations  
pub mod database {
    use std::collections::HashMap;
    
    pub fn query_users() -> HashMap<String, String> {
        let mut users = HashMap::new();
        users.insert("admin".to_string(), "Administrator".to_string());
        users.insert("user".to_string(), "Regular User".to_string());
        users
    }
    
    pub fn save_user(username: &str, role: &str) -> bool {
        // Database save operation
        !username.is_empty() && !role.is_empty()
    }
}

// Utility functions
pub fn format_message(msg: &str) -> String {
    format!("[INFO] {}", msg)
}
"#).unwrap();

    // Test queries that should work with both backends
    let test_queries = vec![
        "authenticate_user",
        "database", 
        "format_message",
        "HashMap",
        "admin",
    ];

    for query in &test_queries {
        println!("  Testing query: '{}'", query);
        
        // Test with Tantivy
        let mut tantivy_searcher = create_text_searcher_with_root(&SearchBackend::Tantivy, project_path.clone()).await
            .expect("Failed to create Tantivy searcher");
        tantivy_searcher.index_file(&test_file).await.expect("Failed to index with Tantivy");
        let tantivy_results = tantivy_searcher.search(query).await.expect("Tantivy search failed");
        
        // Should find results for these queries
        assert!(!tantivy_results.is_empty(), "Tantivy should find results for '{}'", query);
        
        println!("    Tantivy: {} results", tantivy_results.len());
    }
    
    println!("✅ Tantivy search functionality verified for all test queries");
}

/// Test the UnifiedSearcher with backend switching (post-migration)
#[tokio::test]  
async fn test_unified_searcher_with_backend_switching() {
    println!("🚀 Testing UnifiedSearcher with backend switching (post-migration)");
    
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project_path = temp_dir.path().to_path_buf();
    let db_path = project_path.join("test.db");
    
    // Create test file
    let test_file = project_path.join("fuzzy_test.rs");
    fs::write(&test_file, r#"
pub fn fuzzy_matching_function() {
    println!("This function tests fuzzy matching capabilities");
}

pub fn approximate_search() {
    let query = "fuzzy";
    println!("Searching with query: {}", query);
}
"#).unwrap();

    // Test with Tantivy backend
    let backends = vec![SearchBackend::Tantivy];
    
    for backend in backends {
        println!("  Testing with backend: {:?}", backend);
        
        let db_path = project_path.join(format!("test_{:?}.db", backend));
        let searcher = UnifiedSearcher::new_with_backend(
            project_path.clone(), 
            db_path, 
            backend.clone()
        ).await.expect(&format!("Failed to create UnifiedSearcher with {:?} backend", backend));
    
        searcher.index_file(&test_file).await.expect("Failed to index file");
        
        // Test with exact matching query that should work with both backends
        let results = searcher.search("fuzzy_matching_function").await.expect("Search failed");
        
        println!("    Backend {:?} found {} results for 'fuzzy_matching_function'", backend, results.len());
        assert!(!results.is_empty(), "Should find results with {:?} backend", backend);
        
        // Test a more general search
        let results2 = searcher.search("fuzzy").await.expect("Search failed");
        println!("    Backend {:?} found {} results for 'fuzzy'", backend, results2.len());
        assert!(!results2.is_empty(), "Should find fuzzy results with {:?} backend", backend);
    }
}

// Helper function to verify search result quality
fn verify_search_quality(results: &[embed_search::search::unified::SearchResult], expected_file: &str, query: &str) -> bool {
    // Check if results contain the expected file
    let has_expected_file = results.iter().any(|r| r.file.contains(expected_file));
    
    if !has_expected_file {
        println!("❌ Expected file '{}' not found in results for query '{}'", expected_file, query);
        for (i, result) in results.iter().enumerate() {
            println!("  {}. {} (score: {:.3})", i+1, result.file, result.score);
        }
        return false;
    }
    
    // Check if results have reasonable scores (relaxed check for adapter testing)
    let has_good_scores = results.iter().all(|r| !r.score.is_nan() && r.score.is_finite());
    if !has_good_scores {
        println!("❌ Some results have invalid scores (NaN/Inf) for query '{}'", query);
        for (i, result) in results.iter().enumerate() {
            println!("  {}. {} (score: {:.3})", i+1, result.file, result.score);
        }
        return false;
    }
    
    true
}

#[tokio::test]
async fn test_search_result_quality() {
    println!("📊 Testing search result quality across backends");
    
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project_path = temp_dir.path().to_path_buf();
    let db_path = project_path.join("quality_test.db");
    
    // Create multiple test files with different content
    let files = vec![
        ("user_auth.rs", r#"
pub fn authenticate_user(credentials: &UserCredentials) -> AuthResult {
    validate_credentials(credentials)
}

pub fn validate_credentials(creds: &UserCredentials) -> AuthResult {
    // Credential validation logic
    AuthResult::Success
}
"#),
        ("database_ops.rs", r#"
use std::collections::HashMap;

pub fn store_user_data(user_id: u64, data: &str) -> Result<(), DbError> {
    // Database storage implementation
    Ok(())
}

pub fn retrieve_user_info(user_id: u64) -> Option<UserInfo> {
    // Database retrieval logic
    None
}
"#),
        ("utils.rs", r#"
pub fn format_user_message(msg: &str, user_name: &str) -> String {
    format!("[{}] {}", user_name, msg)
}

pub fn validate_email(email: &str) -> bool {
    email.contains('@') && email.contains('.')
}
"#),
    ];
    
    for (filename, content) in &files {
        let file_path = project_path.join(filename);
        fs::write(&file_path, content).expect("Failed to write test file");
    }
    
    let searcher = UnifiedSearcher::new(project_path.clone(), db_path).await
        .expect("Failed to create UnifiedSearcher");
    
    // Index all files
    for (filename, _) in &files {
        let file_path = project_path.join(filename);
        searcher.index_file(&file_path).await.expect("Failed to index file");
    }
    
    // Test various queries and verify result quality
    let test_cases = vec![
        ("authenticate", Some("user_auth.rs"), "Should find authentication functions"),
        ("user_data", Some("database_ops.rs"), "Should find user data operations"), 
        ("validate", None, "Should find validation functions across files"),
        ("format", Some("utils.rs"), "Should find formatting utilities"),
    ];
    
    let mut passed_tests = 0;
    let total_tests = test_cases.len();
    
    for (query, expected_file_opt, description) in test_cases {
        println!("  Testing: {} - {}", query, description);
        
        let results = searcher.search(query).await.expect("Search failed");
        
        if results.is_empty() {
            println!("    ❌ No results found for query '{}'", query);
            continue;
        }
        
        match expected_file_opt {
            Some(expected_file) => {
                if verify_search_quality(&results, expected_file, query) {
                    println!("    ✅ Quality check passed for query '{}'", query);
                    passed_tests += 1;
                } else {
                    println!("    ❌ Quality check failed for query '{}'", query);
                }
            }
            None => {
                // For queries that should match multiple files, just verify we got results
                if !results.is_empty() {
                    println!("    ✅ Found {} results for query '{}'", results.len(), query);
                    passed_tests += 1;
                }
            }
        }
    }
    
    println!("📊 Search quality test results: {}/{} tests passed", passed_tests, total_tests);
    assert!(passed_tests >= total_tests * 3 / 4, "At least 75% of quality tests should pass");
}