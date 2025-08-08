// Import the FIXED BM25 implementation
use embed_search::search::bm25_fixed::{BM25Engine, BM25Match};

#[cfg(test)]
mod edge_case_tests {
    use super::*;

    /// Test single document corpus (N=1) - critical edge case
    #[test]
    fn test_single_document_corpus() {
        let mut bm25 = BM25::new();
        
        // Single document
        let doc = Document {
            id: "doc1".to_string(),
            content: "unique term appears here".to_string(),
        };
        
        bm25.add_document(doc);
        let results = bm25.search("unique", 10);
        
        // Should handle single document gracefully
        assert_eq!(results.len(), 1);
        assert!(results[0].score > 0.0, "Score should be positive for single document");
        assert!(results[0].score.is_finite(), "Score must be finite, not NaN or infinite");
        
        println!("Single document score: {}", results[0].score);
    }

    /// Test large corpus with high-frequency terms (>50% document frequency)
    #[test]
    fn test_large_corpus_high_frequency_terms() {
        let mut bm25 = BM25::new();
        
        // Create 100 documents where "common" appears in 80 of them
        for i in 0..100 {
            let content = if i < 80 {
                format!("common term in document {}", i)
            } else {
                format!("unique content in document {}", i)
            };
            
            let doc = Document {
                id: format!("doc_{}", i),
                content,
            };
            bm25.add_document(doc);
        }
        
        let results = bm25.search("common", 10);
        
        // Should return results even for very common terms
        assert!(!results.is_empty(), "Should find results for high-frequency term");
        assert!(results.len() >= 10, "Should return at least 10 results");
        
        // All scores should be positive (IDF should use epsilon to prevent negative values)
        for result in &results {
            assert!(result.score > 0.0, "Score must be positive, got: {}", result.score);
            assert!(result.score.is_finite(), "Score must be finite, got: {}", result.score);
        }
        
        // Scores should be relatively small but positive for very common terms
        let max_score = results[0].score;
        assert!(max_score > 0.0 && max_score < 10.0, "High-frequency term should have small positive score, got: {}", max_score);
        
        println!("High-frequency term max score: {}", max_score);
    }

    /// Test terms that appear in ALL documents (100% frequency)
    #[test]
    fn test_universal_terms() {
        let mut bm25 = BM25::new();
        
        // Create 50 documents where "universal" appears in ALL of them
        for i in 0..50 {
            let doc = Document {
                id: format!("doc_{}", i),
                content: format!("universal term plus unique content {}", i),
            };
            bm25.add_document(doc);
        }
        
        let results = bm25.search("universal", 10);
        
        // Should handle universal terms without crashes
        assert!(!results.is_empty(), "Should find results even for universal terms");
        
        for result in &results {
            assert!(result.score > 0.0, "Universal term score must be positive (epsilon protection), got: {}", result.score);
            assert!(result.score.is_finite(), "Universal term score must be finite, got: {}", result.score);
        }
        
        // Universal terms should have very small but positive scores
        let max_score = results[0].score;
        assert!(max_score > 0.0 && max_score < 1.0, "Universal term should have very small positive score, got: {}", max_score);
        
        println!("Universal term max score: {}", max_score);
    }

    /// Test case sensitivity edge cases
    #[test]
    fn test_case_sensitivity_edge_cases() {
        let mut bm25 = BM25::new();
        
        // Documents with same terms in different cases
        let docs = vec![
            ("doc1", "UPPERCASE text here"),
            ("doc2", "lowercase text here"),
            ("doc3", "MixedCase Text Here"),
            ("doc4", "text with MIXED case TEXT"),
        ];
        
        for (id, content) in docs {
            bm25.add_document(Document {
                id: id.to_string(),
                content: content.to_string(),
            });
        }
        
        // Test different case queries
        let lowercase_results = bm25.search("text", 10);
        let uppercase_results = bm25.search("TEXT", 10);
        let mixed_results = bm25.search("Text", 10);
        
        // Should handle case variations consistently
        assert!(!lowercase_results.is_empty(), "Should find lowercase 'text'");
        assert!(!uppercase_results.is_empty(), "Should find uppercase 'TEXT'");
        assert!(!mixed_results.is_empty(), "Should find mixed case 'Text'");
        
        // All results should have positive, finite scores
        for results in [&lowercase_results, &uppercase_results, &mixed_results] {
            for result in results {
                assert!(result.score > 0.0, "Case variation score must be positive, got: {}", result.score);
                assert!(result.score.is_finite(), "Case variation score must be finite, got: {}", result.score);
            }
        }
        
        println!("Case sensitivity test passed with {} results each", lowercase_results.len());
    }

