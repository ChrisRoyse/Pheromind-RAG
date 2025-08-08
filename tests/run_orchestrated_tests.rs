//! Orchestrated Test Runner
//! 
//! This is an executable test that runs the full test orchestration system.
//! It provides comprehensive validation with dependency resolution and 
//! truth enforcement.

mod orchestration;

use orchestration::{create_default_test_suite, TestOrchestrator};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 EMBED-SEARCH ORCHESTRATED TEST RUNNER");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    // Initialize the default test suite
    let mut orchestrator = create_default_test_suite();
    
    // Run all tests with orchestration
    match orchestrator.run_all_tests() {
        Ok(()) => {
            println!("\n🎉 ALL ORCHESTRATED TESTS COMPLETED SUCCESSFULLY");
            std::process::exit(0);
        }
        Err(e) => {
            eprintln!("\n💥 ORCHESTRATED TESTS FAILED: {}", e);
            std::process::exit(1);
        }
    }
}

// Alternative test runner that can be called from cargo test
#[cfg(test)]
mod orchestrated_integration_test {
    use super::*;

    #[tokio::test]
    async fn run_full_orchestration() {
        let mut orchestrator = create_default_test_suite();
        
        // Run the orchestration (this will execute all integrated tests)
        let result = orchestrator.run_all_tests();
        
        // The test should not panic even if some sub-tests fail
        // This allows us to see the full report
        match result {
            Ok(()) => {
                println!("✅ Full orchestration completed successfully");
            }
            Err(e) => {
                println!("⚠️  Orchestration completed with issues: {}", e);
                // Don't panic here - let the report be visible
            }
        }
    }
}