use anyhow::Result;
use embed_search::search::{BM25Engine, BM25Document, BM25Token};

#[test]
fn test_bm25_basic_functionality() -> Result<()> {
    // Create BM25 engine with standard parameters
    let mut engine = BM25Engine::with_params(1.2, 0.75);
    
    // Create a test document
    let doc = BM25Document {
        id: "test_1".to_string(),
        file_path: "test.rs".to_string(),
        chunk_index: 0,
        tokens: vec![
            BM25Token {
                text: "function".to_string(),
                position: 0,
                importance_weight: 1.0,
            },
            BM25Token {
                text: "search".to_string(),
                position: 1,
                importance_weight: 1.0,
            },
            BM25Token {
                text: "algorithm".to_string(),
                position: 2,
                importance_weight: 1.0,
            },
        ],
        start_line: 1,
        end_line: 10,
        language: Some("rust".to_string()),
    };
    
    // Add document to engine
    engine.add_document(doc)?;
    
    // Test search for "function"
    let results = engine.search("function", 10)?;
    
    // Verify results
    assert!(!results.is_empty(), "BM25 should find results for 'function'");
    assert_eq!(results[0].doc_id, "test_1", "Should return the correct document");
    assert!(results[0].score > 0.0, "Score should be positive but got: {:.6}", results[0].score);
    
    println!("✅ BM25 FUNCTIONAL - Found {} results with score {:.6}", 
             results.len(), results[0].score);
    
    // Test search for "search" 
    let search_results = engine.search("search", 5)?;
    assert!(!search_results.is_empty(), "Should find 'search' term");
    
    // Test search for non-existent term
    let empty_results = engine.search("nonexistent", 5)?;
    assert!(empty_results.is_empty(), "Should return no results for nonexistent term");
    
    println!("✅ BM25 COMPLETE VERIFICATION PASSED");
    
    Ok(())
}

#[cfg(feature = "tantivy")]
#[tokio::test] 
async fn test_tantivy_basic_functionality() -> Result<()> {
    use embed_search::search::TantivySearcher;
    use tokio::fs;
    
    // Create temp directory and file
    let temp_dir = tempfile::tempdir()?;
    let test_file = temp_dir.path().join("test.rs");
    fs::write(&test_file, "fn calculate_value(x: i32) -> i32 { x * 2 }").await?;
    
    // Create Tantivy searcher
    let mut searcher = TantivySearcher::new().await?;
    
    // Index the test file
    searcher.index_file(&test_file).await?;
    
    // Test exact search
    let results = searcher.search("calculate_value").await?;
    assert!(!results.is_empty(), "Tantivy should find 'calculate_value'");
    
    println!("✅ TANTIVY FUNCTIONAL - Found {} exact results", results.len());
    
    // Test fuzzy search
    let fuzzy_results = searcher.search_fuzzy("calcualte_valu", 2).await?;
    assert!(!fuzzy_results.is_empty(), "Fuzzy search should work");
    
    println!("✅ TANTIVY FUZZY FUNCTIONAL - Found {} fuzzy results", fuzzy_results.len());
    
    Ok(())
}

#[test]
fn test_search_config_functionality() -> Result<()> {
    use embed_search::search::SearchConfig;
    use embed_search::config::Config;
    
    // Test SearchConfig (just feature flags)
    let search_config = SearchConfig::default();
    assert!(search_config.enable_bm25, "BM25 should be enabled by default");
    
    // Test main Config with BM25 parameters (using test config)
    let main_config = Config::new_test_config();
    assert_eq!(main_config.bm25_k1, 1.2);
    assert_eq!(main_config.bm25_b, 0.75);
    
    // Test custom SearchConfig with different features
    let custom_search_config = SearchConfig {
        enable_bm25: true,
        enable_tantivy: false,
        enable_ml: false,
        enable_tree_sitter: false,
        index_path: std::path::PathBuf::from("custom_index"),
    };
    
    assert!(custom_search_config.enable_bm25);
    assert!(!custom_search_config.enable_tantivy);
    
    println!("✅ SEARCH CONFIG FUNCTIONAL - Feature flags configurable");
    println!("✅ MAIN CONFIG FUNCTIONAL - BM25 parameters accessible: k1={}, b={}", main_config.bm25_k1, main_config.bm25_b);
    
    Ok(())
}