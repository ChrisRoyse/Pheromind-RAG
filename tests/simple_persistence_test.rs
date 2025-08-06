use std::path::Path;
use tempfile::TempDir;
use std::fs;

use embed_search::search::tantivy_search::TantivySearcher;

#[tokio::test]
async fn test_basic_persistent_storage() {
    let temp_dir = TempDir::new().unwrap();
    let index_path = temp_dir.path().join("test_index");
    
    // Create test file
    let test_file = temp_dir.path().join("test.rs");
    fs::write(&test_file, r#"
pub fn hello_world() {
    println!("Hello, world!");
}

pub fn goodbye_world() {
    println!("Goodbye, world!");
}
"#).unwrap();

    // Phase 1: Create searcher and index file
    {
        let mut searcher = TantivySearcher::new_with_path(&index_path).await.unwrap();
        searcher.index_file(&test_file).await.unwrap();
        
        // Verify we can search
        let results = searcher.search("hello").await.unwrap();
        assert!(!results.is_empty(), "Should find 'hello' in indexed content");
        
        // Verify index path is set correctly
        assert!(searcher.is_persistent());
        assert_eq!(searcher.index_path().unwrap(), index_path);
    }
    
    // Verify index files were created on disk
    assert!(index_path.exists(), "Index directory should exist on disk");
    assert!(index_path.is_dir(), "Index path should be a directory");
    
    // Phase 2: Create new searcher instance - should load existing index
    {
        let searcher = TantivySearcher::new_with_path(&index_path).await.unwrap();
        
        // Should be able to search without re-indexing
        let hello_results = searcher.search("hello").await.unwrap();
        assert!(!hello_results.is_empty(), "Should find 'hello' in persisted index");
        
        let goodbye_results = searcher.search("goodbye").await.unwrap();
        assert!(!goodbye_results.is_empty(), "Should find 'goodbye' in persisted index");
        
        // Test that we're actually reading from disk
        assert!(searcher.is_persistent());
        
        // Test index stats
        let stats = searcher.get_index_stats().unwrap();
        assert!(stats.num_documents > 0, "Should have indexed documents");
        assert!(stats.is_persistent, "Should be persistent storage");
        println!("Index stats: {}", stats);
    }
    
    println!("✅ Basic persistent storage test passed!");
}

#[tokio::test]
async fn test_fuzzy_search_persistence() {
    let temp_dir = TempDir::new().unwrap();
    let index_path = temp_dir.path().join("fuzzy_index");
    
    // Create test file
    let test_file = temp_dir.path().join("fuzzy.rs");
    fs::write(&test_file, r#"
pub fn authenticate_user() -> bool { true }
pub fn authorize_request() -> bool { true }
pub fn configure_system() -> String { "config".to_string() }
"#).unwrap();

    // Phase 1: Index content
    {
        let mut searcher = TantivySearcher::new_with_path(&index_path).await.unwrap();
        searcher.index_file(&test_file).await.unwrap();
    }
    
    // Phase 2: Test fuzzy search after restart
    {
        let searcher = TantivySearcher::new_with_path(&index_path).await.unwrap();
        
        // Test fuzzy search with typos
        let fuzzy_results = searcher.search_fuzzy("authenticat", 2).await.unwrap();
        assert!(!fuzzy_results.is_empty(), "Fuzzy search should find results for 'authenticat'");
        
        let found_authenticate = fuzzy_results.iter().any(|r| 
            r.content.to_lowercase().contains("authenticate") ||
            r.line_content.to_lowercase().contains("authenticate")
        );
        assert!(found_authenticate, "Fuzzy search should find 'authenticate'");
    }
    
    println!("✅ Fuzzy search persistence test passed!");
}