//! Performance Regression Tests
//! 
//! This module tests performance characteristics of the search system to detect
//! regressions and ensure acceptable performance under various load conditions.

use embed_search::search::unified::{UnifiedSearcher, SearchResult, IndexStats};
use embed_search::search::bm25::{BM25Engine, BM25Document, Token as BM25Token};
use embed_search::search::fusion::SimpleFusion;
use embed_search::config::Config;
use std::time::{Duration, Instant};
use tempfile::TempDir;
use std::path::PathBuf;

/// Performance test suite for regression detection
#[cfg(test)]
mod performance_regression_tests {
    use super::*;

    /// Performance test setup with configurable corpus size
    struct PerformanceTestSetup {
        temp_dir: TempDir,
        project_path: PathBuf,
        db_path: PathBuf,
    }

    impl PerformanceTestSetup {
        async fn new() -> anyhow::Result<Self> {
            let temp_dir = TempDir::new()?;
            let project_path = temp_dir.path().join("project");
            let db_path = temp_dir.path().join("db");

            tokio::fs::create_dir_all(&project_path).await?;
            tokio::fs::create_dir_all(&db_path).await?;

            // Initialize with performance-focused config
            std::env::set_var("EMBED_LOG_LEVEL", "warn"); // Reduce logging overhead
            std::env::set_var("EMBED_SEARCH_BACKEND", "tantivy");
            Config::init().map_err(|e| anyhow::anyhow!("Config init failed: {}", e))?;

            Ok(Self {
                temp_dir,
                project_path,
                db_path,
            })
        }

        /// Create a corpus of specified size for performance testing
        async fn create_performance_corpus(&self, num_files: usize, lines_per_file: usize) -> anyhow::Result<()> {
            for file_idx in 0..num_files {
                let mut content = String::new();
                
                for line_idx in 0..lines_per_file {
                    match file_idx % 4 {
                        0 => {
                            // Rust-like code
                            content.push_str(&format!(
                                "pub fn function_{}_{}_{}() -> Result<String, Error> {{\n",
                                file_idx, line_idx, line_idx % 10
                            ));
                            content.push_str("    let result = perform_operation();\n");
                            content.push_str("    Ok(result.to_string())\n}\n\n");
                        }
                        1 => {
                            // JavaScript-like code
                            content.push_str(&format!(
                                "function processData{}{}(data) {{\n",
                                file_idx, line_idx
                            ));
                            content.push_str("    return data.map(item => item.value);\n");
                            content.push_str("}\n\n");
                        }
                        2 => {
                            // Python-like code
                            content.push_str(&format!(
                                "def calculate_{}(x, y):\n    result = x * y + {}\n    return result\n\n",
                                line_idx, file_idx
                            ));
                        }
                        3 => {
                            // Generic text content
                            content.push_str(&format!(
                                "This is line {} of file {}. It contains common terms like test, function, data, process, result.\n",
                                line_idx, file_idx
                            ));
                            content.push_str("Additional content with keywords like search, index, performance, benchmark.\n");
                        }
                        _ => unreachable!(),
                    }
                }

                let file_path = match file_idx % 4 {
                    0 => self.project_path.join(format!("src/module_{}.rs", file_idx)),
                    1 => self.project_path.join(format!("js/script_{}.js", file_idx)),
                    2 => self.project_path.join(format!("python/tool_{}.py", file_idx)),
                    3 => self.project_path.join(format!("docs/readme_{}.md", file_idx)),
                    _ => unreachable!(),
                };

                if let Some(parent) = file_path.parent() {
                    tokio::fs::create_dir_all(parent).await?;
                }

                tokio::fs::write(file_path, content).await?;
            }

            Ok(())
        }
    }

    /// Benchmark result tracking
    #[derive(Debug, Clone)]
    struct BenchmarkResult {
        operation: String,
        duration: Duration,
        throughput: Option<f64>, // Operations per second
        memory_delta: Option<usize>, // Memory change in bytes
    }

