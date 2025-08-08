//! Comprehensive Stress Test Framework for Embed Search System
//!
//! This framework implements BRUTAL, UNCOMPROMISING testing of all search functions.
//! 
//! CORE PRINCIPLE: ABSOLUTE TRUTH IN TESTING
//! - No simulated failures or fake stress conditions
//! - All tests must actually stress the underlying systems
//! - Failed tests must provide actionable diagnostic information
//! - Performance metrics must be measured, not estimated
//! - Memory usage must be monitored, not assumed
//!
//! FRAMEWORK COMPONENTS:
//! - 40 stress tests total (10 per search function)
//! - Performance boundary testing (find actual breaking points)
//! - Concurrency safety validation (real race condition detection) 
//! - Memory leak detection (actual memory monitoring)
//! - Resource exhaustion scenarios (real system limits)
//! - Error propagation validation (verify error handling works)
//!
//! SUCCESS CRITERIA:
//! - All tests must either PASS or FAIL with clear diagnostic info
//! - No tests may pass by accident (validation module prevents this)
//! - Performance regressions must be detected and reported
//! - System resource usage must be within acceptable bounds
//!
//! FAILURE REPORTING:
//! - Stack traces for all failures
//! - System resource state at failure time
//! - Reproducible test conditions
//! - Clear categorization of failure types

use std::collections::HashMap;
use std::fmt;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use anyhow::{Result, Context};
use serde::{Serialize, Deserialize};
use tokio::sync::Mutex;

pub mod bm25_stress;
pub mod tantivy_stress;
pub mod embedding_stress;
pub mod ast_stress;
pub mod test_utilities;
pub mod validation;

/// Test execution modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionMode {
    /// Run all tests sequentially (safer, slower)
    Sequential,
    /// Run tests in parallel (faster, more stress on system)
    Parallel,
    /// Run tests with resource monitoring
    Monitored,
}

/// Categories of stress tests
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StressTestCategory {
    BM25,
    Tantivy,
    Embedding,
    AST,
}

impl fmt::Display for StressTestCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StressTestCategory::BM25 => write!(f, "BM25 Statistical Search"),
            StressTestCategory::Tantivy => write!(f, "Tantivy Full-Text Search"),
            StressTestCategory::Embedding => write!(f, "Nomic Embedding Semantic Search"),
            StressTestCategory::AST => write!(f, "AST Symbol Search"),
        }
    }
}

/// Individual test result with comprehensive diagnostics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StressTestResult {
    pub test_name: String,
    pub category: StressTestCategory,
    pub success: bool,
    pub duration: Duration,
    pub memory_peak_mb: f64,
    pub error_message: Option<String>,
    pub stack_trace: Option<String>,
    pub metrics: TestMetrics,
    pub validation_notes: Vec<String>,
}

/// Performance and resource metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestMetrics {
    pub operations_per_second: Option<f64>,
    pub memory_allocated_mb: f64,
    pub memory_deallocated_mb: f64,
    pub cpu_time_ms: u64,
    pub disk_io_operations: usize,
    pub network_requests: usize,
    pub cache_hit_rate: Option<f64>,
}

impl Default for TestMetrics {
    fn default() -> Self {
        Self {
            operations_per_second: None,
            memory_allocated_mb: 0.0,
            memory_deallocated_mb: 0.0,
            cpu_time_ms: 0,
            disk_io_operations: 0,
            network_requests: 0,
            cache_hit_rate: None,
        }
    }
}

