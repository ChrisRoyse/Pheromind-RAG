//! Fusion Score Normalization Edge Case Tests
//! 
//! This module tests the fusion system's score normalization and edge case handling,
//! ensuring proper behavior with extreme scores, NaN values, and ranking correctness.

use embed_search::search::fusion::{SimpleFusion, FusedResult, MatchType};
use embed_search::search::ExactMatch;
use embed_search::search::bm25::BM25Match;
use embed_search::error::SearchError;
use std::collections::HashMap;
use rustc_hash::FxHashMap;

#[cfg(feature = "vectordb")]
use embed_search::storage::lancedb_storage::LanceEmbeddingRecord;

#[cfg(feature = "tree-sitter")]
use embed_search::search::symbol_index::Symbol;

/// Test suite for fusion score normalization and edge cases
#[cfg(test)]
mod fusion_score_normalization_tests {
    use super::*;

    /// Mock factory for creating test data
    struct TestDataFactory;

    impl TestDataFactory {
        fn create_exact_match(file_path: &str, line_number: usize, content: &str) -> ExactMatch {
            ExactMatch {
                file_path: file_path.to_string(),
                line_number,
                content: content.to_string(),
                line_content: content.to_string(),
            }
        }

        fn create_bm25_match(doc_id: &str, score: f32) -> BM25Match {
            BM25Match {
                doc_id: doc_id.to_string(),
                score,
                term_scores: FxHashMap::default(),
                matched_terms: vec!["test".to_string()],
            }
        }

        #[cfg(feature = "vectordb")]
        fn create_semantic_match(file_path: &str, chunk_index: u64, similarity: f32) -> LanceEmbeddingRecord {
            LanceEmbeddingRecord {
                id: format!("{}-{}", file_path, chunk_index),
                file_path: file_path.to_string(),
                chunk_index,
                content: "semantic match content".to_string(),
                embedding: vec![0.1; 768],
                start_line: chunk_index * 10,
                end_line: (chunk_index + 1) * 10 - 1,
                similarity_score: Some(similarity),
            }
        }

        #[cfg(feature = "tree-sitter")]
        fn create_symbol_match(file_path: &str, line: usize, name: &str) -> Symbol {
            Symbol {
                name: name.to_string(),
                file_path: file_path.to_string(),
                line,
                text: format!("fn {}() {{}}", name),
                start_line: Some(line),
                end_line: Some(line + 5),
                kind: crate::search::symbol_index::SymbolKind::Function,
            }
        }
    }

    /// Test BM25 score normalization edge cases
    #[test]
    fn test_bm25_score_normalization_edge_cases() {
        let fusion = SimpleFusion::new();

        // Test extreme BM25 scores
        let extreme_scores = vec![0.0, 0.001, 1.0, 10.0, 20.0, 50.0, 100.0, 1000.0];
        let mut bm25_matches = Vec::new();

        for (i, score) in extreme_scores.iter().enumerate() {
            bm25_matches.push(TestDataFactory::create_bm25_match(
                &format!("test{}.rs-{}", i, i),
                *score
            ));
        }

        // Test core fusion (BM25 + Exact only)
        let exact_matches = vec![
            TestDataFactory::create_exact_match("exact.rs", 10, "exact match content")
        ];

        let results = fusion.fuse_results_core(exact_matches, bm25_matches)
            .expect("Core fusion must succeed with extreme BM25 scores");

        // Verify exact match has highest score
        assert!(
            !results.is_empty() && results[0].match_type == MatchType::Exact,
            "Exact match should rank first with score 1.0"
        );
        assert_eq!(results[0].score, 1.0, "Exact match must have perfect score");

        // Verify BM25 scores are properly normalized (0-0.9 range)
        for result in &results {
            if result.match_type == MatchType::Statistical {
                assert!(
                    result.score >= 0.0 && result.score <= 0.9,
                    "BM25 score {} must be normalized to [0.0, 0.9] range",
                    result.score
                );
                
                assert!(
                    result.score.is_finite(),
                    "BM25 score must be finite, got {}",
                    result.score
                );
            }
        }

        // Verify ordering: higher BM25 scores get higher normalized scores
        let bm25_results: Vec<_> = results.iter()
            .filter(|r| r.match_type == MatchType::Statistical)
            .collect();

        for i in 1..bm25_results.len() {
            assert!(
                bm25_results[i-1].score >= bm25_results[i].score,
                "BM25 results should be in descending score order: {} >= {}",
                bm25_results[i-1].score, bm25_results[i].score
            );
        }
    }

