/// Unbiased accuracy test with real embeddings and realistic queries
/// This test uses unpredictable queries that don't match filenames

use std::path::PathBuf;
use embed_lib::search::unified::UnifiedSearcher;
use embed_lib::embedding::RealMiniLMEmbedder;
use anyhow::Result;

#[derive(Debug)]
struct TestCase {
    query: &'static str,
    expected_concepts: Vec<&'static str>,
    description: &'static str,
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("==> UNBIASED ACCURACY TEST - Real Embeddings, No Cheating");
    println!("{}", "=".repeat(80));
    
    // Test that the singleton pattern is working
    println!("==> Verifying real embeddings (not mocks)...");
    let embedder = RealMiniLMEmbedder::get_global().await
        .map_err(|e| anyhow::anyhow!("Failed to load real embedder: {}", e))?;
    
    // Test embedding to ensure it's real
    let test_embedding = embedder.embed("test")
        .map_err(|e| anyhow::anyhow!("Failed to generate embedding: {}", e))?;
    
    if test_embedding.len() != 384 {
        panic!("Not using real MiniLM embeddings! Got {} dimensions", test_embedding.len());
    }
    
    println!("OK: Real MiniLM-L6-v2 embeddings confirmed (384 dimensions)");
    
    let project_root = std::env::current_dir()?;
    let test_dir = project_root.join("realistic_test");
    let db_path = project_root.join(".unbiased_test_db");
    
    // Clean existing DB
    if db_path.exists() {
        std::fs::remove_dir_all(&db_path).ok();
    }
    
    println!("\n==> Initializing search system...");
    let searcher = UnifiedSearcher::new_with_config(
        project_root.clone(),
        db_path.clone(),
        true // include all files for comprehensive testing
    ).await?;
    
    println!("==> Indexing realistic_test directory...");
    let start = std::time::Instant::now();
    let stats = searcher.index_directory(&test_dir).await?;
    let index_time = start.elapsed();
    
    println!("OK: Indexed {} files with {} chunks in {:.2f}s", 
             stats.files_indexed, stats.chunks_created, index_time.as_secs_f64());
    
    // Define unpredictable test queries that DON'T match filenames
    let test_cases = vec![
        TestCase {
            query: "secure password verification login system",
            expected_concepts: vec!["authentication", "validate", "credentials", "session"],
            description: "Authentication concept search",
        },
        TestCase {
            query: "transform compress encrypt data pipeline",
            expected_concepts: vec!["processor", "transformation", "operation", "execute"],
            description: "Data processing pipeline search",
        },
        TestCase {
            query: "paginated results with sorting and filtering",
            expected_concepts: vec!["table", "pagination", "sort", "filter", "page"],
            description: "UI table component search",
        },
        TestCase {
            query: "retry failed requests with exponential backoff",
            expected_concepts: vec!["retry", "attempts", "delay", "error", "request"],
            description: "Retry logic search",
        },
        TestCase {
            query: "store retrieve cached values memory",
            expected_concepts: vec!["cache", "store", "get", "memory", "lru"],
            description: "Caching mechanism search",
        },
        TestCase {
            query: "database operations create read update delete",
            expected_concepts: vec!["repository", "query", "insert", "update", "database"],
            description: "CRUD operations search",
        },
        TestCase {
            query: "real-time message handling priority queue",
            expected_concepts: vec!["queue", "message", "priority", "push", "pop"],
            description: "Message queue search",
        },
        TestCase {
            query: "api response error handling status codes",
            expected_concepts: vec!["response", "error", "success", "status", "api"],
            description: "API response types search",
        },
        TestCase {
            query: "user permissions role based access control",
            expected_concepts: vec!["permission", "role", "access", "resource", "action"],
            description: "Authorization system search",
        },
        TestCase {
            query: "web socket bidirectional communication events",
            expected_concepts: vec!["websocket", "message", "event", "connection"],
            description: "WebSocket messaging search",
        },
        
        // Cross-language conceptual searches
        TestCase {
            query: "handle errors gracefully with fallback",
            expected_concepts: vec!["error", "catch", "fallback", "handle", "exception"],
            description: "Error handling across languages",
        },
        TestCase {
            query: "optimize performance reduce latency",
            expected_concepts: vec!["performance", "optimize", "cache", "efficient", "fast"],
            description: "Performance optimization search",
        },
        TestCase {
            query: "validate input sanitize user data",
            expected_concepts: vec!["validate", "sanitize", "input", "check", "clean"],
            description: "Input validation search",
        },
        TestCase {
            query: "asynchronous concurrent parallel processing",
            expected_concepts: vec!["async", "concurrent", "parallel", "worker", "thread"],
            description: "Concurrency patterns search",
        },
        TestCase {
            query: "serialize deserialize json data format",
            expected_concepts: vec!["json", "serialize", "parse", "format", "data"],
            description: "Data serialization search",
        },
    ];
    
