//! Comprehensive Error Handling Scenario Tests
//! 
//! This module tests error handling across all components of the search system,
//! ensuring proper error propagation, graceful degradation, and meaningful error messages.

use embed_search::error::{EmbedError, SearchError, StorageError, EmbeddingError};
use embed_search::search::unified::{UnifiedSearcher, SearchResult};
use embed_search::search::bm25::{BM25Engine, BM25Document, Token as BM25Token};
use embed_search::search::fusion::SimpleFusion;
use embed_search::config::{Config, SearchBackend};
use rustc_hash::FxHashMap;
use anyhow::Result;
use tempfile::TempDir;
use std::sync::Arc;

/// Test suite for comprehensive error handling scenarios
#[cfg(test)]
mod comprehensive_error_handling_tests {
    use super::*;

    /// Mock error generator for testing different failure modes
    struct ErrorScenarioGenerator;

    impl ErrorScenarioGenerator {
        /// Create a corrupted BM25 document for testing
        fn create_corrupted_bm25_doc() -> BM25Document {
            BM25Document {
                id: String::new(), // Invalid empty ID
                file_path: "/nonexistent/path.rs".to_string(),
                chunk_index: usize::MAX, // Extreme chunk index
                tokens: vec![], // Empty tokens
                start_line: usize::MAX,
                end_line: 0, // end_line < start_line (invalid)
                language: Some("unknown_language".to_string()),
            }
        }

        /// Create invalid search query scenarios
        fn get_invalid_queries() -> Vec<(String, &'static str)> {
            vec![
                ("".to_string(), "empty query"),
                ("   ".to_string(), "whitespace only"),
                ("\0".to_string(), "null character"),
                ("a".repeat(10000), "extremely long query"),
                ("query\nwith\nnewlines".to_string(), "multiline query"),
                ("query\twith\ttabs".to_string(), "query with tabs"),
                ("query with \x1b[31mANSI\x1b[0m codes".to_string(), "query with ANSI codes"),
            ]
        }
    }

    /// Test BM25 engine error handling
    #[test]
    fn test_bm25_engine_error_handling() {
        let mut engine = BM25Engine::new();

        // Test empty query error
        let empty_query_result = engine.search("", 10);
        assert!(
            empty_query_result.is_err(),
            "BM25 should reject empty query"
        );

        match empty_query_result.unwrap_err() {
            e if e.to_string().contains("Empty query") => {
                // Expected error
            }
            e => panic!("Unexpected error for empty query: {}", e),
        }

        // Test whitespace-only query
        let whitespace_result = engine.search("   \t\n  ", 10);
        assert!(
            whitespace_result.is_err(),
            "BM25 should reject whitespace-only query"
        );

        // Test search on empty index
        let empty_index_result = engine.search("test", 10);
        match empty_index_result {
            Ok(results) => {
                assert!(
                    results.is_empty(),
                    "Empty index should return empty results, not error"
                );
            }
            Err(e) => {
                // Also acceptable - depends on implementation
                assert!(
                    e.to_string().contains("empty") || e.to_string().contains("no documents"),
                    "Empty index error should be descriptive: {}",
                    e
                );
            }
        }

        // Test document with invalid data
        let corrupted_doc = ErrorScenarioGenerator::create_corrupted_bm25_doc();
        let add_result = engine.add_document(corrupted_doc);
        
        match add_result {
            Ok(_) => {
                // System might handle invalid data gracefully
                println!("✅ BM25 handled corrupted document gracefully");
            }
            Err(e) => {
                // Should provide meaningful error message
                assert!(
                    !e.to_string().contains("panic") && 
                    !e.to_string().contains("unwrap") &&
                    e.to_string().len() > 10,
                    "Error should be descriptive and not a panic: {}",
                    e
                );
            }
        }
    }

