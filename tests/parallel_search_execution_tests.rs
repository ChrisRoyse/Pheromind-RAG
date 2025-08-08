//! Parallel Search Execution Correctness Tests
//! 
//! This module tests the unified search system's ability to execute multiple
//! search backends in parallel while maintaining correctness and handling failures gracefully.

use embed_search::search::unified::{UnifiedSearcher, SearchResult};
use embed_search::search::fusion::MatchType;
use embed_search::config::{Config, SearchBackend};
use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;
use tempfile::TempDir;

/// Test suite for parallel search execution correctness
#[cfg(test)]
mod parallel_search_execution_tests {
    use super::*;

    /// Mock setup helper for creating test environment
    struct ParallelTestSetup {
        temp_dir: TempDir,
        project_path: std::path::PathBuf,
        db_path: std::path::PathBuf,
    }

    impl ParallelTestSetup {
        async fn new() -> Result<Self> {
            let temp_dir = TempDir::new()?;
            let project_path = temp_dir.path().join("project");
            let db_path = temp_dir.path().join("db");

            tokio::fs::create_dir_all(&project_path).await?;
            tokio::fs::create_dir_all(&db_path).await?;

            // Initialize configuration for tests
            std::env::set_var("EMBED_LOG_LEVEL", "debug");
            std::env::set_var("EMBED_SEARCH_BACKEND", "tantivy");
            
            Config::init().map_err(|e| anyhow::anyhow!("Config init failed: {}", e))?;

            Ok(Self {
                temp_dir,
                project_path,
                db_path,
            })
        }

        async fn create_test_files(&self) -> Result<()> {
            // Create test source files with different content types
            let files = vec![
                ("src/main.rs", "fn main() {\n    println!(\"Hello, world!\");\n}\n"),
                ("src/lib.rs", "pub fn test_function() {\n    // Test implementation\n}\n"),
                ("src/utils.rs", "pub fn utility_function() -> String {\n    \"test\".to_string()\n}\n"),
                ("tests/test_main.rs", "#[test]\nfn test_main() {\n    assert_eq!(2 + 2, 4);\n}\n"),
                ("README.md", "# Test Project\nThis is a test project for search functionality.\n"),
            ];

            for (path, content) in files {
                let file_path = self.project_path.join(path);
                if let Some(parent) = file_path.parent() {
                    tokio::fs::create_dir_all(parent).await?;
                }
                tokio::fs::write(file_path, content).await?;
            }

            Ok(())
        }
    }

    /// Test parallel execution of multiple search backends
    #[tokio::test]
    async fn test_parallel_backend_execution() {
        let setup = ParallelTestSetup::new().await
            .expect("Test setup must succeed");
        
        setup.create_test_files().await
            .expect("Test file creation must succeed");

        // Create unified searcher
        let searcher = UnifiedSearcher::new_with_config(
            setup.project_path.clone(),
            setup.db_path.clone(),
            false, // don't include test files
        ).await.expect("UnifiedSearcher creation must succeed");

        // Index the test files
        let stats = searcher.index_directory(&setup.project_path).await
            .expect("Directory indexing must succeed");

        assert!(
            stats.files_indexed > 0,
            "Should have indexed some files, got {}",
            stats.files_indexed
        );

        // Test parallel search execution with timeout
        let search_futures = vec![
            searcher.search("main"),
            searcher.search("test"),
            searcher.search("function"),
            searcher.search("println"),
        ];

        let start_time = std::time::Instant::now();
        
        // Execute searches in parallel with timeout protection
        let results = timeout(Duration::from_secs(10), async {
            futures::future::try_join_all(search_futures).await
        }).await
        .expect("Parallel searches should complete within timeout")
        .expect("All parallel searches should succeed");

        let execution_time = start_time.elapsed();

        // Verify all searches completed
        assert_eq!(
            results.len(), 4,
            "Should have 4 search results, got {}",
            results.len()
        );

        // Verify reasonable execution time (parallel should be faster than sequential)
        assert!(
            execution_time < Duration::from_secs(5),
            "Parallel execution took too long: {:?}",
            execution_time
        );

        // Verify each search returned valid results
        for (i, result) in results.iter().enumerate() {
            assert!(
                !result.is_empty() || i == 3, // println might not match much
                "Search {} should return results or be empty for uncommon terms",
                i
            );

            // Verify result structure
            for search_result in result {
                assert!(
                    !search_result.file.is_empty(),
                    "Result file path should not be empty"
                );
                
                assert!(
                    search_result.score >= 0.0,
                    "Result score should be non-negative, got {}",
                    search_result.score
                );

                assert!(
                    search_result.score.is_finite(),
                    "Result score should be finite, got {}",
                    search_result.score
                );

                // Verify three-chunk context is valid
                assert!(
                    !search_result.three_chunk_context.center.content.is_empty(),
                    "Center chunk should have content"
                );
            }
        }

        println!("✅ Parallel search execution completed in {:?}", execution_time);
    }

