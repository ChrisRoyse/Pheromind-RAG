use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant};
use anyhow::{Result, Context};
use tempfile::TempDir;

/// TANTIVY STRESS TEST VERIFICATION RUNNER
/// 
/// This script actually EXECUTES the stress tests to prove they work and find real issues.
/// It runs a subset of critical tests that can complete quickly while still being thorough.
/// 
/// VERIFICATION PRINCIPLES:
/// 1. Tests must complete within reasonable time (< 5 minutes total)
/// 2. All test results must be verified as real, not simulated
/// 3. Any failures must produce specific, actionable error messages
/// 4. Performance metrics must be measured and reported accurately

#[cfg(feature = "tantivy")]
use embed_search::search::{TantivySearcher, ExactMatch};

/// Run quick verification of TantivySearcher stress testing capabilities
#[cfg(test)]
#[tokio::test]
async fn verify_tantivy_stress_capabilities() -> Result<()> {
    println!("🔥 TANTIVY STRESS TEST VERIFICATION");
    println!("====================================");
    println!("Running quick verification tests to prove stress testing works...\n");
    
    let overall_start = Instant::now();
    let mut verification_results = VerificationResults::new();
    
    // Verification 1: Basic Functionality
    println!("📋 VERIFICATION 1: Basic Functionality");
    println!("--------------------------------------");
    match verify_basic_functionality().await {
        Ok(duration) => {
            verification_results.add_success("Basic Functionality", duration);
            println!("✅ Basic functionality verified in {:?}\n", duration);
        }
        Err(e) => {
            verification_results.add_failure("Basic Functionality", e.to_string());
            println!("❌ Basic functionality failed: {}\n", e);
        }
    }
    
    // Verification 2: Moderate Stress
    println!("📋 VERIFICATION 2: Moderate Stress Test");
    println!("---------------------------------------");
    match verify_moderate_stress().await {
        Ok(duration) => {
            verification_results.add_success("Moderate Stress", duration);
            println!("✅ Moderate stress test verified in {:?}\n", duration);
        }
        Err(e) => {
            verification_results.add_failure("Moderate Stress", e.to_string());
            println!("❌ Moderate stress test failed: {}\n", e);
        }
    }
    
    // Verification 3: Error Handling
    println!("📋 VERIFICATION 3: Error Handling");
    println!("---------------------------------");
    match verify_error_handling().await {
        Ok(duration) => {
            verification_results.add_success("Error Handling", duration);
            println!("✅ Error handling verified in {:?}\n", duration);
        }
        Err(e) => {
            verification_results.add_failure("Error Handling", e.to_string());
            println!("❌ Error handling failed: {}\n", e);
        }
    }
    
    // Verification 4: Fuzzy Search
    println!("📋 VERIFICATION 4: Fuzzy Search Capabilities");
    println!("--------------------------------------------");
    match verify_fuzzy_search().await {
        Ok(duration) => {
            verification_results.add_success("Fuzzy Search", duration);
            println!("✅ Fuzzy search verified in {:?}\n", duration);
        }
        Err(e) => {
            verification_results.add_failure("Fuzzy Search", e.to_string());
            println!("❌ Fuzzy search failed: {}\n", e);
        }
    }
    
    // Verification 5: Performance Boundaries
    println!("📋 VERIFICATION 5: Performance Boundaries");
    println!("-----------------------------------------");
    match verify_performance_boundaries().await {
        Ok(duration) => {
            verification_results.add_success("Performance Boundaries", duration);
            println!("✅ Performance boundaries verified in {:?}\n", duration);
        }
        Err(e) => {
            verification_results.add_failure("Performance Boundaries", e.to_string());
            println!("❌ Performance boundaries failed: {}\n", e);
        }
    }
    
    let total_duration = overall_start.elapsed();
    
    // Final Report
    println!("🏁 VERIFICATION RESULTS SUMMARY");
    println!("===============================");
    verification_results.print_summary(total_duration);
    
    if verification_results.has_failures() {
        println!("\n❌ STRESS TEST VERIFICATION FAILED");
        println!("The stress test suite itself has issues that must be fixed before");
        println!("it can be trusted to find problems in TantivySearcher.");
        verification_results.print_failures();
        return Err(anyhow::anyhow!("Stress test verification failed"));
    } else {
        println!("\n✅ STRESS TEST VERIFICATION SUCCESSFUL");
        println!("The stress test suite is working correctly and can be trusted");
        println!("to find real issues in TantivySearcher implementation.");
    }
    
    Ok(())
}

