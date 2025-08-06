/// Ultra-fast accuracy test using lightweight components
/// This avoids heavy LanceDB compilation for quick iteration

use std::path::{Path, PathBuf};
use std::collections::HashMap;
use embed_lib::embedding::CachedEmbedder;
use embed_lib::chunking::SimpleRegexChunker;
use anyhow::Result;

struct TestResult {
    query: String,
    expected: Vec<String>,
    found: Vec<String>,
    accuracy: f32,
    search_time_ms: u128,
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 Fast Accuracy Test - Lightweight Mode");
    println!("{}", "=".repeat(80));
    
    let project_root = std::env::current_dir()?;
    let vectortest_path = project_root.join("vectortest");
    
    // Initialize lightweight components
    println!("📚 Initializing embedder (using singleton)...");
    let embedder = CachedEmbedder::new_with_cache_size(1000).await
        .map_err(|e| anyhow::anyhow!("Failed to create embedder: {}", e))?;
    
    let chunker = SimpleRegexChunker::new();
    
    // Index vectortest directory into memory
    println!("🔄 Indexing vectortest directory...");
    let start = std::time::Instant::now();
    
    let mut file_embeddings: HashMap<String, Vec<(Vec<f32>, String)>> = HashMap::new();
    let mut total_chunks = 0;
    