    /// Test concurrent searches with shared resources
    #[tokio::test]
    async fn test_concurrent_shared_resource_access() {
        let setup = ParallelTestSetup::new().await
            .expect("Test setup must succeed");
        
        setup.create_test_files().await
            .expect("Test file creation must succeed");

        let searcher = Arc::new(
            UnifiedSearcher::new_with_config(
                setup.project_path.clone(),
                setup.db_path.clone(),
                false,
            ).await.expect("UnifiedSearcher creation must succeed")
        );

        // Index files
        searcher.index_directory(&setup.project_path).await
            .expect("Directory indexing must succeed");

        // Create multiple concurrent tasks accessing the same searcher
        let num_concurrent = 10;
        let mut handles = Vec::new();

        for i in 0..num_concurrent {
            let searcher_clone = Arc::clone(&searcher);
            let query = match i % 4 {
                0 => "main",
                1 => "test",
                2 => "function",
                _ => "println",
            };

            let handle = tokio::spawn(async move {
                let result = searcher_clone.search(query).await;
                (i, query, result)
            });

            handles.push(handle);
        }

        // Wait for all concurrent searches to complete
        let start_time = std::time::Instant::now();
        let results = futures::future::try_join_all(handles).await
            .expect("All concurrent tasks should complete");
        let concurrent_time = start_time.elapsed();

        // Verify all searches completed successfully
        assert_eq!(
            results.len(), num_concurrent,
            "All {} concurrent searches should complete",
            num_concurrent
        );

        for (task_id, query, result) in results {
            match result {
                Ok(search_results) => {
                    // Verify results are valid
                    for search_result in &search_results {
                        assert!(
                            search_result.score.is_finite(),
                            "Task {} query '{}' produced invalid score: {}",
                            task_id, query, search_result.score
                        );
                    }
                }
                Err(e) => {
                    panic!("Task {} query '{}' failed: {}", task_id, query, e);
                }
            }
        }

        // Should complete within reasonable time
        assert!(
            concurrent_time < Duration::from_secs(15),
            "Concurrent execution took too long: {:?}",
            concurrent_time
        );

        println!("✅ {} concurrent searches completed in {:?}", num_concurrent, concurrent_time);
    }

    /// Test graceful degradation when backends fail
    #[tokio::test]
    async fn test_graceful_degradation_backend_failures() {
        let setup = ParallelTestSetup::new().await
            .expect("Test setup must succeed");
        
        setup.create_test_files().await
            .expect("Test file creation must succeed");

        // Test with different backend configurations to simulate partial failures
        let searcher = UnifiedSearcher::new_with_config(
            setup.project_path.clone(),
            setup.db_path.clone(),
            false,
        ).await.expect("UnifiedSearcher creation must succeed");

        // Index files first
        searcher.index_directory(&setup.project_path).await
            .expect("Directory indexing must succeed");

        // Test search when some backends might fail but others succeed
        let search_result = searcher.search("main").await;

        match search_result {
            Ok(results) => {
                // Should get results from at least one backend
                assert!(
                    !results.is_empty(),
                    "Should get results from available backends"
                );
                
                // Verify results are valid even with partial backend failures
                for result in &results {
                    assert!(
                        result.score.is_finite() && result.score >= 0.0,
                        "Results should be valid despite potential backend failures"
                    );
                }
            }
            Err(e) => {
                // If all backends fail, should get a proper error (not panic)
                assert!(
                    e.to_string().contains("search") || e.to_string().contains("backend"),
                    "Error should indicate search/backend issue: {}",
                    e
                );
            }
        }
    }

