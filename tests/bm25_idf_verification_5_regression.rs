/// BM25 IDF Calculation Regression Test - CRITICAL BUG VERIFICATION
/// 
/// This test prevents the regression of a specific IDF calculation bug that was discovered and fixed.
/// 
/// BUG DESCRIPTION:
/// Before the fix, very common terms and rare terms were getting identical IDF scores due to
/// improper epsilon handling in negative IDF calculations. This broke search relevance ordering.
/// 
/// ROOT CAUSE IDENTIFIED:
/// The epsilon handling for negative raw IDF values was inverted. The code was:
///   `epsilon + (raw_idf.abs() * 0.0001)`
/// This made MORE negative raw IDF (more common terms) produce HIGHER final IDF values.
///
/// EXACT BUG SCENARIO REPRODUCED:
/// - Document 1: ["function", "calculate", "total"] 
/// - Document 2: ["function", "function", "process"]
/// - Query terms: "function" (appears in 2/2 docs = very common) vs "calculate" (appears in 1/2 docs = rare)
/// - BEFORE BUG FIX: Both terms got IDF ≈ 0.001000 (WRONG - identical values)
/// - AFTER BUG FIX: "calculate" IDF (0.010000) > "function" IDF (0.001038) (CORRECT - proper ordering)
/// 
/// FIX APPLIED:
/// Changed epsilon handling to: `epsilon + (1.0 / (raw_idf.abs() + 1.0)) * 0.0001`
/// This ensures more negative raw IDF produces LOWER final IDF values, preserving correct ordering.
/// 
/// MATHEMATICAL EXPECTATIONS:
/// - Common term "function" (df=2): IDF should be low but positive
/// - Rare term "calculate" (df=1): IDF should be significantly higher
/// - Very rare terms should have highest IDF values
/// - IDF ordering MUST be preserved: rarer terms = higher IDF
/// 
/// REGRESSION PREVENTION:
/// This test suite prevents the bug from ever returning by testing all edge cases.

use embed_search::search::bm25::{BM25Engine, BM25Document, Token};
use anyhow::Result;

const EPSILON_THRESHOLD: f32 = 0.0001; // For precise floating point comparisons