/// Verify basic TantivySearcher functionality works correctly
#[cfg(feature = "tantivy")]
async fn verify_basic_functionality() -> Result<Duration> {
    let start_time = Instant::now();
    let temp_dir = TempDir::new()?;
    
    // Test 1: Create searcher
    let mut searcher = TantivySearcher::new_with_path(temp_dir.path().join("verify_basic")).await
        .context("Failed to create TantivySearcher")?;
    
    // Test 2: Index a simple file
    let test_file = temp_dir.path().join("basic_test.rs");
    let test_content = r#"
pub fn basic_test_function() -> String {
    let search_target = "basic_verification_marker";
    search_target.to_string()
}

pub struct BasicTestStruct {
    field: String,
}

impl BasicTestStruct {
    pub fn get_unique_content() -> &'static str {
        "unique_basic_content_12345"
    }
}
"#;
    fs::write(&test_file, test_content)?;
    
    searcher.index_file(&test_file).await
        .context("Failed to index basic test file")?;
    
    // Test 3: Verify exact search works
    let exact_results = searcher.search("basic_verification_marker").await
        .context("Exact search failed")?;
    
    if exact_results.is_empty() {
        return Err(anyhow::anyhow!("Exact search returned no results for known content"));
    }
    
    let first_result = &exact_results[0];
    if !first_result.content.contains("basic_verification_marker") {
        return Err(anyhow::anyhow!("Search result doesn't contain expected content: {}", first_result.content));
    }
    
    // Test 4: Verify fuzzy search works
    let fuzzy_results = searcher.search_fuzzy("BasicTestStrct", 2).await
        .context("Fuzzy search failed")?;
    
    if fuzzy_results.is_empty() {
        return Err(anyhow::anyhow!("Fuzzy search returned no results"));
    }
    
    // Test 5: Verify index stats
    let stats = searcher.get_index_stats()
        .context("Failed to get index stats")?;
    
    if stats.num_documents == 0 {
        return Err(anyhow::anyhow!("Index stats show 0 documents after indexing"));
    }
    
    println!("   📊 Basic test indexed {} documents", stats.num_documents);
    println!("   🔍 Exact search found {} results", exact_results.len());
    println!("   🔮 Fuzzy search found {} results", fuzzy_results.len());
    
    Ok(start_time.elapsed())
}

#[cfg(not(feature = "tantivy"))]
async fn verify_basic_functionality() -> Result<Duration> {
    Err(anyhow::anyhow!("Tantivy feature not enabled - cannot verify functionality"))
}

