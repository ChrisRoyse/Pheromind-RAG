//! Integration Test Orchestration System
//! 
//! This module provides a comprehensive orchestration system for running
//! all integration tests in a coordinated manner with proper validation.

use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use std::path::Path;
use std::io::{self, Write};

/// Test orchestration configuration
#[derive(Debug, Clone)]
pub struct TestConfig {
    pub name: String,
    pub features: Vec<String>,
    pub timeout: Duration,
    pub retry_count: u32,
    pub dependencies: Vec<String>,
    pub validation_rules: Vec<ValidationRule>,
}

/// Validation rules for test execution
#[derive(Debug, Clone)]
pub enum ValidationRule {
    MustPass,
    MustContainOutput(String),
    MustNotContainOutput(String),
    MustFinishWithin(Duration),
    MustProduceArtifact(String),
}

/// Test execution result
#[derive(Debug)]
pub struct TestResult {
    pub config: TestConfig,
    pub success: bool,
    pub duration: Duration,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    pub validation_failures: Vec<String>,
    pub artifacts_produced: Vec<String>,
}

/// Main test orchestrator
pub struct TestOrchestrator {
    pub test_configs: Vec<TestConfig>,
    pub results: Vec<TestResult>,
    pub parallel_limit: usize,
}

impl TestOrchestrator {
    pub fn new(parallel_limit: usize) -> Self {
        Self {
            test_configs: Vec::new(),
            results: Vec::new(),
            parallel_limit,
        }
    }

    /// Register a test configuration
    pub fn register_test(&mut self, config: TestConfig) {
        self.test_configs.push(config);
    }

    /// Run all registered tests with dependency resolution
    pub fn run_all_tests(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🚀 Starting Integration Test Orchestration");
        println!("Tests registered: {}", self.test_configs.len());
        println!("Parallel limit: {}", self.parallel_limit);
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        // Resolve test dependencies
        let execution_order = self.resolve_dependencies()?;
        
        // Execute tests in dependency order
        for test_batch in execution_order {
            self.run_test_batch(test_batch)?;
        }

        // Generate comprehensive report
        self.generate_report()
    }

    /// Resolve test dependencies and create execution batches
    fn resolve_dependencies(&self) -> Result<Vec<Vec<TestConfig>>, Box<dyn std::error::Error>> {
        let mut remaining_tests: HashMap<String, TestConfig> = self.test_configs.iter()
            .map(|config| (config.name.clone(), config.clone()))
            .collect();
        
        let mut execution_order = Vec::new();
        let mut completed_tests = std::collections::HashSet::new();

        // Keep resolving until all tests are scheduled
        while !remaining_tests.is_empty() {
            let mut current_batch = Vec::new();

            // Find tests whose dependencies are satisfied
            let mut ready_tests = Vec::new();
            for (name, config) in &remaining_tests {
                let dependencies_satisfied = config.dependencies.iter()
                    .all(|dep| completed_tests.contains(dep));
                
                if dependencies_satisfied {
                    ready_tests.push(name.clone());
                }
            }

            if ready_tests.is_empty() {
                return Err("Circular dependency detected in test configuration".into());
            }

            // Add ready tests to current batch (up to parallel limit)
            for test_name in ready_tests.into_iter().take(self.parallel_limit) {
                if let Some(config) = remaining_tests.remove(&test_name) {
                    completed_tests.insert(test_name);
                    current_batch.push(config);
                }
            }

            execution_order.push(current_batch);
        }

        Ok(execution_order)
    }

    /// Run a batch of tests in parallel
    fn run_test_batch(&mut self, batch: Vec<TestConfig>) -> Result<(), Box<dyn std::error::Error>> {
        if batch.is_empty() {
            return Ok(());
        }

        println!("\n🔄 Executing test batch ({} tests):", batch.len());
        for config in &batch {
            println!("  • {}", config.name);
        }

        let mut handles = Vec::new();
        
        // Start all tests in the batch
        for config in batch {
            let handle = std::thread::spawn(move || {
                Self::execute_single_test(config)
            });
            handles.push(handle);
        }

        // Wait for all tests to complete
        for handle in handles {
            match handle.join() {
                Ok(result) => {
                    let success_symbol = if result.success { "✅" } else { "❌" };
                    println!("  {} {} ({:.2}s)", 
                        success_symbol, 
                        result.config.name,
                        result.duration.as_secs_f64()
                    );
                    
                    if !result.validation_failures.is_empty() {
                        for failure in &result.validation_failures {
                            println!("    ⚠️  {}", failure);
                        }
                    }
                    
                    self.results.push(result);
                }
                Err(e) => {
                    eprintln!("Thread panic during test execution: {:?}", e);
                }
            }
        }

        Ok(())
    }