/// Test that recreates the EXACT bug scenario that was broken before
#[test]
fn test_idf_bug_regression_exact_scenario() -> Result<()> {
    println!("\n=== BM25 IDF Regression Test: Exact Bug Scenario ===");
    
    let mut engine = BM25Engine::new();
    
    // EXACT SAME DOCUMENTS that revealed the bug
    let doc1 = BM25Document {
        id: "doc1".to_string(),
        file_path: "file1.rs".to_string(),
        chunk_index: 0,
        tokens: vec![
            Token { text: "function".to_string(), position: 0, importance_weight: 1.0 },
            Token { text: "calculate".to_string(), position: 1, importance_weight: 1.0 },
            Token { text: "total".to_string(), position: 2, importance_weight: 1.0 },
        ],
        start_line: 1,
        end_line: 5,
        language: Some("rust".to_string()),
    };
    
    let doc2 = BM25Document {
        id: "doc2".to_string(),
        file_path: "file2.rs".to_string(),
        chunk_index: 0,
        tokens: vec![
            Token { text: "function".to_string(), position: 0, importance_weight: 1.0 },
            Token { text: "function".to_string(), position: 1, importance_weight: 1.0 }, // Duplicate to make it common
            Token { text: "process".to_string(), position: 2, importance_weight: 1.0 },
        ],
        start_line: 1,
        end_line: 5,
        language: Some("rust".to_string()),
    };
    
    engine.add_document(doc1)?;
    engine.add_document(doc2)?;
    
    // Calculate IDF for the EXACT terms that exposed the bug
    let function_idf = engine.calculate_idf("function"); // Common term (appears in 2/2 docs)
    let calculate_idf = engine.calculate_idf("calculate"); // Rare term (appears in 1/2 docs)
    let process_idf = engine.calculate_idf("process"); // Rare term (appears in 1/2 docs)
    let total_idf = engine.calculate_idf("total"); // Rare term (appears in 1/2 docs)
    
    println!("IDF Values (N=2 documents):");
    println!("  function IDF: {:.6} (appears in 2/2 docs = 100%)", function_idf);
    println!("  calculate IDF: {:.6} (appears in 1/2 docs = 50%)", calculate_idf);
    println!("  process IDF: {:.6} (appears in 1/2 docs = 50%)", process_idf);
    println!("  total IDF: {:.6} (appears in 1/2 docs = 50%)", total_idf);
    
    // CRITICAL REGRESSION CHECK: The bug made these values identical (~0.001000)
    assert!(calculate_idf > function_idf + EPSILON_THRESHOLD, 
        "REGRESSION DETECTED: Rare term 'calculate' (IDF={:.6}) should have significantly higher IDF than common term 'function' (IDF={:.6})", 
        calculate_idf, function_idf);
    
    assert!(process_idf > function_idf + EPSILON_THRESHOLD, 
        "REGRESSION DETECTED: Rare term 'process' (IDF={:.6}) should have significantly higher IDF than common term 'function' (IDF={:.6})", 
        process_idf, function_idf);
    
    assert!(total_idf > function_idf + EPSILON_THRESHOLD, 
        "REGRESSION DETECTED: Rare term 'total' (IDF={:.6}) should have significantly higher IDF than common term 'function' (IDF={:.6})", 
        total_idf, function_idf);
    
    // Verify all rare terms have similar IDF (since they appear in same number of docs)
    let rare_term_diff = (calculate_idf - process_idf).abs();
    assert!(rare_term_diff < EPSILON_THRESHOLD, 
        "Terms with same document frequency should have similar IDF values: calculate={:.6}, process={:.6}", 
        calculate_idf, process_idf);
    
    // Verify all IDF values are positive (bug caused some to be ~0.001)
    assert!(function_idf > 0.0, "All IDF values must be positive, function IDF: {:.6}", function_idf);
    assert!(calculate_idf > 0.0, "All IDF values must be positive, calculate IDF: {:.6}", calculate_idf);
    assert!(process_idf > 0.0, "All IDF values must be positive, process IDF: {:.6}", process_idf);
    assert!(total_idf > 0.0, "All IDF values must be positive, total IDF: {:.6}", total_idf);
    
    // Verify reasonable IDF magnitudes (not ~0.001 like the bug)
    // For a 2-document collection, rare terms (df=1) should have IDF > 0.005
    assert!(calculate_idf > 0.005, 
        "Rare terms should have meaningful IDF values, not tiny epsilon: {:.6}", calculate_idf);
    
    println!("✅ BUG REGRESSION PREVENTION: IDF ordering is correct!");
    Ok(())
}

/// Test the specific epsilon handling that was broken
#[test]
fn test_idf_epsilon_handling_regression() -> Result<()> {
    println!("\n=== BM25 IDF Epsilon Handling Regression Test ===");
    
    let mut engine = BM25Engine::new();
    
    // Create scenario where terms have negative raw IDF (very common terms)
    // When df > N/2, raw IDF = ln((N-df+0.5)/(df+0.5)) becomes negative
    
    // Add many documents with a very common term
    for i in 0..10 {
        let doc = BM25Document {
            id: format!("doc{}", i),
            file_path: format!("file{}.rs", i),
            chunk_index: 0,
            tokens: vec![
                Token { text: "the".to_string(), position: 0, importance_weight: 1.0 }, // Very common
                Token { text: format!("unique{}", i), position: 1, importance_weight: 1.0 }, // Unique per doc
            ],
            start_line: 1,
            end_line: 5,
            language: Some("rust".to_string()),
        };
        engine.add_document(doc)?;
    }
    
    let common_idf = engine.calculate_idf("the"); // Appears in 10/10 docs
    let rare_idf = engine.calculate_idf("unique1"); // Appears in 1/10 docs
    let nonexistent_idf = engine.calculate_idf("nonexistent"); // Appears in 0/10 docs
    
    println!("Epsilon handling test (N=10 documents):");
    println!("  'the' IDF: {:.6} (appears in 10/10 docs)", common_idf);
    println!("  'unique1' IDF: {:.6} (appears in 1/10 docs)", rare_idf);
    println!("  'nonexistent' IDF: {:.6} (appears in 0/10 docs)", nonexistent_idf);
    
    // The bug: epsilon handling made all values identical
    // The fix: proper handling preserves ordering even with epsilon adjustments
    assert!(nonexistent_idf > rare_idf, 
        "REGRESSION: Non-existent terms should have highest IDF: nonexistent={:.6} vs rare={:.6}", 
        nonexistent_idf, rare_idf);
    
    assert!(rare_idf > common_idf, 
        "REGRESSION: Rare terms should have higher IDF than common terms: rare={:.6} vs common={:.6}", 
        rare_idf, common_idf);
    
    // Verify the epsilon handling doesn't break ordering
    assert!(common_idf > 0.0, "Common terms should still have positive IDF after epsilon handling: {:.6}", common_idf);
    
    // Verify reasonable separation between categories
    let rare_to_common_ratio = rare_idf / common_idf;
    assert!(rare_to_common_ratio > 2.0, 
        "Rare terms should have significantly higher IDF than common terms (ratio: {:.2})", rare_to_common_ratio);
    
    println!("✅ EPSILON HANDLING REGRESSION PREVENTION: Ordering preserved!");
    Ok(())
}

