//! Test Library for Embed Search System
//!
//! This module provides centralized test infrastructure and shared utilities
//! for all tests in the embed-search project.
//!
//! ## Module Architecture
//! 
//! - `stress_test_framework/`: Comprehensive stress testing framework
//! - `fixtures/`: Shared test data and reference implementations
//! - `integration/`: Integration test suites
//! - Individual test files: Specific functionality tests
//!
//! ## Import Strategy
//! 
//! All test modules can access shared functionality via `crate::` imports,
//! following Rust's standard module resolution patterns.

#![allow(dead_code)]
#![allow(unused_imports)]

// Re-export the main embed-search library for tests
pub use embed_search::*;

// Test framework modules
pub mod stress_test_framework;

// Test fixtures and utilities  
pub mod fixtures {
    pub mod reference_embeddings;
    pub mod semantic_similarity_benchmarks;
    
    // Load test configuration
    pub fn load_test_config() -> embed_search::config::Config {
        // Initialize config if needed
        if let Err(_) = embed_search::config::Config::init() {
            // Config already initialized, that's ok
        }
        embed_search::config::Config::get().expect("Failed to get test configuration")
    }
}

// Common test utilities
pub mod test_utils {
    use std::path::PathBuf;
    use tempfile::TempDir;
    
    pub fn create_temp_dir() -> TempDir {
        TempDir::new().expect("Failed to create temporary directory")
    }
    
    pub fn get_test_data_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("stress_test_environment")
    }
}

// Performance testing utilities
pub mod performance {
    use std::time::{Duration, Instant};
    use serde::{Serialize, Deserialize};
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct PerformanceBaseline {
        pub test_name: String,
        pub baseline_duration: Duration,
        pub memory_usage_mb: f64,
        pub operations_per_second: f64,
        pub timestamp: chrono::DateTime<chrono::Utc>,
    }
    
    impl PerformanceBaseline {
        pub fn new(test_name: String, duration: Duration, memory_mb: f64, ops_per_sec: f64) -> Self {
            Self {
                test_name,
                baseline_duration: duration,
                memory_usage_mb: memory_mb,
                operations_per_second: ops_per_sec,
                timestamp: chrono::Utc::now(),
            }
        }
    }
    
    pub fn measure_performance<F, R>(test_name: &str, f: F) -> (R, PerformanceBaseline)
    where
        F: FnOnce() -> R,
    {
        let start = Instant::now();
        let result = f();
        let duration = start.elapsed();
        
        // Simple memory estimation (would use actual measurement in production)
        let memory_mb = 10.0; // Placeholder
        let ops_per_sec = 1000.0 / duration.as_millis().max(1u128) as f64;
        
        let baseline = PerformanceBaseline::new(
            test_name.to_string(),
            duration,
            memory_mb,
            ops_per_sec,
        );
        
        (result, baseline)
    }
}

// Math utilities for tests
pub mod math_utils {
    /// Safe max function for floats that handles type inference issues
    pub fn safe_max(a: f64, b: f64) -> f64 {
        a.max(b)
    }
    
    /// Calculate percentage difference between two values
    pub fn percentage_diff(old_value: f64, new_value: f64) -> f64 {
        if old_value == 0.0 {
            return if new_value == 0.0 { 0.0 } else { 100.0 };
        }
        ((new_value - old_value) / old_value.abs()) * 100.0
    }
}

// Re-export commonly used test dependencies
pub use anyhow::{Result, Context};
pub use serde::{Serialize, Deserialize};
pub use serde_json;
pub use tokio;
pub use std::time::{Duration, Instant};
pub use std::collections::HashMap;
pub use tempfile;
pub use chrono;