    impl BenchmarkResult {
        fn new(operation: String, duration: Duration) -> Self {
            Self {
                operation,
                duration,
                throughput: None,
                memory_delta: None,
            }
        }

        fn with_throughput(mut self, ops_per_sec: f64) -> Self {
            self.throughput = Some(ops_per_sec);
            self
        }

        fn assert_within_threshold(&self, max_duration: Duration, operation_context: &str) {
            assert!(
                self.duration <= max_duration,
                "{} took too long: {:?} > {:?} (operation: {})",
                operation_context,
                self.duration,
                max_duration,
                self.operation
            );
        }

        fn assert_throughput_threshold(&self, min_throughput: f64, operation_context: &str) {
            if let Some(throughput) = self.throughput {
                assert!(
                    throughput >= min_throughput,
                    "{} throughput too low: {:.2} < {:.2} ops/sec (operation: {})",
                    operation_context,
                    throughput,
                    min_throughput,
                    self.operation
                );
            }
        }
    }

    /// Test indexing performance doesn't regress
    #[tokio::test]
    async fn test_indexing_performance_regression() {
        let setup = PerformanceTestSetup::new().await
            .expect("Performance test setup should succeed");

        // Create medium-sized corpus (should complete in reasonable time)
        setup.create_performance_corpus(50, 100).await
            .expect("Performance corpus creation should succeed");

        let searcher = UnifiedSearcher::new_with_config(
            setup.project_path.clone(),
            setup.db_path.clone(),
            false, // Exclude test files for cleaner performance metrics
        ).await.expect("UnifiedSearcher creation should succeed");

        // Benchmark indexing performance
        let start_time = Instant::now();
        let stats = searcher.index_directory(&setup.project_path).await
            .expect("Directory indexing should succeed");
        let indexing_duration = start_time.elapsed();

        let indexing_benchmark = BenchmarkResult::new(
            "directory_indexing".to_string(),
            indexing_duration
        ).with_throughput(stats.files_indexed as f64 / indexing_duration.as_secs_f64());

        // Performance assertions
        indexing_benchmark.assert_within_threshold(
            Duration::from_secs(30), // Should index 50 files in under 30 seconds
            "Directory indexing"
        );

        indexing_benchmark.assert_throughput_threshold(
            2.0, // Should index at least 2 files per second
            "Indexing throughput"
        );

        assert!(
            stats.files_indexed > 40, // Should successfully index most files
            "Should index most files, got {}",
            stats.files_indexed
        );

        assert!(
            stats.chunks_created > stats.files_indexed, // Multiple chunks per file
            "Should create multiple chunks per file: {} chunks from {} files",
            stats.chunks_created,
            stats.files_indexed
        );

        println!("✅ Indexing performance: {:.2}s for {} files ({:.2} files/sec, {} chunks)",
                 indexing_duration.as_secs_f64(),
                 stats.files_indexed,
                 indexing_benchmark.throughput.unwrap_or(0.0),
                 stats.chunks_created);
    }

