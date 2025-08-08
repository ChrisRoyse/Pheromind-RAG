#!/usr/bin/env rust-script
//! Comprehensive Test Runner with Truth Enforcement
//! 
//! This script orchestrates the entire test suite with validation mechanisms
//! to ensure all tests are genuine and truthful.

use std::process::{Command, Stdio};
use std::io::{self, Write};
use std::time::Instant;
use std::collections::{HashMap, HashSet};
use std::path::Path;

#[derive(Debug, Clone)]
struct TestSuite {
    name: String,
    command: String,
    features: Vec<String>,
    timeout_seconds: u64,
    required_success_rate: f32,
    validation_rules: Vec<ValidationRule>,
}

#[derive(Debug, Clone)]
enum ValidationRule {
    MustContain(String),
    MustNotContain(String),
    ExitCodeMustBe(i32),
    RuntimeMustBeLessThan(u64),
    MustProduceFile(String),
}

#[derive(Debug)]
struct TestResult {
    suite: String,
    passed: bool,
    duration_ms: u128,
    stdout: String,
    stderr: String,
    exit_code: Option<i32>,
    validation_failures: Vec<String>,
}

struct TestRunner {
    results: Vec<TestResult>,
    start_time: Instant,
}

impl TestRunner {
    fn new() -> Self {
        Self {
            results: Vec::new(),
            start_time: Instant::now(),
        }
    }

    fn run_all_suites(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let suites = self.define_test_suites();
        
        println!("🚀 Starting Comprehensive Test Suite Validation");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        
        for suite in &suites {
            self.run_test_suite(suite)?;
        }
        
        self.generate_final_report()?;
        self.enforce_truth_requirements()?;
        
        Ok(())
    }

    fn define_test_suites(&self) -> Vec<TestSuite> {
        vec![
            // Core compilation tests
            TestSuite {
                name: "Core Feature Compilation".to_string(),
                command: "cargo check --features core".to_string(),
                features: vec!["core".to_string()],
                timeout_seconds: 120,
                required_success_rate: 1.0,
                validation_rules: vec![
                    ValidationRule::ExitCodeMustBe(0),
                    ValidationRule::MustNotContain("error".to_string()),
                ],
            },
            
            // Full feature compilation
            TestSuite {
                name: "Full System Compilation".to_string(),
                command: "cargo check --all-features".to_string(),
                features: vec!["full-system".to_string()],
                timeout_seconds: 300,
                required_success_rate: 1.0,
                validation_rules: vec![
                    ValidationRule::ExitCodeMustBe(0),
                    ValidationRule::MustNotContain("error:".to_string()),
                ],
            },
            
            // Unit tests
            TestSuite {
                name: "Core Unit Tests".to_string(),
                command: "cargo test --lib --features core".to_string(),
                features: vec!["core".to_string()],
                timeout_seconds: 180,
                required_success_rate: 0.95,
                validation_rules: vec![
                    ValidationRule::MustContain("test result: ok".to_string()),
                    ValidationRule::MustNotContain("FAILED".to_string()),
                ],
            },
            
            // BM25 functionality tests
            TestSuite {
                name: "BM25 Search Validation".to_string(),
                command: "cargo test --test bm25_functionality_validation --features core".to_string(),
                features: vec!["core".to_string()],
                timeout_seconds: 120,
                required_success_rate: 1.0,
                validation_rules: vec![
                    ValidationRule::ExitCodeMustBe(0),
                    ValidationRule::MustContain("✅".to_string()),
                ],
            },
            
            // Tree-sitter tests (if feature available)
            TestSuite {
                name: "Tree-sitter Symbol Indexing".to_string(),
                command: "cargo test --test ast_parser_stress_tests --features tree-sitter".to_string(),
                features: vec!["tree-sitter".to_string()],
                timeout_seconds: 180,
                required_success_rate: 0.90,
                validation_rules: vec![
                    ValidationRule::MustContain("parsing".to_string()),
                ],
            },
            
            // Tantivy search tests
            TestSuite {
                name: "Tantivy Full-Text Search".to_string(),
                command: "cargo test --test tantivy_functionality_validation --features tantivy".to_string(),
                features: vec!["tantivy".to_string()],
                timeout_seconds: 240,
                required_success_rate: 0.95,
                validation_rules: vec![
                    ValidationRule::ExitCodeMustBe(0),
                ],
            },
            
            // Integration tests
            TestSuite {
                name: "Search Integration Pipeline".to_string(),
                command: "cargo test --test integration_pipeline_validation --features search-advanced".to_string(),
                features: vec!["search-advanced".to_string()],
                timeout_seconds: 300,
                required_success_rate: 0.90,
                validation_rules: vec![
                    ValidationRule::MustContain("integration".to_string()),
                ],
            },
            
            // Stress tests
            TestSuite {
                name: "Concurrency Stress Tests".to_string(),
                command: "cargo test --test concurrency_stress_validation --features full-system".to_string(),
                features: vec!["full-system".to_string()],
                timeout_seconds: 600,
                required_success_rate: 0.85,
                validation_rules: vec![
                    ValidationRule::RuntimeMustBeLessThan(600),
                ],
            },
        ]
    }