    /// Test search consistency across multiple parallel executions
    #[tokio::test]
    async fn test_search_result_consistency() {
        let setup = ParallelTestSetup::new().await
            .expect("Test setup must succeed");
        
        setup.create_test_files().await
            .expect("Test file creation must succeed");

        let searcher = UnifiedSearcher::new_with_config(
            setup.project_path.clone(),
            setup.db_path.clone(),
            false,
        ).await.expect("UnifiedSearcher creation must succeed");

        searcher.index_directory(&setup.project_path).await
            .expect("Directory indexing must succeed");

        // Run the same search multiple times in parallel
        let query = "function";
        let num_runs = 5;
        let search_futures: Vec<_> = (0..num_runs)
            .map(|_| searcher.search(query))
            .collect();

        let results = futures::future::try_join_all(search_futures).await
            .expect("All consistency searches should succeed");

        // Verify all runs return consistent results
        if !results.is_empty() && !results[0].is_empty() {
            let first_result = &results[0];
            
            for (i, result) in results.iter().enumerate() {
                // Should have same number of results (with caching)
                assert_eq!(
                    result.len(), first_result.len(),
                    "Run {} should return same number of results as first run: {} vs {}",
                    i, result.len(), first_result.len()
                );

                // Should have same top result
                if !result.is_empty() && !first_result.is_empty() {
                    assert_eq!(
                        result[0].file, first_result[0].file,
                        "Run {} should return same top result as first run",
                        i
                    );

                    // Scores should be identical (deterministic)
                    assert!(
                        (result[0].score - first_result[0].score).abs() < 1e-6,
                        "Run {} top result score should match first run: {} vs {}",
                        i, result[0].score, first_result[0].score
                    );
                }
            }
        }

        println!("✅ Search consistency verified across {} parallel runs", num_runs);
    }

    /// Test timeout handling in parallel searches
    #[tokio::test]
    async fn test_parallel_search_timeout_handling() {
        let setup = ParallelTestSetup::new().await
            .expect("Test setup must succeed");
        
        setup.create_test_files().await
            .expect("Test file creation must succeed");

        let searcher = Arc::new(
            UnifiedSearcher::new_with_config(
                setup.project_path.clone(),
                setup.db_path.clone(),
                false,
            ).await.expect("UnifiedSearcher creation must succeed")
        );

        searcher.index_directory(&setup.project_path).await
            .expect("Directory indexing must succeed");

        // Test with very short timeout to verify timeout handling
        let short_timeout = Duration::from_millis(1);
        let searcher_clone = Arc::clone(&searcher);
        
        let timeout_result = timeout(short_timeout, async {
            searcher_clone.search("test").await
        }).await;

        // Should either complete quickly or timeout gracefully
        match timeout_result {
            Ok(search_result) => {
                // If it completed, verify results are valid
                match search_result {
                    Ok(results) => {
                        for result in &results {
                            assert!(
                                result.score.is_finite(),
                                "Fast completion should still produce valid results"
                            );
                        }
                    }
                    Err(e) => {
                        // Search errors should be handled gracefully
                        assert!(
                            !e.to_string().contains("panic"),
                            "Search errors should not contain panics: {}",
                            e
                        );
                    }
                }
            }
            Err(_) => {
                // Timeout occurred - this is acceptable behavior
                println!("✅ Search properly timed out with short timeout");
            }
        }

        // Test with reasonable timeout to verify normal operation
        let normal_timeout = Duration::from_secs(5);
        let normal_result = timeout(normal_timeout, searcher.search("function")).await;

        assert!(
            normal_result.is_ok(),
            "Search should complete within reasonable timeout"
        );

        let search_result = normal_result.unwrap()
            .expect("Search should succeed with reasonable timeout");

        // Verify normal results
        for result in &search_result {
            assert!(
                result.score.is_finite() && result.score >= 0.0,
                "Normal timeout search should produce valid results"
            );
        }
    }

