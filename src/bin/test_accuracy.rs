/// Lightweight accuracy testing binary
/// Run with: cargo run --bin test_accuracy --release

use std::path::PathBuf;
use embed_lib::search::unified::UnifiedSearcher;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🎯 Embedding Search Accuracy Test");
    println!("{}", "=".repeat(80));
    
    let project_root = std::env::current_dir()?;
    let vectortest_path = project_root.join("vectortest");
    let db_path = project_root.join(".accuracy_test_db");
    
    // Clean existing DB
    if db_path.exists() {
        std::fs::remove_dir_all(&db_path).ok();
    }
    
    println!("📚 Initializing search system...");
    let searcher = UnifiedSearcher::new_with_config(
        project_root.clone(),
        db_path,
        true // include all files for testing
    ).await?;
    
    println!("🔄 Indexing vectortest directory...");
    let start = std::time::Instant::now();
    let stats = searcher.index_directory(&vectortest_path).await?;
    let index_time = start.elapsed();
    
    println!("✅ Indexed {} files with {} chunks in {:.2}s", 
             stats.files_indexed, stats.chunks_created, index_time.as_secs_f64());
    
    // Define test queries with expected results
    let test_cases = vec![
        ("database migration SQL", vec!["database_migration.sql"]),
        ("Python authentication", vec!["auth_service.py"]),
        ("Rust cache memory", vec!["memory_cache.rs"]),
        ("TypeScript payment", vec!["payment_gateway.ts"]),
        ("JavaScript controller", vec!["user_controller.js"]),
        ("Go analytics", vec!["analytics_dashboard.go"]),
        ("C++ websocket", vec!["websocket_server.cpp"]),
        ("C# data processor", vec!["DataProcessor.cs"]),
        ("Java order", vec!["OrderService.java"]),
        ("Ruby catalog", vec!["product_catalog.rb"]),
        
        // Semantic searches
        ("API documentation", vec!["API_DOCUMENTATION.md"]),
        ("architecture design", vec!["ARCHITECTURE_OVERVIEW.md"]),
        ("deployment guide", vec!["DEPLOYMENT_GUIDE.md"]),
        ("troubleshooting errors", vec!["TROUBLESHOOTING.md"]),
        ("contributing", vec!["CONTRIBUTING.md"]),
        
        // Cross-language concepts
        ("caching optimization", vec!["memory_cache.rs"]),
        ("user authentication", vec!["auth_service.py"]),
        ("real-time communication", vec!["websocket_server.cpp"]),
        ("payment processing", vec!["payment_gateway.ts"]),
        ("data transformation", vec!["DataProcessor.cs"]),
    ];
    
    println!("\n🔍 Running accuracy tests...\n");
    
    let mut passed = 0;
    let mut total = 0;
    let mut total_accuracy = 0.0;
    
    for (query, expected_files) in test_cases {
        total += 1;
        print!("Test {}: \"{}\" ", total, query);
        
        let start = std::time::Instant::now();
        let results = searcher.search(query).await?;
        let search_time = start.elapsed();
        
        // Check if expected files are in top 5 results
        let mut found = false;
        for expected in &expected_files {
            for (idx, result) in results.iter().enumerate().take(5) {
                if result.file.contains(expected) {
                    found = true;
                    print!("✅ (found at position {}, {:.0}ms)", idx + 1, search_time.as_millis());
                    break;
                }
            }
            if found { break; }
        }
        
        if !found {
            print!("❌ (expected {:?}, got ", expected_files);
            if results.is_empty() {
                print!("no results");
            } else {
                print!("{}", results[0].file.split('/').last().unwrap_or(&results[0].file));
            }
            print!(", {:.0}ms)", search_time.as_millis());
        } else {
            passed += 1;
            total_accuracy += 100.0;
        }
        
        println!();
    }
    
    let accuracy = (passed as f64 / total as f64) * 100.0;
    
    println!("\n{}", "=".repeat(80));
    println!("📊 RESULTS SUMMARY");
    println!("{}", "=".repeat(80));
    println!("Tests Passed: {}/{}", passed, total);
    println!("Accuracy: {:.1}%", accuracy);
    
    if accuracy >= 90.0 {
        println!("🎉 SUCCESS: Search accuracy exceeds 90% threshold!");
    } else {
        println!("⚠️  WARNING: Search accuracy {:.1}% is below 90% threshold", accuracy);
    }
    
    // Clean up test DB
    std::fs::remove_dir_all(&db_path).ok();
    
    Ok(())
}