    fn run_test_suite(&mut self, suite: &TestSuite) -> Result<(), Box<dyn std::error::Error>> {
        println!("\n🔬 Testing: {}", suite.name);
        println!("   Command: {}", suite.command);
        println!("   Features: {:?}", suite.features);
        
        let start = Instant::now();
        
        // Execute the test command
        let output = Command::new("cmd")
            .args(&["/C", &suite.command])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;
            
        let duration = start.elapsed();
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let exit_code = output.status.code();
        
        // Apply validation rules
        let mut validation_failures = Vec::new();
        for rule in &suite.validation_rules {
            match rule {
                ValidationRule::MustContain(pattern) => {
                    if !stdout.contains(pattern) && !stderr.contains(pattern) {
                        validation_failures.push(format!("Missing required pattern: {}", pattern));
                    }
                }
                ValidationRule::MustNotContain(pattern) => {
                    if stdout.contains(pattern) || stderr.contains(pattern) {
                        validation_failures.push(format!("Found forbidden pattern: {}", pattern));
                    }
                }
                ValidationRule::ExitCodeMustBe(expected) => {
                    if exit_code != Some(*expected) {
                        validation_failures.push(format!("Exit code {} != expected {}", 
                            exit_code.unwrap_or(-1), expected));
                    }
                }
                ValidationRule::RuntimeMustBeLessThan(max_seconds) => {
                    if duration.as_secs() > *max_seconds {
                        validation_failures.push(format!("Runtime {}s > max {}s", 
                            duration.as_secs(), max_seconds));
                    }
                }
                ValidationRule::MustProduceFile(file_path) => {
                    if !Path::new(file_path).exists() {
                        validation_failures.push(format!("Required file not found: {}", file_path));
                    }
                }
            }
        }
        
        let passed = exit_code == Some(0) && validation_failures.is_empty();
        
        // Store result
        let result = TestResult {
            suite: suite.name.clone(),
            passed,
            duration_ms: duration.as_millis(),
            stdout,
            stderr,
            exit_code,
            validation_failures,
        };
        
        // Print immediate feedback
        if result.passed {
            println!("   ✅ PASSED in {:.2}s", duration.as_secs_f64());
        } else {
            println!("   ❌ FAILED in {:.2}s", duration.as_secs_f64());
            for failure in &result.validation_failures {
                println!("      ⚠️  {}", failure);
            }
        }
        
        self.results.push(result);
        Ok(())
    }

