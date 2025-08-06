use std::path::PathBuf;
use std::sync::Arc;
use once_cell::sync::Lazy;
use tokio::sync::OnceCell;
use embed_search::search::unified::{UnifiedSearcher, IndexStats};

/// Shared test setup that only indexes once
static TEST_SETUP: Lazy<OnceCell<Arc<TestEnvironment>>> = Lazy::new(|| OnceCell::new());

struct TestEnvironment {
    searcher: UnifiedSearcher,
    vectortest_path: PathBuf,
}

impl TestEnvironment {
    async fn get_or_init() -> Arc<Self> {
        TEST_SETUP.get_or_init(|| async {
            println!("🔧 Initializing shared test environment (one-time setup)...");
            
            let project_root = std::env::current_dir().unwrap();
            let vectortest_path = project_root.join("vectortest");
            let db_path = project_root.join("test_accuracy_db");
            
            // Clean up any existing test database
            if db_path.exists() {
                std::fs::remove_dir_all(&db_path).ok();
            }
            
            // Create searcher with test file exclusion disabled to index everything
            let searcher = UnifiedSearcher::new_with_config(
                project_root.clone(),
                db_path,
                true // include test files for comprehensive testing
            ).await.expect("Failed to create searcher");
            
            // Index the vectortest directory ONCE
            println!("📚 Indexing vectortest directory (one-time operation)...");
            println!("📚 vectortest_path: {:?}", vectortest_path);
            println!("📚 vectortest exists: {}", vectortest_path.exists());
            match searcher.index_directory(&vectortest_path).await {
                Ok(stats) => {
                    println!("✅ Indexed {} files with {} chunks", stats.files_indexed, stats.chunks_created);
                },
                Err(e) => {
                    println!("❌ Failed to index directory: {}", e);
                    panic!("Index failed: {}", e);
                }
            }
            
            Arc::new(Self {
                searcher,
                vectortest_path,
            })
        }).await.clone()
    }
}

/// Test query with expected results
struct AccuracyTest {
    query: &'static str,
    expected_files: Vec<&'static str>,
    min_expected: usize,
    description: &'static str,
}

/// Core accuracy tests for the search system
mod accuracy_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_search_accuracy_suite() {
        // Temporarily use direct setup to see debug output
        println!("🔧 Initializing test environment (direct setup for debugging)...");
        
        let project_root = std::env::current_dir().unwrap();
        let vectortest_path = project_root.join("vectortest");
        let db_path = project_root.join("test_accuracy_db");
        
        // Clean up any existing test database
        if db_path.exists() {
            std::fs::remove_dir_all(&db_path).ok();
        }
        
        // Create searcher with test file exclusion disabled to index everything
        let searcher = UnifiedSearcher::new_with_config(
            project_root.clone(),
            db_path,
            true // include test files for comprehensive testing
        ).await.expect("Failed to create searcher");
        
        // Index the vectortest directory ONCE
        println!("📚 Indexing vectortest directory (direct setup)...");
        println!("📚 vectortest_path: {:?}", vectortest_path);
        println!("📚 vectortest exists: {}", vectortest_path.exists());
        println!("📚 vectortest is_dir: {}", vectortest_path.is_dir());
        
        // Try to list directory contents manually
        match std::fs::read_dir(&vectortest_path) {
            Ok(entries) => {
                println!("📚 Directory contents:");
                for entry in entries {
                    if let Ok(entry) = entry {
                        println!("  - {:?}", entry.path());
                    }
                }
            },
            Err(e) => println!("❌ Cannot read directory: {}", e),
        }
        
        println!("📚 About to call index_directory...");
        let stats = match searcher.index_directory(&vectortest_path).await {
            Ok(stats) => {
                println!("✅ Indexed {} files with {} chunks", stats.files_indexed, stats.chunks_created);
                stats
            },
            Err(e) => {
                println!("❌ Failed to index directory: {}", e);
                panic!("Index failed: {}", e);
            }
        };
        println!("📚 index_directory call completed successfully");
        
        let env = Arc::new(TestEnvironment {
            searcher,
            vectortest_path: vectortest_path.clone(),
        });
        