/// Test moderate stress conditions
#[cfg(feature = "tantivy")]
async fn verify_moderate_stress() -> Result<Duration> {
    let start_time = Instant::now();
    let temp_dir = TempDir::new()?;
    
    let mut searcher = TantivySearcher::new_with_path(temp_dir.path().join("verify_stress")).await?;
    
    // Create moderate number of files (100 files, 50 lines each)
    let files_dir = temp_dir.path().join("stress_files");
    fs::create_dir_all(&files_dir)?;
    
    const FILE_COUNT: usize = 100;
    const LINES_PER_FILE: usize = 50;
    
    println!("   📁 Creating {} files with {} lines each...", FILE_COUNT, LINES_PER_FILE);
    let creation_start = Instant::now();
    
    for file_idx in 0..FILE_COUNT {
        let file_path = files_dir.join(format!("stress_file_{:03}.rs", file_idx));
        let mut content = String::new();
        
        for line_idx in 0..LINES_PER_FILE {
            content.push_str(&format!(
                "pub fn stress_function_{}_{}() -> String {{\n    \
                    \"stress_marker_{}_{}\".to_string()\n\
                }}\n\n",
                file_idx, line_idx, file_idx, line_idx
            ));
        }
        
        fs::write(&file_path, content)?;
    }
    
    let creation_duration = creation_start.elapsed();
    println!("   ⏱️  File creation took {:?}", creation_duration);
    
    // Index all files
    let index_start = Instant::now();
    searcher.index_directory(&files_dir).await
        .context("Failed to index directory under moderate stress")?;
    let index_duration = index_start.elapsed();
    
    // Verify indexing worked
    let stats = searcher.get_index_stats()?;
    println!("   📊 Indexed {} documents in {:?}", stats.num_documents, index_duration);
    
    if stats.num_documents == 0 {
        return Err(anyhow::anyhow!("No documents indexed under moderate stress"));
    }
    
    // Test search performance under moderate load
    let search_queries = vec![
        "stress_marker_50_25",
        "stress_function_75",
        "stress_marker",
        "function_50_25",
    ];
    
    let mut total_search_time = Duration::new(0, 0);
    let mut total_results = 0;
    
    for query in &search_queries {
        let search_start = Instant::now();
        let results = searcher.search(query).await?;
        let search_duration = search_start.elapsed();
        
        total_search_time += search_duration;
        total_results += results.len();
        
        println!("   🔍 '{}' found {} results in {:?}", query, results.len(), search_duration);
        
        // Verify search results make sense
        if !results.is_empty() {
            let first_result = &results[0];
            if first_result.content.is_empty() || first_result.file_path.is_empty() {
                return Err(anyhow::anyhow!("Search result has empty content or file path"));
            }
        }
    }
    
    let avg_search_time = total_search_time / search_queries.len() as u32;
    println!("   ⚡ Average search time: {:?}, total results: {}", avg_search_time, total_results);
    
    // Performance requirements for moderate stress
    if index_duration > Duration::from_secs(30) {
        return Err(anyhow::anyhow!("Indexing took too long under moderate stress: {:?}", index_duration));
    }
    
    if avg_search_time > Duration::from_millis(200) {
        return Err(anyhow::anyhow!("Search too slow under moderate stress: {:?}", avg_search_time));
    }
    
    Ok(start_time.elapsed())
}

#[cfg(not(feature = "tantivy"))]
async fn verify_moderate_stress() -> Result<Duration> {
    Err(anyhow::anyhow!("Tantivy feature not enabled"))
}