    /// Test invalid/corrupted score detection and error handling
    #[test]
    fn test_invalid_score_detection() {
        let fusion = SimpleFusion::new();

        // Test NaN score rejection
        let nan_bm25 = TestDataFactory::create_bm25_match("nan-test.rs-0", f32::NAN);
        let result = fusion.fuse_results_core(vec![], vec![nan_bm25]);
        
        assert!(
            result.is_err(),
            "Fusion must reject NaN scores and return error"
        );

        match result.unwrap_err() {
            SearchError::CorruptedData { description } => {
                assert!(
                    description.contains("NaN") || description.contains("infinite"),
                    "Error should mention NaN/infinite values: {}",
                    description
                );
            }
            _ => panic!("Expected CorruptedData error for NaN score"),
        }

        // Test infinite score rejection
        let inf_bm25 = TestDataFactory::create_bm25_match("inf-test.rs-0", f32::INFINITY);
        let result = fusion.fuse_results_core(vec![], vec![inf_bm25]);
        
        assert!(
            result.is_err(),
            "Fusion must reject infinite scores and return error"
        );
    }

    /// Test score normalization consistency across different ranges
    #[test]
    fn test_score_normalization_consistency() {
        let fusion = SimpleFusion::new();

        // Test with different BM25 score ranges
        let test_ranges = vec![
            vec![0.1, 0.2, 0.3],           // Low scores
            vec![1.0, 2.0, 3.0],           // Medium scores  
            vec![10.0, 15.0, 20.0],        // High scores
            vec![50.0, 75.0, 100.0],       // Very high scores
        ];

        for (range_idx, scores) in test_ranges.iter().enumerate() {
            let mut bm25_matches = Vec::new();
            for (i, &score) in scores.iter().enumerate() {
                bm25_matches.push(TestDataFactory::create_bm25_match(
                    &format!("range{}-{}.rs-{}", range_idx, i, i),
                    score
                ));
            }

            let results = fusion.fuse_results_core(vec![], bm25_matches)
                .expect("Normalization should work for all ranges");

            // Verify all scores are in normalized range
            for result in &results {
                assert!(
                    result.score >= 0.0 && result.score <= 0.9,
                    "Range {} score {} must be in [0.0, 0.9]",
                    range_idx, result.score
                );
            }

            // Verify relative ordering is preserved within range
            for i in 1..results.len() {
                assert!(
                    results[i-1].score >= results[i].score,
                    "Relative ordering must be preserved in range {}: {} >= {}",
                    range_idx, results[i-1].score, results[i].score
                );
            }
        }
    }

    /// Test edge case with zero and near-zero scores
    #[test]
    fn test_zero_and_near_zero_scores() {
        let fusion = SimpleFusion::new();

        let bm25_matches = vec![
            TestDataFactory::create_bm25_match("zero.rs-0", 0.0),
            TestDataFactory::create_bm25_match("tiny.rs-0", 0.0001),
            TestDataFactory::create_bm25_match("small.rs-0", 0.001),
            TestDataFactory::create_bm25_match("larger.rs-0", 0.01),
        ];

        let results = fusion.fuse_results_core(vec![], bm25_matches)
            .expect("Zero scores should be handled gracefully");

        // All scores should be valid
        for result in &results {
            assert!(
                result.score >= 0.0,
                "Score should be non-negative, got {}",
                result.score
            );
            
            assert!(
                result.score.is_finite(),
                "Score should be finite, got {}",
                result.score
            );
        }

        // Zero scores should still be included but with zero normalized score
        let zero_result = results.iter()
            .find(|r| r.file_path == "zero.rs")
            .expect("Zero score result should be included");
        
        assert_eq!(
            zero_result.score, 0.0,
            "Zero BM25 score should remain zero after normalization"
        );
    }

