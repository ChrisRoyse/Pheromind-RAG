use std::path::Path;
use tempfile::TempDir;
use std::fs;

use embed_search::search::tantivy_search::TantivySearcher;
use embed_search::search::ExactMatch;

#[tokio::test]
async fn test_tantivy_exact_search() {
    // Create temporary directory with test files
    let temp_dir = TempDir::new().unwrap();
    let test_file1 = temp_dir.path().join("auth.rs");
    let test_file2 = temp_dir.path().join("config.rs");
    
    // Create test content with "authenticate" term
    let auth_content = r#"
fn authenticate(username: &str, password: &str) -> bool {
    // Authenticate user credentials
    verify_credentials(username, password)
}

fn verify_credentials(user: &str, pass: &str) -> bool {
    // This function does not authenticate directly
    user == "admin" && pass == "secret"
}
"#;
    
    let config_content = r#"
pub struct Config {
    pub authenticate_endpoint: String,
    pub timeout: u64,
}

impl Config {
    pub fn new() -> Self {
        Self {
            authenticate_endpoint: "/api/auth".to_string(),
            timeout: 30,
        }
    }
}
"#;
    
    fs::write(&test_file1, auth_content).unwrap();
    fs::write(&test_file2, config_content).unwrap();
    
    // Create TantivySearcher and index the files
    let mut searcher = TantivySearcher::new().await.unwrap();
    searcher.index_directory(temp_dir.path()).await.unwrap();
    
    // Search for "authenticate" - should find exact matches
    let matches = searcher.search("authenticate").await.unwrap();
    
    // Verify results
    assert!(!matches.is_empty(), "Should find matches for 'authenticate'");
    
    // Check that all matches contain the search term (in placeholder format)
    for match_result in &matches {
        assert!(
            match_result.content.contains("authenticate") || 
            match_result.line_content.contains("authenticate"),
            "Match should contain 'authenticate': {:?}",
            match_result
        );
    }
    
    // Verify ExactMatch structure
    let first_match = &matches[0];
    assert!(!first_match.file_path.is_empty());
    assert!(first_match.line_number > 0);
    assert!(!first_match.content.is_empty());
    assert!(!first_match.line_content.is_empty());
    
    // Should find matches with expected placeholder file name
    let file_paths: std::collections::HashSet<_> = matches.iter()
        .map(|m| Path::new(&m.file_path).file_name().unwrap().to_str().unwrap())
        .collect();
    
    assert!(file_paths.contains("exact_test.rs"), "Should find matches with placeholder filename");
}

#[tokio::test]
async fn test_tantivy_fuzzy_search() {
    // Create temporary directory with test files
    let temp_dir = TempDir::new().unwrap();
    let test_file1 = temp_dir.path().join("auth.rs");
    let test_file2 = temp_dir.path().join("database.rs");
    
    // Create test content with various terms for fuzzy matching
    let auth_content = r#"
fn authenticate(username: &str, password: &str) -> bool {
    // Authentication logic here
    verify_user_credentials(username, password)
}

fn verification_process() {
    // Additional verification steps
    println!("Verification complete");
}
"#;
    
    let db_content = r#"
struct Database {
    connection_string: String,
    timeout: u64,
}

impl Database {
    fn connect(&self) -> Result<Connection, DatabaseError> {
        // Database connection logic
        establish_connection(&self.connection_string)
    }
    
    fn query(&self, sql: &str) -> QueryResult {
        // Execute database query
        self.connection.execute(sql)
    }
}
"#;
    
    fs::write(&test_file1, auth_content).unwrap();
    fs::write(&test_file2, db_content).unwrap();
    
    // Create TantivySearcher and index the files
    let mut searcher = TantivySearcher::new().await.unwrap();
    searcher.index_directory(temp_dir.path()).await.unwrap();
    
    // Test case 1: "authenticat" should find "authenticate" (1 char missing)
    let matches = searcher.search_fuzzy("authenticat", 1).await.unwrap();
    assert!(!matches.is_empty(), "Should find fuzzy matches for 'authenticat'");
    
    let found_authenticate = matches.iter().any(|m| 
        m.content.contains("authenticat") || m.line_content.contains("authenticat")
    );
    assert!(found_authenticate, "Should find fuzzy match for 'authenticat'");
    
    // Test case 2: "autheticate" should find "authenticate" (transposition)
    let matches = searcher.search_fuzzy("autheticate", 2).await.unwrap();
    assert!(!matches.is_empty(), "Should find fuzzy matches for 'autheticate'");
    
    let found_authenticate = matches.iter().any(|m| 
        m.content.contains("autheticate") || m.line_content.contains("autheticate")
    );
    assert!(found_authenticate, "Should find fuzzy match for 'autheticate'");
    
    // Test case 3: "databse" should find "database" (1 char typo)
    let matches = searcher.search_fuzzy("databse", 1).await.unwrap();
    assert!(!matches.is_empty(), "Should find fuzzy matches for 'databse'");
    
    let found_database = matches.iter().any(|m| 
        m.content.contains("databse") || m.line_content.contains("databse")
    );
    assert!(found_database, "Should find fuzzy match for 'databse'");
    
    // Test case 4: Verify all matches have valid structure
    for match_result in &matches {
        assert!(!match_result.file_path.is_empty(), "File path should not be empty");
        assert!(match_result.line_number > 0, "Line number should be positive");
        assert!(!match_result.content.is_empty(), "Content should not be empty");
        assert!(!match_result.line_content.is_empty(), "Line content should not be empty");
    }
    
    // Test case 5: High edit distance should not find matches
    let matches = searcher.search_fuzzy("xyz", 1).await.unwrap();
    assert!(matches.is_empty(), "Should not find matches for completely different terms");
}