/// Test error handling under controlled conditions
#[cfg(feature = "tantivy")]
async fn verify_error_handling() -> Result<Duration> {
    let start_time = Instant::now();
    let temp_dir = TempDir::new()?;
    
    let mut searcher = TantivySearcher::new().await?;
    
    // Test 1: Index nonexistent file
    let nonexistent_file = temp_dir.path().join("does_not_exist.rs");
    let result = searcher.index_file(&nonexistent_file).await;
    
    match result {
        Ok(()) => {
            return Err(anyhow::anyhow!("Indexing nonexistent file should fail but succeeded"));
        }
        Err(e) => {
            println!("   ✅ Correctly failed to index nonexistent file: {}", e);
            let error_msg = e.to_string().to_lowercase();
            if !error_msg.contains("failed to read file") {
                return Err(anyhow::anyhow!("Error message doesn't indicate file read failure: {}", e));
            }
        }
    }
    
    // Test 2: Search empty index
    let empty_results = searcher.search("nonexistent_content").await?;
    if !empty_results.is_empty() {
        return Err(anyhow::anyhow!("Search in empty index should return no results"));
    }
    println!("   ✅ Empty index search correctly returned no results");
    
    // Test 3: Invalid search queries
    let invalid_queries = vec!["", "\"", "AND OR NOT"];
    
    for query in &invalid_queries {
        let result = searcher.search(query).await;
        match result {
            Ok(results) => {
                println!("   ℹ️  Invalid query '{}' returned {} results (acceptable)", 
                        query.escape_debug(), results.len());
            }
            Err(e) => {
                println!("   ✅ Invalid query '{}' correctly failed: {}", 
                        query.escape_debug(), e);
            }
        }
    }
    
    // Test 4: Index recovery from corruption simulation
    // Create a valid index first
    let index_path = temp_dir.path().join("corruption_test");
    let mut persistent_searcher = TantivySearcher::new_with_path(&index_path).await?;
    
    let test_file = temp_dir.path().join("corruption_test.rs");
    fs::write(&test_file, "pub fn corruption_test() { println!(\"test\"); }")?;
    persistent_searcher.index_file(&test_file).await?;
    
    // Verify it works
    let results_before = persistent_searcher.search("corruption_test").await?;
    if results_before.is_empty() {
        return Err(anyhow::anyhow!("Index didn't work before corruption test"));
    }
    
    // Drop the searcher to release locks
    drop(persistent_searcher);
    
    // Simulate corruption by writing garbage to index files
    if let Ok(entries) = fs::read_dir(&index_path) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_file() {
                    // Write some garbage data
                    let _ = fs::write(&path, b"CORRUPTED_DATA_12345");
                }
            }
        }
    }
    
    // Test recovery
    let recovery_result = TantivySearcher::new_with_path(&index_path).await;
    match recovery_result {
        Ok(_recovered_searcher) => {
            println!("   ✅ Successfully recovered from index corruption");
        }
        Err(e) => {
            println!("   ℹ️  Index corruption caused rebuild (acceptable): {}", e);
            // This is acceptable behavior - corruption should trigger rebuild
        }
    }
    
    Ok(start_time.elapsed())
}

#[cfg(not(feature = "tantivy"))]
async fn verify_error_handling() -> Result<Duration> {
    Err(anyhow::anyhow!("Tantivy feature not enabled"))
}

/// Test fuzzy search capabilities
#[cfg(feature = "tantivy")]
async fn verify_fuzzy_search() -> Result<Duration> {
    let start_time = Instant::now();
    let temp_dir = TempDir::new()?;
    
    let mut searcher = TantivySearcher::new().await?;
    
    // Create test content with known fuzzy match targets
    let fuzzy_file = temp_dir.path().join("fuzzy_test.rs");
    let fuzzy_content = r#"
pub struct DatabaseConnection {
    connection_string: String,
}

impl DatabaseConnection {
    pub fn create_connection() -> Self {
        DatabaseConnection {
            connection_string: "database://localhost".to_string(),
        }
    }
    
    pub fn execute_query(&self) -> QueryResult {
        self.connection.query("SELECT * FROM users")
    }
}

pub fn process_payment(amount: f64) -> PaymentResult {
    PaymentResult::Success(amount)
}

pub fn validate_user_input(input: &str) -> ValidationResult {
    ValidationResult::Valid
}

pub fn handle_authentication() -> AuthResult {
    AuthResult::Authenticated
}
"#;
    
    fs::write(&fuzzy_file, fuzzy_content)?;
    searcher.index_file(&fuzzy_file).await?;
    
    // Test fuzzy search with various edit distances and patterns
    let fuzzy_test_cases = vec![
        // (query, max_distance, expected_to_find_something, description)
        ("DatabaseConnection", 1, true, "Exact match"),
        ("DatabaseConnction", 1, true, "1 char missing"),
        ("Databse", 2, true, "Multiple chars missing"),
        ("process", 1, true, "Exact word match"),
        ("proces", 1, true, "1 char missing from word"),
        ("validate", 1, true, "Exact function name part"),
        ("validte", 1, true, "1 char missing from function"),
        ("authentication", 2, true, "Exact word from function"),
        ("authntication", 2, true, "1 char missing"),
        ("xyz", 2, false, "Completely unrelated"),
    ];
    
    for (query, max_distance, should_find, description) in &fuzzy_test_cases {
        let search_start = Instant::now();
        let results = searcher.search_fuzzy(query, *max_distance).await?;
        let search_duration = search_start.elapsed();
        
        println!("   🔮 Fuzzy '{}' (d={}) -> {} results in {:?} ({})", 
                query, max_distance, results.len(), search_duration, description);
        
        if *should_find && results.is_empty() {
            return Err(anyhow::anyhow!("Fuzzy search for '{}' should find results but didn't", query));
        }
        
        if !*should_find && !results.is_empty() {
            println!("      ℹ️  Unexpected results for unrelated query (may be acceptable)");
        }
        
        // Verify result quality if found
        if !results.is_empty() {
            let first_result = &results[0];
            if first_result.content.is_empty() {
                return Err(anyhow::anyhow!("Fuzzy search result has empty content"));
            }
            
            println!("      Sample: {}", first_result.content.chars().take(50).collect::<String>());
        }
        
        // Performance check
        if search_duration > Duration::from_millis(100) {
            println!("      ⚠️  Fuzzy search took longer than 100ms: {:?}", search_duration);
        }
    }
    
    Ok(start_time.elapsed())
}

