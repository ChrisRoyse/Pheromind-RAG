//! Comprehensive BM25 IDF Tests with Negative Value Handling
//! 
//! This module provides exhaustive testing for BM25 IDF calculation edge cases,
//! particularly focusing on negative IDF values that occur when terms appear in
//! more than half of the documents in the corpus.

use embed_search::search::bm25::{BM25Engine, BM25Document, Token as BM25Token};

/// Test suite for BM25 IDF calculation with negative values and edge cases
#[cfg(test)]
mod comprehensive_bm25_idf_tests {
    use super::*;

    /// Mock document creator for consistent test setup
    fn create_test_document(id: &str, file_path: &str, tokens: Vec<&str>) -> BM25Document {
        let bm25_tokens: Vec<BM25Token> = tokens.iter()
            .enumerate()
            .map(|(pos, token)| BM25Token {
                text: token.to_string(),
                position: pos,
                importance_weight: 1.0,
            })
            .collect();

        BM25Document {
            id: id.to_string(),
            file_path: file_path.to_string(),
            chunk_index: 0,
            tokens: bm25_tokens,
            start_line: 1,
            end_line: 10,
            language: Some("rust".to_string()),
        }
    }

    /// Test BM25 IDF calculation with negative values
    /// This test verifies the critical fix for negative IDF handling
    #[test]
    fn test_bm25_idf_negative_values_handling() {
        let mut engine = BM25Engine::new();
        
        // Create corpus where "common" appears in 4 out of 5 documents (80%)
        // This should produce negative IDF: log((5-4+0.5)/(4+0.5)) = log(1.5/4.5) = log(1/3) ≈ -1.099
        let documents = vec![
            create_test_document("doc1", "file1.rs", vec!["common", "term1"]),
            create_test_document("doc2", "file2.rs", vec!["common", "term2"]),
            create_test_document("doc3", "file3.rs", vec!["common", "term3"]),
            create_test_document("doc4", "file4.rs", vec!["common", "term4"]),
            create_test_document("doc5", "file5.rs", vec!["rare", "term5"]),
        ];

        // Add documents to engine
        for doc in documents {
            engine.add_document(doc).expect("Document indexing must succeed");
        }

        // Calculate IDF values
        let common_idf = engine.calculate_idf("common");
        let rare_idf = engine.calculate_idf("rare");
        let nonexistent_idf = engine.calculate_idf("nonexistent");

        // Verify that rare terms have higher IDF than common terms
        assert!(
            rare_idf > common_idf,
            "CRITICAL: Rare term IDF ({:.6}) must be higher than common term IDF ({:.6}). \
             This is fundamental to TF-IDF ranking correctness.",
            rare_idf, common_idf
        );

        // Verify common term gets positive but small IDF (from fix)
        assert!(
            common_idf > 0.0,
            "FIXED: Common term IDF ({:.6}) must be positive after negative IDF handling fix. \
             Negative IDFs were causing ranking inversions.",
            common_idf
        );

        // Verify common term IDF is small (indicating high frequency)
        assert!(
            common_idf < 0.01,
            "Common term IDF ({:.6}) should be very small, indicating high document frequency",
            common_idf
        );

        // Verify nonexistent term has highest IDF
        assert!(
            nonexistent_idf > rare_idf,
            "Nonexistent term IDF ({:.6}) must be higher than rare term IDF ({:.6})",
            nonexistent_idf, rare_idf
        );

        // Verify all IDFs are finite and valid
        assert!(common_idf.is_finite(), "Common IDF must be finite, got {}", common_idf);
        assert!(rare_idf.is_finite(), "Rare IDF must be finite, got {}", rare_idf);
        assert!(nonexistent_idf.is_finite(), "Nonexistent IDF must be finite, got {}", nonexistent_idf);

        println!("✅ IDF Values: common={:.6}, rare={:.6}, nonexistent={:.6}", 
                 common_idf, rare_idf, nonexistent_idf);
    }

