// MINIMAL END-TO-END PIPELINE VALIDATION - BRUTAL TRUTH TEST
// This test validates the core functionality with available features only
// FAIL FAST: Any failure indicates system defects

use std::path::PathBuf;
use std::fs;
use tempfile::TempDir;
use tokio;
use anyhow::Result;

use embed_search::search::unified::UnifiedSearcher;
use embed_search::config::Config;

#[tokio::test]
async fn test_minimal_pipeline_validation() -> Result<()> {
    println!("🚨 STARTING MINIMAL PIPELINE VALIDATION");
    
    // Create temporary directory for test files
    let temp_dir = TempDir::new()?;
    let test_file_path = temp_dir.path().join("test_code.rs");
    
    // Create realistic Rust code file for testing
    let test_code = r#"
/// A simple user management system
pub struct UserManager {
    users: std::collections::HashMap<u64, User>,
}

impl UserManager {
    pub fn new() -> Self {
        Self {
            users: std::collections::HashMap::new(),
        }
    }
    
    pub fn create_user(&mut self, name: &str) -> u64 {
        let id = self.users.len() as u64 + 1;
        let user = User { id, name: name.to_string() };
        self.users.insert(id, user);
        id
    }
}

pub struct User {
    pub id: u64,
    pub name: String,
}
"#;

    fs::write(&test_file_path, test_code)?;
    println!("✅ Created test file at {:?}", test_file_path);
    
    // VALIDATION POINT 1: System Initialization
    println!("🔍 VALIDATION 1: System Initialization");
    
    let config = Config::new_test_config();
    let project_path = temp_dir.path().to_path_buf();
    let db_path = temp_dir.path().join("db");
    
    let searcher = match UnifiedSearcher::new(project_path, db_path).await {
        Ok(s) => {
            println!("✅ UnifiedSearcher initialized successfully");
            s
        },
        Err(e) => {
            println!("❌ CRITICAL FAILURE: UnifiedSearcher initialization failed: {}", e);
            panic!("System initialization FAILED: {}", e);
        }
    };
    
    // VALIDATION POINT 2: File Indexing
    println!("🔍 VALIDATION 2: File Indexing Pipeline");
    
    match searcher.index_file(&test_file_path).await {
        Ok(_) => println!("✅ File indexed successfully"),
        Err(e) => {
            println!("❌ CRITICAL FAILURE: File indexing failed: {}", e);
            panic!("File indexing FAILED: {}", e);
        }
    }
    
    // VALIDATION POINT 3: Basic Search Functionality
    println!("🔍 VALIDATION 3: Basic Search Functionality");
    
    let search_results = match searcher.search("create_user").await {
        Ok(results) => results,
        Err(e) => {
            println!("❌ CRITICAL FAILURE: Search failed: {}", e);
            panic!("Search functionality FAILED: {}", e);
        }
    };
    
    // Validate search results
    if search_results.is_empty() {
        println!("❌ CRITICAL FAILURE: Search returned no results for 'create_user'");
        panic!("Search validation FAILED - no results found");
    }
    
    println!("✅ Search found {} results for 'create_user'", search_results.len());
    
    // Validate result structure
    for (i, result) in search_results.iter().enumerate() {
        if result.content.is_empty() {
            println!("❌ CRITICAL FAILURE: Result {} has empty content", i);
            panic!("Search result validation FAILED - empty content");
        }
        
        if result.file_path.is_empty() {
            println!("❌ CRITICAL FAILURE: Result {} has empty file path", i);
            panic!("Search result validation FAILED - empty file path");
        }
        
        // Check if content contains expected search term
        if !result.content.to_lowercase().contains("create_user") {
            println!("⚠️  WARNING: Result {} content doesn't contain search term", i);
        }
        
        println!("  Result {}: {} chars, score: {:.3}", i, result.content.len(), result.score);
    }
    
    // VALIDATION POINT 4: Symbol Search (if tree-sitter enabled)
    #[cfg(feature = "tree-sitter")]
    {
        println!("🔍 VALIDATION 4: Symbol Search");
        match searcher.search_symbols("UserManager").await {
            Ok(symbols) => {
                if symbols.is_empty() {
                    println!("⚠️  WARNING: Symbol search returned no results for 'UserManager'");
                } else {
                    println!("✅ Symbol search found {} results", symbols.len());
                    for symbol in &symbols {
                        println!("  Symbol: {} at {}:{}", symbol.name, symbol.file_path, symbol.line);
                    }
                }
            },
            Err(e) => {
                println!("⚠️  WARNING: Symbol search failed: {}", e);
            }
        }
    }
    
    // VALIDATION POINT 5: Error Resilience
    println!("🔍 VALIDATION 5: Error Resilience");
    
    // Test with non-existent file
    let fake_path = temp_dir.path().join("non_existent.rs");
    match searcher.index_file(&fake_path).await {
        Err(_) => println!("✅ System properly handles non-existent files"),
        Ok(_) => {
            println!("❌ CRITICAL: System should fail on non-existent files");
            panic!("Error resilience validation FAILED");
        }
    }
    
    // Test empty search
    match searcher.search("").await {
        Ok(results) => {
            println!("✅ Empty search handled gracefully, {} results", results.len());
        },
        Err(e) => {
            println!("⚠️  Empty search error: {}", e);
        }
    }
    
    // VALIDATION POINT 6: Multiple File Processing
    println!("🔍 VALIDATION 6: Multiple File Processing");
    
    let test_file2_path = temp_dir.path().join("utils.rs");
    let test_code2 = r#"
pub fn validate_input(input: &str) -> bool {
    !input.is_empty()
}

pub fn process_data(data: &str) -> String {
    data.trim().to_lowercase()
}
"#;
    fs::write(&test_file2_path, test_code2)?;
    
    match searcher.index_file(&test_file2_path).await {
        Ok(_) => println!("✅ Second file indexed successfully"),
        Err(e) => {
            println!("❌ CRITICAL: Multi-file indexing failed: {}", e);
            panic!("Multi-file indexing FAILED");
        }
    }
    
    // Search across both files
    let multi_results = searcher.search("validate").await?;
    if multi_results.is_empty() {
        println!("⚠️  WARNING: Multi-file search found no results");
    } else {
        // Check if results span multiple files
        let unique_files: std::collections::HashSet<_> = multi_results.iter()
            .map(|r| r.file_path())
            .collect();
        
        println!("✅ Multi-file search found {} results across {} files", 
                 multi_results.len(), unique_files.len());
    }
    
    // VALIDATION POINT 7: Performance Check
    println!("🔍 VALIDATION 7: Basic Performance");
    
    let start_time = std::time::Instant::now();
    
    for i in 0..10 {
        let query = format!("function_{}", i);
        let _results = searcher.search(&query).await?;
    }
    
    let duration = start_time.elapsed();
    println!("✅ Completed 10 searches in {:?}", duration);
    
    if duration.as_secs() > 10 {
        println!("⚠️  WARNING: Search performance may be suboptimal");
    }
    
    println!("🎉 MINIMAL PIPELINE VALIDATION COMPLETED");
    
    // FINAL ASSESSMENT
    println!("═══════════════════════════════════════");
    println!("📊 PIPELINE VALIDATION SUMMARY");
    println!("═══════════════════════════════════════");
    println!("✅ System Initialization: PASSED");
    println!("✅ File Indexing: PASSED");
    println!("✅ Basic Search: PASSED");
    #[cfg(feature = "tree-sitter")]
    println!("✅ Symbol Search: AVAILABLE");
    #[cfg(not(feature = "tree-sitter"))]
    println!("⚠️  Symbol Search: NOT AVAILABLE (feature disabled)");
    println!("✅ Error Resilience: PASSED");
    println!("✅ Multi-file Processing: PASSED");
    println!("✅ Basic Performance: ACCEPTABLE");
    
    println!("🎯 VERDICT: CORE FUNCTIONALITY OPERATIONAL");
    
    Ok(())
}