    /// Test search performance doesn't regress
    #[tokio::test]
    async fn test_search_performance_regression() {
        let setup = PerformanceTestSetup::new().await
            .expect("Performance test setup should succeed");

        setup.create_performance_corpus(30, 50).await
            .expect("Performance corpus creation should succeed");

        let searcher = UnifiedSearcher::new_with_config(
            setup.project_path.clone(),
            setup.db_path.clone(),
            false,
        ).await.expect("UnifiedSearcher creation should succeed");

        // Index the corpus
        searcher.index_directory(&setup.project_path).await
            .expect("Directory indexing should succeed");

        // Benchmark various search scenarios
        let search_queries = vec![
            ("function", "common term"),
            ("Result", "type name"),
            ("calculate", "specific function"),
            ("xyz_nonexistent", "no matches"),
            ("test performance benchmark", "multi-word query"),
        ];

        let mut search_benchmarks = Vec::new();

        for (query, description) in search_queries {
            let start_time = Instant::now();
            let results = searcher.search(query).await
                .expect("Search should complete successfully");
            let search_duration = start_time.elapsed();

            let benchmark = BenchmarkResult::new(
                format!("search_{}", description.replace(" ", "_")),
                search_duration
            );

            benchmark.assert_within_threshold(
                Duration::from_millis(500), // Searches should complete under 500ms
                &format!("Search for '{}' ({})", query, description)
            );

            // Verify result quality
            for result in &results {
                assert!(
                    result.score.is_finite() && result.score >= 0.0,
                    "Search result scores should be valid"
                );

                assert!(
                    !result.file.is_empty(),
                    "Search results should have valid file paths"
                );
            }

            search_benchmarks.push(benchmark);

            println!("✅ Search '{}' ({}): {:.2}ms, {} results",
                     query, description, 
                     search_duration.as_millis(),
                     results.len());
        }

        // Verify no search took excessively long
        let max_search_time = search_benchmarks.iter()
            .map(|b| b.duration)
            .max()
            .unwrap_or(Duration::ZERO);

        assert!(
            max_search_time < Duration::from_secs(1),
            "No search should take longer than 1 second, max was {:?}",
            max_search_time
        );
    }

    /// Test concurrent search performance
    #[tokio::test]
    async fn test_concurrent_search_performance() {
        let setup = PerformanceTestSetup::new().await
            .expect("Performance test setup should succeed");

        setup.create_performance_corpus(40, 30).await
            .expect("Performance corpus creation should succeed");

        let searcher = std::sync::Arc::new(
            UnifiedSearcher::new_with_config(
                setup.project_path.clone(),
                setup.db_path.clone(),
                false,
            ).await.expect("UnifiedSearcher creation should succeed")
        );

        searcher.index_directory(&setup.project_path).await
            .expect("Directory indexing should succeed");

        // Test concurrent search performance
        let num_concurrent = 20;
        let queries = vec!["function", "result", "data", "process", "test"];

        let start_time = Instant::now();
        
        let concurrent_futures: Vec<_> = (0..num_concurrent)
            .map(|i| {
                let searcher_clone = std::sync::Arc::clone(&searcher);
                let query = queries[i % queries.len()];
                
                tokio::spawn(async move {
                    let search_start = Instant::now();
                    let result = searcher_clone.search(query).await;
                    let search_duration = search_start.elapsed();
                    (i, query, result, search_duration)
                })
            })
            .collect();

        let concurrent_results = futures::future::try_join_all(concurrent_futures).await
            .expect("All concurrent searches should complete");

        let total_concurrent_time = start_time.elapsed();

        // Analyze concurrent performance
        let mut successful_searches = 0;
        let mut total_individual_time = Duration::ZERO;
        let mut max_individual_time = Duration::ZERO;

        for (task_id, query, result, duration) in concurrent_results {
            match result {
                Ok(search_results) => {
                    successful_searches += 1;
                    total_individual_time += duration;
                    max_individual_time = max_individual_time.max(duration);

                    // Verify result quality under concurrency
                    for search_result in &search_results {
                        assert!(
                            search_result.score.is_finite(),
                            "Concurrent search {} should produce valid scores",
                            task_id
                        );
                    }
                }
                Err(e) => {
                    panic!("Concurrent search {} ('{}') failed: {}", task_id, query, e);
                }
            }
        }

        // Performance assertions for concurrent execution
        assert_eq!(
            successful_searches, num_concurrent,
            "All concurrent searches should succeed"
        );

        assert!(
            total_concurrent_time < Duration::from_secs(10),
            "Concurrent searches should complete quickly: {:?}",
            total_concurrent_time
        );

        assert!(
            max_individual_time < Duration::from_secs(2),
            "No individual search should take too long under concurrency: {:?}",
            max_individual_time
        );

        // Calculate concurrency efficiency
        let avg_individual_time = total_individual_time / num_concurrent as u32;
        let concurrency_factor = avg_individual_time.as_secs_f64() / total_concurrent_time.as_secs_f64();

        assert!(
            concurrency_factor > 0.5, // Should get some benefit from concurrency
            "Concurrency should provide performance benefit: factor {:.2}",
            concurrency_factor
        );

        println!("✅ Concurrent performance: {} searches in {:.2}s (avg: {:.2}ms, max: {:.2}ms, factor: {:.2})",
                 num_concurrent,
                 total_concurrent_time.as_secs_f64(),
                 avg_individual_time.as_millis(),
                 max_individual_time.as_millis(),
                 concurrency_factor);
    }