    /// Test score calculation error handling
    #[test]
    fn test_bm25_score_calculation_errors() {
        let mut engine = BM25Engine::new();

        // Add valid document first
        let valid_doc = BM25Document {
            id: "test-0".to_string(),
            file_path: "test.rs".to_string(),
            chunk_index: 0,
            tokens: vec![
                BM25Token {
                    text: "valid".to_string(),
                    position: 0,
                    importance_weight: 1.0,
                }
            ],
            start_line: 1,
            end_line: 10,
            language: Some("rust".to_string()),
        };

        engine.add_document(valid_doc)
            .expect("Valid document should be added successfully");

        // Test score calculation with nonexistent document
        let nonexistent_score = engine.calculate_bm25_score(&["valid".to_string()], "nonexistent-doc");
        assert!(
            nonexistent_score.is_err(),
            "Score calculation should fail for nonexistent document"
        );

        match nonexistent_score.unwrap_err() {
            e if e.to_string().contains("not found") => {
                // Expected error
            }
            e => panic!("Unexpected error for nonexistent document: {}", e),
        }

        // Test with extremely long query terms
        let long_term = "a".repeat(1000);
        let long_term_result = engine.search(&long_term, 10);
        
        match long_term_result {
            Ok(results) => {
                // Should handle gracefully
                assert!(
                    results.is_empty(),
                    "Long term should typically return no results"
                );
            }
            Err(e) => {
                // Should provide meaningful error
                assert!(
                    !e.to_string().contains("panic"),
                    "Long term error should be graceful: {}",
                    e
                );
            }
        }
    }

    /// Test fusion error handling scenarios
    #[test]
    fn test_fusion_error_handling() {
        let fusion = SimpleFusion::new();

        // Test with corrupted exact matches (empty file paths)
        let corrupted_exact = crate::search::ExactMatch {
            file_path: String::new(), // Invalid empty path
            line_number: 0, // Invalid line number
            content: String::new(),
            line_content: String::new(),
        };

        // Core fusion should handle edge cases gracefully
        let fusion_result = fusion.fuse_results_core(vec![corrupted_exact], vec![]);
        
        match fusion_result {
            Ok(results) => {
                // Might handle gracefully by filtering out invalid entries
                for result in &results {
                    assert!(
                        !result.file_path.is_empty(),
                        "Fusion should filter out empty file paths"
                    );
                }
            }
            Err(e) => {
                // Should provide meaningful error
                assert!(
                    !e.to_string().contains("panic") &&
                    e.to_string().len() > 5,
                    "Fusion error should be descriptive: {}",
                    e
                );
            }
        }

        // Test with invalid BM25 matches
        let invalid_bm25 = embed_search::search::bm25::BM25Match {
            doc_id: "invalid-format".to_string(), // Missing chunk index
            score: f32::NAN, // Invalid score
            term_scores: FxHashMap::default(),
            matched_terms: vec![],
        };

        let bm25_fusion_result = fusion.fuse_results_core(vec![], vec![invalid_bm25]);
        assert!(
            bm25_fusion_result.is_err(),
            "Fusion should reject NaN scores"
        );

        match bm25_fusion_result.unwrap_err() {
            SearchError::CorruptedData { description } => {
                assert!(
                    description.contains("NaN") || description.contains("infinite"),
                    "Error should mention NaN/infinite: {}",
                    description
                );
            }
            SearchError::InvalidDocId { doc_id, expected_format } => {
                assert_eq!(doc_id, "invalid-format");
                assert!(expected_format.contains("filepath-chunkindex"));
            }
            e => panic!("Unexpected error type: {:?}", e),
        }
    }