/// Test the mathematical correctness of IDF formula with edge cases
#[test]  
fn test_idf_mathematical_correctness() -> Result<()> {
    println!("\n=== BM25 IDF Mathematical Correctness Test ===");
    
    let mut engine = BM25Engine::new();
    
    // Test with known mathematical expectations
    // N=5 documents, various document frequency scenarios
    
    let docs = vec![
        ("doc1", vec!["term_all", "term_four", "term_three", "term_one"]),      // 4 terms
        ("doc2", vec!["term_all", "term_four", "term_three"]),                  // 3 terms  
        ("doc3", vec!["term_all", "term_four"]),                                // 2 terms
        ("doc4", vec!["term_all"]),                                             // 1 term
        ("doc5", vec!["term_all"]),                                             // 1 term
    ];
    
    for (doc_id, terms) in docs {
        let tokens: Vec<Token> = terms.into_iter().enumerate().map(|(pos, term)| {
            Token { text: term.to_string(), position: pos, importance_weight: 1.0 }
        }).collect();
        
        let doc = BM25Document {
            id: doc_id.to_string(),
            file_path: format!("{}.rs", doc_id),
            chunk_index: 0,
            tokens,
            start_line: 1,
            end_line: 5,
            language: Some("rust".to_string()),
        };
        engine.add_document(doc)?;
    }
    
    // Calculate IDF for terms with different document frequencies
    let idf_all = engine.calculate_idf("term_all");      // df=5, appears in all docs
    let idf_four = engine.calculate_idf("term_four");    // df=3, appears in 3 docs  
    let idf_three = engine.calculate_idf("term_three");  // df=2, appears in 2 docs
    let idf_one = engine.calculate_idf("term_one");      // df=1, appears in 1 doc
    let idf_none = engine.calculate_idf("term_none");    // df=0, appears in 0 docs
    
    println!("Mathematical correctness (N=5):");
    println!("  term_all (df=5): IDF={:.6}", idf_all);
    println!("  term_four (df=3): IDF={:.6}", idf_four);  
    println!("  term_three (df=2): IDF={:.6}", idf_three);
    println!("  term_one (df=1): IDF={:.6}", idf_one);
    println!("  term_none (df=0): IDF={:.6}", idf_none);
    
    // Verify strict IDF ordering: lower document frequency = higher IDF
    assert!(idf_none > idf_one, "df=0 should have higher IDF than df=1: {:.6} vs {:.6}", idf_none, idf_one);
    assert!(idf_one > idf_three, "df=1 should have higher IDF than df=2: {:.6} vs {:.6}", idf_one, idf_three);
    assert!(idf_three > idf_four, "df=2 should have higher IDF than df=3: {:.6} vs {:.6}", idf_three, idf_four);
    assert!(idf_four > idf_all, "df=3 should have higher IDF than df=5: {:.6} vs {:.6}", idf_four, idf_all);
    
    // Verify mathematical bounds
    // For df=5, N=5: raw IDF = ln((5-5+0.5)/(5+0.5)) = ln(0.5/5.5) ≈ -2.4 (negative)
    // The fix should handle this properly with epsilon adjustment
    assert!(idf_all > 0.0, "Even very common terms should have positive IDF: {:.6}", idf_all);
    
    // For df=0: IDF should be ln(N+1) = ln(6) ≈ 1.79  
    let expected_idf_none = (6.0_f32).ln();
    let idf_none_diff = (idf_none - expected_idf_none).abs();
    assert!(idf_none_diff < 0.1, 
        "Non-existent term IDF should be ~ln(N+1): expected={:.6}, actual={:.6}", 
        expected_idf_none, idf_none);
    
    println!("✅ MATHEMATICAL CORRECTNESS: All IDF calculations verified!");
    Ok(())
}

