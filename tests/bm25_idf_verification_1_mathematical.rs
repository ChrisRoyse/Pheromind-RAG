#[cfg(test)]
mod bm25_idf_mathematical_verification {
    use crate::search::bm25::BM25Scorer;
    use std::collections::HashMap;

    /// Mathematical verification of BM25 IDF calculation
    /// Tests the exact formula: IDF = log((N - df + 0.5) / (df + 0.5))
    /// Where N = total documents, df = document frequency of term
    #[test]
    fn test_bm25_idf_mathematical_correctness() {
        // Create 5 test documents with known term distributions
        let documents = vec![
            vec!["rare".to_string(), "common".to_string(), "universal".to_string()], // Doc 0
            vec!["rare".to_string(), "medium".to_string(), "common".to_string(), "universal".to_string()], // Doc 1
            vec!["medium".to_string(), "frequent".to_string(), "common".to_string(), "universal".to_string()], // Doc 2
            vec!["medium".to_string(), "frequent".to_string(), "common".to_string(), "universal".to_string()], // Doc 3
            vec!["frequent".to_string(), "common".to_string(), "universal".to_string()], // Doc 4
        ];

        // Term distribution analysis:
        // "rare": appears in 2 documents (0, 1) -> df = 2
        // "medium": appears in 3 documents (1, 2, 3) -> df = 3  
        // "frequent": appears in 3 documents (2, 3, 4) -> df = 3
        // "common": appears in 5 documents (0, 1, 2, 3, 4) -> df = 5
        // "universal": appears in 5 documents (0, 1, 2, 3, 4) -> df = 5
        // "nonexistent": appears in 0 documents -> df = 0

        let total_docs = documents.len() as f64; // N = 5.0
        let scorer = BM25Scorer::new(documents, 1.5, 0.75);

        // Calculate expected IDF values using exact BM25 formula
        // IDF = log((N - df + 0.5) / (df + 0.5))

        // For "rare" (df = 2):
        let expected_rare_idf = ((total_docs - 2.0 + 0.5) / (2.0 + 0.5)).ln();
        // = log((5 - 2 + 0.5) / (2 + 0.5)) = log(3.5 / 2.5) = log(1.4) ≈ 0.3365

        // For "medium" and "frequent" (df = 3):
        let expected_medium_idf = ((total_docs - 3.0 + 0.5) / (3.0 + 0.5)).ln();
        // = log((5 - 3 + 0.5) / (3 + 0.5)) = log(2.5 / 3.5) = log(0.7143) ≈ -0.3365

        // For "common" and "universal" (df = 5):
        let expected_common_idf = ((total_docs - 5.0 + 0.5) / (5.0 + 0.5)).ln();
        // = log((5 - 5 + 0.5) / (5 + 0.5)) = log(0.5 / 5.5) = log(0.0909) ≈ -2.3979

        // For "nonexistent" (df = 0):
        let expected_nonexistent_idf = ((total_docs - 0.0 + 0.5) / (0.0 + 0.5)).ln();
        // = log((5 - 0 + 0.5) / (0 + 0.5)) = log(5.5 / 0.5) = log(11.0) ≈ 2.3979

        // Test each IDF calculation with precise tolerance
        let tolerance = 1e-10; // Very strict tolerance for mathematical precision

        // Test "rare" term (highest IDF among existing terms)
        let actual_rare_idf = scorer.calculate_idf("rare");
        assert!(
            (actual_rare_idf - expected_rare_idf).abs() < tolerance,
            "IDF for 'rare' is incorrect. Expected: {:.10}, Actual: {:.10}, Diff: {:.10}",
            expected_rare_idf, actual_rare_idf, (actual_rare_idf - expected_rare_idf).abs()
        );

        // Test "medium" term
        let actual_medium_idf = scorer.calculate_idf("medium");
        assert!(
            (actual_medium_idf - expected_medium_idf).abs() < tolerance,
            "IDF for 'medium' is incorrect. Expected: {:.10}, Actual: {:.10}, Diff: {:.10}",
            expected_medium_idf, actual_medium_idf, (actual_medium_idf - expected_medium_idf).abs()
        );

        // Test "frequent" term (should equal "medium" since same df)
        let actual_frequent_idf = scorer.calculate_idf("frequent");
        assert!(
            (actual_frequent_idf - expected_medium_idf).abs() < tolerance,
            "IDF for 'frequent' is incorrect. Expected: {:.10}, Actual: {:.10}, Diff: {:.10}",
            expected_medium_idf, actual_frequent_idf, (actual_frequent_idf - expected_medium_idf).abs()
        );

        // Test "common" term (lowest IDF among existing terms)
        let actual_common_idf = scorer.calculate_idf("common");
        assert!(
            (actual_common_idf - expected_common_idf).abs() < tolerance,
            "IDF for 'common' is incorrect. Expected: {:.10}, Actual: {:.10}, Diff: {:.10}",
            expected_common_idf, actual_common_idf, (actual_common_idf - expected_common_idf).abs()
        );

        // Test "universal" term (should equal "common" since same df)
        let actual_universal_idf = scorer.calculate_idf("universal");
        assert!(
            (actual_universal_idf - expected_common_idf).abs() < tolerance,
            "IDF for 'universal' is incorrect. Expected: {:.10}, Actual: {:.10}, Diff: {:.10}",
            expected_common_idf, actual_universal_idf, (actual_universal_idf - expected_common_idf).abs()
        );

        // Test "nonexistent" term (should have highest IDF)
        let actual_nonexistent_idf = scorer.calculate_idf("nonexistent");
        assert!(
            (actual_nonexistent_idf - expected_nonexistent_idf).abs() < tolerance,
            "IDF for 'nonexistent' is incorrect. Expected: {:.10}, Actual: {:.10}, Diff: {:.10}",
            expected_nonexistent_idf, actual_nonexistent_idf, (actual_nonexistent_idf - expected_nonexistent_idf).abs()
        );
    }

