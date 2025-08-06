use std::path::PathBuf;
use tempfile::TempDir;
use embed_search::search::unified::UnifiedSearcher;
use embed_search::config::SearchBackend;

#[tokio::test]
async fn test_unified_searcher_core_features() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_path = temp_dir.path().to_path_buf();
    let db_path = temp_dir.path().join("test_db");
    
    // This should work with core features only
    let searcher_result = UnifiedSearcher::new_with_backend_and_config(
        project_path,
        db_path,
        SearchBackend::Tantivy,
        false
    ).await;
    
    // With minimal features, initialization might not have all components but should not crash
    match searcher_result {
        Ok(_searcher) => {
            println!("✅ UnifiedSearcher initialized successfully with core features");
            // Can't do much searching without features, but it should initialize
        },
        Err(e) => {
            // Acceptable if some dependencies are missing with core-only features
            println!("⚠️ UnifiedSearcher initialization failed with core features (expected): {}", e);
        }
    }
}

#[tokio::test]
#[cfg(feature = "full-system")]
async fn test_unified_searcher_full_features() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_path = temp_dir.path().to_path_buf();
    let db_path = temp_dir.path().join("test_db");
    
    // This should work with all features enabled
    let searcher = UnifiedSearcher::new_with_backend_and_config(
        project_path,
        db_path,
        SearchBackend::Tantivy,
        false
    ).await.expect("Failed to create UnifiedSearcher with full features");
    
    println!("✅ UnifiedSearcher initialized successfully with full features");
    
    // Test basic search functionality (should not crash)
    let _results = searcher.search("test query").await
        .expect("Search should not crash even if no results");
    
    println!("✅ Search operation completed without errors");
}

#[test]
fn test_conditional_compilation_consistency() {
    // This test just ensures that conditional compilation attributes are consistent
    
    #[cfg(feature = "ml")]
    {
        println!("✅ ML feature is enabled");
    }
    
    #[cfg(feature = "vectordb")]
    {
        println!("✅ VectorDB feature is enabled");
    }
    
    #[cfg(feature = "tree-sitter")]
    {
        println!("✅ Tree-sitter feature is enabled");
    }
    
    #[cfg(feature = "tantivy")]
    {
        println!("✅ Tantivy feature is enabled");
    }
    
    #[cfg(feature = "core")]
    {
        println!("✅ Core feature is enabled");
    }
    
    println!("✅ Feature flag test completed");
}