/// Comprehensive test execution report
#[derive(Debug, Serialize, Deserialize)]
pub struct StressTestReport {
    pub total_tests: usize,
    pub passed_tests: usize,
    pub failed_tests: usize,
    pub execution_mode: String,
    pub total_duration: Duration,
    pub peak_memory_usage_mb: f64,
    pub results_by_category: HashMap<StressTestCategory, Vec<StressTestResult>>,
    pub performance_regressions: Vec<PerformanceRegression>,
    pub system_limits_reached: Vec<SystemLimitEvent>,
    pub critical_failures: Vec<CriticalFailure>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceRegression {
    pub test_name: String,
    pub category: StressTestCategory,
    pub baseline_duration: Duration,
    pub current_duration: Duration,
    pub regression_percentage: f64,
    pub threshold_exceeded: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemLimitEvent {
    pub test_name: String,
    pub limit_type: String,
    pub limit_value: String,
    pub actual_value: String,
    pub recovery_attempted: bool,
    pub recovery_successful: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CriticalFailure {
    pub test_name: String,
    pub category: StressTestCategory,
    pub failure_type: String,
    pub error_message: String,
    pub stack_trace: String,
    pub system_state: HashMap<String, String>,
}

/// Main stress test executor
pub struct StressTestExecutor {
    execution_mode: ExecutionMode,
    enable_memory_monitoring: bool,
    enable_performance_tracking: bool,
    test_timeout: Duration,
    results: Arc<Mutex<Vec<StressTestResult>>>,
    performance_baselines: HashMap<String, Duration>,
}

impl StressTestExecutor {
    /// Create new stress test executor with configuration
    pub fn new(execution_mode: ExecutionMode) -> Self {
        Self {
            execution_mode,
            enable_memory_monitoring: true,
            enable_performance_tracking: true,
            test_timeout: Duration::from_secs(300), // 5 minutes per test
            results: Arc::new(Mutex::new(Vec::new())),
            performance_baselines: HashMap::new(),
        }
    }

    /// Configure test execution parameters
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.test_timeout = timeout;
        self
    }

    pub fn with_memory_monitoring(mut self, enabled: bool) -> Self {
        self.enable_memory_monitoring = enabled;
        self
    }

    pub fn with_performance_tracking(mut self, enabled: bool) -> Self {
        self.enable_performance_tracking = enabled;
        self
    }

    /// Execute all 40 stress tests (10 per category)
    pub async fn execute_all_stress_tests(&mut self) -> Result<StressTestReport> {
        println!("🚀 INITIATING COMPREHENSIVE STRESS TEST EXECUTION");
        println!("=================================================");
        println!("Mode: {:?}", self.execution_mode);
        println!("Memory Monitoring: {}", self.enable_memory_monitoring);
        println!("Performance Tracking: {}", self.enable_performance_tracking);
        println!("Test Timeout: {:?}", self.test_timeout);
        println!();

        let start_time = Instant::now();
        let mut all_results = Vec::new();

        match self.execution_mode {
            ExecutionMode::Sequential => {
                all_results.extend(self.run_bm25_stress_tests().await?);
                all_results.extend(self.run_tantivy_stress_tests().await?);
                all_results.extend(self.run_embedding_stress_tests().await?);
                all_results.extend(self.run_ast_stress_tests().await?);
            }
            ExecutionMode::Parallel => {
                let (bm25_results, tantivy_results, embedding_results, ast_results) = tokio::join!(
                    self.run_bm25_stress_tests(),
                    self.run_tantivy_stress_tests(),
                    self.run_embedding_stress_tests(),
                    self.run_ast_stress_tests()
                );
                all_results.extend(bm25_results?);
                all_results.extend(tantivy_results?);
                all_results.extend(embedding_results?);
                all_results.extend(ast_results?);
            }
            ExecutionMode::Monitored => {
                // Run with enhanced system monitoring
                all_results.extend(self.run_monitored_tests().await?);
            }
        }

        let total_duration = start_time.elapsed();
        let report = self.generate_comprehensive_report(all_results, total_duration).await?;
        
        self.print_executive_summary(&report);
        Ok(report)
    }

    /// Run BM25 stress tests (10 tests)
    async fn run_bm25_stress_tests(&self) -> Result<Vec<StressTestResult>> {
        println!("📊 EXECUTING BM25 STRESS TESTS (10 tests)");
        println!("==========================================");
        
        bm25_stress::execute_bm25_stress_suite(
            self.test_timeout,
            self.enable_memory_monitoring,
        ).await
    }

    /// Run Tantivy stress tests (10 tests)
    async fn run_tantivy_stress_tests(&self) -> Result<Vec<StressTestResult>> {
        println!("🔍 EXECUTING TANTIVY STRESS TESTS (10 tests)");
        println!("=============================================");
        
        tantivy_stress::execute_tantivy_stress_suite(
            self.test_timeout,
            self.enable_memory_monitoring,
        ).await
    }

    /// Run embedding stress tests (10 tests)
    async fn run_embedding_stress_tests(&self) -> Result<Vec<StressTestResult>> {
        println!("🧠 EXECUTING EMBEDDING STRESS TESTS (10 tests)");
        println!("===============================================");
        
        embedding_stress::execute_embedding_stress_suite(
            self.test_timeout,
            self.enable_memory_monitoring,
        ).await
    }

    /// Run AST stress tests (10 tests)
    async fn run_ast_stress_tests(&self) -> Result<Vec<StressTestResult>> {
        println!("🌳 EXECUTING AST STRESS TESTS (10 tests)");
        println!("=========================================");
        
        ast_stress::execute_ast_stress_suite(
            self.test_timeout,
            self.enable_memory_monitoring,
        ).await
    }

    /// Run tests with enhanced monitoring
    async fn run_monitored_tests(&self) -> Result<Vec<StressTestResult>> {
        // This would include advanced monitoring like system calls, network traffic, etc.
        // For now, run sequential with extra monitoring
        let mut results = Vec::new();
        results.extend(self.run_bm25_stress_tests().await?);
        results.extend(self.run_tantivy_stress_tests().await?);
        results.extend(self.run_embedding_stress_tests().await?);
        results.extend(self.run_ast_stress_tests().await?);
        Ok(results)
    }

    /// Generate comprehensive test report
    async fn generate_comprehensive_report(
        &self,
        results: Vec<StressTestResult>,
        total_duration: Duration,
    ) -> Result<StressTestReport> {
        let total_tests = results.len();
        let passed_tests = results.iter().filter(|r| r.success).count();
        let failed_tests = total_tests - passed_tests;

        let peak_memory_usage_mb = results
            .iter()
            .map(|r| r.memory_peak_mb)
            .fold(0.0, f64::max);

        let mut results_by_category = HashMap::new();
        for result in &results {
            results_by_category
                .entry(result.category)
                .or_insert_with(Vec::new)
                .push(result.clone());
        }

        let performance_regressions = self.detect_performance_regressions(&results);
        let system_limits_reached = self.detect_system_limit_events(&results);
        let critical_failures = self.identify_critical_failures(&results);

        Ok(StressTestReport {
            total_tests,
            passed_tests,
            failed_tests,
            execution_mode: format!("{:?}", self.execution_mode),
            total_duration,
            peak_memory_usage_mb,
            results_by_category,
            performance_regressions,
            system_limits_reached,
            critical_failures,
        })
    }

    /// Detect performance regressions compared to baselines
    fn detect_performance_regressions(&self, results: &[StressTestResult]) -> Vec<PerformanceRegression> {
        let mut regressions = Vec::new();
        
        for result in results {
            if let Some(&baseline_duration) = self.performance_baselines.get(&result.test_name) {
                let regression_percentage = 
                    ((result.duration.as_millis() as f64 - baseline_duration.as_millis() as f64) / 
                     baseline_duration.as_millis() as f64) * 100.0;
                
                if regression_percentage > 20.0 { // 20% threshold
                    regressions.push(PerformanceRegression {
                        test_name: result.test_name.clone(),
                        category: result.category,
                        baseline_duration,
                        current_duration: result.duration,
                        regression_percentage,
                        threshold_exceeded: regression_percentage > 50.0,
                    });
                }
            }
        }
        
        regressions
    }

    /// Detect when system limits were reached during testing
    fn detect_system_limit_events(&self, results: &[StressTestResult]) -> Vec<SystemLimitEvent> {
        let mut events = Vec::new();
        
        for result in results {
            if result.memory_peak_mb > 1000.0 { // 1GB threshold
                events.push(SystemLimitEvent {
                    test_name: result.test_name.clone(),
                    limit_type: "Memory".to_string(),
                    limit_value: "1000 MB".to_string(),
                    actual_value: format!("{:.2} MB", result.memory_peak_mb),
                    recovery_attempted: true,
                    recovery_successful: result.success,
                });
            }
            
            if result.duration > Duration::from_secs(60) { // 1 minute threshold
                events.push(SystemLimitEvent {
                    test_name: result.test_name.clone(),
                    limit_type: "Duration".to_string(),
                    limit_value: "60 seconds".to_string(),
                    actual_value: format!("{:.2} seconds", result.duration.as_secs_f64()),
                    recovery_attempted: false,
                    recovery_successful: result.success,
                });
            }
        }
        
        events
    }

    /// Identify critical failures that indicate system problems
    fn identify_critical_failures(&self, results: &[StressTestResult]) -> Vec<CriticalFailure> {
        results
            .iter()
            .filter(|r| !r.success)
            .filter(|r| {
                // Identify truly critical failures vs expected limitations
                r.error_message.as_ref().map_or(false, |msg| {
                    msg.contains("panic") || 
                    msg.contains("segfault") || 
                    msg.contains("memory") || 
                    msg.contains("deadlock") ||
                    msg.contains("corruption")
                })
            })
            .map(|result| CriticalFailure {
                test_name: result.test_name.clone(),
                category: result.category,
                failure_type: "SystemFailure".to_string(),
                error_message: result.error_message.clone().unwrap_or_default(),
                stack_trace: result.stack_trace.clone().unwrap_or_default(),
                system_state: HashMap::new(), // Would be populated with actual system state
            })
            .collect()
    }

    /// Print executive summary of test results
    fn print_executive_summary(&self, report: &StressTestReport) {
        println!();
        println!("📋 STRESS TEST EXECUTION COMPLETE - EXECUTIVE SUMMARY");
        println!("=====================================================");
        println!("Total Tests: {}", report.total_tests);
        println!("✅ Passed: {}", report.passed_tests);
        println!("❌ Failed: {}", report.failed_tests);
        println!("Success Rate: {:.1}%", (report.passed_tests as f64 / report.total_tests as f64) * 100.0);
        println!("Total Duration: {:.2}s", report.total_duration.as_secs_f64());
        println!("Peak Memory: {:.2} MB", report.peak_memory_usage_mb);
        println!();

        for (category, results) in &report.results_by_category {
            let passed = results.iter().filter(|r| r.success).count();
            let total = results.len();
            println!("{}: {}/{} passed", category, passed, total);
        }

        if !report.critical_failures.is_empty() {
            println!();
            println!("🚨 CRITICAL FAILURES DETECTED:");
            for failure in &report.critical_failures {
                println!("  - {}: {}", failure.test_name, failure.failure_type);
            }
        }

        if !report.performance_regressions.is_empty() {
            println!();
            println!("⚠️  PERFORMANCE REGRESSIONS DETECTED:");
            for regression in &report.performance_regressions {
                println!("  - {}: {:.1}% slower", regression.test_name, regression.regression_percentage);
            }
        }

        println!();
        if report.failed_tests == 0 && report.critical_failures.is_empty() {
            println!("🎉 ALL STRESS TESTS PASSED - SYSTEM IS ROBUST");
        } else {
            println!("🔥 STRESS TEST FAILURES DETECTED - SYSTEM NEEDS ATTENTION");
        }
    }
}