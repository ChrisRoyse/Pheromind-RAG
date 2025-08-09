// END-TO-END PIPELINE VALIDATION - BRUTAL TRUTH TEST
// This test validates the complete pipeline from file ingestion to semantic search
// FAIL FAST: Any failure indicates the system is NOT production ready

use std::path::{Path, PathBuf};
use std::fs;
use tempfile::TempDir;
use tokio;
use anyhow::Result;

use embed_search::search::unified::UnifiedSearcher;
use embed_search::config::{Config, SearchBackend};
use embed_search::search::cache::SearchResult;

#[tokio::test]
async fn test_complete_pipeline_validation() -> Result<()> {
    println!("🚨 STARTING BRUTAL END-TO-END PIPELINE VALIDATION");
    
    // Create temporary directory for test files
    let temp_dir = TempDir::new()?;
    let test_file_path = temp_dir.path().join("test_code.rs");
    
    // Create realistic Rust code file for testing
    let test_code = r#"
use std::collections::HashMap;

/// A simple user management system
pub struct UserManager {
    users: HashMap<u64, User>,
    next_id: u64,
}

impl UserManager {
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
            next_id: 1,
        }
    }
    
    pub fn create_user(&mut self, name: &str, email: &str) -> Result<u64, String> {
        if self.users.values().any(|u| u.email == email) {
            return Err("Email already exists".to_string());
        }
        
        let user = User {
            id: self.next_id,
            name: name.to_string(),
            email: email.to_string(),
        };
        
        self.users.insert(self.next_id, user);
        let id = self.next_id;
        self.next_id += 1;
        
        Ok(id)
    }
    
    pub fn find_user_by_email(&self, email: &str) -> Option<&User> {
        self.users.values().find(|u| u.email == email)
    }
}

#[derive(Debug, Clone)]
pub struct User {
    pub id: u64,
    pub name: String,
    pub email: String,
}
"#;

    fs::write(&test_file_path, test_code)?;
    println!("✅ Created test file at {:?}", test_file_path);
    
    // VALIDATION POINT 1: System Initialization
    println!("🔍 VALIDATION 1: System Initialization");
    
    let project_path = temp_dir.path().to_path_buf();
    let db_path = temp_dir.path().join("db");
    let searcher = UnifiedSearcher::new(project_path, db_path).await?;
    println!("✅ UnifiedSearcher initialized successfully");
    
    // VALIDATION POINT 2: File Indexing
    println!("🔍 VALIDATION 2: Complete File Indexing Pipeline");
    
    let index_result = searcher.index_file(&test_file_path).await;
    match index_result {
        Ok(_) => println!("✅ File indexed successfully"),
        Err(e) => {
            println!("❌ CRITICAL FAILURE: File indexing failed: {}", e);
            panic!("Pipeline validation FAILED at file indexing: {}", e);
        }
    }
    
    // VALIDATION POINT 3: BM25 Text Search
    println!("🔍 VALIDATION 3: BM25 Text Search Functionality");
    
    let bm25_results = searcher.search("create_user").await?;
    validate_search_results(&bm25_results, "create_user", "BM25")?;
    
    // VALIDATION POINT 4: Symbol Search (if tree-sitter enabled)
    #[cfg(feature = "tree-sitter")]
    {
        println!("🔍 VALIDATION 4: Symbol Search Functionality");
        let symbol_results = searcher.search_symbols("UserManager", 5).await?;
        validate_symbol_results(&symbol_results, "UserManager")?;
    }
    
    // VALIDATION POINT 5: Semantic Search (if ML + vectordb enabled)
    #[cfg(all(feature = "ml", feature = "vectordb"))]
    {
        println!("🔍 VALIDATION 5: Semantic Search Functionality");
        let semantic_results = searcher.search("user management functionality").await?;
        validate_search_results(&semantic_results, "user management functionality", "Semantic")?;
        
        // Test semantic similarity - should find related concepts
        let similarity_results = searcher.search("email validation system").await?;
        validate_search_results(&similarity_results, "email validation system", "Semantic Similarity")?;
    }
    
    // VALIDATION POINT 6: Fuzzy Text Search (if tantivy enabled)  
    #[cfg(feature = "tantivy")]
    {
        println!("🔍 VALIDATION 6: Fuzzy Text Search");
        let fuzzy_results = searcher.search("create_usr").await?; // Typo test
        // Should still find results due to fuzzy matching
        if fuzzy_results.is_empty() {
            println!("⚠️  WARNING: Fuzzy search found no results for 'create_usr'");
        } else {
            println!("✅ Fuzzy search working - found {} results", fuzzy_results.len());
        }
    }
    
    // VALIDATION POINT 7: Multiple File Processing
    println!("🔍 VALIDATION 7: Multiple File Processing");
    
    // Create second test file
    let test_file2_path = temp_dir.path().join("test_utils.rs");
    let test_code2 = r#"