        let test_cases = vec![
            // Language-specific searches
            AccuracyTest {
                query: "database migration SQL",
                expected_files: vec!["database_migration.sql"],
                min_expected: 1,
                description: "SQL database migration search",
            },
            AccuracyTest {
                query: "authentication service Python",
                expected_files: vec!["auth_service.py"],
                min_expected: 1,
                description: "Python authentication service search",
            },
            AccuracyTest {
                query: "memory cache Rust implementation",
                expected_files: vec!["memory_cache.rs"],
                min_expected: 1,
                description: "Rust memory cache search",
            },
            AccuracyTest {
                query: "payment gateway TypeScript",
                expected_files: vec!["payment_gateway.ts"],
                min_expected: 1,
                description: "TypeScript payment gateway search",
            },
            AccuracyTest {
                query: "user controller JavaScript",
                expected_files: vec!["user_controller.js"],
                min_expected: 1,
                description: "JavaScript user controller search",
            },
            AccuracyTest {
                query: "analytics dashboard Go",
                expected_files: vec!["analytics_dashboard.go"],
                min_expected: 1,
                description: "Go analytics dashboard search",
            },
            AccuracyTest {
                query: "websocket server C++",
                expected_files: vec!["websocket_server.cpp"],
                min_expected: 1,
                description: "C++ websocket server search",
            },
            AccuracyTest {
                query: "data processor C#",
                expected_files: vec!["DataProcessor.cs"],
                min_expected: 1,
                description: "C# data processor search",
            },
            AccuracyTest {
                query: "order service Java",
                expected_files: vec!["OrderService.java"],
                min_expected: 1,
                description: "Java order service search",
            },
            AccuracyTest {
                query: "product catalog Ruby",
                expected_files: vec!["product_catalog.rb"],
                min_expected: 1,
                description: "Ruby product catalog search",
            },
            
            // Cross-file semantic searches
            AccuracyTest {
                query: "API documentation endpoints",
                expected_files: vec!["API_DOCUMENTATION.md"],
                min_expected: 1,
                description: "API documentation search",
            },
            AccuracyTest {
                query: "system architecture overview design",
                expected_files: vec!["ARCHITECTURE_OVERVIEW.md"],
                min_expected: 1,
                description: "Architecture documentation search",
            },
            AccuracyTest {
                query: "deployment configuration guide",
                expected_files: vec!["DEPLOYMENT_GUIDE.md"],
                min_expected: 1,
                description: "Deployment guide search",
            },
            AccuracyTest {
                query: "troubleshooting errors debugging",
                expected_files: vec!["TROUBLESHOOTING.md"],
                min_expected: 1,
                description: "Troubleshooting documentation search",
            },
            AccuracyTest {
                query: "contributing guidelines pull request",
                expected_files: vec!["CONTRIBUTING.md"],
                min_expected: 1,
                description: "Contributing guidelines search",
            },
            
            // Semantic concept searches
            AccuracyTest {
                query: "caching performance optimization",
                expected_files: vec!["memory_cache.rs"],
                min_expected: 1,
                description: "Cache-related code search",
            },
            AccuracyTest {
                query: "user authentication security",
                expected_files: vec!["auth_service.py"],
                min_expected: 1,
                description: "Authentication security search",
            },
            AccuracyTest {
                query: "real-time communication websocket",
                expected_files: vec!["websocket_server.cpp"],
                min_expected: 1,
                description: "Real-time communication search",
            },
            AccuracyTest {
                query: "data processing pipeline",
                expected_files: vec!["DataProcessor.cs"],
                min_expected: 1,
                description: "Data processing search",
            },
            AccuracyTest {
                query: "payment transaction handling",
                expected_files: vec!["payment_gateway.ts"],
                min_expected: 1,
                description: "Payment handling search",
            },
        ];
        
        let mut total_tests = 0;
        let mut passed_tests = 0;
        let mut accuracy_scores = Vec::new();
        
        println!("\n🎯 Running Search Accuracy Tests\n");
        println!("{}", "=".repeat(80));
        
