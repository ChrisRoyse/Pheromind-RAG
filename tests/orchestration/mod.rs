//! Test Orchestration Module
//! 
//! This module provides comprehensive test orchestration capabilities
//! for the embed-search system, including dependency resolution,
//! parallel execution, and truth enforcement validation.

pub mod test_orchestrator;

pub use test_orchestrator::{
    TestOrchestrator,
    TestConfig, 
    ValidationRule,
    TestResult,
    create_default_test_suite,
};

#[cfg(test)]
mod integration_tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_orchestration_system() {
        // This test validates the orchestration system itself
        let orchestrator = create_default_test_suite();
        
        // Verify test configuration
        assert!(orchestrator.test_configs.len() >= 5);
        
        // Verify dependency chains exist
        let has_dependencies = orchestrator.test_configs
            .iter()
            .any(|config| !config.dependencies.is_empty());
        assert!(has_dependencies, "Test suite should have dependency chains");
        
        // Verify timeout configurations are reasonable
        for config in &orchestrator.test_configs {
            assert!(config.timeout <= Duration::from_secs(900), 
                "Test timeout {} should be reasonable (< 15 minutes)", config.name);
            assert!(config.timeout >= Duration::from_secs(30),
                "Test timeout {} should not be too short (> 30 seconds)", config.name);
        }
    }
}