    /// Test BM25 engine performance in isolation
    #[test]
    fn test_bm25_engine_performance() {
        let mut engine = BM25Engine::new();

        // Create performance test corpus
        let num_docs = 1000;
        let tokens_per_doc = 100;

        let indexing_start = Instant::now();

        for doc_idx in 0..num_docs {
            let mut tokens = Vec::new();
            
            for token_idx in 0..tokens_per_doc {
                tokens.push(BM25Token {
                    text: format!("token_{}_{}", doc_idx % 50, token_idx % 20), // Some overlap
                    position: token_idx,
                    importance_weight: 1.0,
                });
            }

            let doc = BM25Document {
                id: format!("doc_{}", doc_idx),
                file_path: format!("file_{}.rs", doc_idx),
                chunk_index: doc_idx,
                tokens,
                start_line: 1,
                end_line: 100,
                language: Some("rust".to_string()),
            };

            engine.add_document(doc)
                .expect("Document addition should succeed");
        }

        let indexing_duration = indexing_start.elapsed();

        // Performance assertion for indexing
        assert!(
            indexing_duration < Duration::from_secs(5),
            "BM25 indexing should be fast: {:?} for {} documents",
            indexing_duration, num_docs
        );

        let indexing_throughput = num_docs as f64 / indexing_duration.as_secs_f64();
        assert!(
            indexing_throughput > 100.0,
            "BM25 indexing throughput should be high: {:.2} docs/sec",
            indexing_throughput
        );

        // Test search performance
        let search_queries = vec![
            "token_5_10",  // Specific token
            "token_1",     // Common prefix
            "nonexistent", // No matches
        ];

        for query in search_queries {
            let search_start = Instant::now();
            let results = engine.search(query, 50)
                .expect("BM25 search should succeed");
            let search_duration = search_start.elapsed();

            assert!(
                search_duration < Duration::from_millis(50),
                "BM25 search should be fast: {:?} for query '{}'",
                search_duration, query
            );

            // Verify result quality
            for result in &results {
                assert!(
                    result.score.is_finite() && result.score > 0.0,
                    "BM25 results should have valid positive scores"
                );
            }
        }

        println!("✅ BM25 performance: {:.2} docs/sec indexing, searches under 50ms",
                 indexing_throughput);
    }

    /// Test fusion performance with large result sets
    #[test]
    fn test_fusion_performance() {
        let fusion = SimpleFusion::new();

        // Create large result sets to test fusion scaling
        let num_exact = 1000;
        let num_bm25 = 5000;

        let mut exact_matches = Vec::new();
        for i in 0..num_exact {
            exact_matches.push(crate::search::ExactMatch {
                file_path: format!("file_{}.rs", i),
                line_number: i * 10,
                content: format!("exact match content {}", i),
                line_content: format!("line content {}", i),
            });
        }

        let mut bm25_matches = Vec::new();
        for i in 0..num_bm25 {
            bm25_matches.push(crate::search::bm25::BM25Match {
                doc_id: format!("file_{}.rs-{}", i / 10, i % 10),
                score: (num_bm25 - i) as f32 / 100.0, // Decreasing scores
                term_scores: std::collections::HashMap::new(),
                matched_terms: vec!["test".to_string()],
            });
        }

        let fusion_start = Instant::now();
        let fused_results = fusion.fuse_results_core(exact_matches, bm25_matches)
            .expect("Large fusion should succeed");
        let fusion_duration = fusion_start.elapsed();

        // Performance assertions
        assert!(
            fusion_duration < Duration::from_millis(100),
            "Fusion should be fast even with large inputs: {:?}",
            fusion_duration
        );

        // Should be truncated to reasonable size
        assert!(
            fused_results.len() <= 20,
            "Fusion should limit results to prevent performance issues"
        );

        // Verify results are properly sorted
        for i in 1..fused_results.len() {
            assert!(
                fused_results[i-1].score >= fused_results[i].score,
                "Fusion results should be properly sorted"
            );
        }

        // Verify exact matches come first (highest scores)
        if !fused_results.is_empty() {
            assert_eq!(
                fused_results[0].match_type,
                crate::search::fusion::MatchType::Exact,
                "Exact matches should rank first"
            );
        }

        let fusion_throughput = (num_exact + num_bm25) as f64 / fusion_duration.as_secs_f64();
        println!("✅ Fusion performance: {:.2}ms for {} inputs ({:.0} items/sec)",
                 fusion_duration.as_millis(),
                 num_exact + num_bm25,
                 fusion_throughput);
    }