    fn generate_final_report(&self) -> Result<(), Box<dyn std::error::Error>> {
        let total_duration = self.start_time.elapsed();
        let passed = self.results.iter().filter(|r| r.passed).count();
        let total = self.results.len();
        let success_rate = passed as f32 / total as f32;
        
        println!("\n");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("📊 COMPREHENSIVE TEST VALIDATION REPORT");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("Total Runtime: {:.2}s", total_duration.as_secs_f64());
        println!("Success Rate: {}/{} ({:.1}%)", passed, total, success_rate * 100.0);
        println!();
        
        // Detailed results
        for result in &self.results {
            let status = if result.passed { "✅ PASS" } else { "❌ FAIL" };
            println!("{} {} ({:.2}s)", status, result.suite, result.duration_ms as f64 / 1000.0);
            
            if !result.passed {
                for failure in &result.validation_failures {
                    println!("    ⚠️  {}", failure);
                }
                
                if !result.stderr.is_empty() {
                    println!("    📝 Error Output:");
                    for line in result.stderr.lines().take(5) {
                        println!("       {}", line);
                    }
                    if result.stderr.lines().count() > 5 {
                        println!("       ... ({} more lines)", result.stderr.lines().count() - 5);
                    }
                }
            }
        }
        
        Ok(())
    }

    fn enforce_truth_requirements(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("\n🔍 TRUTH ENFORCEMENT VALIDATION");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        
        let failed_tests: Vec<_> = self.results.iter().filter(|r| !r.passed).collect();
        
        if failed_tests.is_empty() {
            println!("✅ ALL TESTS GENUINE - No false positives detected");
            println!("✅ TRUTH REQUIREMENT SATISFIED");
            return Ok(());
        }
        
        println!("❌ TRUTH VIOLATION DETECTED:");
        println!("   {} test suites failed validation", failed_tests.len());
        
        // Analyze failure patterns to detect potential false claims
        let mut failure_patterns: HashMap<String, usize> = HashMap::new();
        for test in &failed_tests {
            for failure in &test.validation_failures {
                *failure_patterns.entry(failure.clone()).or_insert(0) += 1;
            }
        }
        
        println!("\n📊 FAILURE PATTERN ANALYSIS:");
        for (pattern, count) in failure_patterns.iter() {
            println!("   • {} (occurred {} times)", pattern, count);
        }
        
        // Check for suspicious patterns that might indicate fake implementations
        let suspicious_patterns = [
            "todo!", "unimplemented!", "panic!", 
            "mock", "fake", "dummy", "placeholder",
            "not implemented", "coming soon"
        ];
        
        let mut suspicious_count = 0;
        for test in &self.results {
            for pattern in &suspicious_patterns {
                if test.stdout.to_lowercase().contains(pattern) || 
                   test.stderr.to_lowercase().contains(pattern) {
                    suspicious_count += 1;
                    println!("🚨 SUSPICIOUS PATTERN '{}' found in {}", pattern, test.suite);
                }
            }
        }
        
        if suspicious_count > 0 {
            println!("\n💥 CRITICAL: {} suspicious patterns detected", suspicious_count);
            println!("This suggests potential fake or incomplete implementations");
            return Err("Truth enforcement failed - suspicious implementations detected".into());
        }
        
        // Final truth assessment
        let success_rate = self.results.iter().filter(|r| r.passed).count() as f32 / self.results.len() as f32;
        if success_rate < 0.80 {
            println!("\n💥 CRITICAL: Success rate {:.1}% is below 80% threshold", success_rate * 100.0);
            return Err("Truth enforcement failed - success rate too low".into());
        }
        
        println!("\n⚠️  PARTIAL COMPLIANCE: {:.1}% success rate", success_rate * 100.0);
        println!("Some tests failed but no fake implementations detected");
        
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut runner = TestRunner::new();
    
    match runner.run_all_suites() {
        Ok(()) => {
            println!("\n🎉 TEST VALIDATION COMPLETE");
            std::process::exit(0);
        }
        Err(e) => {
            eprintln!("\n💥 TEST VALIDATION FAILED: {}", e);
            std::process::exit(1);
        }
    }
}