    /// Test doc ID parsing edge cases and error handling
    #[test]
    fn test_doc_id_parsing_edge_cases() {
        let fusion = SimpleFusion::new();

        // Test malformed doc IDs
        let invalid_doc_ids = vec![
            "no-dash",              // Missing chunk index
            "multiple-dash-es-1",   // Multiple dashes (should use last)
            "-5",                   // Missing file path
            "file.rs-",             // Missing chunk index
            "file.rs-abc",          // Non-numeric chunk index
            "file.rs--1",           // Double dash
        ];

        for invalid_id in invalid_doc_ids {
            let bm25_match = TestDataFactory::create_bm25_match(invalid_id, 5.0);
            let result = fusion.fuse_results_core(vec![], vec![bm25_match]);

            match invalid_id {
                "multiple-dash-es-1" => {
                    // Should succeed - takes last dash split
                    assert!(
                        result.is_ok(),
                        "Multiple dashes should work by taking last split: {}",
                        invalid_id
                    );
                }
                "file.rs-abc" => {
                    // Should fail - non-numeric chunk index
                    assert!(
                        result.is_err(),
                        "Non-numeric chunk index should fail: {}",
                        invalid_id
                    );
                    
                    if let Err(SearchError::InvalidDocId { doc_id, expected_format }) = result {
                        assert_eq!(doc_id, invalid_id);
                        assert!(expected_format.contains("numeric"));
                    }
                }
                _ => {
                    // Other cases should fail
                    assert!(
                        result.is_err(),
                        "Invalid doc ID should fail: {}",
                        invalid_id
                    );
                }
            }
        }
    }

    /// Test semantic score missing similarity handling
    #[cfg(feature = "vectordb")]
    #[test]
    fn test_semantic_score_missing_similarity() {
        let fusion = SimpleFusion::new();

        // Create semantic match with missing similarity score
        let mut semantic_match = TestDataFactory::create_semantic_match("test.rs", 0, 0.8);
        semantic_match.similarity_score = None; // Remove similarity score

        let result = fusion.fuse_results(vec![], vec![semantic_match]);
        
        assert!(
            result.is_err(),
            "Fusion should fail when similarity score is missing"
        );

        match result.unwrap_err() {
            SearchError::MissingSimilarityScore { file_path, chunk_index } => {
                assert_eq!(file_path, "test.rs");
                assert_eq!(chunk_index, 0);
            }
            _ => panic!("Expected MissingSimilarityScore error"),
        }
    }

    /// Test ranking optimization with corrupted scores
    #[test]
    fn test_ranking_optimization_corrupted_scores() {
        let fusion = SimpleFusion::new();

        // Create results with valid scores initially
        let mut fused_results = vec![
            FusedResult {
                file_path: "test1.rs".to_string(),
                line_number: Some(10),
                chunk_index: None,
                score: 0.8,
                match_type: MatchType::Exact,
                content: "test content".to_string(),
                start_line: 10,
                end_line: 10,
            },
            FusedResult {
                file_path: "test2.rs".to_string(),
                line_number: None,
                chunk_index: Some(1),
                score: 0.6,
                match_type: MatchType::Semantic,
                content: "semantic content".to_string(),
                start_line: 20,
                end_line: 30,
            },
        ];

        // Manually corrupt one score to test detection
        fused_results[1].score = f32::NAN;

        let result = fusion.optimize_ranking(&mut fused_results, "test query");
        
        assert!(
            result.is_err(),
            "Ranking optimization should detect and reject NaN scores"
        );

        match result.unwrap_err() {
            SearchError::CorruptedData { description } => {
                assert!(
                    description.contains("Invalid score") && description.contains("test2.rs"),
                    "Error should identify corrupted score and file: {}",
                    description
                );
            }
            _ => panic!("Expected CorruptedData error for ranking optimization"),
        }
    }

