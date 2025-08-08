// Test for UnifiedSearcher graceful degradation with partial features
use embed_search::search::unified::UnifiedSearcher;
use embed_search::config::Config;
use std::path::PathBuf;

#[tokio::test]
async fn test_unified_search_with_partial_features() {
    // Initialize config first
    if Config::init().is_err() {
        // Config might already be initialized, continue
    }
    
    let project_path = PathBuf::from(".");
    let db_path = PathBuf::from("./test_db");
    
    // This should not fail regardless of which features are enabled
    let searcher_result = UnifiedSearcher::new(project_path, db_path).await;
    
    match searcher_result {
        Ok(searcher) => {
            println!("✅ UnifiedSearcher created successfully with available features");
            
            // Test search - this should work regardless of available features
            let search_result = searcher.search("test query").await;
            
            match search_result {
                Ok(results) => {
                    println!("✅ Search completed successfully with {} results", results.len());
                    // Even with no results, this is success if it doesn't crash
                }
                Err(e) => {
                    println!("❌ Search failed: {}", e);
                    panic!("Search should not fail due to missing features");
                }
            }
        }
        Err(e) => {
            println!("❌ UnifiedSearcher creation failed: {}", e);
            // This is only acceptable if BM25 (the fallback) is completely broken
            panic!("UnifiedSearcher should work with at least BM25 as fallback");
        }
    }
}

#[tokio::test]
async fn test_unified_search_with_no_optional_features() {
    // This test verifies that UnifiedSearcher works with just the basic BM25 engine
    // even when tantivy, vectordb, and tree-sitter features are all disabled
    
    if Config::init().is_err() {
        // Config might already be initialized, continue
    }
    
    let project_path = PathBuf::from(".");
    let db_path = PathBuf::from("./test_db_basic");
    
    let searcher = UnifiedSearcher::new(project_path, db_path).await
        .expect("UnifiedSearcher should work with BM25 fallback");
    
    let results = searcher.search("function").await
        .expect("Search should work even with only BM25");
    
    println!("✅ Basic search returned {} results", results.len());
    // Success means no panics and graceful degradation
}

#[cfg(feature = "tantivy")]
#[tokio::test]
async fn test_unified_search_with_tantivy_only() {
    // Test that UnifiedSearcher works when only tantivy is enabled
    
    if Config::init().is_err() {
        // Config might already be initialized, continue
    }
    
    let project_path = PathBuf::from(".");
    let db_path = PathBuf::from("./test_db_tantivy");
    
    let searcher = UnifiedSearcher::new(project_path, db_path).await
        .expect("UnifiedSearcher should work with tantivy + BM25");
    
    let results = searcher.search("struct").await
        .expect("Search should work with tantivy + BM25");
    
    println!("✅ Tantivy+BM25 search returned {} results", results.len());
}

#[test]
fn test_compilation_with_partial_features() {
    // This test just needs to compile to prove the conditional compilation works
    println!("✅ Code compiles successfully with current feature set");
    
    #[cfg(feature = "tantivy")]
    println!("  - tantivy feature enabled");
    
    #[cfg(feature = "vectordb")]
    println!("  - vectordb feature enabled");
    
    #[cfg(feature = "tree-sitter")]  
    println!("  - tree-sitter feature enabled");
    
    #[cfg(feature = "ml")]
    println!("  - ml feature enabled");
    
    // If none are enabled, BM25 should still be available
    #[cfg(not(any(feature = "tantivy", feature = "vectordb", feature = "tree-sitter", feature = "ml")))]
    println!("  - only BM25 baseline available (no optional features)");
}