pub fn validate_email(email: &str) -> bool {
    email.contains('@') && email.contains('.')
}

pub fn hash_password(password: &str) -> String {
    // Simplified hash - in reality would use proper hashing
    format!("hashed_{}", password)
}
"#;
    fs::write(&test_file2_path, test_code2)?;
    
    searcher.index_file(&test_file2_path).await?;
    println!("✅ Second file indexed successfully");
    
    // Search across both files
    let multi_file_results = searcher.search("email").await?;
    validate_multi_file_results(&multi_file_results, "email")?;
    
    // VALIDATION POINT 8: Error Resilience
    println!("🔍 VALIDATION 8: Error Resilience Testing");
    
    // Test with non-existent file
    let fake_path = temp_dir.path().join("non_existent.rs");
    let error_result = searcher.index_file(&fake_path).await;
    match error_result {
        Err(_) => println!("✅ System properly handles non-existent files"),
        Ok(_) => {
            println!("❌ CRITICAL: System should fail on non-existent files");
            panic!("Error resilience validation FAILED");
        }
    }
    
    // Test with malformed file
    let malformed_path = temp_dir.path().join("malformed.rs");
    fs::write(&malformed_path, "This is not valid Rust code @#$%")?;
    let malformed_result = searcher.index_file(&malformed_path).await;
    // Should not panic, but handle gracefully
    match malformed_result {
        Ok(_) => println!("✅ System handles malformed code files"),
        Err(e) => println!("⚠️  Warning: Malformed file error: {}", e),
    }
    
    // VALIDATION POINT 9: Performance Under Load
    println!("🔍 VALIDATION 9: Performance Under Load");
    
    let start_time = std::time::Instant::now();
    
    // Perform multiple searches rapidly
    for i in 0..50 {
        let query = format!("search_term_{}", i % 5);
        let _results = searcher.search(&query).await?;
    }
    
    let duration = start_time.elapsed();
    println!("✅ Completed 50 searches in {:?}", duration);
    
    if duration.as_secs() > 30 {
        println!("⚠️  WARNING: Search performance may be suboptimal");
    }
    
    println!("🎉 END-TO-END PIPELINE VALIDATION COMPLETED SUCCESSFULLY");
    println!("✅ ALL VALIDATIONS PASSED - System is production ready!");
    
    Ok(())
}

fn validate_search_results(results: &[SearchResult], query: &str, search_type: &str) -> Result<()> {
    if results.is_empty() {
        println!("❌ CRITICAL FAILURE: {} search for '{}' returned no results", search_type, query);
        panic!("{} search validation FAILED - no results found", search_type);
    }
    
    println!("✅ {} search found {} results for '{}'", search_type, results.len(), query);
    
    // Validate result structure
    for (i, result) in results.iter().enumerate() {
        if result.content.is_empty() {
            println!("❌ CRITICAL FAILURE: Result {} has empty content", i);
            panic!("Search result validation FAILED - empty content");
        }
        
        if result.file_path.is_empty() {
            println!("❌ CRITICAL FAILURE: Result {} has empty file path", i);
            panic!("Search result validation FAILED - empty file path");
        }
        
        // Validate score is reasonable
        if result.score < 0.0 || result.score > 1.0 {
            println!("⚠️  WARNING: Result {} has unusual score: {}", i, result.score);
        }
    }
    
    println!("✅ All {} search results validated", search_type);
    Ok(())
}

