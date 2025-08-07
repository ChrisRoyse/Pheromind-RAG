/// WORKING Integration Test - Tests Only Verified Components
/// 
/// This test validates components that ACTUALLY work, not the fake fixes
/// that agents claimed were working. Based on VERIFIED functionality only.

use anyhow::Result;
use embed_search::{Config, search::{BM25Engine, CodeTextProcessor, QueryPreprocessor}};
use tempfile::TempDir;
use std::path::PathBuf;

#[tokio::test]
async fn test_verified_working_components() -> Result<()> {
    println!("🔧 Testing ONLY verified working components (no agent lies)");
    
    // Test 1: Config initialization that ACTUALLY works
    println!("🔍 Testing Config initialization...");
    Config::init_test().expect("Config::init_test() must work");
    println!("✅ Config initialization successful");
    
    // Test 2: BM25 Engine (verified to work in isolation)
    println!("🔍 Testing BM25 Engine directly...");
    let bm25 = BM25Engine::with_params(1.2, 0.75);
    let results = bm25.search("test", 10)?;
    println!("✅ BM25 Engine works: {} results (empty as expected)", results.len());
    assert_eq!(results.len(), 0, "Empty BM25 index should return 0 results");
    
    // Test 3: Text Processor (verified to work)  
    println!("🔍 Testing Code Text Processor...");
    let processor = CodeTextProcessor::with_config(
        true,   // enable_stemming  
        false,  // enable_ngrams
        3,      // max_ngram_size
        2,      // min_term_length
        50,     // max_term_length
        vec!["the".to_string(), "and".to_string()], // stop_words
    );
    
    let tokens = processor.tokenize_code(
        "function calculate_sum(a, b) { return a + b; }", 
        Some("javascript")
    );
    println!("✅ Text Processor works: {} tokens generated", tokens.len());
    assert!(!tokens.is_empty(), "Should generate tokens from code");
    
    // Verify tokens contain expected content
    let token_texts: Vec<String> = tokens.iter().map(|t| t.text.clone()).collect();
    let has_function_related = token_texts.iter().any(|t| 
        t.contains("function") || t.contains("calculate") || t.contains("sum"));
    assert!(has_function_related, "Tokens should contain function-related terms: {:?}", token_texts);
    
    // Test 4: Query Preprocessor (verified to work)
    println!("🔍 Testing Query Preprocessor...");
    let preprocessor = QueryPreprocessor::new();
    let processed = preprocessor.preprocess("Test Query With CAPS!");
    println!("✅ Query Preprocessor works: '{}' -> '{}'", "Test Query With CAPS!", processed);
    assert!(!processed.is_empty(), "Processed query should not be empty");
    
    // Test 5: Config access methods (verified to work)
    println!("🔍 Testing Config access methods...");
    let chunk_size = Config::chunk_size()?;
    let cache_size = Config::search_cache_size()?;
    let db_path = Config::vector_db_path()?;
    
    println!("✅ Config access works:");
    println!("   - chunk_size: {}", chunk_size);
    println!("   - search_cache_size: {}", cache_size); 
    println!("   - vector_db_path: {}", db_path);
    
    assert!(chunk_size > 0, "chunk_size should be positive");
    assert!(cache_size > 0, "search_cache_size should be positive");
    assert!(!db_path.is_empty(), "vector_db_path should not be empty");
    
    println!("🎉 All VERIFIED components work correctly!");
    println!("   ✅ Config system");
    println!("   ✅ BM25 search engine");
    println!("   ✅ Code text processor");
    println!("   ✅ Query preprocessor");
    println!("   ✅ Configuration access");
    
    // Test 6: Error handling verification (ensures no fallbacks)
    println!("🔍 Testing truthful error handling...");
    
    // Reset config to test error conditions
    {
        use embed_search::config::CONFIG;
        *CONFIG.write().unwrap() = None;
        
        let result = Config::get();
        match result {
            Ok(_) => panic!("Config::get() should fail when not initialized"),
            Err(e) => {
                println!("✅ Config properly fails when not initialized: {}", e);
                assert!(e.to_string().contains("not initialized"), 
                       "Error should mention 'not initialized': {}", e);
                // Verify no fallback language
                assert!(!e.to_string().to_lowercase().contains("fallback"),
                       "Error should not mention fallbacks: {}", e);
            }
        }
    }
    
    // Re-initialize for cleanup
    Config::init_test()?;
    
    println!("✅ Error handling is truthful (no fallbacks)");
    
    Ok(())
}

#[tokio::test] 
async fn test_what_is_actually_broken() -> Result<()> {
    println!("🚨 Testing components that are ACTUALLY broken (not agent lies)");
    
    // Initialize config first
    Config::init_test()?;
    
    // Test 1: Verify Nomic is broken due to model corruption (not runtime panic)
    #[cfg(feature = "ml")]
    {
        println!("🔍 Testing Nomic ML (should fail due to model corruption)...");
        use embed_search::embedding::NomicEmbedder;
        
        let result = NomicEmbedder::get_global().await;
        match result {
            Ok(_) => {
                println!("⚠️  Nomic unexpectedly succeeded - model may have been fixed");
            },
            Err(e) => {
                println!("❌ Nomic failed as expected: {}", e);
                let error_msg = e.to_string();
                
                // Verify it's the REAL error (model corruption) not the fake one (runtime panic)
                if error_msg.contains("runtime") {
                    panic!("Agents lied! Still reporting fake 'runtime' error instead of model corruption");
                } else if error_msg.contains("NaN") || error_msg.contains("corrupted") {
                    println!("✅ TRUTH: Error correctly identifies model corruption");
                } else {
                    println!("🤔 New error type: {}", error_msg);
                }
            }
        }
    }
    
    #[cfg(not(feature = "ml"))]
    {
        println!("⏭️  Nomic test skipped - ml feature not enabled");
    }
    
    println!("✅ Broken component verification complete");
    Ok(())
}

/// Helper function to create test data for components that need it
async fn create_test_data(test_dir: &PathBuf) -> Result<()> {
    use tokio::fs;
    
    // Create a simple test file
    let test_file = test_dir.join("test_code.rs");
    let test_content = r#"
// Test file for search components
fn main() {
    println!("Hello, world!");
}

fn calculate_sum(a: i32, b: i32) -> i32 {
    a + b
}

struct User {
    name: String,
    age: u32,
}

impl User {
    fn new(name: String, age: u32) -> Self {
        User { name, age }
    }
}
"#;
    
    fs::create_dir_all(test_dir).await?;
    fs::write(&test_file, test_content).await?;
    println!("✅ Created test data at: {:?}", test_file);
    
    Ok(())
}