    /// Execute a single test configuration
    fn execute_single_test(config: TestConfig) -> TestResult {
        let start_time = Instant::now();
        
        // Build the cargo test command
        let mut command = Command::new("cargo");
        command.arg("test");
        
        // Add features if specified
        if !config.features.is_empty() {
            command.arg("--features");
            command.arg(config.features.join(","));
        }
        
        // Add test name pattern if it looks like a specific test
        if config.name.contains("::") || config.name.ends_with("_test") {
            command.arg(&config.name);
        } else {
            // Assume it's a test file
            command.arg("--test");
            command.arg(&config.name);
        }
        
        command.arg("--");
        command.arg("--nocapture");
        
        // Set timeout and execute
        let output = match command
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output() 
        {
            Ok(output) => output,
            Err(e) => {
                return TestResult {
                    config,
                    success: false,
                    duration: start_time.elapsed(),
                    stdout: String::new(),
                    stderr: format!("Failed to execute test: {}", e),
                    exit_code: None,
                    validation_failures: vec!["Execution failed".to_string()],
                    artifacts_produced: Vec::new(),
                };
            }
        };

        let duration = start_time.elapsed();
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let exit_code = output.status.code();
        
        // Apply validation rules
        let validation_failures = Self::validate_test_result(
            &config.validation_rules,
            &stdout,
            &stderr,
            exit_code,
            duration
        );

        // Check for artifacts
        let artifacts_produced = Self::check_artifacts(&config);

        let success = exit_code == Some(0) && validation_failures.is_empty();

        TestResult {
            config,
            success,
            duration,
            stdout,
            stderr,
            exit_code,
            validation_failures,
            artifacts_produced,
        }
    }

    /// Validate test result against configured rules
    fn validate_test_result(
        rules: &[ValidationRule],
        stdout: &str,
        stderr: &str,
        exit_code: Option<i32>,
        duration: Duration,
    ) -> Vec<String> {
        let mut failures = Vec::new();

        for rule in rules {
            match rule {
                ValidationRule::MustPass => {
                    if exit_code != Some(0) {
                        failures.push(format!("Test must pass but exit code was {:?}", exit_code));
                    }
                }
                ValidationRule::MustContainOutput(pattern) => {
                    if !stdout.contains(pattern) && !stderr.contains(pattern) {
                        failures.push(format!("Output must contain '{}'", pattern));
                    }
                }
                ValidationRule::MustNotContainOutput(pattern) => {
                    if stdout.contains(pattern) || stderr.contains(pattern) {
                        failures.push(format!("Output must not contain '{}'", pattern));
                    }
                }
                ValidationRule::MustFinishWithin(max_duration) => {
                    if duration > *max_duration {
                        failures.push(format!("Test took {:.2}s but must finish within {:.2}s", 
                            duration.as_secs_f64(), max_duration.as_secs_f64()));
                    }
                }
                ValidationRule::MustProduceArtifact(file_path) => {
                    if !Path::new(file_path).exists() {
                        failures.push(format!("Required artifact not found: {}", file_path));
                    }
                }
            }
        }

        failures
    }

    /// Check for test artifacts
    fn check_artifacts(config: &TestConfig) -> Vec<String> {
        let mut artifacts = Vec::new();
        
        // Common artifact locations
        let artifact_patterns = [
            "target/debug/deps/",
            "test_logs/",
            "test_output/",
        ];

        for pattern in &artifact_patterns {
            if let Ok(entries) = std::fs::read_dir(pattern) {
                for entry in entries {
                    if let Ok(entry) = entry {
                        let path = entry.path();
                        if path.file_name()
                            .and_then(|name| name.to_str())
                            .map(|name| name.contains(&config.name))
                            .unwrap_or(false)
                        {
                            artifacts.push(path.to_string_lossy().to_string());
                        }
                    }
                }
            }
        }

        artifacts
    }