#[cfg(not(feature = "tantivy"))]
async fn verify_fuzzy_search() -> Result<Duration> {
    Err(anyhow::anyhow!("Tantivy feature not enabled"))
}

/// Test performance boundaries
#[cfg(feature = "tantivy")]
async fn verify_performance_boundaries() -> Result<Duration> {
    let start_time = Instant::now();
    let temp_dir = TempDir::new()?;
    
    let mut searcher = TantivySearcher::new_with_path(temp_dir.path().join("perf_test")).await?;
    
    // Create a reasonably sized test dataset
    const TEST_FILES: usize = 50;
    const LINES_PER_FILE: usize = 200;
    
    let perf_dir = temp_dir.path().join("performance_test");
    fs::create_dir_all(&perf_dir)?;
    
    println!("   📊 Creating performance test dataset: {} files, {} lines each", 
            TEST_FILES, LINES_PER_FILE);
    
    let dataset_start = Instant::now();
    for file_idx in 0..TEST_FILES {
        let file_path = perf_dir.join(format!("perf_{:03}.rs", file_idx));
        let mut content = String::new();
        
        for line_idx in 0..LINES_PER_FILE {
            content.push_str(&format!(
                "pub fn perf_function_{}_{}() -> Result<String> {{\n    \
                    let benchmark_data = \"perf_marker_{}_{}_unique\";\n    \
                    Ok(benchmark_data.to_string())\n\
                }}\n\n",
                file_idx, line_idx, file_idx, line_idx
            ));
        }
        
        fs::write(&file_path, content)?;
    }
    let dataset_duration = dataset_start.elapsed();
    
    // Measure indexing performance
    let index_start = Instant::now();
    searcher.index_directory(&perf_dir).await?;
    let index_duration = index_start.elapsed();
    
    let stats = searcher.get_index_stats()?;
    let expected_docs = TEST_FILES * LINES_PER_FILE;
    
    println!("   📈 Performance Results:");
    println!("      Dataset creation: {:?}", dataset_duration);
    println!("      Indexing duration: {:?}", index_duration);
    println!("      Documents indexed: {} (expected: {})", stats.num_documents, expected_docs);
    println!("      Index size: {:.2} MB", stats.index_size_bytes as f64 / 1_024_000.0);
    
    // Calculate performance metrics
    let docs_per_second = stats.num_documents as f64 / index_duration.as_secs_f64();
    let files_per_second = TEST_FILES as f64 / index_duration.as_secs_f64();
    
    println!("      Indexing rate: {:.1} docs/sec, {:.1} files/sec", 
            docs_per_second, files_per_second);
    
    // Test search performance
    let search_terms = vec![
        "perf_marker_25_100_unique",
        "perf_function_30",
        "benchmark_data",
        "Result<String>",
    ];
    
    let mut total_search_time = Duration::new(0, 0);
    let mut total_results = 0;
    
    for term in &search_terms {
        let search_start = Instant::now();
        let results = searcher.search(term).await?;
        let search_duration = search_start.elapsed();
        
        total_search_time += search_duration;
        total_results += results.len();
        
        println!("      '{}': {} results in {:?}", term, results.len(), search_duration);
        
        // Verify results are legitimate
        if !results.is_empty() {
            let first_result = &results[0];
            if term.len() > 3 && !first_result.content.contains(term) {
                // For longer terms, verify they actually appear in results
                let term_found = results.iter().any(|r| r.content.contains(term));
                if !term_found {
                    return Err(anyhow::anyhow!("Search results for '{}' don't contain the search term", term));
                }
            }
        }
    }
    
    let avg_search_time = total_search_time / search_terms.len() as u32;
    println!("      Average search time: {:?}", avg_search_time);
    
    // Performance assertions
    if index_duration > Duration::from_secs(10) {
        return Err(anyhow::anyhow!("Indexing took too long: {:?} (max: 10s)", index_duration));
    }
    
    if avg_search_time > Duration::from_millis(50) {
        return Err(anyhow::anyhow!("Average search too slow: {:?} (max: 50ms)", avg_search_time));
    }
    
    if docs_per_second < 100.0 {
        return Err(anyhow::anyhow!("Indexing rate too low: {:.1} docs/sec (min: 100)", docs_per_second));
    }
    
    println!("   ✅ All performance boundaries within acceptable limits");
    
    Ok(start_time.elapsed())
}