/// Test search ranking behavior with the fixed IDF
#[test]
fn test_search_ranking_with_fixed_idf() -> Result<()> {
    println!("\n=== Search Ranking with Fixed IDF Test ===");
    
    let mut engine = BM25Engine::new();
    
    // Documents that should have clear ranking with fixed IDF
    let docs = vec![
        ("perfect_match", vec!["rare", "calculate", "optimization"]),
        ("partial_match", vec!["common", "calculate", "function"]),  
        ("weak_match", vec!["common", "common", "function"]),
    ];
    
    for (doc_id, terms) in docs {
        let tokens: Vec<Token> = terms.into_iter().enumerate().map(|(pos, term)| {
            Token { text: term.to_string(), position: pos, importance_weight: 1.0 }
        }).collect();
        
        let doc = BM25Document {
            id: doc_id.to_string(),
            file_path: format!("{}.rs", doc_id),
            chunk_index: 0,
            tokens,
            start_line: 1,
            end_line: 5,
            language: Some("rust".to_string()),
        };
        engine.add_document(doc)?;
    }
    
    // Search for rare term - should strongly prefer documents with rare terms
    let results = engine.search("rare calculate", 10)?;
    
    println!("Search results for 'rare calculate':");
    for (i, result) in results.iter().enumerate() {
        println!("  {}. {} (score: {:.4})", i+1, result.doc_id, result.score);
    }
    
    // With fixed IDF, documents with rare terms should rank higher
    assert!(!results.is_empty(), "Should find results for search query");
    assert_eq!(results[0].doc_id, "perfect_match", 
        "Document with rare terms should rank first with fixed IDF");
    
    // Verify score ordering reflects proper IDF calculation
    if results.len() >= 2 {
        assert!(results[0].score > results[1].score + EPSILON_THRESHOLD, 
            "Document with rarer terms should have significantly higher score: {:.4} vs {:.4}", 
            results[0].score, results[1].score);
    }
    
    println!("✅ SEARCH RANKING: Fixed IDF produces correct relevance ordering!");
    Ok(())
}

/// Comprehensive regression test covering all aspects of the IDF bug fix
#[test]
fn test_comprehensive_idf_regression() -> Result<()> {
    println!("\n=== COMPREHENSIVE IDF BUG REGRESSION TEST ===");
    println!("This test validates that the specific IDF calculation bug is permanently fixed.");
    println!("Bug: Epsilon handling caused common and rare terms to get identical IDF values.");
    println!("Fix: Proper negative IDF handling with preserved ordering.\n");
    
    // Run all regression checks in one comprehensive test
    test_idf_bug_regression_exact_scenario()?;
    test_idf_epsilon_handling_regression()?;
    test_idf_mathematical_correctness()?;
    test_search_ranking_with_fixed_idf()?;
    
    println!("\n🎯 REGRESSION TEST SUMMARY:");
    println!("✅ Exact bug scenario - PASS");
    println!("✅ Epsilon handling - PASS");  
    println!("✅ Mathematical correctness - PASS");
    println!("✅ Search ranking behavior - PASS");
    println!("\n🚫 BUG REGRESSION PREVENTION: SUCCESSFUL");
    println!("The IDF calculation bug cannot resurface without breaking these tests.");
    
    Ok(())
}