        for test_case in test_cases {
            total_tests += 1;
            
            let results = env.searcher.search(test_case.query)
                .await
                .expect("Search failed");
            
            // Check if expected files are in results
            let mut found_count = 0;
            let mut found_files = Vec::new();
            
            for expected_file in &test_case.expected_files {
                for result in &results {
                    if result.file.contains(expected_file) {
                        found_count += 1;
                        found_files.push(expected_file.to_string());
                        break;
                    }
                }
            }
            
            let accuracy = (found_count as f64 / test_case.expected_files.len() as f64) * 100.0;
            accuracy_scores.push(accuracy);
            
            let passed = found_count >= test_case.min_expected;
            if passed {
                passed_tests += 1;
            }
            
            // Print test result
            println!("Test: {}", test_case.description);
            println!("  Query: \"{}\"", test_case.query);
            println!("  Expected: {:?}", test_case.expected_files);
            println!("  Found: {:?} ({}/{} files)", found_files, found_count, test_case.expected_files.len());
            println!("  Accuracy: {:.1}%", accuracy);
            println!("  Status: {}", if passed { "✅ PASSED" } else { "❌ FAILED" });
            println!("  Total results returned: {}", results.len());
            
            // Show top 3 results for debugging
            if !passed && !results.is_empty() {
                println!("  Top 3 results:");
                for (i, result) in results.iter().take(3).enumerate() {
                    println!("    {}. {} (score: {:.3})", i+1, result.file, result.score);
                }
            }
            println!();
        }
        
        // Calculate overall accuracy
        let overall_accuracy = accuracy_scores.iter().sum::<f64>() / accuracy_scores.len() as f64;
        let pass_rate = (passed_tests as f64 / total_tests as f64) * 100.0;
        
        println!("{}", "=".repeat(80));
        println!("📊 ACCURACY TEST SUMMARY");
        println!("{}", "=".repeat(80));
        println!("Total Tests: {}", total_tests);
        println!("Passed: {} / {}", passed_tests, total_tests);
        println!("Pass Rate: {:.1}%", pass_rate);
        println!("Average Accuracy: {:.1}%", overall_accuracy);
        println!();
        
        // Assert minimum accuracy threshold
        assert!(
            overall_accuracy >= 90.0,
            "Search accuracy {:.1}% is below 90% threshold",
            overall_accuracy
        );
        
        println!("✅ Search system achieved {:.1}% accuracy (>90% required)", overall_accuracy);
    }
    
    #[tokio::test]
    async fn test_semantic_similarity_accuracy() {
        let env = TestEnvironment::get_or_init().await;
        
        // Test semantic understanding with similar concepts
        let semantic_tests = vec![
            ("function that handles HTTP requests", vec!["user_controller.js", "payment_gateway.ts"]),
            ("code for managing state in memory", vec!["memory_cache.rs"]),
            ("service for user identity verification", vec!["auth_service.py"]),
            ("real-time bidirectional communication", vec!["websocket_server.cpp"]),
            ("structured data transformation logic", vec!["DataProcessor.cs"]),
        ];
        
        let mut semantic_accuracy = 0.0;
        
        for (query, expected_files) in semantic_tests {
            let results = env.searcher.search(query).await.unwrap();
            
            let mut found = false;
            for expected in expected_files {
                if results.iter().take(5).any(|r| r.file.contains(expected)) {
                    found = true;
                    break;
                }
            }
            
            if found {
                semantic_accuracy += 20.0; // Each test worth 20%
            }
            
            println!("Semantic test: \"{}\" - {}", query, if found { "✅" } else { "❌" });
        }
        
        println!("\n📊 Semantic Understanding Accuracy: {:.1}%", semantic_accuracy);
        assert!(semantic_accuracy >= 80.0, "Semantic accuracy too low");
    }
}

#[tokio::test]
async fn test_search_performance() {
    let env = TestEnvironment::get_or_init().await;
    
    // Test that search returns quickly
    let start = std::time::Instant::now();
    
    let queries = vec![
        "database migration",
        "authentication service",
        "websocket server",
        "payment processing",
        "cache implementation",
    ];
    
    for query in queries {
        let query_start = std::time::Instant::now();
        let _ = env.searcher.search(query).await.unwrap();
        let query_time = query_start.elapsed();
        
        assert!(
            query_time.as_millis() < 500,
            "Query '{}' took {}ms (>500ms threshold)",
            query,
            query_time.as_millis()
        );
    }
    
    let total_time = start.elapsed();
    println!("✅ All queries completed in {:.2}s (avg {:.0}ms per query)", 
             total_time.as_secs_f64(),
             total_time.as_millis() / 5);
}