    #[test]
    fn test_bm25_idf_monotonic_ordering() {
        // Create test corpus with different term frequencies
        let documents = vec![
            vec!["alpha".to_string(), "beta".to_string(), "gamma".to_string(), "delta".to_string(), "epsilon".to_string()],
            vec!["alpha".to_string(), "beta".to_string(), "gamma".to_string(), "delta".to_string()],
            vec!["alpha".to_string(), "beta".to_string(), "gamma".to_string()],
            vec!["alpha".to_string(), "beta".to_string()],
            vec!["alpha".to_string()],
        ];

        // Term frequencies:
        // "alpha": 5 docs (df=5) - lowest IDF
        // "beta": 4 docs (df=4) 
        // "gamma": 3 docs (df=3)
        // "delta": 2 docs (df=2)
        // "epsilon": 1 doc (df=1) - highest IDF

        let scorer = BM25Scorer::new(documents, 1.5, 0.75);

        let idf_alpha = scorer.calculate_idf("alpha");
        let idf_beta = scorer.calculate_idf("beta");
        let idf_gamma = scorer.calculate_idf("gamma");
        let idf_delta = scorer.calculate_idf("delta");
        let idf_epsilon = scorer.calculate_idf("epsilon");

        // Verify monotonic decreasing order (higher df = lower IDF)
        assert!(
            idf_epsilon > idf_delta,
            "IDF ordering violation: epsilon (df=1, IDF={:.6}) should > delta (df=2, IDF={:.6})",
            idf_epsilon, idf_delta
        );

        assert!(
            idf_delta > idf_gamma,
            "IDF ordering violation: delta (df=2, IDF={:.6}) should > gamma (df=3, IDF={:.6})",
            idf_delta, idf_gamma
        );

        assert!(
            idf_gamma > idf_beta,
            "IDF ordering violation: gamma (df=3, IDF={:.6}) should > beta (df=4, IDF={:.6})",
            idf_gamma, idf_beta
        );

        assert!(
            idf_beta > idf_alpha,
            "IDF ordering violation: beta (df=4, IDF={:.6}) should > alpha (df=5, IDF={:.6})",
            idf_beta, idf_alpha
        );

        // Additional verification: all existing terms should have finite, real values
        assert!(idf_alpha.is_finite(), "Alpha IDF should be finite, got: {}", idf_alpha);
        assert!(idf_beta.is_finite(), "Beta IDF should be finite, got: {}", idf_beta);
        assert!(idf_gamma.is_finite(), "Gamma IDF should be finite, got: {}", idf_gamma);
        assert!(idf_delta.is_finite(), "Delta IDF should be finite, got: {}", idf_delta);
        assert!(idf_epsilon.is_finite(), "Epsilon IDF should be finite, got: {}", idf_epsilon);
    }