    /// Test UnifiedSearcher error scenarios
    #[tokio::test]
    async fn test_unified_searcher_error_scenarios() {
        let temp_dir = TempDir::new()
            .expect("Temp directory creation should succeed");

        // Initialize config for test
        std::env::set_var("EMBED_LOG_LEVEL", "error");
        std::env::set_var("EMBED_SEARCH_BACKEND", "tantivy");
        Config::init().expect("Config initialization should succeed");

        let project_path = temp_dir.path().join("project");
        let db_path = temp_dir.path().join("db");

        tokio::fs::create_dir_all(&project_path).await
            .expect("Project directory creation should succeed");
        tokio::fs::create_dir_all(&db_path).await
            .expect("Database directory creation should succeed");

        // Test with invalid project path
        let invalid_project = temp_dir.path().join("nonexistent");
        let searcher_result = UnifiedSearcher::new(invalid_project.clone(), db_path.clone()).await;
        
        match searcher_result {
            Ok(searcher) => {
                // Might succeed but operations should fail gracefully
                let search_result = searcher.search("test").await;
                match search_result {
                    Ok(results) => {
                        assert!(
                            results.is_empty(),
                            "Search on invalid project should return empty results"
                        );
                    }
                    Err(e) => {
                        assert!(
                            !e.to_string().contains("panic"),
                            "Search error should be graceful: {}",
                            e
                        );
                    }
                }
            }
            Err(e) => {
                assert!(
                    e.to_string().contains("path") || e.to_string().contains("directory"),
                    "Invalid path error should be descriptive: {}",
                    e
                );
            }
        }

        // Test with valid paths but edge case operations
        let searcher = UnifiedSearcher::new(project_path.clone(), db_path).await
            .expect("Valid UnifiedSearcher creation should succeed");

        // Test search with invalid queries
        for (query, description) in ErrorScenarioGenerator::get_invalid_queries() {
            let search_result = searcher.search(&query).await;
            
            match search_result {
                Ok(results) => {
                    // Empty results are acceptable for edge cases
                    assert!(
                        results.is_empty(),
                        "Edge case query '{}' ({}) should return empty results",
                        query, description
                    );
                }
                Err(e) => {
                    // Errors should be descriptive and graceful
                    let error_msg = e.to_string();
                    assert!(
                        !error_msg.contains("panic") && 
                        !error_msg.contains("unwrap") &&
                        error_msg.len() > 5,
                        "Error for '{}' ({}) should be descriptive: {}",
                        query, description, error_msg
                    );
                }
            }
        }

        // Test indexing nonexistent files
        let nonexistent_file = project_path.join("nonexistent.rs");
        let index_result = searcher.index_file(&nonexistent_file).await;
        
        assert!(
            index_result.is_err(),
            "Indexing nonexistent file should fail"
        );

        match index_result.unwrap_err() {
            e if e.to_string().contains("No such file") ||
                 e.to_string().contains("not found") => {
                // Expected file not found error
            }
            e => {
                // Should still be a descriptive I/O error
                assert!(
                    !e.to_string().contains("panic"),
                    "File not found error should be graceful: {}",
                    e
                );
            }
        }
    }