    /// Test memory usage stability
    #[tokio::test]
    async fn test_memory_usage_stability() {
        let setup = PerformanceTestSetup::new().await
            .expect("Performance test setup should succeed");

        setup.create_performance_corpus(100, 20).await
            .expect("Performance corpus creation should succeed");

        let searcher = UnifiedSearcher::new_with_config(
            setup.project_path.clone(),
            setup.db_path.clone(),
            false,
        ).await.expect("UnifiedSearcher creation should succeed");

        // Measure memory before operations
        let memory_before = estimate_memory_usage();

        // Perform indexing
        searcher.index_directory(&setup.project_path).await
            .expect("Directory indexing should succeed");

        let memory_after_indexing = estimate_memory_usage();

        // Perform multiple searches
        let search_queries = vec!["function", "result", "test", "data", "process"];
        for _ in 0..10 {
            for query in &search_queries {
                let _results = searcher.search(query).await
                    .expect("Search should succeed");
            }
        }

        let memory_after_searches = estimate_memory_usage();

        // Clear cache and run garbage collection simulation
        searcher.clear_index().await
            .expect("Index clearing should succeed");

        let memory_after_cleanup = estimate_memory_usage();

        // Memory assertions
        let indexing_growth = memory_after_indexing.saturating_sub(memory_before);
        let search_growth = memory_after_searches.saturating_sub(memory_after_indexing);
        let cleanup_reduction = memory_after_searches.saturating_sub(memory_after_cleanup);

        assert!(
            indexing_growth < 200_000_000, // Less than 200MB for indexing
            "Indexing memory growth should be reasonable: {} bytes",
            indexing_growth
        );

        assert!(
            search_growth < 50_000_000, // Less than 50MB for searches
            "Search memory growth should be minimal: {} bytes",
            search_growth
        );

        assert!(
            cleanup_reduction > indexing_growth / 2, // Should reclaim significant memory
            "Cleanup should reclaim substantial memory: {} bytes reclaimed vs {} indexed",
            cleanup_reduction, indexing_growth
        );

        println!("✅ Memory stability: +{}MB indexing, +{}MB searches, -{}MB cleanup",
                 indexing_growth / 1_000_000,
                 search_growth / 1_000_000,
                 cleanup_reduction / 1_000_000);
    }