    #[test]
    fn test_bm25_idf_edge_cases() {
        // Single document corpus
        let single_doc = vec![
            vec!["solo".to_string(), "term".to_string()],
        ];
        let single_scorer = BM25Scorer::new(single_doc, 1.5, 0.75);

        // For single document, all existing terms have df=1, N=1
        // IDF = log((1 - 1 + 0.5) / (1 + 0.5)) = log(0.5 / 1.5) = log(1/3) ≈ -1.0986
        let expected_single_idf = (0.5 / 1.5).ln();
        let actual_solo_idf = single_scorer.calculate_idf("solo");
        
        assert!(
            (actual_solo_idf - expected_single_idf).abs() < 1e-10,
            "Single doc IDF incorrect. Expected: {:.10}, Actual: {:.10}",
            expected_single_idf, actual_solo_idf
        );

        // Non-existent term in single doc: df=0, N=1
        // IDF = log((1 - 0 + 0.5) / (0 + 0.5)) = log(1.5 / 0.5) = log(3) ≈ 1.0986
        let expected_nonexistent_single_idf = (1.5 / 0.5).ln();
        let actual_nonexistent_single_idf = single_scorer.calculate_idf("ghost");
        
        assert!(
            (actual_nonexistent_single_idf - expected_nonexistent_single_idf).abs() < 1e-10,
            "Single doc non-existent term IDF incorrect. Expected: {:.10}, Actual: {:.10}",
            expected_nonexistent_single_idf, actual_nonexistent_single_idf
        );
    }

    #[test]
    fn test_bm25_idf_formula_precision() {
        // Test with exact values that expose precision issues
        let documents = vec![
            vec!["test".to_string()],
            vec!["test".to_string()],
            vec!["other".to_string()],
        ];

        let scorer = BM25Scorer::new(documents, 1.5, 0.75);
        
        // For "test": N=3, df=2
        // IDF = log((3 - 2 + 0.5) / (2 + 0.5)) = log(1.5 / 2.5) = log(0.6)
        let expected_test_idf = (1.5 / 2.5).ln();
        let actual_test_idf = scorer.calculate_idf("test");

        // Use very strict tolerance to catch floating-point errors
        assert!(
            (actual_test_idf - expected_test_idf).abs() < 1e-15,
            "Precision test failed for 'test'. Expected: {:.15}, Actual: {:.15}, Error: {:.15}",
            expected_test_idf, actual_test_idf, (actual_test_idf - expected_test_idf).abs()
        );

        // For "other": N=3, df=1
        // IDF = log((3 - 1 + 0.5) / (1 + 0.5)) = log(2.5 / 1.5) = log(5/3)
        let expected_other_idf = (2.5 / 1.5).ln();
        let actual_other_idf = scorer.calculate_idf("other");

        assert!(
            (actual_other_idf - expected_other_idf).abs() < 1e-15,
            "Precision test failed for 'other'. Expected: {:.15}, Actual: {:.15}, Error: {:.15}",
            expected_other_idf, actual_other_idf, (actual_other_idf - expected_other_idf).abs()
        );
    }
}