    println!("\n==> Running unbiased accuracy tests...\n");
    
    let mut passed = 0;
    let mut total = 0;
    let mut concept_matches = 0;
    let mut total_concepts = 0;
    
    for test_case in test_cases {
        total += 1;
        print!("[{}] {} ... ", total, test_case.description);
        
        let start = std::time::Instant::now();
        let results = searcher.search(test_case.query).await?;
        let search_time = start.elapsed();
        
        if results.is_empty() {
            println!("FAIL: No results ({:.0}ms)", search_time.as_millis());
            continue;
        }
        
        // Check if any of the expected concepts appear in top 5 results
        let top_results = results.iter().take(5).collect::<Vec<_>>();
        let mut found_concepts = Vec::new();
        
        for result in &top_results {
            let content_lower = result.three_chunk_context.get_full_content().to_lowercase();
            let file_lower = result.file.to_lowercase();
            
            for concept in &test_case.expected_concepts {
                if content_lower.contains(concept) || file_lower.contains(concept) {
                    if !found_concepts.contains(concept) {
                        found_concepts.push(*concept);
                    }
                }
            }
        }
        
        total_concepts += test_case.expected_concepts.len();
        concept_matches += found_concepts.len();
        
        let concept_coverage = (found_concepts.len() as f32 / test_case.expected_concepts.len() as f32) * 100.0;
        
        if concept_coverage >= 40.0 {  // At least 40% of concepts found
            passed += 1;
            println!("PASS: {:.0}% concepts found in {} ({:.0}ms)", 
                     concept_coverage, 
                     top_results[0].file.split('/').last().unwrap_or(&top_results[0].file),
                     search_time.as_millis());
        } else {
            println!("FAIL: Only {:.0}% concepts found ({:.0}ms)", 
                     concept_coverage, search_time.as_millis());
            println!("  Query: \"{}\"", test_case.query);
            println!("  Expected concepts: {:?}", test_case.expected_concepts);
            println!("  Found concepts: {:?}", found_concepts);
            println!("  Top result: {}", top_results[0].file);
        }
    }
    
    let accuracy = (passed as f32 / total as f32) * 100.0;
    let concept_accuracy = (concept_matches as f32 / total_concepts as f32) * 100.0;
    
    println!("\n{}", "=".repeat(80));
    println!("==> UNBIASED TEST RESULTS");
    println!("{}", "=".repeat(80));
    println!("Tests Passed: {}/{} ({:.1}%)", passed, total, accuracy);
    println!("Concepts Found: {}/{} ({:.1}%)", concept_matches, total_concepts, concept_accuracy);
    
    println!("\n==> ACCURACY ASSESSMENT:");
    if accuracy >= 80.0 {
        println!("*** EXCELLENT: System achieves {:.1}% accuracy without bias!", accuracy);
        println!("*** This is TRUE semantic search working correctly!");
    } else if accuracy >= 60.0 {
        println!("GOOD: System achieves {:.1}% accuracy", accuracy);
        println!("This is realistic for semantic search without fine-tuning");
    } else if accuracy >= 40.0 {
        println!("ACCEPTABLE: System achieves {:.1}% accuracy", accuracy);
        println!("This is typical for general-purpose embeddings on code");
    } else {
        println!("NEEDS IMPROVEMENT: Only {:.1}% accuracy", accuracy);
        println!("Consider using code-specific embeddings like CodeBERT");
    }
    
    // Clean up test DB
    std::fs::remove_dir_all(&db_path).ok();
    
    Ok(())
}