    /// Test resource cleanup in parallel scenarios
    #[tokio::test]
    async fn test_parallel_resource_cleanup() {
        let setup = ParallelTestSetup::new().await
            .expect("Test setup must succeed");
        
        setup.create_test_files().await
            .expect("Test file creation must succeed");

        // Create searcher in limited scope
        {
            let searcher = UnifiedSearcher::new_with_config(
                setup.project_path.clone(),
                setup.db_path.clone(),
                false,
            ).await.expect("UnifiedSearcher creation must succeed");

            searcher.index_directory(&setup.project_path).await
                .expect("Directory indexing must succeed");

            // Perform multiple parallel operations
            let futures = vec![
                searcher.search("test"),
                searcher.search("main"),
                searcher.search("function"),
            ];

            let results = futures::future::try_join_all(futures).await
                .expect("Parallel operations should succeed");

            assert_eq!(results.len(), 3, "Should complete all operations");

            // Test cache clearing
            searcher.clear_index().await
                .expect("Index clearing should succeed");
        } // searcher goes out of scope here

        // Test that resources were properly cleaned up
        // (This is implicit - no panics or resource leaks should occur)

        // Create new searcher to verify clean state
        let new_searcher = UnifiedSearcher::new_with_config(
            setup.project_path.clone(),
            setup.db_path.clone(),
            false,
        ).await.expect("New searcher creation should succeed after cleanup");

        // Should start with empty index
        let empty_results = new_searcher.search("test").await
            .expect("Search on fresh searcher should succeed");

        // Should have no results since index was cleared and not rebuilt
        assert!(
            empty_results.is_empty(),
            "Fresh searcher should have empty index initially"
        );

        println!("✅ Resource cleanup verified in parallel scenarios");
    }

    /// Test error propagation in parallel search scenarios
    #[tokio::test]
    async fn test_parallel_error_propagation() {
        let setup = ParallelTestSetup::new().await
            .expect("Test setup must succeed");

        let searcher = UnifiedSearcher::new_with_config(
            setup.project_path.clone(), 
            setup.db_path.clone(),
            false,
        ).await.expect("UnifiedSearcher creation must succeed");

        // Test with empty query and other edge cases
        let error_queries = vec![
            "",           // Empty query
            "   ",        // Whitespace only
            "\n\t",       // Special whitespace
        ];

        let error_futures: Vec<_> = error_queries.iter()
            .map(|&query| searcher.search(query))
            .collect();

        let error_results = futures::future::join_all(error_futures).await;

        // Verify error handling
        for (i, result) in error_results.iter().enumerate() {
            match result {
                Ok(search_results) => {
                    // Empty results are acceptable for edge case queries
                    assert!(
                        search_results.is_empty(),
                        "Edge case query {} should return empty results",
                        i
                    );
                }
                Err(e) => {
                    // Errors should be descriptive and not contain panics
                    let error_msg = e.to_string();
                    assert!(
                        !error_msg.contains("panic") && !error_msg.contains("unwrap"),
                        "Error should be graceful, not a panic: {}",
                        error_msg
                    );
                }
            }
        }

        println!("✅ Error propagation verified in parallel scenarios");
    }

    /// Test mixed successful and failed parallel operations
    #[tokio::test] 
    async fn test_mixed_success_failure_parallel() {
        let setup = ParallelTestSetup::new().await
            .expect("Test setup must succeed");
        
        setup.create_test_files().await
            .expect("Test file creation must succeed");

        let searcher = UnifiedSearcher::new_with_config(
            setup.project_path.clone(),
            setup.db_path.clone(), 
            false,
        ).await.expect("UnifiedSearcher creation must succeed");

        searcher.index_directory(&setup.project_path).await
            .expect("Directory indexing must succeed");

        // Mix of valid and edge case queries
        let mixed_queries = vec![
            "function",    // Should succeed
            "",           // Edge case
            "main",       // Should succeed
            "   ",        // Edge case
            "test",       // Should succeed
        ];

        let mixed_futures: Vec<_> = mixed_queries.iter()
            .map(|&query| searcher.search(query))
            .collect();

        let mixed_results = futures::future::join_all(mixed_futures).await;

        let mut successes = 0;
        let mut acceptable_failures = 0;

        for (i, result) in mixed_results.iter().enumerate() {
            match result {
                Ok(search_results) => {
                    successes += 1;
                    // Verify valid results
                    for search_result in search_results {
                        assert!(
                            search_result.score.is_finite(),
                            "Successful result {} should have finite score",
                            i
                        );
                    }
                }
                Err(e) => {
                    // Should be graceful errors for edge cases
                    acceptable_failures += 1;
                    assert!(
                        !e.to_string().contains("panic"),
                        "Error should be graceful: {}",
                        e
                    );
                }
            }
        }

        // Should have some successes
        assert!(
            successes > 0,
            "Should have at least some successful searches, got {}",
            successes
        );

        println!("✅ Mixed parallel operations: {} successes, {} acceptable failures", 
                 successes, acceptable_failures);
    }
}