    /// Test error recovery and graceful degradation
    #[tokio::test]
    async fn test_error_recovery_and_degradation() {
        let temp_dir = TempDir::new()
            .expect("Temp directory creation should succeed");

        std::env::set_var("EMBED_LOG_LEVEL", "warn");
        Config::init().expect("Config initialization should succeed");

        let project_path = temp_dir.path().join("project");
        let db_path = temp_dir.path().join("db");

        tokio::fs::create_dir_all(&project_path).await
            .expect("Directory creation should succeed");
        tokio::fs::create_dir_all(&db_path).await
            .expect("Directory creation should succeed");

        // Create test files with some invalid content
        let large_content = "x".repeat(1_000_000);
        let test_files = vec![
            ("valid.rs", "fn main() { println!(\"Hello\"); }"),
            ("empty.rs", ""), // Empty file
            ("binary.bin", "\x00\x01\x02\x03\x04"), // Binary content
            ("large.rs", large_content.as_str()), // Very large file
        ];

        for (filename, content) in test_files {
            let file_path = project_path.join(filename);
            tokio::fs::write(file_path, content).await
                .expect("Test file creation should succeed");
        }

        let searcher = UnifiedSearcher::new(project_path.clone(), db_path).await
            .expect("UnifiedSearcher creation should succeed");

        // Test directory indexing with mixed file types
        let index_stats = searcher.index_directory(&project_path).await;
        
        match index_stats {
            Ok(stats) => {
                // Should successfully index at least the valid files
                assert!(
                    stats.files_indexed > 0,
                    "Should index at least some valid files"
                );

                // Some files might cause errors but shouldn't stop the process
                println!("✅ Indexed {} files with {} errors", 
                         stats.files_indexed, stats.errors);
            }
            Err(e) => {
                // Complete failure should still be handled gracefully
                assert!(
                    !e.to_string().contains("panic"),
                    "Indexing error should be graceful: {}",
                    e
                );
            }
        }

        // Test search functionality even with partial indexing
        let search_results = searcher.search("main").await;
        
        match search_results {
            Ok(results) => {
                // Should work with successfully indexed files
                for result in &results {
                    assert!(
                        result.score.is_finite() && result.score >= 0.0,
                        "Results should be valid even with partial failures"
                    );
                }
            }
            Err(e) => {
                // Search failure should be handled gracefully
                assert!(
                    !e.to_string().contains("panic"),
                    "Search error should be graceful: {}",
                    e
                );
            }
        }
    }

    /// Test concurrent error handling
    #[tokio::test]
    async fn test_concurrent_error_handling() {
        let temp_dir = TempDir::new()
            .expect("Temp directory creation should succeed");

        std::env::set_var("EMBED_LOG_LEVEL", "error");
        Config::init().expect("Config initialization should succeed");

        let project_path = temp_dir.path().join("project");
        let db_path = temp_dir.path().join("db");

        tokio::fs::create_dir_all(&project_path).await
            .expect("Directory creation should succeed");
        tokio::fs::create_dir_all(&db_path).await
            .expect("Directory creation should succeed");

        let searcher = Arc::new(
            UnifiedSearcher::new(project_path, db_path).await
                .expect("UnifiedSearcher creation should succeed")
        );

        // Launch multiple concurrent operations that might fail
        let mut handles = Vec::new();
        
        for i in 0..10 {
            let searcher_clone = Arc::clone(&searcher);
            let handle = tokio::spawn(async move {
                let invalid_queries = ErrorScenarioGenerator::get_invalid_queries();
                let query = &invalid_queries[i % invalid_queries.len()].0;
                
                let result = searcher_clone.search(query).await;
                (i, query, result)
            });
            handles.push(handle);
        }

        // Wait for all concurrent operations
        let results = futures::future::try_join_all(handles).await
            .expect("All concurrent tasks should complete without panic");

        // Verify all operations completed without panicking
        let num_results = results.len();
        for (task_id, _query, result) in results {
            match result {
                Ok(search_results) => {
                    // Empty results are fine for invalid queries
                    assert!(
                        search_results.is_empty(),
                        "Task {} with invalid query should return empty results",
                        task_id
                    );
                }
                Err(e) => {
                    // Errors should be graceful
                    assert!(
                        !e.to_string().contains("panic") && 
                        !e.to_string().contains("unwrap"),
                        "Task {} error should be graceful: {}",
                        task_id, e
                    );
                }
            }
        }

        println!("✅ All {} concurrent error scenarios completed gracefully", num_results);
    }