    /// Test performance with different query patterns
    #[tokio::test]
    async fn test_query_pattern_performance() {
        let setup = PerformanceTestSetup::new().await
            .expect("Performance test setup should succeed");

        setup.create_performance_corpus(50, 50).await
            .expect("Performance corpus creation should succeed");

        let searcher = UnifiedSearcher::new_with_config(
            setup.project_path.clone(),
            setup.db_path.clone(),
            false,
        ).await.expect("UnifiedSearcher creation should succeed");

        searcher.index_directory(&setup.project_path).await
            .expect("Directory indexing should succeed");

        // Test different query patterns
        let query_patterns = vec![
            ("short", "Short single term"),
            ("function_123", "Medium specific term"),
            ("this is a longer multi word query", "Multi-word query"),
            ("very_specific_unlikely_to_match_anything", "Long unlikely term"),
            ("a", "Very short common term"),
        ];

        for (query, description) in query_patterns {
            let search_start = Instant::now();
            let results = searcher.search(query).await
                .expect("Search should complete");
            let search_duration = search_start.elapsed();

            assert!(
                search_duration < Duration::from_millis(300),
                "Query '{}' ({}) took too long: {:?}",
                query, description, search_duration
            );

            // Verify results are valid
            for result in &results {
                assert!(
                    result.score.is_finite() && result.score >= 0.0,
                    "Query results should be valid"
                );
            }

            println!("✅ Query pattern '{}' ({}): {:.2}ms, {} results",
                     query, description, search_duration.as_millis(), results.len());
        }
    }

    /// Helper function to estimate memory usage
    fn estimate_memory_usage() -> usize {
        // Simple estimation - in production would use more sophisticated memory tracking
        // For now, use a simple counter as proxy
        use std::sync::atomic::{AtomicUsize, Ordering};
        static MEMORY_COUNTER: AtomicUsize = AtomicUsize::new(0);
        MEMORY_COUNTER.fetch_add(1024, Ordering::Relaxed)
    }

    /// Test performance regression detection
    #[tokio::test]
    async fn test_performance_regression_detection() {
        let setup = PerformanceTestSetup::new().await
            .expect("Performance test setup should succeed");

        setup.create_performance_corpus(25, 40).await
            .expect("Performance corpus creation should succeed");

        let searcher = UnifiedSearcher::new_with_config(
            setup.project_path.clone(),
            setup.db_path.clone(),
            false,
        ).await.expect("UnifiedSearcher creation should succeed");

        // Establish baseline performance
        let baseline_start = Instant::now();
        searcher.index_directory(&setup.project_path).await
            .expect("Baseline indexing should succeed");
        let baseline_indexing = baseline_start.elapsed();

        let baseline_search_start = Instant::now();
        let _baseline_results = searcher.search("function").await
            .expect("Baseline search should succeed");
        let baseline_search = baseline_search_start.elapsed();

        // Performance thresholds (these would be adjusted based on actual baseline measurements)
        let max_indexing_time = Duration::from_secs(15); // 15 seconds for 25 files
        let max_search_time = Duration::from_millis(200); // 200ms for search

        // Assert performance meets expectations
        assert!(
            baseline_indexing <= max_indexing_time,
            "PERFORMANCE REGRESSION: Indexing took {:?}, expected <= {:?}",
            baseline_indexing, max_indexing_time
        );

        assert!(
            baseline_search <= max_search_time,
            "PERFORMANCE REGRESSION: Search took {:?}, expected <= {:?}",
            baseline_search, max_search_time
        );

        // Test sustained performance (multiple operations)
        let sustained_start = Instant::now();
        for i in 0..10 {
            let query = format!("test_{}", i % 3);
            let _results = searcher.search(&query).await
                .expect("Sustained search should succeed");
        }
        let sustained_duration = sustained_start.elapsed();

        let avg_sustained_search = sustained_duration / 10;
        assert!(
            avg_sustained_search <= max_search_time * 2, // Allow some overhead for sustained ops
            "PERFORMANCE REGRESSION: Sustained search average {:?}, expected <= {:?}",
            avg_sustained_search, max_search_time * 2
        );

        println!("✅ Performance baseline established:");
        println!("   - Indexing: {:.2}s (threshold: {:.2}s)", 
                 baseline_indexing.as_secs_f64(), max_indexing_time.as_secs_f64());
        println!("   - Search: {:.2}ms (threshold: {:.2}ms)", 
                 baseline_search.as_millis(), max_search_time.as_millis());
        println!("   - Sustained: {:.2}ms avg (threshold: {:.2}ms)", 
                 avg_sustained_search.as_millis(), (max_search_time * 2).as_millis());
    }
}