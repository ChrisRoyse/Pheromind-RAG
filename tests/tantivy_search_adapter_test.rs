use embed_search::config::SearchBackend;
use embed_search::search::search_adapter::{create_text_searcher, TextSearcher};
use tokio;
use tempfile::TempDir;
use std::fs;

#[cfg(feature = "tantivy")]
#[tokio::test]
async fn test_create_tantivy_text_searcher() {
    // Test that we can create a Tantivy text searcher
    let searcher_result = create_text_searcher(&SearchBackend::Tantivy).await;
    assert!(searcher_result.is_ok(), "Should be able to create Tantivy searcher");
    
    let mut searcher = searcher_result.unwrap();
    
    // Create a temporary test file
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.rs");
    fs::write(&test_file, "fn main() { println!(\"Hello, world!\"); }").unwrap();
    
    // Test that we can index a file
    let index_result = searcher.index_file(&test_file).await;
    assert!(index_result.is_ok(), "Should be able to index file");
    
    // Test that we can search (though results may be empty without proper indexing)
    let search_result = searcher.search("main").await;
    assert!(search_result.is_ok(), "Should be able to search");
}

#[cfg(feature = "tantivy")]
#[tokio::test] 
async fn test_search_backend_enum_coverage() {
    // Ensure we can handle all variants of SearchBackend
    match SearchBackend::Tantivy {
        SearchBackend::Tantivy => {
            // Test creation
            let result = create_text_searcher(&SearchBackend::Tantivy).await;
            assert!(result.is_ok(), "Tantivy backend should be supported");
        }
    }
}

#[test]
fn test_search_backend_only_tantivy() {
    // This test ensures at compile time that only Tantivy variant exists
    let backend = SearchBackend::Tantivy;
    
    match backend {
        SearchBackend::Tantivy => {
            // Good - only variant that should exist
            assert!(true);
        }
        // If any other variants exist, this won't compile
    }
}