    // Process each file in vectortest
    for entry in std::fs::read_dir(&vectortest_path)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() {
            let file_name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();
            
            if let Ok(content) = std::fs::read_to_string(&path) {
                let chunks = chunker.chunk_file(&content);
                let mut file_chunks = Vec::new();
                
                for chunk in chunks {
                    if let Ok(embedding) = embedder.embed(&chunk.content) {
                        file_chunks.push((embedding, chunk.content));
                        total_chunks += 1;
                    }
                }
                
                file_embeddings.insert(file_name, file_chunks);
            }
        }
    }
    
    let index_time = start.elapsed();
    println!("✅ Indexed {} files with {} chunks in {:.2}s", 
             file_embeddings.len(), total_chunks, index_time.as_secs_f64());
    
    // Define test queries
    let test_cases = vec![
        ("database migration SQL", vec!["database_migration.sql"]),
        ("Python authentication service", vec!["auth_service.py"]),
        ("Rust memory cache", vec!["memory_cache.rs"]),
        ("TypeScript payment gateway", vec!["payment_gateway.ts"]),
        ("JavaScript user controller", vec!["user_controller.js"]),
        ("Go analytics dashboard", vec!["analytics_dashboard.go"]),
        ("C++ websocket server", vec!["websocket_server.cpp"]),
        ("C# data processor", vec!["DataProcessor.cs"]),
        ("Java order service", vec!["OrderService.java"]),
        ("Ruby product catalog", vec!["product_catalog.rb"]),
        
        // Documentation searches
        ("API documentation", vec!["API_DOCUMENTATION.md"]),
        ("architecture overview", vec!["ARCHITECTURE_OVERVIEW.md"]),
        ("deployment guide", vec!["DEPLOYMENT_GUIDE.md"]),
        ("troubleshooting", vec!["TROUBLESHOOTING.md"]),
        ("contributing", vec!["CONTRIBUTING.md"]),
        
        // Semantic concept searches
        ("caching optimization memory", vec!["memory_cache.rs"]),
        ("user authentication security", vec!["auth_service.py"]),
        ("real-time websocket communication", vec!["websocket_server.cpp"]),
        ("payment processing transactions", vec!["payment_gateway.ts"]),
        ("data transformation pipeline", vec!["DataProcessor.cs"]),
    ];
    
    println!("\n🔍 Running accuracy tests...\n");
    
    let mut test_results = Vec::new();
    
    for (query, expected_files) in test_cases {
        let start = std::time::Instant::now();
        
        // Generate query embedding
        let query_embedding = embedder.embed(query)
            .map_err(|e| anyhow::anyhow!("Failed to embed query: {}", e))?;
        
        // Search across all files
        let mut all_scores: Vec<(f32, String)> = Vec::new();
        
        for (file_name, chunks) in &file_embeddings {
            let mut best_score = 0.0_f32;
            
            for (chunk_embedding, _content) in chunks {
                let score = cosine_similarity(&query_embedding, chunk_embedding);
                if score > best_score {
                    best_score = score;
                }
            }
            
            all_scores.push((best_score, file_name.clone()));
        }
        
        // Sort by score (descending)
        all_scores.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        
        let search_time = start.elapsed();
        
        // Check if expected files are in top 5
        let top_5: Vec<String> = all_scores.iter()
            .take(5)
            .map(|(_, name)| name.clone())
            .collect();
        
        let mut found = Vec::new();
        for expected in &expected_files {
            if top_5.iter().any(|name| name.contains(expected)) {
                found.push(expected.to_string());
            }
        }
        
        let accuracy = (found.len() as f32 / expected_files.len() as f32) * 100.0;
        
        test_results.push(TestResult {
            query: query.to_string(),
            expected: expected_files.into_iter().map(|s| s.to_string()).collect(),
            found: found.clone(),
            accuracy,
            search_time_ms: search_time.as_millis(),
        });
        
        // Print inline result
        print!("Test: \"{}\" ", query);
        if accuracy == 100.0 {
            println!("✅ ({:.0}ms)", search_time.as_millis());
        } else if accuracy > 0.0 {
            println!("⚠️  {:.0}% ({:.0}ms)", accuracy, search_time.as_millis());
        } else {
            println!("❌ ({:.0}ms) - got {}", search_time.as_millis(), 
                     if all_scores.is_empty() { "no results".to_string() } else { all_scores[0].1.clone() });
        }
    }
    
    // Calculate overall metrics
    let total_tests = test_results.len();
    let passed_tests = test_results.iter().filter(|r| r.accuracy == 100.0).count();
    let avg_accuracy: f32 = test_results.iter().map(|r| r.accuracy).sum::<f32>() / total_tests as f32;
    let avg_time: u128 = test_results.iter().map(|r| r.search_time_ms).sum::<u128>() / total_tests as u128;
    
    println!("\n{}", "=".repeat(80));
    println!("📊 ACCURACY TEST RESULTS");
    println!("{}", "=".repeat(80));
    println!("Total Tests: {}", total_tests);
    println!("Fully Passed: {}/{} ({:.1}%)", passed_tests, total_tests, 
             (passed_tests as f32 / total_tests as f32) * 100.0);
    println!("Average Accuracy: {:.1}%", avg_accuracy);
    println!("Average Search Time: {}ms", avg_time);
    
    // Show failed tests
    let failed_tests: Vec<_> = test_results.iter()
        .filter(|r| r.accuracy < 100.0)
        .collect();
    
    if !failed_tests.is_empty() {
        println!("\n⚠️  Partially Failed Tests:");
        for test in failed_tests {
            println!("  - \"{}\" expected {:?}, found {:?} ({:.0}%)", 
                     test.query, test.expected, test.found, test.accuracy);
        }
    }
    
    // Final verdict
    println!("\n{}", "=".repeat(80));
    if avg_accuracy >= 90.0 {
        println!("🎉 SUCCESS: Search accuracy {:.1}% exceeds 90% threshold!", avg_accuracy);
        println!("✨ Grade: A+ - System achieves excellent search accuracy!");
    } else if avg_accuracy >= 80.0 {
        println!("✅ GOOD: Search accuracy {:.1}% is good but below 90%", avg_accuracy);
        println!("📈 Grade: B+ - Minor improvements needed for A+");
    } else {
        println!("⚠️  NEEDS IMPROVEMENT: Search accuracy {:.1}% is below target", avg_accuracy);
        println!("📉 Grade: C - Significant improvements needed");
    }
    
    Ok(())
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }
    
    // Assuming embeddings are already normalized (they are from MiniLM)
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}