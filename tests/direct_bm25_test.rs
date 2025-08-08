use embed_search::search::bm25::{BM25Engine, BM25Document, Token};
use anyhow::Result;

/// Direct test of BM25 engine functionality without UnifiedSearcher
/// This tests the core BM25 implementation in isolation
#[tokio::test]
async fn test_direct_bm25_functionality() -> Result<()> {
    let mut engine = BM25Engine::new();
    
    // Create test documents with known content patterns
    let doc1 = BM25Document {
        id: "doc1".to_string(),
        file_path: "auth.rs".to_string(),
        chunk_index: 0,
        tokens: vec![
            Token { text: "authentication".to_string(), position: 0, importance_weight: 1.0 },
            Token { text: "user".to_string(), position: 1, importance_weight: 1.0 },
            Token { text: "login".to_string(), position: 2, importance_weight: 1.0 },
            Token { text: "password".to_string(), position: 3, importance_weight: 1.0 },
            Token { text: "verify".to_string(), position: 4, importance_weight: 1.0 },
        ],
        start_line: 0,
        end_line: 10,
        language: Some("rust".to_string()),
    };
    
    let doc2 = BM25Document {
        id: "doc2".to_string(),
        file_path: "database.rs".to_string(),
        chunk_index: 0,
        tokens: vec![
            Token { text: "database".to_string(), position: 0, importance_weight: 1.0 },
            Token { text: "connection".to_string(), position: 1, importance_weight: 1.0 },
            Token { text: "query".to_string(), position: 2, importance_weight: 1.0 },
            Token { text: "user".to_string(), position: 3, importance_weight: 1.0 },
            Token { text: "data".to_string(), position: 4, importance_weight: 1.0 },
        ],
        start_line: 0,
        end_line: 10,
        language: Some("rust".to_string()),
    };
    
    let doc3 = BM25Document {
        id: "doc3".to_string(),
        file_path: "ui.rs".to_string(),
        chunk_index: 0,
        tokens: vec![
            Token { text: "user".to_string(), position: 0, importance_weight: 1.0 },
            Token { text: "interface".to_string(), position: 1, importance_weight: 1.0 },
            Token { text: "component".to_string(), position: 2, importance_weight: 1.0 },
            Token { text: "render".to_string(), position: 3, importance_weight: 1.0 },
        ],
        start_line: 0,
        end_line: 10,
        language: Some("rust".to_string()),
    };
    
    // Add documents to BM25 engine
    engine.add_document(doc1)?;
    engine.add_document(doc2)?;
    engine.add_document(doc3)?;
    
    // Get engine statistics
    let stats = engine.get_stats();
    println!("BM25 Engine Statistics:");
    println!("  Total documents: {}", stats.total_documents);
    println!("  Total terms: {}", stats.total_terms);
    println!("  Average document length: {:.2}", stats.avg_document_length);
    println!("  k1: {}", stats.k1);
    println!("  b: {}", stats.b);
    
    // Verify basic functionality
    assert_eq!(stats.total_documents, 3);
    assert!(stats.total_terms > 0);
    assert!(stats.avg_document_length > 0.0);
    
    println!("\n=== Testing BM25 Search Functionality ===");
    
    // Test 1: Single term search
    let results = engine.search("authentication", 10)?;
    println!("\nTest 1 - Single term 'authentication':");
    println!("  Results found: {}", results.len());
    for (i, result) in results.iter().enumerate() {
        println!("  {}. {} (score: {:.4})", i+1, result.doc_id, result.score);
    }
    assert!(!results.is_empty(), "Should find results for 'authentication'");
    assert_eq!(results[0].doc_id, "doc1"); // Should match auth document
    
    // Test 2: Multi-term search
    let results = engine.search("user database", 10)?;
    println!("\nTest 2 - Multi-term 'user database':");
    println!("  Results found: {}", results.len());
    for (i, result) in results.iter().enumerate() {
        println!("  {}. {} (score: {:.4}, matched: {:?})", 
                i+1, result.doc_id, result.score, result.matched_terms);
    }
    assert!(!results.is_empty(), "Should find results for 'user database'");
    
    // Test 3: IDF calculation
    println!("\n=== Testing IDF Calculations ===");
    let idf_user = engine.calculate_idf("user");
    let idf_authentication = engine.calculate_idf("authentication");
    let idf_nonexistent = engine.calculate_idf("nonexistent");
    
    println!("  IDF('user'): {:.4} (appears in multiple docs)", idf_user);
    println!("  IDF('authentication'): {:.4} (appears in one doc)", idf_authentication);
    println!("  IDF('nonexistent'): {:.4} (doesn't exist)", idf_nonexistent);
    
    // Rare terms should have higher IDF than common terms
    assert!(idf_authentication > idf_user, 
            "Rare term 'authentication' should have higher IDF than common term 'user'");
    assert!(idf_nonexistent > idf_authentication,
            "Non-existent term should have highest IDF");
    
    // Test 4: BM25 score calculation for specific document
    println!("\n=== Testing Manual BM25 Score Calculation ===");
    let score1 = engine.calculate_bm25_score(&["user".to_string()], "doc1")?;
    let score2 = engine.calculate_bm25_score(&["user".to_string()], "doc2")?;
    let score3 = engine.calculate_bm25_score(&["user".to_string()], "doc3")?;
    
    println!("  BM25 score for 'user' in doc1: {:.4}", score1);
    println!("  BM25 score for 'user' in doc2: {:.4}", score2);
    println!("  BM25 score for 'user' in doc3: {:.4}", score3);
    
    // All should be positive scores since all docs contain "user"
    assert!(score1 > 0.0, "Score for doc1 should be positive");
    assert!(score2 > 0.0, "Score for doc2 should be positive");
    assert!(score3 > 0.0, "Score for doc3 should be positive");
    
    // Test 5: Term frequency saturation test
    println!("\n=== Testing Term Frequency Saturation ===");
    
    // Create document with repeated terms
    let doc_repetitive = BM25Document {
        id: "doc_rep".to_string(),
        file_path: "repetitive.rs".to_string(),
        chunk_index: 0,
        tokens: vec![
            Token { text: "function".to_string(), position: 0, importance_weight: 1.0 },
            Token { text: "function".to_string(), position: 1, importance_weight: 1.0 },
            Token { text: "function".to_string(), position: 2, importance_weight: 1.0 },
            Token { text: "function".to_string(), position: 3, importance_weight: 1.0 },
            Token { text: "function".to_string(), position: 4, importance_weight: 1.0 },
        ],
        start_line: 0,
        end_line: 10,
        language: Some("rust".to_string()),
    };
    
    let doc_normal = BM25Document {
        id: "doc_norm".to_string(),
        file_path: "normal.rs".to_string(),
        chunk_index: 0,
        tokens: vec![
            Token { text: "function".to_string(), position: 0, importance_weight: 1.0 },
            Token { text: "calculate".to_string(), position: 1, importance_weight: 1.0 },
            Token { text: "result".to_string(), position: 2, importance_weight: 1.0 },
        ],
        start_line: 0,
        end_line: 10,
        language: Some("rust".to_string()),
    };
    
    engine.add_document(doc_repetitive)?;
    engine.add_document(doc_normal)?;
    
    let results = engine.search("function", 10)?;
    println!("  Search results for 'function' after adding repetitive docs:");
    for (i, result) in results.iter().enumerate() {
        println!("  {}. {} (score: {:.4})", i+1, result.doc_id, result.score);
    }
    
    // BM25 should prevent the repetitive document from completely dominating
    // The score difference should be reasonable due to saturation
    if results.len() >= 2 {
        let top_score = results[0].score;
        let second_score = results[1].score;
        let ratio = top_score / second_score;
        println!("  Score ratio (top/second): {:.2}", ratio);
        assert!(ratio < 10.0, "BM25 saturation should prevent extreme score differences");
    }
    
    println!("\n✅ All BM25 functionality tests passed!");
    Ok(())
}