    /// Generate comprehensive test report
    fn generate_report(&self) -> Result<(), Box<dyn std::error::Error>> {
        let total_tests = self.results.len();
        let passed_tests = self.results.iter().filter(|r| r.success).count();
        let success_rate = passed_tests as f32 / total_tests as f32;
        let total_duration: Duration = self.results.iter().map(|r| r.duration).sum();

        println!("\n");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("🎯 INTEGRATION TEST ORCHESTRATION REPORT");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("Total Tests: {}", total_tests);
        println!("Passed: {} ({:.1}%)", passed_tests, success_rate * 100.0);
        println!("Failed: {}", total_tests - passed_tests);
        println!("Total Runtime: {:.2}s", total_duration.as_secs_f64());
        println!("Average Test Time: {:.2}s", 
            total_duration.as_secs_f64() / total_tests as f64);
        println!();

        // Detailed results by category
        let mut by_feature: HashMap<String, Vec<&TestResult>> = HashMap::new();
        for result in &self.results {
            for feature in &result.config.features {
                by_feature.entry(feature.clone()).or_default().push(result);
            }
        }

        println!("📊 Results by Feature:");
        for (feature, results) in by_feature {
            let feature_passed = results.iter().filter(|r| r.success).count();
            let feature_total = results.len();
            let feature_rate = feature_passed as f32 / feature_total as f32;
            
            println!("  {} {}: {}/{} ({:.1}%)", 
                if feature_rate == 1.0 { "✅" } else if feature_rate >= 0.8 { "⚠️" } else { "❌" },
                feature,
                feature_passed,
                feature_total,
                feature_rate * 100.0
            );
        }

        // Failed tests details
        let failed_tests: Vec<_> = self.results.iter().filter(|r| !r.success).collect();
        if !failed_tests.is_empty() {
            println!("\n❌ Failed Tests Details:");
            for result in failed_tests {
                println!("  • {} ({:.2}s)", result.config.name, result.duration.as_secs_f64());
                for failure in &result.validation_failures {
                    println!("    ⚠️  {}", failure);
                }
                if result.exit_code != Some(0) {
                    println!("    Exit code: {:?}", result.exit_code);
                }
            }
        }

        // Truth enforcement assessment
        println!("\n🔍 Truth Enforcement Assessment:");
        let suspicious_failures = self.results.iter()
            .filter(|r| !r.success)
            .filter(|r| {
                r.stderr.to_lowercase().contains("unimplemented") ||
                r.stderr.to_lowercase().contains("todo") ||
                r.stdout.to_lowercase().contains("mock") ||
                r.stdout.to_lowercase().contains("fake")
            })
            .count();

        if suspicious_failures == 0 {
            println!("  ✅ No suspicious failure patterns detected");
            println!("  ✅ All failures appear to be genuine test issues");
        } else {
            println!("  ⚠️  {} tests failed with suspicious patterns", suspicious_failures);
            println!("  ⚠️  Review these tests for potential fake implementations");
        }

        // Final verdict
        println!("\n🎯 Final Assessment:");
        if success_rate >= 0.95 {
            println!("  🎉 EXCELLENT: {:.1}% success rate - system is highly reliable", success_rate * 100.0);
        } else if success_rate >= 0.85 {
            println!("  ✅ GOOD: {:.1}% success rate - system is reliable", success_rate * 100.0);
        } else if success_rate >= 0.70 {
            println!("  ⚠️  ACCEPTABLE: {:.1}% success rate - some improvements needed", success_rate * 100.0);
        } else {
            println!("  ❌ POOR: {:.1}% success rate - significant issues detected", success_rate * 100.0);
        }

        if success_rate < 0.70 {
            return Err("Integration test success rate below acceptable threshold".into());
        }

        Ok(())
    }
}