    /// Test single-term documents (minimal content)
    #[test]
    fn test_single_term_documents() {
        let mut bm25 = BM25::new();
        
        // Documents with only one unique term each
        for i in 0..20 {
            let doc = Document {
                id: format!("single_term_doc_{}", i),
                content: if i < 10 {
                    "singleton".to_string()  // 10 docs with same single term
                } else {
                    format!("unique{}", i)   // 10 docs with different single terms
                },
            };
            bm25.add_document(doc);
        }
        
        let results = bm25.search("singleton", 10);
        
        assert!(!results.is_empty(), "Should find single-term documents");
        assert_eq!(results.len(), 10, "Should find all 10 documents with 'singleton'");
        
        for result in &results {
            assert!(result.score > 0.0, "Single-term document score must be positive, got: {}", result.score);
            assert!(result.score.is_finite(), "Single-term document score must be finite, got: {}", result.score);
        }
        
        println!("Single-term documents test passed with {} results", results.len());
    }

    /// Test documents with massive term repetition
    #[test]
    fn test_massive_term_repetition() {
        let mut bm25 = BM25::new();
        
        // Create documents with extreme term repetition
        for i in 0..10 {
            let repeated_content = if i < 5 {
                // 5 documents with "repeated" appearing 1000 times
                vec!["repeated"; 1000].join(" ")
            } else {
                // 5 documents with "repeated" appearing 10 times + unique content
                format!("{} unique_content_{}", vec!["repeated"; 10].join(" "), i)
            };
            
            let doc = Document {
                id: format!("massive_rep_doc_{}", i),
                content: repeated_content,
            };
            bm25.add_document(doc);
        }
        
        let results = bm25.search("repeated", 10);
        
        assert!(!results.is_empty(), "Should handle massive term repetition");
        assert!(results.len() >= 5, "Should find documents with massive repetition");
        
        for result in &results {
            assert!(result.score > 0.0, "Massive repetition score must be positive, got: {}", result.score);
            assert!(result.score.is_finite(), "Massive repetition score must be finite, got: {}", result.score);
        }
        
        // Documents with more repetitions should score higher
        let scores: Vec<f64> = results.iter().map(|r| r.score).collect();
        let max_score = scores.iter().fold(0.0f64, |a, &b| a.max(b));
        assert!(max_score > 1.0, "Massive repetition should produce high scores, got max: {}", max_score);
        
        println!("Massive repetition test passed, max score: {}", max_score);
    }

    /// Test empty and whitespace-only edge cases
    #[test]
    fn test_empty_and_whitespace_documents() {
        let mut bm25 = BM25::new();
        
        // Add various edge case documents
        let edge_docs = vec![
            ("empty", ""),
            ("whitespace", "   \t\n  "),
            ("single_char", "a"),
            ("single_space", " "),
            ("normal", "normal document content"),
        ];
        
        for (id, content) in edge_docs {
            let doc = Document {
                id: id.to_string(),
                content: content.to_string(),
            };
            bm25.add_document(doc);
        }
        
        // Search for content that exists
        let results = bm25.search("normal", 10);
        assert!(!results.is_empty(), "Should find normal content");
        
        // Search for content that might not exist
        let empty_results = bm25.search("nonexistent", 10);
        // Should handle gracefully (empty results are acceptable)
        
        for result in &results {
            assert!(result.score > 0.0, "Edge case document score must be positive, got: {}", result.score);
            assert!(result.score.is_finite(), "Edge case document score must be finite, got: {}", result.score);
        }
        
        println!("Empty/whitespace edge cases handled correctly");
    }

