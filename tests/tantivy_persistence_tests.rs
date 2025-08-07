#[cfg(feature = "tantivy")]
#[cfg(feature = "tantivy")]
use tempfile::TempDir;
#[cfg(feature = "tantivy")]
use std::fs;
#[cfg(feature = "tantivy")]

#[cfg(feature = "tantivy")]
use embed_search::search::tantivy_search::TantivySearcher;
#[cfg(feature = "tantivy")]

/// Test suite for Tantivy persistent storage functionality
/// 
/// These tests ensure that TantivySearcher:
/// 1. Persists index data to disk
/// 2. Can reload existing indexes on restart
/// 3. Handles index corruption gracefully
/// 4. Maintains fuzzy search functionality with persistent storage
/// 5. Supports incremental updates to existing indexes

#[cfg(feature = "tantivy")]
#[tokio::test]
async fn test_index_persists_after_restart() {
    let temp_dir = TempDir::new().unwrap();
    let index_path = temp_dir.path().join("tantivy_index");
    
    // Create test file
    let test_file = temp_dir.path().join("test.rs");
    fs::write(&test_file, r#"
pub fn authenticate_user(username: &str) -> bool {
    username == "admin"
}

pub fn configure_database() -> String {
    "connection_string".to_string()
}
"#).unwrap();

    // Phase 1: Create searcher and index file
    {
        let mut searcher = TantivySearcher::new_with_path(&index_path).await.unwrap();
        searcher.index_file(&test_file).await.unwrap();
        
        // Verify we can search
        let results = searcher.search("authenticate").await.unwrap();
        assert!(!results.is_empty(), "Should find 'authenticate' in indexed content");
        
        // Searcher goes out of scope here - simulating process restart
    }
    
    // Verify index files were created on disk
    assert!(index_path.exists(), "Index directory should exist on disk");
    assert!(index_path.is_dir(), "Index path should be a directory");
    
    // Phase 2: Create new searcher instance - should load existing index
    {
        let searcher = TantivySearcher::new_with_path(&index_path).await.unwrap();
        
        // Should be able to search without re-indexing
        let results = searcher.search("authenticate").await.unwrap();
        assert!(!results.is_empty(), "Should find 'authenticate' in persisted index");
        
        let results = searcher.search("configure").await.unwrap();
        assert!(!results.is_empty(), "Should find 'configure' in persisted index");
    }
}

#[cfg(feature = "tantivy")]
#[tokio::test]
async fn test_fuzzy_search_with_persistent_storage() {
    let temp_dir = TempDir::new().unwrap();
    let index_path = temp_dir.path().join("fuzzy_index");
    
    // Create test file with words that can be fuzzy matched
    let test_file = temp_dir.path().join("fuzzy_test.rs");
    fs::write(&test_file, r#"
pub fn authentication_service() -> bool { true }
pub fn authorization_handler() -> bool { true }
pub fn configuration_manager() -> String { "config".to_string() }
pub fn database_connection() -> String { "db".to_string() }
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
        let typo_tests = vec![
            ("authenticaton", "authentication"),  // Missing 'i'
            ("authorizaton", "authorization"),    // Missing 'i' 
            ("configuraton", "configuration"),    // Missing 'i'
            ("databse", "database"),              // Transposed letters
        ];
        
        for (typo, expected) in typo_tests {
            let fuzzy_results = searcher.search_fuzzy(typo, 2).await.unwrap();
            
            let found_expected = fuzzy_results.iter().any(|result| 
                result.content.to_lowercase().contains(expected) ||
                result.line_content.to_lowercase().contains(expected)
            );
            
            assert!(found_expected, 
                "Fuzzy search for '{}' should find '{}' in persisted index", 
                typo, expected);
        }
    }
}

#[cfg(feature = "tantivy")]
#[tokio::test]
async fn test_incremental_indexing() {
    let temp_dir = TempDir::new().unwrap();
    let index_path = temp_dir.path().join("incremental_index");
    
    // Phase 1: Index first file
    let file1 = temp_dir.path().join("file1.rs");
    fs::write(&file1, "pub fn function_one() {}").unwrap();
    
    {
        let mut searcher = TantivySearcher::new_with_path(&index_path).await.unwrap();
        searcher.index_file(&file1).await.unwrap();
        
        let results = searcher.search("function_one").await.unwrap();
        assert_eq!(results.len(), 1, "Should find function_one");
    }
    
    // Phase 2: Add second file to existing index
    let file2 = temp_dir.path().join("file2.rs");
    fs::write(&file2, "pub fn function_two() {}").unwrap();
    
    {
        let mut searcher = TantivySearcher::new_with_path(&index_path).await.unwrap();
        searcher.index_file(&file2).await.unwrap();
        
        // Should find both functions now
        let results1 = searcher.search("function_one").await.unwrap();
        let results2 = searcher.search("function_two").await.unwrap();
        
        assert!(!results1.is_empty(), "Should still find function_one");
        assert!(!results2.is_empty(), "Should now find function_two");
    }
    
    // Phase 3: Restart and verify both files are indexed
    {
        let searcher = TantivySearcher::new_with_path(&index_path).await.unwrap();
        
        let results1 = searcher.search("function_one").await.unwrap();
        let results2 = searcher.search("function_two").await.unwrap();
        
        assert!(!results1.is_empty(), "function_one should persist across restarts");
        assert!(!results2.is_empty(), "function_two should persist across restarts");
    }
}

#[cfg(feature = "tantivy")]
#[tokio::test]
async fn test_index_corruption_recovery() {
    let temp_dir = TempDir::new().unwrap();
    let index_path = temp_dir.path().join("corruption_test");
    
    // Create valid index first
    let test_file = temp_dir.path().join("test.rs");
    fs::write(&test_file, "pub fn test_function() {}").unwrap();
    
    {
        let mut searcher = TantivySearcher::new_with_path(&index_path).await.unwrap();
        searcher.index_file(&test_file).await.unwrap();
    }
    
    // Simulate corruption by writing garbage to index files
    for entry in fs::read_dir(&index_path).unwrap() {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_file() {
            fs::write(entry.path(), b"corrupted_data").unwrap();
        }
    }
    
    // Should gracefully handle corruption by rebuilding index
    {
        let result = TantivySearcher::new_with_path(&index_path).await;
        
        // Should either succeed by rebuilding or fail with clear error
        match result {
            Ok(mut searcher) => {
                // If it succeeded, it should have rebuilt the index
                // Re-index the file
                searcher.index_file(&test_file).await.unwrap();
                
                let results = searcher.search("test_function").await.unwrap();
                assert!(!results.is_empty(), "Should work after corruption recovery");
            }
            Err(e) => {
                // Should provide clear error message about corruption
                let error_msg = format!("{}", e);
                assert!(error_msg.contains("corrupt") || error_msg.contains("invalid") || error_msg.contains("rebuild"),
                    "Error should indicate corruption: {}", error_msg);
            }
        }
    }
}

#[cfg(feature = "tantivy")]
#[tokio::test]
async fn test_concurrent_access_safety() {
    let temp_dir = TempDir::new().unwrap();
    let index_path = temp_dir.path().join("concurrent_index");
    
    // Create test file
    let test_file = temp_dir.path().join("concurrent_test.rs");
    fs::write(&test_file, r#"
pub fn concurrent_function_alpha() {}
pub fn concurrent_function_beta() {}
pub fn concurrent_function_gamma() {}
"#).unwrap();

    // Test that multiple searchers can safely read from the same index
    {
        // First searcher indexes the file
        let mut searcher1 = TantivySearcher::new_with_path(&index_path).await.unwrap();
        searcher1.index_file(&test_file).await.unwrap();
    }
    
    // Create multiple readers
    let searcher1 = TantivySearcher::new_with_path(&index_path).await.unwrap();
    let searcher2 = TantivySearcher::new_with_path(&index_path).await.unwrap();
    let searcher3 = TantivySearcher::new_with_path(&index_path).await.unwrap();
    
    // Search concurrently
    let (results1, results2, results3) = tokio::join!(
        searcher1.search("concurrent_function"),
        searcher2.search("concurrent_function"),
        searcher3.search("concurrent_function")
    );
    
    let results1 = results1.unwrap();
    let results2 = results2.unwrap(); 
    let results3 = results3.unwrap();
    
    // All should find the same results
    assert!(!results1.is_empty(), "Searcher 1 should find results");
    assert!(!results2.is_empty(), "Searcher 2 should find results"); 
    assert!(!results3.is_empty(), "Searcher 3 should find results");
    
    // Results should be consistent
    assert_eq!(results1.len(), results2.len(), "Results should be consistent across searchers");
    assert_eq!(results2.len(), results3.len(), "Results should be consistent across searchers");
}

#[cfg(feature = "tantivy")]
#[tokio::test]
async fn test_large_index_performance() {
    let temp_dir = TempDir::new().unwrap();
    let index_path = temp_dir.path().join("large_index");
    
    // Create large test dataset
    let mut total_lines = 0;
    for i in 0..10 {
        let filename = format!("large_file_{}.rs", i);
        let file_path = temp_dir.path().join(&filename);
        
        let mut content = String::new();
        for j in 0..100 {
            content.push_str(&format!("pub fn function_{}_{j}() {{\n", i));
            content.push_str(&format!("    let var_{j} = \"test_string_{j}\";\n"));
            content.push_str(&format!("    process_data_{j}(var_{j});\n"));
            content.push_str("}\n\n");
            total_lines += 4;
        }
        
        fs::write(&file_path, content).unwrap();
    }
    
    println!("Created test dataset with {} lines across 10 files", total_lines);
    
    // Test indexing performance
    let start_time = std::time::Instant::now();
    {
        let mut searcher = TantivySearcher::new_with_path(&index_path).await.unwrap();
        
        for i in 0..10 {
            let filename = format!("large_file_{}.rs", i);
            let file_path = temp_dir.path().join(&filename);
            searcher.index_file(&file_path).await.unwrap();
        }
    }
    let indexing_time = start_time.elapsed();
    println!("Indexing time: {:?}", indexing_time);
    
    // Indexing should complete in reasonable time (less than 30 seconds)
    assert!(indexing_time.as_secs() < 30, "Indexing should complete within 30 seconds");
    
    // Test search performance after restart
    {
        let searcher = TantivySearcher::new_with_path(&index_path).await.unwrap();
        
        let search_start = std::time::Instant::now();
        let results = searcher.search("function").await.unwrap();
        let search_time = search_start.elapsed();
        
        println!("Search time: {:?}, Results found: {}", search_time, results.len());
        
        // Search should be fast (less than 100ms) and find many results
        assert!(search_time.as_millis() < 100, "Search should be fast (<100ms)");
        assert!(results.len() > 100, "Should find many function matches");
        
        // Test fuzzy search performance
        let fuzzy_start = std::time::Instant::now();
        let fuzzy_results = searcher.search_fuzzy("functoin", 2).await.unwrap();
        let fuzzy_time = fuzzy_start.elapsed();
        
        println!("Fuzzy search time: {:?}, Fuzzy results: {}", fuzzy_time, fuzzy_results.len());
        
        // Fuzzy search should still be reasonably fast
        assert!(fuzzy_time.as_millis() < 500, "Fuzzy search should be reasonably fast (<500ms)");
        assert!(!fuzzy_results.is_empty(), "Fuzzy search should find results");
    }
}

#[cfg(feature = "tantivy")]
#[tokio::test]
async fn test_empty_index_handling() {
    let temp_dir = TempDir::new().unwrap();
    let index_path = temp_dir.path().join("empty_index");
    
    // Create searcher with no content
    let searcher = TantivySearcher::new_with_path(&index_path).await.unwrap();
    
    // Should handle empty index gracefully
    let results = searcher.search("anything").await.unwrap();
    assert!(results.is_empty(), "Empty index should return no results");
    
    let fuzzy_results = searcher.search_fuzzy("anything", 2).await.unwrap();
    assert!(fuzzy_results.is_empty(), "Empty index should return no fuzzy results");
}

#[cfg(feature = "tantivy")]
#[tokio::test] 
async fn test_invalid_index_path_handling() {
    // Test with read-only path (should fail gracefully)
    let result = TantivySearcher::new_with_path("/read_only_path/that/does/not/exist").await;
    
    match result {
        Ok(_) => panic!("Should not succeed with invalid path"),
        Err(e) => {
            let error_msg = format!("{}", e);
            assert!(error_msg.contains("permission") || 
                   error_msg.contains("not found") || 
                   error_msg.contains("access"),
                   "Should provide meaningful error for invalid path: {}", error_msg);
        }
    }
}