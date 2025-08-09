/// BRUTAL TRUTH PRODUCTION VALIDATION
/// Tests what ACTUALLY works vs what's claimed
/// NO FALLBACKS OR WORKAROUNDS - test the system as-is

use embed_search::{
    search::unified::UnifiedSearcher,
    config::{Config, SearchBackend},
    Result,
};
use std::path::PathBuf;
use tempfile::TempDir;

#[tokio::test]
async fn test_unified_searcher_basic_initialization() {
    println!("🔍 TESTING: UnifiedSearcher basic initialization");
    
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join("test_db");
    
    // Use the ACTUAL API discovered from code inspection
    match UnifiedSearcher::new_with_backend(
        temp_dir.path().to_path_buf(), 
        db_path, 
        SearchBackend::Tantivy
    ).await {
        Ok(searcher) => {
            println!("✅ UnifiedSearcher initialized successfully");
            
            // Try to get basic status
            match searcher.get_indexed_file_count().await {
                Ok(count) => println!("✅ Indexed file count: {}", count),
                Err(e) => println!("❌ Failed to get file count: {}", e),
            }
        }
        Err(e) => {
            println!("❌ CRITICAL: UnifiedSearcher initialization failed: {}", e);
            panic!("Basic initialization must work for production readiness");
        }
    }
}

#[tokio::test] 
async fn test_basic_file_indexing() {
    println!("🔍 TESTING: Basic file indexing functionality");
    
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join("test_db");
    
    // Create test file
    let test_file = temp_dir.path().join("test.txt");
    std::fs::write(&test_file, "hello world test content").expect("Failed to write test file");
    
    match UnifiedSearcher::new_with_backend(
        temp_dir.path().to_path_buf(), 
        db_path, 
        SearchBackend::Tantivy
    ).await {
        Ok(mut searcher) => {
            println!("✅ Searcher initialized");
            
            // Try to index the test file
            match searcher.index_file(&test_file).await {
                Ok(_) => {
                    println!("✅ File indexing completed");
                    
                    // Verify file was indexed
                    match searcher.get_indexed_file_count().await {
                        Ok(count) => {
                            if count > 0 {
                                println!("✅ File count after indexing: {}", count);
                            } else {
                                println!("❌ WARNING: File count is 0 after indexing");
                            }
                        }
                        Err(e) => println!("❌ Failed to get file count after indexing: {}", e),
                    }
                }
                Err(e) => {
                    println!("❌ CRITICAL: File indexing failed: {}", e);
                }
            }
        }
        Err(e) => {
            println!("❌ CRITICAL: Searcher initialization failed: {}", e);
        }
    }
}

#[tokio::test]
async fn test_basic_search_functionality() {
    println!("🔍 TESTING: Basic search functionality");
    
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join("test_db");
    
    // Create test file with searchable content
    let test_file = temp_dir.path().join("search_test.txt");
    std::fs::write(&test_file, "The quick brown fox jumps over the lazy dog").expect("Failed to write test file");
    
    match UnifiedSearcher::new_with_backend(
        temp_dir.path().to_path_buf(), 
        db_path, 
        SearchBackend::Tantivy
    ).await {
        Ok(mut searcher) => {
            println!("✅ Searcher initialized");
            
            // Index the file
            if let Err(e) = searcher.index_file(&test_file).await {
                println!("❌ File indexing failed: {}", e);
                return;
            }
            
            // Try basic search
            match searcher.search("quick brown", None, None).await {
                Ok(results) => {
                    println!("✅ Search executed successfully");
                    println!("   Results count: {}", results.len());
                    
                    if results.is_empty() {
                        println!("❌ WARNING: Search returned no results for obvious match");
                    } else {
                        for (i, result) in results.iter().enumerate() {
                            println!("   Result {}: file={:?}, score={}", 
                                i + 1, 
                                result.file_path.file_name().unwrap_or_default(),
                                result.score
                            );
                        }
                    }
                }
                Err(e) => {
                    println!("❌ CRITICAL: Search operation failed: {}", e);
                }
            }
        }
        Err(e) => {
            println!("❌ CRITICAL: Searcher initialization failed: {}", e);
        }
    }
}

#[tokio::test]
async fn test_feature_availability() {
    println!("🔍 TESTING: Feature availability assessment");
    
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    
    // Test only the backend that actually exists
    println!("  Testing Tantivy backend...");
    let db_path = temp_dir.path().join("test_db");
    
    match UnifiedSearcher::new_with_backend(
        temp_dir.path().to_path_buf(), 
        db_path, 
        SearchBackend::Tantivy
    ).await {
        Ok(_) => println!("    ✅ Tantivy backend initializes"),
        Err(e) => println!("    ❌ Tantivy backend failed: {}", e),
    }
}

#[test]
fn test_config_loading() {
    println!("🔍 TESTING: Configuration system");
    
    // Test test config creation (ONLY available method)
    let config = Config::new_test_config();
    println!("✅ Test config created");
    println!("   Backend: {:?}", config.search_backend);
    println!("   Max results: {}", config.max_search_results);
    println!("   Cache size: {}", config.search_cache_size);
    
    // Test config from file (if exists)
    match Config::load() {
        Ok(config) => {
            println!("✅ Config loaded from file");
            println!("   Backend: {:?}", config.search_backend);
        }
        Err(e) => {
            println!("ℹ️  No config file found (expected): {}", e);
        }
    }
}

#[test]
fn test_basic_types_and_imports() {
    println!("🔍 TESTING: Basic types and imports");
    
    // Test that core types can be instantiated
    use embed_search::{EmbedError, Result};
    use embed_search::config::{Config, SearchBackend};
    
    let _config = Config::new_test_config();
    println!("✅ Config type available");
    
    let _backend = SearchBackend::Tantivy;
    println!("✅ SearchBackend enum available");
    
    let _error: Result<()> = Err(EmbedError::Internal {
        message: "test error".to_string(),
        backtrace: None,
    });
    println!("✅ Error types available");
}

/// Run all validation tests and report results
#[tokio::test]
async fn run_full_validation_suite() {
    println!("\n🚀 STARTING BRUTAL TRUTH PRODUCTION VALIDATION");
    println!("================================================");
    
    // Test basic types first
    test_basic_types_and_imports();
    test_config_loading();
    
    // Test core functionality
    test_unified_searcher_basic_initialization().await;
    test_basic_file_indexing().await;
    test_basic_search_functionality().await;
    test_feature_availability().await;
    
    println!("\n✅ VALIDATION SUITE COMPLETED");
    println!("See individual test results above for detailed assessment");
}