    /// Test error message quality and informativeness
    #[test]
    fn test_error_message_quality() {
        // Test SearchError variants
        let invalid_doc_id_error = SearchError::InvalidDocId {
            doc_id: "malformed-id".to_string(),
            expected_format: "filepath-chunkindex".to_string(),
        };

        let error_msg = invalid_doc_id_error.to_string();
        assert!(error_msg.contains("malformed-id"), "Error should include the invalid doc ID");
        assert!(error_msg.contains("filepath-chunkindex"), "Error should include expected format");
        assert!(error_msg.len() > 20, "Error message should be descriptive");

        // Test missing similarity score error
        let missing_score_error = SearchError::MissingSimilarityScore {
            file_path: "test.rs".to_string(),
            chunk_index: 5,
        };

        let score_error_msg = missing_score_error.to_string();
        assert!(score_error_msg.contains("test.rs"), "Error should include file path");
        assert!(score_error_msg.contains("5"), "Error should include chunk index");
        assert!(score_error_msg.contains("similarity"), "Error should mention similarity score");

        // Test corrupted data error
        let corrupted_error = SearchError::CorruptedData {
            description: "NaN score detected in fusion results".to_string(),
        };

        let corrupted_msg = corrupted_error.to_string();
        assert!(corrupted_msg.contains("NaN"), "Error should include corruption details");
        assert!(corrupted_msg.contains("Corrupted"), "Error should indicate corruption");

        // Test query invalid error
        let query_error = SearchError::QueryInvalid {
            message: "Query contains null characters".to_string(),
            query: "test\0query".to_string(),
        };

        let query_msg = query_error.to_string();
        assert!(query_msg.contains("null characters"), "Error should explain the problem");
        assert!(query_msg.contains("test"), "Error should include part of the query");

        println!("✅ All error messages are informative and specific");
    }

    /// Test error chain and source preservation
    #[test]
    fn test_error_chain_preservation() {
        use std::error::Error as StdError;

        // Create nested error chain
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let embed_error = EmbedError::from(io_error);

        // Verify error chain is preserved
        let error_msg = embed_error.to_string();
        assert!(error_msg.contains("file not found"), "Original error message should be preserved");

        // Check source chain
        let mut source_count = 0;
        let mut current_error: &dyn StdError = &embed_error;
        
        while let Some(source) = current_error.source() {
            current_error = source;
            source_count += 1;
            
            // Prevent infinite loops
            if source_count > 10 {
                break;
            }
        }

        assert!(source_count > 0, "Error should have source chain");

        // Test conversion chains
        let storage_error = StorageError::ConnectionFailed {
            message: "Database connection failed".to_string(),
            url: Some("localhost:5432".to_string()),
        };

        let converted_embed_error = EmbedError::from(storage_error);
        assert!(converted_embed_error.to_string().contains("Database connection failed"));

        let embedding_error = EmbeddingError::ModelNotLoaded {
            model_name: "nomic-embed".to_string(),
        };

        let converted_embed_error2 = EmbedError::from(embedding_error);
        assert!(converted_embed_error2.to_string().contains("nomic-embed"));

        println!("✅ Error chain preservation verified");
    }

    /// Test resource cleanup on errors
    #[tokio::test]
    async fn test_resource_cleanup_on_errors() {
        let temp_dir = TempDir::new()
            .expect("Temp directory creation should succeed");

        std::env::set_var("EMBED_LOG_LEVEL", "error");
        Config::init().expect("Config initialization should succeed");

        let project_path = temp_dir.path().join("project");
        let db_path = temp_dir.path().join("db");

        tokio::fs::create_dir_all(&project_path).await
            .expect("Directory creation should succeed");
        tokio::fs::create_dir_all(&db_path).await
            .expect("Directory creation should succeed");

        // Test resource cleanup when operations fail
        {
            let searcher = UnifiedSearcher::new(project_path.clone(), db_path.clone()).await
                .expect("UnifiedSearcher creation should succeed");

            // Perform operations that might fail
            let _search_result = searcher.search("").await; // Empty query
            let _invalid_index = searcher.index_file(&project_path.join("nonexistent.rs")).await;
            
            // Clear index (might succeed or fail)
            let _clear_result = searcher.clear_index().await;
        } // searcher should be cleaned up here

        // Verify we can create new searcher (resources were released)
        let new_searcher = UnifiedSearcher::new(project_path, db_path).await;
        assert!(
            new_searcher.is_ok(),
            "Should be able to create new searcher after cleanup"
        );

        println!("✅ Resource cleanup on errors verified");
    }
}