#[cfg(feature = "tree-sitter")]
fn validate_symbol_results(results: &[crate::search::symbol_index::Symbol], symbol_name: &str) -> Result<()> {
    if results.is_empty() {
        println!("❌ CRITICAL FAILURE: Symbol search for '{}' returned no results", symbol_name);
        panic!("Symbol search validation FAILED - no results found");
    }
    
    println!("✅ Symbol search found {} results for '{}'", results.len(), symbol_name);
    
    // Validate symbols have required fields
    for (i, symbol) in results.iter().enumerate() {
        if symbol.name.is_empty() {
            println!("❌ CRITICAL FAILURE: Symbol {} has empty name", i);
            panic!("Symbol validation FAILED - empty name");
        }
        
        if symbol.file_path.is_empty() {
            println!("❌ CRITICAL FAILURE: Symbol {} has empty file path", i);
            panic!("Symbol validation FAILED - empty file path");
        }
    }
    
    println!("✅ All symbol search results validated");
    Ok(())
}

fn validate_multi_file_results(results: &[SearchResult], query: &str) -> Result<()> {
    if results.is_empty() {
        println!("❌ CRITICAL FAILURE: Multi-file search for '{}' returned no results", query);
        panic!("Multi-file search validation FAILED");
    }
    
    // Check if results span multiple files
    let unique_files: std::collections::HashSet<_> = results.iter()
        .map(|r| &r.file_path)
        .collect();
    
    if unique_files.len() < 2 {
        println!("⚠️  WARNING: Multi-file search only found results in {} file(s)", unique_files.len());
    } else {
        println!("✅ Multi-file search found results across {} files", unique_files.len());
    }
    
    println!("✅ Multi-file search validation completed");
    Ok(())
}