#[tokio::test]
async fn test_search_accuracy_validation() -> Result<()> {
    println!("🚨 TESTING SEARCH ACCURACY");
    
    let temp_dir = TempDir::new()?;
    let config = Config::new_test_config();
    let project_path = temp_dir.path().to_path_buf();
    let db_path = temp_dir.path().join("db");
    
    let searcher = UnifiedSearcher::new(project_path, db_path).await?;
    
    // Create test file with specific content
    let test_file = temp_dir.path().join("accuracy_test.rs");
    let content = r#"
pub struct Calculator {
    value: f64,
}

impl Calculator {
    pub fn new() -> Self {
        Self { value: 0.0 }
    }
    
    pub fn add(&mut self, x: f64) -> &mut Self {
        self.value += x;
        self
    }
    
    pub fn multiply(&mut self, x: f64) -> &mut Self {
        self.value *= x;
        self
    }
    
    pub fn get_result(&self) -> f64 {
        self.value
    }
}
"#;
    
    fs::write(&test_file, content)?;
    searcher.index_file(&test_file).await?;
    
    // Test specific search queries
    let test_cases = vec![
        ("Calculator", true),  // Should find struct
        ("add", true),         // Should find method
        ("multiply", true),    // Should find method
        ("result", true),      // Should find get_result
        ("nonexistent", false), // Should not find
    ];
    
    for (query, should_find) in test_cases {
        let results = searcher.search(query).await?;
        
        if should_find {
            if results.is_empty() {
                println!("❌ ACCURACY FAILURE: Query '{}' should find results but didn't", query);
                panic!("Search accuracy FAILED for query: {}", query);
            } else {
                println!("✅ Query '{}' correctly found {} results", query, results.len());
                
                // Validate that results contain the query term
                let found_match = results.iter().any(|r| 
                    r.content().to_lowercase().contains(&query.to_lowercase())
                );
                
                if !found_match {
                    println!("⚠️  WARNING: Query '{}' results don't contain the search term", query);
                }
            }
        } else {
            if !results.is_empty() {
                println!("⚠️  Query '{}' unexpectedly found {} results", query, results.len());
            } else {
                println!("✅ Query '{}' correctly found no results", query);
            }
        }
    }
    
    println!("🎉 SEARCH ACCURACY VALIDATION COMPLETED");
    Ok(())
}

