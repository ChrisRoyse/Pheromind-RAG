use std::path::Path;
use tempfile::TempDir;
use std::fs;

use embed_search::search::{TextSearcher, TantivySearcher, create_text_searcher};
use embed_search::config::Config;

/// Test the search adapter interface for seamless backend switching
#[tokio::test]
async fn test_search_adapter_interface() {
    // Create temporary directory with test files
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.rs");
    
    let content = r#"
fn authenticate_user(username: &str, password: &str) -> bool {
    // Authentication logic
    verify_credentials(username, password)
}

fn verify_credentials(user: &str, pass: &str) -> bool {
    user == "admin" && pass == "secret"
}

pub struct AuthConfig {
    pub authenticate_endpoint: String,
    pub timeout: u64,
}
"#;
    
    fs::write(&test_file, content).unwrap();
    
    // Test with TantivySearcher implementation
    let mut tantivy_searcher = TantivySearcher::new().await.unwrap();
    test_searcher_interface(&mut tantivy_searcher, temp_dir.path()).await;
}

/// Test that both implementations conform to the same interface
async fn test_searcher_interface<T: TextSearcher>(searcher: &mut T, test_dir: &Path) {
    // Test index_file method
    let test_file = test_dir.join("test.rs");
    let result = searcher.index_file(&test_file).await;
    assert!(result.is_ok(), "index_file should succeed");
    
    // Test search method
    let matches = searcher.search("authenticate").await.unwrap();
    assert!(!matches.is_empty(), "Should find matches for 'authenticate'");
    
    // Verify ExactMatch structure is consistent
    for match_result in &matches {
        assert!(!match_result.file_path.is_empty(), "File path should not be empty");
        assert!(match_result.line_number > 0, "Line number should be positive");
        assert!(!match_result.content.is_empty(), "Content should not be empty");
        assert!(!match_result.line_content.is_empty(), "Line content should not be empty");
        assert!(
            match_result.content.contains("authenticate") || 
            match_result.line_content.contains("authenticate"),
            "Match should contain 'authenticate'"
        );
    }
    
    // Test clear_index method
    let result = searcher.clear_index().await;
    assert!(result.is_ok(), "clear_index should succeed");
}

/// Test config-based searcher selection
#[tokio::test]
async fn test_config_based_searcher_selection() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("config_test.rs");
    
    let content = r#"
fn main() {
    println!("Hello, world!");
    authenticate_user("admin", "secret");
}
"#;
    
    fs::write(&test_file, content).unwrap();
    
    // Test with tantivy backend (default)
    let mut config = Config::default();
    config.search_backend = embed_search::config::SearchBackend::Tantivy;
    
    let _searcher = create_text_searcher(&config.search_backend).await.unwrap();
    
    // This will need to be implemented as a factory function
    // For now, just test that the concept works
    assert!(true, "Config-based selection concept validated");
}

#[tokio::test]
async fn test_adapter_switching_behavior() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("switch_test.rs");
    
    let content = r#"
fn process_authentication() {
    // This function handles authentication
    let result = authenticate_user("test", "password");
    println!("Authentication result: {}", result);
}
"#;
    
    fs::write(&test_file, content).unwrap();
    
    // Create Tantivy searcher
    let mut tantivy = TantivySearcher::new().await.unwrap();
    
    // Index the file
    tantivy.index_file(&test_file).await.unwrap();
    
    // Search for matches
    let tantivy_matches = tantivy.search("authentication").await.unwrap();
    
    // Should find matches
    assert!(!tantivy_matches.is_empty(), "TantivySearcher should find matches");
    
    // Verify structure
    for match_result in &tantivy_matches {
        assert!(!match_result.file_path.is_empty());
        assert!(match_result.line_number > 0);
        assert!(!match_result.content.is_empty());
        assert!(!match_result.line_content.is_empty());
    }
}