/// Test BM25 edge cases and error handling
#[tokio::test]
async fn test_bm25_edge_cases() -> Result<()> {
    let mut engine = BM25Engine::new();
    
    // Test empty query
    let result = engine.search("", 10);
    assert!(result.is_err(), "Empty query should return error");
    
    // Test search without any documents
    let result = engine.search("test", 10);
    match result {
        Ok(results) => assert!(results.is_empty(), "Should return empty results when no documents"),
        Err(_) => {} // Error is also acceptable
    }
    
    // Add a document
    let doc = BM25Document {
        id: "test_doc".to_string(),
        file_path: "test.rs".to_string(),
        chunk_index: 0,
        tokens: vec![
            Token { text: "test".to_string(), position: 0, importance_weight: 1.0 },
        ],
        start_line: 0,
        end_line: 1,
        language: Some("rust".to_string()),
    };
    engine.add_document(doc)?;
    
    // Test search for non-existent term
    let results = engine.search("nonexistent", 10)?;
    assert!(results.is_empty(), "Should return empty results for non-existent terms");
    
    // Test whitespace query
    let results = engine.search("   ", 10);
    assert!(results.is_err(), "Whitespace-only query should return error");
    
    // Test case insensitive search
    let results_lower = engine.search("test", 10)?;
    let results_upper = engine.search("TEST", 10)?;
    assert_eq!(results_lower.len(), results_upper.len(), "Search should be case insensitive");
    
    println!("✅ All BM25 edge case tests passed!");
    Ok(())
}

/// Test BM25 parameter customization
#[tokio::test]
async fn test_bm25_parameter_effects() -> Result<()> {
    // Test different k1 values
    let mut engine_low_k1 = BM25Engine::with_params(0.5, 0.75);
    let mut engine_high_k1 = BM25Engine::with_params(2.0, 0.75);
    
    // Add same document to both engines
    let doc = BM25Document {
        id: "test".to_string(),
        file_path: "test.rs".to_string(),
        chunk_index: 0,
        tokens: vec![
            Token { text: "term".to_string(), position: 0, importance_weight: 1.0 },
            Token { text: "term".to_string(), position: 1, importance_weight: 1.0 },
        ],
        start_line: 0,
        end_line: 1,
        language: Some("rust".to_string()),
    };
    
    engine_low_k1.add_document(doc.clone())?;
    engine_high_k1.add_document(doc)?;
    
    // Calculate scores - higher k1 should affect term frequency impact
    let score_low_k1 = engine_low_k1.calculate_bm25_score(&["term".to_string()], "test")?;
    let score_high_k1 = engine_high_k1.calculate_bm25_score(&["term".to_string()], "test")?;
    
    println!("Score with k1=0.5: {:.4}", score_low_k1);
    println!("Score with k1=2.0: {:.4}", score_high_k1);
    
    // Scores should be different due to different k1 values
    assert!(score_low_k1 != score_high_k1, "Different k1 values should produce different scores");
    
    // Test b parameter effects would require documents of different lengths
    // This is a simplified test to verify parameter customization works
    
    println!("✅ BM25 parameter customization test passed!");
    Ok(())
}