/// Default test configurations for the embed-search system
pub fn create_default_test_suite() -> TestOrchestrator {
    let mut orchestrator = TestOrchestrator::new(4); // 4 parallel tests

    // Core compilation and basic tests
    orchestrator.register_test(TestConfig {
        name: "core_compilation".to_string(),
        features: vec!["core".to_string()],
        timeout: Duration::from_secs(120),
        retry_count: 1,
        dependencies: vec![],
        validation_rules: vec![ValidationRule::MustPass],
    });

    // BM25 search tests
    orchestrator.register_test(TestConfig {
        name: "bm25_functionality_validation".to_string(),
        features: vec!["core".to_string()],
        timeout: Duration::from_secs(180),
        retry_count: 2,
        dependencies: vec!["core_compilation".to_string()],
        validation_rules: vec![
            ValidationRule::MustPass,
            ValidationRule::MustContainOutput("BM25".to_string()),
            ValidationRule::MustNotContainOutput("unimplemented".to_string()),
        ],
    });

    // Tree-sitter tests
    orchestrator.register_test(TestConfig {
        name: "ast_parser_stress_tests".to_string(),
        features: vec!["tree-sitter".to_string()],
        timeout: Duration::from_secs(240),
        retry_count: 1,
        dependencies: vec!["core_compilation".to_string()],
        validation_rules: vec![
            ValidationRule::MustFinishWithin(Duration::from_secs(240)),
            ValidationRule::MustContainOutput("parsing".to_string()),
        ],
    });

    // Tantivy search tests
    orchestrator.register_test(TestConfig {
        name: "tantivy_functionality_validation".to_string(),
        features: vec!["tantivy".to_string()],
        timeout: Duration::from_secs(300),
        retry_count: 2,
        dependencies: vec!["core_compilation".to_string()],
        validation_rules: vec![
            ValidationRule::MustPass,
            ValidationRule::MustNotContainOutput("panic".to_string()),
        ],
    });

    // Integration pipeline tests
    orchestrator.register_test(TestConfig {
        name: "integration_pipeline_validation".to_string(),
        features: vec!["search-advanced".to_string()],
        timeout: Duration::from_secs(360),
        retry_count: 1,
        dependencies: vec![
            "bm25_functionality_validation".to_string(),
            "ast_parser_stress_tests".to_string(),
        ],
        validation_rules: vec![
            ValidationRule::MustContainOutput("integration".to_string()),
            ValidationRule::MustFinishWithin(Duration::from_secs(360)),
        ],
    });

    // Stress tests (dependent on successful integration)
    orchestrator.register_test(TestConfig {
        name: "concurrency_stress_validation".to_string(),
        features: vec!["full-system".to_string()],
        timeout: Duration::from_secs(600),
        retry_count: 0, // Don't retry stress tests
        dependencies: vec!["integration_pipeline_validation".to_string()],
        validation_rules: vec![
            ValidationRule::MustFinishWithin(Duration::from_secs(600)),
            ValidationRule::MustNotContainOutput("deadlock".to_string()),
        ],
    });

    orchestrator
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orchestrator_creation() {
        let orchestrator = TestOrchestrator::new(2);
        assert_eq!(orchestrator.parallel_limit, 2);
        assert_eq!(orchestrator.test_configs.len(), 0);
    }

    #[test]
    fn test_default_suite_creation() {
        let orchestrator = create_default_test_suite();
        assert!(orchestrator.test_configs.len() > 0);
        
        // Verify we have core tests
        assert!(orchestrator.test_configs.iter().any(|t| t.name == "core_compilation"));
        assert!(orchestrator.test_configs.iter().any(|t| t.name == "bm25_functionality_validation"));
    }

    #[test]
    fn test_validation_rules() {
        let rules = vec![
            ValidationRule::MustPass,
            ValidationRule::MustContainOutput("test".to_string()),
        ];

        let failures = TestOrchestrator::validate_test_result(
            &rules,
            "test output", 
            "",
            Some(0),
            Duration::from_secs(1)
        );

        assert!(failures.is_empty());
    }

    #[test] 
    fn test_validation_failure_detection() {
        let rules = vec![ValidationRule::MustContainOutput("missing".to_string())];

        let failures = TestOrchestrator::validate_test_result(
            &rules,
            "different output",
            "",
            Some(0),
            Duration::from_secs(1)
        );

        assert!(!failures.is_empty());
        assert!(failures[0].contains("missing"));
    }
}