    /// Test special characters and Unicode
    #[test]
    fn test_special_characters_unicode() {
        let mut bm25 = BM25::new();
        
        // Documents with special characters
        let special_docs = vec![
            ("special1", "hello@world.com with email"),
            ("special2", "price: $99.99 for item"),
            ("special3", "unicode: café naïve résumé"),
            ("special4", "symbols: !@#$%^&*()"),
            ("special5", "numbers: 123 456 789"),
        ];
        
        for (id, content) in special_docs {
            let doc = Document {
                id: id.to_string(),
                content: content.to_string(),
            };
            bm25.add_document(doc);
        }
        
        // Test searches with special characters
        let email_results = bm25.search("hello@world.com", 10);
        let unicode_results = bm25.search("café", 10);
        let symbol_results = bm25.search("$99.99", 10);
        
        // Should handle special characters without crashing
        for results in [&email_results, &unicode_results, &symbol_results] {
            for result in results {
                assert!(result.score >= 0.0, "Special character score must be non-negative, got: {}", result.score);
                assert!(result.score.is_finite(), "Special character score must be finite, got: {}", result.score);
            }
        }
        
        println!("Special characters and Unicode handled correctly");
    }

    /// Test IDF calculation precision with extreme values
    #[test]
    fn test_idf_calculation_precision() {
        let mut bm25 = BM25::new();
        
        // Create a scenario that tests IDF precision
        // 1000 documents where a term appears in 999 of them (very high frequency)
        for i in 0..1000 {
            let content = if i < 999 {
                format!("common_term document_content_{}", i)
            } else {
                format!("rare_term document_content_{}", i)
            };
            
            let doc = Document {
                id: format!("precision_doc_{}", i),
                content,
            };
            bm25.add_document(doc);
        }
        
        let common_results = bm25.search("common_term", 10);
        let rare_results = bm25.search("rare_term", 10);
        
        // Common term should still have positive score (epsilon protection)
        assert!(!common_results.is_empty(), "Should find very common term");
        for result in &common_results {
            assert!(result.score > 0.0, "Very common term must have positive score, got: {}", result.score);
            assert!(result.score.is_finite(), "Very common term score must be finite, got: {}", result.score);
        }
        
        // Rare term should have much higher score
        assert!(!rare_results.is_empty(), "Should find rare term");
        for result in &rare_results {
            assert!(result.score > 0.0, "Rare term must have positive score, got: {}", result.score);
            assert!(result.score.is_finite(), "Rare term score must be finite, got: {}", result.score);
        }
        
        // Rare term should score significantly higher than common term
        let common_score = common_results[0].score;
        let rare_score = rare_results[0].score;
        
        assert!(rare_score > common_score, 
                "Rare term should score higher than common term. Rare: {}, Common: {}", 
                rare_score, common_score);
        
        // The ratio should be reasonable (rare should be much higher but not infinite)
        let ratio = rare_score / common_score;
        assert!(ratio > 10.0 && ratio < 1000.0, 
                "Score ratio should be reasonable, got: {}", ratio);
        
        println!("IDF precision test passed. Common: {}, Rare: {}, Ratio: {}", 
                 common_score, rare_score, ratio);
    }

    /// Test zero division protection in edge cases
    #[test]
    fn test_zero_division_protection() {
        let mut bm25 = BM25::new();
        
        // This tests internal robustness - add minimal documents
        let doc1 = Document {
            id: "minimal1".to_string(),
            content: "a".to_string(),
        };
        
        let doc2 = Document {
            id: "minimal2".to_string(),
            content: "a a a".to_string(),
        };
        
        bm25.add_document(doc1);
        bm25.add_document(doc2);
        
        let results = bm25.search("a", 10);
        
        // Should handle minimal case without division by zero
        assert!(!results.is_empty(), "Should handle minimal documents");
        
        for result in &results {
            assert!(result.score > 0.0, "Minimal case score must be positive, got: {}", result.score);
            assert!(result.score.is_finite(), "Minimal case score must be finite, got: {}", result.score);
            assert!(!result.score.is_nan(), "Score must not be NaN, got: {}", result.score);
        }
        
        println!("Zero division protection test passed with {} results", results.len());
    }
}