    /// Test extreme negative IDF case (term in all documents)
    #[test] 
    fn test_bm25_idf_universal_term() {
        let mut engine = BM25Engine::new();
        
        // Create corpus where "universal" appears in ALL documents
        // Raw IDF would be: log((3-3+0.5)/(3+0.5)) = log(0.5/3.5) ≈ -1.946 (very negative)
        let documents = vec![
            create_test_document("doc1", "file1.rs", vec!["universal", "unique1"]),
            create_test_document("doc2", "file2.rs", vec!["universal", "unique2"]),
            create_test_document("doc3", "file3.rs", vec!["universal", "unique3"]),
        ];

        for doc in documents {
            engine.add_document(doc).expect("Document indexing must succeed");
        }

        let universal_idf = engine.calculate_idf("universal");
        let unique_idf = engine.calculate_idf("unique1");

        // Universal term should have lowest IDF but still positive (after fix)
        assert!(
            universal_idf > 0.0,
            "Universal term IDF ({:.6}) must be positive after fix",
            universal_idf
        );

        // Universal term should have much lower IDF than unique terms
        assert!(
            unique_idf > universal_idf,
            "Unique term IDF ({:.6}) must be higher than universal term IDF ({:.6})",
            unique_idf, universal_idf
        );

        // Universal term IDF should be extremely small
        assert!(
            universal_idf < 0.001,
            "Universal term IDF ({:.6}) should be very small due to high frequency",
            universal_idf
        );
    }

    /// Test IDF calculation mathematical precision
    #[test]
    fn test_bm25_idf_mathematical_precision() {
        let mut engine = BM25Engine::new();
        
        // Create precise test case for mathematical verification
        let documents = vec![
            create_test_document("doc1", "file1.rs", vec!["test"]),
            create_test_document("doc2", "file2.rs", vec!["test"]),
            create_test_document("doc3", "file3.rs", vec!["other"]),
        ];

        for doc in documents {
            engine.add_document(doc).expect("Document indexing must succeed");
        }

        let test_idf = engine.calculate_idf("test");
        
        // For "test": N=3, df=2
        // Raw IDF = log((3-2+0.5)/(2+0.5)) = log(1.5/2.5) = log(0.6) ≈ -0.511
        // After fix: should be small positive value
        let expected_raw_idf = (1.5f32 / 2.5f32).ln(); // ≈ -0.511
        
        // Verify the fix transforms negative to small positive
        if expected_raw_idf < 0.0 {
            // Should be positive after fix
            assert!(
                test_idf > 0.0,
                "IDF for medium-frequency term should be positive after fix, got {}",
                test_idf
            );
            
            // Should be small due to negative raw value mapping
            assert!(
                test_idf < 0.01,
                "IDF for medium-frequency term should be small, got {}",
                test_idf
            );
        }

        // Verify mathematical consistency
        assert!(test_idf.is_finite(), "IDF must be finite");
        assert!(!test_idf.is_nan(), "IDF must not be NaN");
    }

    /// Test edge case with single document corpus
    #[test]
    fn test_bm25_idf_single_document() {
        let mut engine = BM25Engine::new();
        
        let doc = create_test_document("doc1", "file1.rs", vec!["only", "term"]);
        engine.add_document(doc).expect("Single document indexing must succeed");

        let only_idf = engine.calculate_idf("only");
        let nonexistent_idf = engine.calculate_idf("missing");

        // Single document: N=1, df=1 for existing terms
        // Raw IDF = log((1-1+0.5)/(1+0.5)) = log(0.5/1.5) = log(1/3) ≈ -1.099 (negative)
        // After fix: should be small positive
        assert!(
            only_idf > 0.0,
            "Single document term IDF should be positive after fix, got {}",
            only_idf
        );

        // Nonexistent term should have higher IDF
        assert!(
            nonexistent_idf > only_idf,
            "Nonexistent term IDF ({}) should be higher than existing term IDF ({})",
            nonexistent_idf, only_idf
        );
    }