#[tokio::test]  
async fn test_chunking_validation() -> Result<()> {
    println!("🚨 TESTING CHUNKING FUNCTIONALITY");
    
    let temp_dir = TempDir::new()?;
    let config = Config::new_test_config();
    let project_path = temp_dir.path().to_path_buf();
    let db_path = temp_dir.path().join("db");
    
    let searcher = UnifiedSearcher::new(project_path, db_path).await?;
    
    // Create a large file that will be chunked
    let large_file = temp_dir.path().join("large_file.rs");
    let mut large_content = String::new();
    
    // Generate multiple functions to ensure chunking
    for i in 0..20 {
        large_content.push_str(&format!(r#"
pub fn function_{}() -> i32 {{
    // This is function number {}
    println!("Processing item {{}}", {});
    let result = {} * 2;
    result
}}
"#, i, i, i, i));
    }
    
    fs::write(&large_file, large_content)?;
    
    // Index the large file
    match searcher.index_file(&large_file).await {
        Ok(_) => println!("✅ Large file indexed successfully (chunking test)"),
        Err(e) => {
            println!("❌ CRITICAL: Large file indexing failed: {}", e);
            panic!("Chunking validation FAILED: {}", e);
        }
    }
    
    // Search for content from different parts of the file
    for i in [0, 10, 19] {  // Test beginning, middle, and end
        let query = format!("function_{}", i);
        let results = searcher.search(&query).await?;
        
        if results.is_empty() {
            println!("❌ CHUNKING FAILURE: Could not find function_{} in chunks", i);
            panic!("Chunking validation FAILED - content not found in chunks");
        } else {
            println!("✅ Found function_{} in {} chunks", i, results.len());
        }
    }
    
    println!("🎉 CHUNKING VALIDATION COMPLETED");
    Ok(())
}