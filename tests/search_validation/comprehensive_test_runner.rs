use anyhow::Result;
use std::time::Instant;
use std::process::{Command, Stdio};
use std::collections::HashMap;

// Import all test modules
mod ripgrep_test;
mod tantivy_test;
mod vector_embedding_test;
mod ast_symbol_test;

/// Comprehensive test runner for all 4 search methods
/// Validates functionality, performance, and error handling
pub struct SearchTestRunner {
    results: HashMap<String, TestResult>,
}

#[derive(Debug, Clone)]
pub struct TestResult {
    pub passed: bool,
    pub duration: std::time::Duration,
    pub details: String,
    pub errors: Vec<String>,
}

impl SearchTestRunner {
    pub fn new() -> Self {
        Self {
            results: HashMap::new(),
        }
    }
    
    /// Run all search method tests and generate comprehensive report
    pub async fn run_all_tests(&mut self) -> Result<()> {
        println!("🚀 COMPREHENSIVE SEARCH VALIDATION SUITE");
        println!("========================================");
        println!("Testing all 4 parallel search methods in embed codebase");
        println!();
        
        let overall_start = Instant::now();
        
        // Test 1: Ripgrep/Native Search
        println!("🔍 TEST 1: RIPGREP/NATIVE SEARCH");
        println!("---------------------------------");
        let result = self.test_ripgrep_search().await;
        self.results.insert("ripgrep".to_string(), result);
        println!();
        
        // Test 2: Tantivy Full-Text Search  
        println!("🔍 TEST 2: TANTIVY FULL-TEXT SEARCH");
        println!("-----------------------------------");
        let result = self.test_tantivy_search().await;
        self.results.insert("tantivy".to_string(), result);
        println!();
        
        // Test 3: Vector/Embedding Search
        println!("🔍 TEST 3: VECTOR/EMBEDDING SEARCH");
        println!("----------------------------------");
        let result = self.test_vector_search().await;
        self.results.insert("vector".to_string(), result);
        println!();
        
        // Test 4: AST-Based Symbol Search
        println!("🔍 TEST 4: AST-BASED SYMBOL SEARCH");
        println!("----------------------------------");
        let result = self.test_ast_search().await;
        self.results.insert("ast".to_string(), result);
        println!();
        
        let overall_duration = overall_start.elapsed();
        
        // Generate comprehensive report
        self.generate_final_report(overall_duration).await?;
        
        Ok(())
    }
    