    /// Test IDF monotonicity with frequency progression
    #[test]
    fn test_bm25_idf_frequency_monotonicity() {
        let mut engine = BM25Engine::new();
        
        // Create 10-document corpus with terms of increasing frequency
        let mut documents = Vec::new();
        for i in 0..10 {
            let mut tokens = vec!["everywhere"]; // Appears in all 10 docs
            
            if i < 5 { tokens.push("frequent"); }    // Appears in 5 docs
            if i < 2 { tokens.push("uncommon"); }    // Appears in 2 docs  
            if i == 0 { tokens.push("rare"); }       // Appears in 1 doc
            
            documents.push(create_test_document(&format!("doc{}", i), &format!("file{}.rs", i), tokens));
        }

        for doc in documents {
            engine.add_document(doc).expect("Document indexing must succeed");
        }

        let everywhere_idf = engine.calculate_idf("everywhere"); // df=10, most common
        let frequent_idf = engine.calculate_idf("frequent");     // df=5
        let uncommon_idf = engine.calculate_idf("uncommon");     // df=2
        let rare_idf = engine.calculate_idf("rare");             // df=1, rarest

        // Verify monotonic ordering: rarer terms have higher IDF
        assert!(
            rare_idf > uncommon_idf,
            "Rare term IDF ({:.6}) must be higher than uncommon term IDF ({:.6})",
            rare_idf, uncommon_idf
        );

        assert!(
            uncommon_idf > frequent_idf,
            "Uncommon term IDF ({:.6}) must be higher than frequent term IDF ({:.6})",
            uncommon_idf, frequent_idf
        );

        assert!(
            frequent_idf > everywhere_idf,
            "Frequent term IDF ({:.6}) must be higher than everywhere term IDF ({:.6})",
            frequent_idf, everywhere_idf
        );

        // All IDFs should be positive after fix
        assert!(everywhere_idf > 0.0, "Everywhere IDF should be positive: {}", everywhere_idf);
        assert!(frequent_idf > 0.0, "Frequent IDF should be positive: {}", frequent_idf);
        assert!(uncommon_idf > 0.0, "Uncommon IDF should be positive: {}", uncommon_idf);
        assert!(rare_idf > 0.0, "Rare IDF should be positive: {}", rare_idf);

        println!("✅ Monotonic IDF sequence: everywhere={:.6} < frequent={:.6} < uncommon={:.6} < rare={:.6}",
                 everywhere_idf, frequent_idf, uncommon_idf, rare_idf);
    }

    /// Test BM25 scoring with negative IDF terms
    #[test]
    fn test_bm25_scoring_with_negative_idf_terms() {
        let mut engine = BM25Engine::new();
        
        // Create documents where "common" would have negative raw IDF
        let documents = vec![
            create_test_document("doc1", "file1.rs", vec!["common", "specific", "term"]),
            create_test_document("doc2", "file2.rs", vec!["common", "other", "words"]),
            create_test_document("doc3", "file3.rs", vec!["common", "different", "content"]),
            create_test_document("doc4", "file4.rs", vec!["rare", "unique", "special"]),
        ];

        for doc in documents {
            engine.add_document(doc).expect("Document indexing must succeed");
        }

        // Search for common term
        let common_results = engine.search("common", 10)
            .expect("Search for common term must succeed");

        // Search for rare term  
        let rare_results = engine.search("rare", 10)
            .expect("Search for rare term must succeed");

        // Verify common term matches multiple documents
        assert_eq!(
            common_results.len(), 3,
            "Common term should match 3 documents, got {}",
            common_results.len()
        );

        // Verify rare term matches one document
        assert_eq!(
            rare_results.len(), 1, 
            "Rare term should match 1 document, got {}",
            rare_results.len()
        );

        // Verify rare term gets higher score than common term in any document
        let rare_score = rare_results[0].score;
        let best_common_score = common_results.iter().map(|r| r.score).fold(0.0, f32::max);

        assert!(
            rare_score > best_common_score,
            "Rare term score ({:.6}) should be higher than best common term score ({:.6})",
            rare_score, best_common_score
        );

        // Verify all scores are finite
        for result in &common_results {
            assert!(
                result.score.is_finite(),
                "Common term score must be finite, got {}",
                result.score
            );
        }

        for result in &rare_results {
            assert!(
                result.score.is_finite(),
                "Rare term score must be finite, got {}",
                result.score
            );
        }
    }

