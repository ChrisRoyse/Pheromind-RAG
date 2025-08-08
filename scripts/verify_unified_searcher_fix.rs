// Test script to verify UnifiedSearcher fix implementation
// This tests whether the claimed graceful degradation is actually working

use std::path::PathBuf;
use tempfile::TempDir;

// Test the UnifiedSearcher with different feature combinations
async fn test_unified_searcher() -> Result<(), Box<dyn std::error::Error>> {
    // Create temporary directories
    let temp_dir = TempDir::new()?;
    let project_path = temp_dir.path().to_path_buf();
    let db_path = project_path.join("db");
    
    println!("Testing UnifiedSearcher with different feature combinations...");
    
    // Test 1: Core features only (should work with BM25)
    println!("\n=== Test 1: Core features only ===");
    
    #[cfg(not(any(feature = "ml", feature = "vectordb", feature = "tantivy", feature = "tree-sitter")))]
    {
        use embed_search::search::unified::UnifiedSearcher;
        use embed_search::config::Config;
        
        // Initialize config first
        if Config::get().is_err() {
            Config::init_test()?;
        }
        
        match UnifiedSearcher::new(project_path.clone(), db_path.clone()).await {
            Ok(searcher) => {
                println!("✅ UnifiedSearcher created successfully with core features");
                
                // Test search
                match searcher.search("test query").await {
                    Ok(results) => {
                        println!("✅ Search executed successfully, returned {} results", results.len());
                    }
                    Err(e) => {
                        println!("❌ Search failed: {}", e);
                        return Err(e.into());
                    }
                }
            }
            Err(e) => {
                println!("❌ UnifiedSearcher creation failed with core features: {}", e);
                return Err(e.into());
            }
        }
    }
    
    #[cfg(any(feature = "ml", feature = "vectordb", feature = "tantivy", feature = "tree-sitter"))]
    {
        println!("⚠️ Skipped core-only test because optional features are enabled");
    }
    
    // Test 2: With tantivy
    println!("\n=== Test 2: With tantivy feature ===");
    
    #[cfg(feature = "tantivy")]
    {
        use embed_search::search::unified::UnifiedSearcher;
        use embed_search::config::{Config, SearchBackend};
        
        if Config::get().is_err() {
            Config::init_test()?;
        }
        
        match UnifiedSearcher::new_with_backend(project_path.clone(), db_path.clone(), SearchBackend::Tantivy).await {
            Ok(searcher) => {
                println!("✅ UnifiedSearcher created with tantivy backend");
                
                match searcher.search("test query").await {
                    Ok(results) => {
                        println!("✅ Search with tantivy executed successfully, returned {} results", results.len());
                    }
                    Err(e) => {
                        println!("❌ Search with tantivy failed: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("❌ UnifiedSearcher creation failed with tantivy: {}", e);
            }
        }
    }
    
    #[cfg(not(feature = "tantivy"))]
    {
        println!("⚠️ Tantivy test skipped - feature not enabled");
    }
    
    // Test 3: With tree-sitter
    println!("\n=== Test 3: With tree-sitter feature ===");
    
    #[cfg(feature = "tree-sitter")]
    {
        use embed_search::search::unified::UnifiedSearcher;
        use embed_search::config::Config;
        
        if Config::get().is_err() {
            Config::init_test()?;
        }
        
        match UnifiedSearcher::new(project_path.clone(), db_path.clone()).await {
            Ok(searcher) => {
                println!("✅ UnifiedSearcher created with tree-sitter");
                
                match searcher.search("function").await {
                    Ok(results) => {
                        println!("✅ Symbol search executed successfully, returned {} results", results.len());
                    }
                    Err(e) => {
                        println!("❌ Symbol search failed: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("❌ UnifiedSearcher creation failed with tree-sitter: {}", e);
            }
        }
    }
    
    #[cfg(not(feature = "tree-sitter"))]
    {
        println!("⚠️ Tree-sitter test skipped - feature not enabled");
    }
    
    // Test 4: With ML features (should fail gracefully if not working)
    println!("\n=== Test 4: With ML features ===");
    
    #[cfg(all(feature = "ml", feature = "vectordb"))]
    {
        use embed_search::search::unified::UnifiedSearcher;
        use embed_search::config::Config;
        
        if Config::get().is_err() {
            Config::init_test()?;
        }
        
        match UnifiedSearcher::new(project_path.clone(), db_path.clone()).await {
            Ok(searcher) => {
                println!("✅ UnifiedSearcher created with ML features");
                
                match searcher.search("semantic query").await {
                    Ok(results) => {
                        println!("✅ Semantic search executed successfully, returned {} results", results.len());
                    }
                    Err(e) => {
                        println!("⚠️ Semantic search failed (may be expected if embeddings not working): {}", e);
                    }
                }
            }
            Err(e) => {
                println!("❌ UnifiedSearcher creation failed with ML features: {}", e);
            }
        }
    }
    
    #[cfg(not(all(feature = "ml", feature = "vectordb")))]
    {
        println!("⚠️ ML test skipped - features not enabled");
    }
    
    println!("\n=== UnifiedSearcher Test Summary ===");
    println!("The test has completed. Review the results above to determine:");
    println!("1. Does UnifiedSearcher work with minimal features?");
    println!("2. Does it gracefully degrade when some features are disabled?");
    println!("3. Are there any compilation or runtime errors?");
    
    Ok(())
}

#[tokio::main]
async fn main() {
    match test_unified_searcher().await {
        Ok(_) => println!("✅ All tests completed"),
        Err(e) => {
            println!("❌ Test failed: {}", e);
            std::process::exit(1);
        }
    }
}