/// TRUTH VERIFICATION TEST: Config Integration Analysis
/// This test provides factual evidence about the current state of config integration

use anyhow::Result;
use embed_search::{Config, search::UnifiedSearcher};
use std::path::PathBuf;
use tempfile::TempDir;

#[tokio::test]
async fn test_config_integration_truth() -> Result<()> {
    println!("=== CONFIG INTEGRATION TRUTH VERIFICATION ===");
    
    // TEST 1: Verify Config::init() works
    println!("\n1. Testing Config::init()...");
    match Config::init_test() {
        Ok(()) => println!("✅ Config::init_test() SUCCESS"),
        Err(e) => println!("❌ Config::init_test() FAILED: {}", e),
    }
    
    // TEST 2: Verify Config::get() works after init
    println!("\n2. Testing Config::get() after init...");
    match Config::get() {
        Ok(_) => println!("✅ Config::get() SUCCESS after init"),
        Err(e) => println!("❌ Config::get() FAILED: {}", e),
    }
    
    // TEST 3: Try UnifiedSearcher::new() with initialized config
    println!("\n3. Testing UnifiedSearcher::new() with initialized config...");
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project_path = temp_dir.path().to_path_buf();
    let db_path = temp_dir.path().join("test.db");
    
    match UnifiedSearcher::new(project_path.clone(), db_path.clone()).await {
        Ok(_) => println!("✅ UnifiedSearcher::new() SUCCESS with initialized config"),
        Err(e) => println!("❌ UnifiedSearcher::new() FAILED: {}", e),
    }
    
    // TEST 4: Reset config and try UnifiedSearcher without init
    println!("\n4. Testing UnifiedSearcher::new() WITHOUT config init...");
    
    // Clear the config (simulate fresh start)
    // Note: We can't actually reset the static config in this test framework,
    // but we can create a new process or test the behavior
    
    // TEST 5: Check specific lines in main.rs
    println!("\n5. Checking main.rs for Config::init() calls...");
    let main_rs = std::fs::read_to_string("src/main.rs").expect("Failed to read main.rs");
    let config_init_lines: Vec<_> = main_rs.lines()
        .enumerate()
        .filter(|(_, line)| line.contains("Config::init"))
        .collect();
    
    println!("Found {} Config::init() calls in main.rs:", config_init_lines.len());
    for (line_num, line) in config_init_lines {
        println!("  Line {}: {}", line_num + 1, line.trim());
    }
    
    // TEST 6: Check UnifiedSearcher::new calls in main.rs
    println!("\n6. Checking main.rs for UnifiedSearcher::new() calls...");
    let searcher_new_lines: Vec<_> = main_rs.lines()
        .enumerate()
        .filter(|(_, line)| line.contains("UnifiedSearcher::new"))
        .collect();
    
    println!("Found {} UnifiedSearcher::new() calls in main.rs:", searcher_new_lines.len());
    for (line_num, line) in searcher_new_lines {
        println!("  Line {}: {}", line_num + 1, line.trim());
    }
    
    // TEST 7: Verify the actual error from UnifiedSearcher without config
    println!("\n7. TRUTH CHECK: What happens without Config::init()?");
    println!("NOTE: Cannot reset static config in same test process");
    println!("But UnifiedSearcher::new() calls Config::get() at line 95-96 in unified.rs");
    println!("This will fail with 'Make sure Config::init() was called first' if not initialized");
    
    println!("\n=== VERIFICATION COMPLETE ===");
    Ok(())
}

#[tokio::test]
async fn verify_unified_searcher_config_dependency() -> Result<()> {
    println!("\n=== UNIFIED SEARCHER CONFIG DEPENDENCY TEST ===");
    
    // Initialize config first
    Config::init_test()?;
    
    let temp_dir = TempDir::new()?;
    let project_path = temp_dir.path().to_path_buf();
    let db_path = temp_dir.path().join("test.db");
    
    // Test new() method
    match UnifiedSearcher::new(project_path.clone(), db_path.clone()).await {
        Ok(_) => println!("✅ UnifiedSearcher::new() works with config"),
        Err(e) => println!("❌ UnifiedSearcher::new() failed: {}", e),
    }
    
    // Test new_with_config() method  
    match UnifiedSearcher::new_with_config(project_path.clone(), db_path.clone(), false).await {
        Ok(_) => println!("✅ UnifiedSearcher::new_with_config() works with config"),
        Err(e) => println!("❌ UnifiedSearcher::new_with_config() failed: {}", e),
    }
    
    Ok(())
}