    /// Test error handling in IDF calculation
    #[test]
    fn test_bm25_idf_error_handling() {
        let engine = BM25Engine::new(); // Empty engine

        // Test with empty corpus
        let empty_idf = engine.calculate_idf("test");
        
        // Should return high IDF for nonexistent term in empty corpus
        assert!(
            empty_idf > 0.0,
            "IDF for term in empty corpus should be positive, got {}",
            empty_idf
        );
        
        assert!(
            empty_idf.is_finite(),
            "IDF for term in empty corpus must be finite, got {}",
            empty_idf
        );

        // Test case sensitivity
        let mut engine = BM25Engine::new();
        let doc = create_test_document("doc1", "file1.rs", vec!["CamelCase"]);
        engine.add_document(doc).expect("Document indexing must succeed");

        let lower_idf = engine.calculate_idf("camelcase");
        let upper_idf = engine.calculate_idf("CAMELCASE");
        let mixed_idf = engine.calculate_idf("CamelCase");

        // All should be equal due to case normalization
        assert!(
            (lower_idf - upper_idf).abs() < 1e-10,
            "Case-insensitive IDF calculation failed: {} vs {}",
            lower_idf, upper_idf
        );

        assert!(
            (lower_idf - mixed_idf).abs() < 1e-10,
            "Case-insensitive IDF calculation failed: {} vs {}",
            lower_idf, mixed_idf
        );
    }

    /// Stress test with large corpus and extreme frequencies
    #[test]
    fn test_bm25_idf_stress_test() {
        let mut engine = BM25Engine::new();
        
        // Create 100 documents with varying term distributions
        for i in 0..100 {
            let mut tokens = vec!["universal"]; // In all 100 documents
            
            if i % 2 == 0 { tokens.push("frequent"); }   // In 50 documents
            if i % 10 == 0 { tokens.push("uncommon"); }  // In 10 documents
            if i == 0 { tokens.push("unique"); }         // In 1 document only
            
            let doc = create_test_document(&format!("doc{}", i), &format!("file{}.rs", i), tokens);
            engine.add_document(doc).expect("Stress test document indexing must succeed");
        }

        // Test IDFs for different frequency ranges
        let universal_idf = engine.calculate_idf("universal"); // df=100 (100%)
        let frequent_idf = engine.calculate_idf("frequent");   // df=50 (50%)  
        let uncommon_idf = engine.calculate_idf("uncommon");   // df=10 (10%)
        let unique_idf = engine.calculate_idf("unique");       // df=1 (1%)

        // Verify ordering
        assert!(unique_idf > uncommon_idf, "Unique > Uncommon IDF failed");
        assert!(uncommon_idf > frequent_idf, "Uncommon > Frequent IDF failed");
        assert!(frequent_idf > universal_idf, "Frequent > Universal IDF failed");

        // Verify all are positive and finite
        let idfs = vec![universal_idf, frequent_idf, uncommon_idf, unique_idf];
        for (i, idf) in idfs.iter().enumerate() {
            assert!(idf > &0.0, "IDF {} should be positive, got {}", i, idf);
            assert!(idf.is_finite(), "IDF {} should be finite, got {}", i, idf);
        }

        // Universal term should be very small
        assert!(
            universal_idf < 0.001,
            "Universal term IDF should be very small, got {}",
            universal_idf
        );

        println!("✅ Stress test IDFs: universal={:.6}, frequent={:.6}, uncommon={:.6}, unique={:.6}",
                 universal_idf, frequent_idf, uncommon_idf, unique_idf);
    }
}