    /// Test weighted fusion scoring edge cases
    #[test]
    fn test_weighted_fusion_edge_cases() {
        let fusion = SimpleFusion::new();

        // Create results with different match types
        let exact_match = TestDataFactory::create_exact_match("exact.rs", 10, "exact");
        let bm25_match = TestDataFactory::create_bm25_match("bm25.rs-1", 15.0);

        #[cfg(all(feature = "vectordb", feature = "tree-sitter"))]
        {
            let semantic_match = TestDataFactory::create_semantic_match("semantic.rs", 0, 0.9);
            let symbol_match = TestDataFactory::create_symbol_match("symbol.rs", 5, "test_func");

            let results = fusion.fuse_all_results_with_bm25(
                vec![exact_match],
                vec![semantic_match],
                vec![symbol_match],
                vec![bm25_match],
            ).expect("4-way fusion should work");

            // Verify weighted scoring maintains correct ordering
            for result in &results {
                match result.match_type {
                    MatchType::Exact => assert_eq!(result.score, 1.0),
                    MatchType::Symbol => assert!(result.score >= 0.95),
                    MatchType::Statistical => assert!(result.score <= 0.9),
                    MatchType::Semantic => assert!(result.score <= 0.7 * 0.9), // semantic weight * max score
                }
            }

            // Verify exact matches come first
            assert_eq!(results[0].match_type, MatchType::Exact);
        }

        #[cfg(not(all(feature = "vectordb", feature = "tree-sitter")))]
        {
            // Test core fusion only
            let results = fusion.fuse_results_core(vec![exact_match], vec![bm25_match])
                .expect("Core fusion should work");

            assert_eq!(results[0].match_type, MatchType::Exact);
            assert_eq!(results[0].score, 1.0);
        }
    }

    /// Test deduplication with edge cases
    #[test]
    fn test_deduplication_edge_cases() {
        let fusion = SimpleFusion::new();

        // Create exact duplicates
        let duplicate_exact = vec![
            TestDataFactory::create_exact_match("test.rs", 10, "content1"),
            TestDataFactory::create_exact_match("test.rs", 10, "content2"), // Same location, different content
        ];

        let results = fusion.fuse_results_core(duplicate_exact, vec![])
            .expect("Deduplication should handle exact duplicates");

        // Should only have one result after deduplication
        assert_eq!(
            results.len(), 1,
            "Duplicates should be removed, got {} results",
            results.len()
        );

        // Should keep first occurrence
        assert_eq!(results[0].content, "content1");
    }

    /// Test result truncation at limits
    #[test]
    fn test_result_truncation() {
        let fusion = SimpleFusion::new();

        // Create more than 20 BM25 matches (fusion limit)
        let mut bm25_matches = Vec::new();
        for i in 0..30 {
            bm25_matches.push(TestDataFactory::create_bm25_match(
                &format!("file{}.rs-0", i),
                (30 - i) as f32, // Descending scores
            ));
        }

        let results = fusion.fuse_results_core(vec![], bm25_matches)
            .expect("Large result set should be handled");

        // Should be truncated to 20 results
        assert_eq!(
            results.len(), 20,
            "Results should be truncated to 20, got {}",
            results.len()
        );

        // Should keep highest scoring results
        for i in 1..results.len() {
            assert!(
                results[i-1].score >= results[i].score,
                "Results should be in descending order after truncation"
            );
        }
    }

    /// Performance stress test with extreme cases
    #[test]
    fn test_fusion_performance_stress() {
        let fusion = SimpleFusion::new();

        // Create large numbers of each match type
        let mut bm25_matches = Vec::new();
        for i in 0..1000 {
            bm25_matches.push(TestDataFactory::create_bm25_match(
                &format!("large{}.rs-{}", i % 100, i % 10),
                (i as f32) / 100.0,
            ));
        }

        let start = std::time::Instant::now();
        let results = fusion.fuse_results_core(vec![], bm25_matches)
            .expect("Large fusion should complete");
        let duration = start.elapsed();

        // Should complete quickly (within 1 second for 1000 items)
        assert!(
            duration.as_secs() < 1,
            "Large fusion took too long: {:?}",
            duration
        );

        // Should be truncated properly
        assert_eq!(results.len(), 20);

        // All scores should be valid
        for result in &results {
            assert!(
                result.score.is_finite() && result.score >= 0.0,
                "Score must be finite and non-negative: {}",
                result.score
            );
        }

        println!("✅ Stress test completed in {:?} with {} results", duration, results.len());
    }
}