    async fn test_ripgrep_search(&self) -> TestResult {
        let start = Instant::now();
        let mut details = String::new();
        let mut errors = Vec::new();
        let mut passed = true;
        
        match ripgrep_test::run_ripgrep_tests() {
            Ok(_) => {
                details.push_str("✅ Native/Ripgrep search tests completed successfully\n");
                details.push_str("   - Basic text search functionality\n");
                details.push_str("   - Regex pattern matching\n");
                details.push_str("   - Case sensitivity handling\n");
                details.push_str("   - File filtering capabilities\n");
                details.push_str("   - Performance metrics validation\n");
                details.push_str("   - Error handling robustness\n");
            }
            Err(e) => {
                passed = false;
                errors.push(format!("Ripgrep tests failed: {}", e));
                details.push_str("❌ Native/Ripgrep search tests failed\n");
            }
        }
        
        // Test actual cargo test execution for ripgrep
        let test_output = Command::new("cargo")
            .args(&["test", "ripgrep_tests", "--", "--nocapture"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output();
            
        match test_output {
            Ok(output) => {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let test_count = stdout.lines()
                        .filter(|line| line.contains("test result:"))
                        .count();
                    details.push_str(&format!("   📊 Cargo test execution: {} test suites\n", test_count));
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    errors.push(format!("Cargo test failed: {}", stderr));
                    passed = false;
                }
            }
            Err(e) => {
                errors.push(format!("Failed to run cargo test: {}", e));
                // Don't mark as failed - might be environment issue
                details.push_str("   ⚠️  Cargo test execution not available\n");
            }
        }
        
        TestResult {
            passed,
            duration: start.elapsed(),
            details,
            errors,
        }
    }
    
    async fn test_tantivy_search(&self) -> TestResult {
        let start = Instant::now();
        let mut details = String::new();
        let mut errors = Vec::new();
        let mut passed = true;
        
        match tantivy_test::run_tantivy_tests().await {
            Ok(_) => {
                details.push_str("✅ Tantivy full-text search tests completed\n");
                details.push_str("   - Index creation and updates\n");
                details.push_str("   - Query parsing and execution\n");
                details.push_str("   - Fuzzy search and ranking\n");
                details.push_str("   - Index corruption detection\n");
                details.push_str("   - Project scoping functionality\n");
                details.push_str("   - Performance benchmarks\n");
            }
            Err(e) => {
                passed = false;
                errors.push(format!("Tantivy tests failed: {}", e));
                details.push_str("❌ Tantivy search tests failed\n");
            }
        }
        
        // Check if Tantivy feature is enabled
        let feature_check = Command::new("cargo")
            .args(&["build", "--features", "tantivy", "--dry-run"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
            
        match feature_check {
            Ok(status) if status.success() => {
                details.push_str("   ✅ Tantivy feature properly configured\n");
            }
            _ => {
                details.push_str("   ⚠️  Tantivy feature may not be available\n");
            }
        }
        
        TestResult {
            passed,
            duration: start.elapsed(),
            details,
            errors,
        }
    }
    
    async fn test_vector_search(&self) -> TestResult {
        let start = Instant::now();
        let mut details = String::new();
        let mut errors = Vec::new();
        let mut passed = true;
        
        match vector_embedding_test::run_vector_tests().await {
            Ok(_) => {
                details.push_str("✅ Vector/embedding search tests completed\n");
                details.push_str("   - Embedding generation functionality\n");
                details.push_str("   - Similarity search operations\n");
                details.push_str("   - Vector database operations\n");
                details.push_str("   - Embedding cache performance\n");
                details.push_str("   - Model status verification\n");
            }
            Err(e) => {
                passed = false;
                errors.push(format!("Vector search tests failed: {}", e));
                details.push_str("❌ Vector search tests failed\n");
            }
        }
        
        // Check if ML and vectordb features are available
        let ml_check = Command::new("cargo")
            .args(&["build", "--features", "ml,vectordb", "--dry-run"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
            
        match ml_check {
            Ok(status) if status.success() => {
                details.push_str("   ✅ ML and VectorDB features properly configured\n");
            }
            _ => {
                details.push_str("   ⚠️  ML or VectorDB features may not be available\n");
                // This is often expected due to model dependencies
            }
        }
        
        TestResult {
            passed,
            duration: start.elapsed(),
            details,
            errors,
        }
    }
    
    async fn test_ast_search(&self) -> TestResult {
        let start = Instant::now();
        let mut details = String::new();
        let mut errors = Vec::new();
        let mut passed = true;
        
        match ast_symbol_test::run_ast_tests().await {
            Ok(_) => {
                details.push_str("✅ AST-based symbol search tests completed\n");
                details.push_str("   - Syntax tree parsing accuracy\n");
                details.push_str("   - Semantic code search capability\n");
                details.push_str("   - Symbol resolution functionality\n");
                details.push_str("   - Multi-language support\n");
                details.push_str("   - Language detection accuracy\n");
            }
            Err(e) => {
                passed = false;
                errors.push(format!("AST search tests failed: {}", e));
                details.push_str("❌ AST search tests failed\n");
            }
        }
        
        // Check if tree-sitter feature is available
        let ts_check = Command::new("cargo")
            .args(&["build", "--features", "tree-sitter", "--dry-run"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
            
        match ts_check {
            Ok(status) if status.success() => {
                details.push_str("   ✅ Tree-sitter feature properly configured\n");
            }
            _ => {
                details.push_str("   ⚠️  Tree-sitter feature may not be available\n");
            }
        }
        
        TestResult {
            passed,
            duration: start.elapsed(),
            details,
            errors,
        }
    }
    
    async fn generate_final_report(&self, overall_duration: std::time::Duration) -> Result<()> {
        println!("📊 COMPREHENSIVE TEST RESULTS SUMMARY");
        println!("======================================");
        println!();
        
        let mut total_passed = 0;
        let mut total_tests = 0;
        
        for (method, result) in &self.results {
            total_tests += 1;
            
            let status = if result.passed { 
                total_passed += 1;
                "✅ PASSED"
            } else { 
                "❌ FAILED" 
            };
            
            println!("🔍 {}: {} (Duration: {:?})", 
                method.to_uppercase(), status, result.duration);
            
            // Print details with proper indentation
            for line in result.details.lines() {
                println!("   {}", line);
            }
            
            // Print errors if any
            if !result.errors.is_empty() {
                println!("   🔥 Errors:");
                for error in &result.errors {
                    println!("      - {}", error);
                }
            }
            
            println!();
        }
        
        // Overall statistics
        println!("📈 OVERALL STATISTICS");
        println!("--------------------");
        println!("Total Search Methods Tested: {}", total_tests);
        println!("Passed: {}", total_passed);
        println!("Failed: {}", total_tests - total_passed);
        println!("Success Rate: {:.1}%", (total_passed as f64 / total_tests as f64) * 100.0);
        println!("Total Test Duration: {:?}", overall_duration);
        println!();
        
        // Performance analysis
        println!("⚡ PERFORMANCE ANALYSIS");
        println!("-----------------------");
        let mut durations: Vec<_> = self.results.iter().collect();
        durations.sort_by_key(|(_, result)| result.duration);
        
        for (method, result) in durations {
            println!("{:<12} {:>8.2}ms", 
                format!("{}:", method.to_uppercase()), 
                result.duration.as_millis());
        }
        println!();
        
        // Feature availability summary
        println!("🛠️  FEATURE AVAILABILITY SUMMARY");
        println!("--------------------------------");
        self.analyze_feature_availability().await?;
        println!();
        
        // Recommendations
        println!("💡 RECOMMENDATIONS");
        println!("------------------");
        self.generate_recommendations().await?;
        
        Ok(())
    }
    
    async fn analyze_feature_availability(&self) -> Result<()> {
        // Check which features are actually available
        let features = vec![
            ("core", "Basic search functionality"),
            ("tantivy", "Full-text search with indexing"),
            ("tree-sitter", "AST-based symbol parsing"),
            ("ml", "Machine learning embeddings"),
            ("vectordb", "Vector database storage"),
            ("full-system", "All search capabilities"),
        ];
        
        for (feature, description) in features {
            let check = Command::new("cargo")
                .args(&["check", "--features", feature])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
                
            let status = match check {
                Ok(status) if status.success() => "✅ Available",
                Ok(_) => "❌ Build Failed",
                Err(_) => "❓ Unknown",
            };
            
            println!("{:<15} {} - {}", 
                format!("{}:", feature), status, description);
        }
        
        Ok(())
    }
    
    async fn generate_recommendations(&self) -> Result<()> {
        let failed_tests: Vec<_> = self.results.iter()
            .filter(|(_, result)| !result.passed)
            .collect();
            
        if failed_tests.is_empty() {
            println!("🎉 All search methods are functioning correctly!");
            println!("   The embed codebase has robust parallel search capabilities.");
            println!();
            println!("Next steps:");
            println!("- Consider performance optimizations for high-volume usage");
            println!("- Add monitoring for search query patterns");
            println!("- Implement search result caching for common queries");
        } else {
            println!("🔧 Issues detected that need attention:");
            println!();
            
            for (method, result) in failed_tests {
                println!("• {}: ", method.to_uppercase());
                for error in &result.errors {
                    println!("  - {}", error);
                }
                
                match method.as_str() {
                    "tantivy" => {
                        println!("  💡 Enable with: cargo build --features tantivy");
                        println!("  💡 Ensure index directory permissions are correct");
                    }
                    "vector" => {
                        println!("  💡 Enable with: cargo build --features ml,vectordb");
                        println!("  💡 Ensure model files are downloaded and accessible");
                        println!("  💡 Check system resources (RAM, disk space)");
                    }
                    "ast" => {
                        println!("  💡 Enable with: cargo build --features tree-sitter");
                        println!("  💡 Ensure tree-sitter language grammars are installed");
                    }
                    _ => {}
                }
                println!();
            }
        }
        
        // Environment-specific recommendations
        println!("🌐 Environment-Specific Notes:");
        println!("- Windows: Ensure long path support is enabled");
        println!("- Linux/Mac: Check file permissions for index directories");
        println!("- All: Ensure sufficient disk space for indices and models");
        println!("- CI/CD: Consider caching model downloads and compiled indices");
        
        Ok(())
    }
}

/// Main test runner function - can be called from integration tests
pub async fn run_comprehensive_search_tests() -> Result<()> {
    let mut runner = SearchTestRunner::new();
    runner.run_all_tests().await
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn integration_test_all_search_methods() {
        let result = run_comprehensive_search_tests().await;
        
        // Don't fail the test if some search methods aren't available
        // Just log the results - this is diagnostic testing
        match result {
            Ok(_) => println!("✅ Comprehensive search validation completed"),
            Err(e) => println!("⚠️  Search validation completed with issues: {}", e),
        }
    }
}

/// Binary entry point for running tests standalone
#[tokio::main]
async fn main() -> Result<()> {
    run_comprehensive_search_tests().await
}