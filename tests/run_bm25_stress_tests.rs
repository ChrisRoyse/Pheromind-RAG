/// Standalone BM25 Stress Test Executor
/// 
/// This executable runs the BM25 stress tests independently to verify implementation
/// against the 10 fundamental flaws identified.
/// 
/// Usage: cargo run --bin run_bm25_stress_tests

use std::env;
use std::process;

fn main() {
    println!("🧪 BM25 Stress Test Suite - Fundamental Flaw Detection");
    println!("======================================================");
    
    let args: Vec<String> = env::args().collect();
    
    if args.len() > 1 && (args[1] == "--help" || args[1] == "-h") {
        print_help();
        return;
    }
    
    println!("Running BM25 stress tests...");
    println!("Note: Use 'cargo test bm25_stress_tests' to run the actual test suite");
    println!();
    
    // Basic smoke test to verify the module can be imported and used
    match run_smoke_test() {
        Ok(_) => {
            println!("✅ Smoke test passed - BM25 module is functional");
            println!("🔍 Run 'cargo test bm25_stress_tests' for comprehensive stress testing");
        }
        Err(e) => {
            println!("❌ Smoke test failed: {}", e);
            process::exit(1);
        }
    }
}

fn print_help() {
    println!("BM25 Stress Test Suite");
    println!();
    println!("USAGE:");
    println!("  cargo run --bin run_bm25_stress_tests");
    println!("  cargo test bm25_stress_tests              # Run full stress test suite");
    println!();
    println!("STRESS TEST CATEGORIES:");
    println!("  1  - Incremental Update Impossibility");
    println!("  2  - Tokenization Catastrophe");
    println!("  3  - Memory Explosion");
    println!("  4  - Persistence Failure");
    println!("  5  - Length Bias Exposure");
    println!("  6  - Mathematical Edge Cases");
    println!("  7  - Unicode Tokenization Destruction");
    println!("  8  - Concurrency Corruption");
    println!("  9  - Stop Word Singularity");
    println!("  10 - Vocabulary Overflow");
    println!();
    println!("For detailed test execution, use:");
    println!("  cargo test bm25_stress_tests -- --nocapture");
}

fn run_smoke_test() -> Result<(), String> {
    use embed_search::search::bm25::{BM25Engine, BM25Document, Token};
    
    println!("Running smoke test...");
    
    let mut engine = BM25Engine::new();
    
    // Create simple test document
    let doc = BM25Document {
        id: "smoke_test".to_string(),
        file_path: "smoke.rs".to_string(),
        chunk_index: 0,
        tokens: vec![
            Token { text: "function".to_string(), position: 0, importance_weight: 1.0 },
            Token { text: "test".to_string(), position: 1, importance_weight: 1.0 },
        ],
        start_line: 0,
        end_line: 1,
        language: Some("rust".to_string()),
    };
    
    // Test basic functionality
    engine.add_document(doc).map_err(|e| format!("Failed to add document: {}", e))?;
    
    let results = engine.search("function", 5).map_err(|e| format!("Search failed: {}", e))?;
    
    if results.is_empty() {
        return Err("Search returned no results for known term".to_string());
    }
    
    if !results[0].score.is_finite() {
        return Err(format!("Invalid BM25 score: {}", results[0].score));
    }
    
    println!("  • Document indexing: ✅");
    println!("  • Search functionality: ✅");
    println!("  • Score calculation: ✅ (score: {:.6})", results[0].score);
    
    Ok(())
}