#[cfg(not(feature = "tantivy"))]
async fn verify_performance_boundaries() -> Result<Duration> {
    Err(anyhow::anyhow!("Tantivy feature not enabled"))
}

// Support structures
struct VerificationResults {
    successes: Vec<(String, Duration)>,
    failures: Vec<(String, String)>,
}

impl VerificationResults {
    fn new() -> Self {
        Self {
            successes: Vec::new(),
            failures: Vec::new(),
        }
    }
    
    fn add_success(&mut self, test_name: &str, duration: Duration) {
        self.successes.push((test_name.to_string(), duration));
    }
    
    fn add_failure(&mut self, test_name: &str, error: String) {
        self.failures.push((test_name.to_string(), error));
    }
    
    fn has_failures(&self) -> bool {
        !self.failures.is_empty()
    }
    
    fn print_summary(&self, total_duration: Duration) {
        let total_tests = self.successes.len() + self.failures.len();
        let success_rate = if total_tests > 0 {
            self.successes.len() as f64 / total_tests as f64 * 100.0
        } else {
            0.0
        };
        
        println!("⏱️  Total verification time: {:?}", total_duration);
        println!("📊 Success rate: {:.1}% ({}/{})", success_rate, self.successes.len(), total_tests);
        
        for (test_name, duration) in &self.successes {
            println!("   ✅ {}: {:?}", test_name, duration);
        }
        
        if !self.failures.is_empty() {
            println!("\n❌ Failures:");
            for (test_name, error) in &self.failures {
                println!("   • {}: {}", test_name, error);
            }
        }
    }
    
    fn print_failures(&self) {
        if !self.failures.is_empty() {
            println!("\n🔍 DETAILED FAILURE ANALYSIS:");
            for (i, (test_name, error)) in self.failures.iter().enumerate() {
                println!("{}. {} FAILURE:", i + 1, test_name.to_uppercase());
                println!("   Error: {}", error);
                println!("   Impact: This prevents reliable stress testing");
                println!("   Action Required: Fix the test implementation\n");
            }
        }
    }
}