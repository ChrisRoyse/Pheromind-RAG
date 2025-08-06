use embed_search::search::unified::UnifiedSearcher;
use std::path::PathBuf;

#[tokio::test]
async fn test_empty_query_returns_no_results() {
    let project_root = std::env::current_dir().unwrap();
    let db_path = project_root.join("test_empty_query_db");
    
    // Clean up any existing test database
    if db_path.exists() {
        std::fs::remove_dir_all(&db_path).ok();
    }
    
    // Create searcher
    let searcher = UnifiedSearcher::new_with_config(
        project_root.clone(),
        db_path,
        true // include test files
    ).await.expect("Failed to create searcher");
    
    // Test empty string
    let results = searcher.search("").await.unwrap();
    assert_eq!(results.len(), 0, "Empty query should return no results");
    
    // Test whitespace only
    let results = searcher.search("   ").await.unwrap();
    assert_eq!(results.len(), 0, "Whitespace-only query should return no results");
    
    // Test tabs and newlines
    let results = searcher.search("\t\n").await.unwrap();
    assert_eq!(results.len(), 0, "Tab/newline query should return no results");
    
    println!("✅ Empty query test passed!");
}