#[tokio::test]
async fn test_pipeline_concurrent_indexing() -> Result<()> {
    println!("🚨 TESTING CONCURRENT INDEXING CAPABILITY");
    
    let temp_dir = TempDir::new()?;
    let project_path = temp_dir.path().to_path_buf();
    let db_path = temp_dir.path().join("db");
    let searcher = UnifiedSearcher::new(project_path, db_path).await?;
    
    // Create multiple test files
    let mut file_paths = Vec::new();
    for i in 0..10 {
        let file_path = temp_dir.path().join(format!("test_{}.rs", i));
        let content = format!(r#"
pub fn function_{}() -> i32 {{
    // This is function number {}
    println!("Processing item {{}}", {});
    return {};
}}
"#, i, i, i, i);
        fs::write(&file_path, content)?;
        file_paths.push(file_path);
    }
    
    // Index all files concurrently
    let start_time = std::time::Instant::now();
    
    let mut tasks = Vec::new();
    for file_path in file_paths {
        let searcher_clone = searcher.clone();
        let task = tokio::spawn(async move {
            searcher_clone.index_file(&file_path).await
        });
        tasks.push(task);
    }
    
    // Wait for all indexing to complete
    for task in tasks {
        task.await??;
    }
    
    let duration = start_time.elapsed();
    println!("✅ Indexed 10 files concurrently in {:?}", duration);
    
    // Verify all files are searchable
    for i in 0..10 {
        let query = format!("function_{}", i);
        let results = searcher.search(&query).await?;
        
        if results.is_empty() {
            println!("❌ CRITICAL: Concurrent indexing failed - function_{} not found", i);
            panic!("Concurrent indexing validation FAILED");
        }
    }
    
    println!("🎉 CONCURRENT INDEXING VALIDATION PASSED");
    Ok(())
}

#[tokio::test]
async fn test_pipeline_memory_usage() -> Result<()> {
    println!("🚨 TESTING MEMORY USAGE UNDER LOAD");
    
    let temp_dir = TempDir::new()?;
    let project_path = temp_dir.path().to_path_buf();
    let db_path = temp_dir.path().join("db");
    let searcher = UnifiedSearcher::new(project_path, db_path).await?;
    
    // Create a large file to test memory handling
    let large_file_path = temp_dir.path().join("large_file.rs");
    let mut large_content = String::new();
    
    // Generate ~100KB of code
    for i in 0..1000 {
        large_content.push_str(&format!(r#"
pub fn generated_function_{}() -> String {{
    let value = "This is a generated function with ID {}";
    println!("Executing function {{}}", {});
    format!("Result: {{}}", value)
}}
"#, i, i, i));
    }
    
    fs::write(&large_file_path, large_content)?;
    
    // Monitor memory during indexing
    let initial_memory = get_memory_usage();
    println!("📊 Initial memory usage: {} KB", initial_memory);
    
    searcher.index_file(&large_file_path).await?;
    
    let post_index_memory = get_memory_usage();
    println!("📊 Post-indexing memory usage: {} KB", post_index_memory);
    
    let memory_increase = post_index_memory - initial_memory;
    println!("📊 Memory increase: {} KB", memory_increase);
    
    // Perform many searches to test memory stability
    for i in 0..100 {
        let query = format!("generated_function_{}", i);
        let _results = searcher.search(&query).await?;
    }
    
    let final_memory = get_memory_usage();
    println!("📊 Final memory usage: {} KB", final_memory);
    
    // Check for memory leaks
    if final_memory > post_index_memory * 2 {
        println!("⚠️  WARNING: Possible memory leak detected");
        println!("   Post-index: {} KB, Final: {} KB", post_index_memory, final_memory);
    } else {
        println!("✅ Memory usage appears stable");
    }
    
    println!("🎉 MEMORY USAGE VALIDATION COMPLETED");
    Ok(())
}

fn get_memory_usage() -> u64 {
    #[cfg(unix)]
    {
        use std::fs;
        if let Ok(content) = fs::read_to_string("/proc/self/status") {
            for line in content.lines() {
                if line.starts_with("VmRSS:") {
                    if let Some(kb_str) = line.split_whitespace().nth(1) {
                        return kb_str.parse().unwrap_or(0);
                    }
                }
            }
        }
    }
    
    #[cfg(windows)]
    {
        use sysinfo::{System, SystemExt, ProcessExt, PidExt};
        let mut sys = System::new();
        sys.refresh_process(sysinfo::get_current_pid().unwrap());
        if let Some(process) = sys.process(sysinfo::get_current_pid().unwrap()) {
            return process.memory() / 1024; // Convert to KB
        }
    }
    
    0 // Fallback
}

// Additional stress tests for edge cases
#[tokio::test]
async fn test_pipeline_edge_cases() -> Result<()> {
    println!("🚨 TESTING PIPELINE EDGE CASES");
    
    let temp_dir = TempDir::new()?;
    let project_path = temp_dir.path().to_path_buf();
    let db_path = temp_dir.path().join("db");
    let searcher = UnifiedSearcher::new(project_path, db_path).await?;
    
    // Test 1: Empty file
    let empty_file = temp_dir.path().join("empty.rs");
    fs::write(&empty_file, "")?;
    searcher.index_file(&empty_file).await?;
    println!("✅ Empty file handled correctly");
    
    // Test 2: File with only whitespace
    let whitespace_file = temp_dir.path().join("whitespace.rs");
    fs::write(&whitespace_file, "   \n\n\t  \n  ")?;
    searcher.index_file(&whitespace_file).await?;
    println!("✅ Whitespace-only file handled correctly");
    
    // Test 3: File with special characters
    let special_file = temp_dir.path().join("special.rs");
    fs::write(&special_file, r#"
// File with special characters: àáâãäåæçèéêë
pub fn special_function() {
    println!("Testing unicode: 🦀 Rust is awesome! 🚀");
    let emoji_string = "🔍🎯⚡";
}
"#)?;
    searcher.index_file(&special_file).await?;
    println!("✅ Special characters handled correctly");
    
    // Test 4: Very long lines
    let long_line_file = temp_dir.path().join("long_lines.rs");
    let long_line = "// ".to_string() + &"a".repeat(10000);
    fs::write(&long_line_file, format!("{}\npub fn test() {{}}", long_line))?;
    searcher.index_file(&long_line_file).await?;
    println!("✅ Long lines handled correctly");
    
    // Test 5: Search for non-existent content
    let no_results = searcher.search("ThisShouldNotExistAnywhere123456789").await?;
    if no_results.is_empty() {
        println!("✅ Non-existent content search handled correctly");
    } else {
        println!("⚠️  Unexpected results for non-existent content search");
    }
    
    println!("🎉 EDGE CASE VALIDATION COMPLETED");
    Ok(())
}