/// Isolated BM25 functionality test without UnifiedSearcher dependencies
/// This directly tests BM25 engine and verifies actual search functionality
use embed_search::search::bm25::{BM25Engine, BM25Document, Token};
use anyhow::Result;

#[tokio::test]
async fn test_bm25_complete_functionality() -> Result<()> {
    println!("=== BM25 Complete Functionality Test ===");
    
    // Create BM25 engine with custom parameters
    let mut engine = BM25Engine::with_params(1.2, 0.75);
    
    // Create comprehensive test documents
    let documents = vec![
        BM25Document {
            id: "auth_service".to_string(),
            file_path: "auth/service.rs".to_string(),
            chunk_index: 0,
            tokens: vec![
                Token { text: "authentication".to_string(), position: 0, importance_weight: 1.0 },
                Token { text: "service".to_string(), position: 1, importance_weight: 1.0 },
                Token { text: "user".to_string(), position: 2, importance_weight: 1.0 },
                Token { text: "login".to_string(), position: 3, importance_weight: 1.0 },
                Token { text: "password".to_string(), position: 4, importance_weight: 1.0 },
                Token { text: "validate".to_string(), position: 5, importance_weight: 1.0 },
                Token { text: "token".to_string(), position: 6, importance_weight: 1.0 },
                Token { text: "jwt".to_string(), position: 7, importance_weight: 1.0 },
            ],
            start_line: 1,
            end_line: 20,
            language: Some("rust".to_string()),
        },
        BM25Document {
            id: "database_connection".to_string(),
            file_path: "db/connection.rs".to_string(),
            chunk_index: 0,
            tokens: vec![
                Token { text: "database".to_string(), position: 0, importance_weight: 1.0 },
                Token { text: "connection".to_string(), position: 1, importance_weight: 1.0 },
                Token { text: "pool".to_string(), position: 2, importance_weight: 1.0 },
                Token { text: "query".to_string(), position: 3, importance_weight: 1.0 },
                Token { text: "execute".to_string(), position: 4, importance_weight: 1.0 },
                Token { text: "transaction".to_string(), position: 5, importance_weight: 1.0 },
                Token { text: "user".to_string(), position: 6, importance_weight: 1.0 },
                Token { text: "data".to_string(), position: 7, importance_weight: 1.0 },
            ],
            start_line: 1,
            end_line: 25,
            language: Some("rust".to_string()),
        },
        BM25Document {
            id: "user_interface".to_string(),
            file_path: "ui/components.tsx".to_string(),
            chunk_index: 0,
            tokens: vec![
                Token { text: "user".to_string(), position: 0, importance_weight: 1.0 },
                Token { text: "interface".to_string(), position: 1, importance_weight: 1.0 },
                Token { text: "component".to_string(), position: 2, importance_weight: 1.0 },
                Token { text: "render".to_string(), position: 3, importance_weight: 1.0 },
                Token { text: "react".to_string(), position: 4, importance_weight: 1.0 },
                Token { text: "profile".to_string(), position: 5, importance_weight: 1.0 },
            ],
            start_line: 1,
            end_line: 30,
            language: Some("typescript".to_string()),
        },
        BM25Document {
            id: "data_processor".to_string(),
            file_path: "analytics/processor.py".to_string(),
            chunk_index: 0,
            tokens: vec![
                Token { text: "data".to_string(), position: 0, importance_weight: 1.0 },
                Token { text: "processing".to_string(), position: 1, importance_weight: 1.0 },
                Token { text: "pipeline".to_string(), position: 2, importance_weight: 1.0 },
                Token { text: "analytics".to_string(), position: 3, importance_weight: 1.0 },
                Token { text: "transform".to_string(), position: 4, importance_weight: 1.0 },
                Token { text: "validate".to_string(), position: 5, importance_weight: 1.0 },
                Token { text: "export".to_string(), position: 6, importance_weight: 1.0 },
            ],
            start_line: 1,
            end_line: 40,
            language: Some("python".to_string()),
        },
        BM25Document {
            id: "test_file".to_string(),
            file_path: "tests/auth_test.rs".to_string(),
            chunk_index: 0,
            tokens: vec![
                Token { text: "test".to_string(), position: 0, importance_weight: 1.0 },
                Token { text: "authentication".to_string(), position: 1, importance_weight: 1.0 },
                Token { text: "user".to_string(), position: 2, importance_weight: 1.0 },
                Token { text: "mock".to_string(), position: 3, importance_weight: 1.0 },
                Token { text: "assert".to_string(), position: 4, importance_weight: 1.0 },
            ],
            start_line: 1,
            end_line: 15,
            language: Some("rust".to_string()),
        },
    ];
    
    // Index all documents
    for doc in documents {
        engine.add_document(doc)?;
        println!("✅ Added document to BM25 index");
    }
    
    // Get engine statistics
    let stats = engine.get_stats();
    println!("\n📊 BM25 Engine Statistics:");
    println!("  Total documents: {}", stats.total_documents);
    println!("  Total terms: {}", stats.total_terms);
    println!("  Average document length: {:.2}", stats.avg_document_length);
    println!("  k1 parameter: {}", stats.k1);
    println!("  b parameter: {}", stats.b);
    
    // Verify basic integrity
    assert_eq!(stats.total_documents, 5, "Should have indexed 5 documents");
    assert!(stats.total_terms > 0, "Should have indexed terms");
    assert!(stats.avg_document_length > 0.0, "Should have calculated average document length");
    
    println!("\n=== BM25 Search Tests ===");
    
    // Test 1: Authentication query
    println!("\n🔍 Test 1: Authentication search");
    let results = engine.search("authentication user", 10)?;
    println!("Found {} results for 'authentication user':", results.len());
    for (i, result) in results.iter().enumerate() {
        println!("  {}. {} (score: {:.4}) - matched: {:?}", 
                i+1, result.doc_id, result.score, result.matched_terms);
    }
    assert!(!results.is_empty(), "Should find results for authentication query");
    assert!(results[0].doc_id == "auth_service", "Auth service should rank highest for authentication query");
    
    // Test 2: Database query
    println!("\n🔍 Test 2: Database connection search");
    let results = engine.search("database connection", 10)?;
    println!("Found {} results for 'database connection':", results.len());
    for (i, result) in results.iter().enumerate() {
        println!("  {}. {} (score: {:.4}) - matched: {:?}", 
                i+1, result.doc_id, result.score, result.matched_terms);
    }
    assert!(!results.is_empty(), "Should find results for database query");
    assert!(results[0].doc_id == "database_connection", "Database connection should rank highest");
    
    // Test 3: User interface query
    println!("\n🔍 Test 3: User interface search");
    let results = engine.search("user interface", 10)?;
    println!("Found {} results for 'user interface':", results.len());
    for (i, result) in results.iter().enumerate() {
        println!("  {}. {} (score: {:.4}) - matched: {:?}", 
                i+1, result.doc_id, result.score, result.matched_terms);
    }
    assert!(!results.is_empty(), "Should find results for UI query");
    // Note: "user" appears in multiple docs, but "interface" only in UI doc
    let ui_found = results.iter().any(|r| r.doc_id == "user_interface");
    assert!(ui_found, "Should find user interface document");
    
    // Test 4: Data processing query
    println!("\n🔍 Test 4: Data processing search");
    let results = engine.search("data processing pipeline", 10)?;
    println!("Found {} results for 'data processing pipeline':", results.len());
    for (i, result) in results.iter().enumerate() {
        println!("  {}. {} (score: {:.4}) - matched: {:?}", 
                i+1, result.doc_id, result.score, result.matched_terms);
    }
    assert!(!results.is_empty(), "Should find results for data processing query");
    assert!(results[0].doc_id == "data_processor", "Data processor should rank highest");
    
    // Test 5: Rare term query
    println!("\n🔍 Test 5: Rare term search");
    let results = engine.search("jwt", 10)?;
    println!("Found {} results for 'jwt':", results.len());
    for (i, result) in results.iter().enumerate() {
        println!("  {}. {} (score: {:.4}) - matched: {:?}", 
                i+1, result.doc_id, result.score, result.matched_terms);
    }
    assert!(!results.is_empty(), "Should find results for JWT query");
    assert!(results[0].doc_id == "auth_service", "Auth service should have JWT");
    
    // Test 6: Multi-term ranking
    println!("\n🔍 Test 6: Multi-term ranking test");
    let results = engine.search("user validate", 10)?;
    println!("Found {} results for 'user validate':", results.len());
    let mut scores: Vec<(String, f32)> = results.iter()
        .map(|r| (r.doc_id.clone(), r.score))
        .collect();
    scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    
    for (doc, score) in &scores {
        println!("  {} - score: {:.4}", doc, score);
    }
    
    // Documents with both terms should rank higher than those with only one
    let auth_score = scores.iter().find(|(id, _)| id == "auth_service").map(|(_, s)| *s).unwrap_or(0.0);
    let data_score = scores.iter().find(|(id, _)| id == "data_processor").map(|(_, s)| *s).unwrap_or(0.0);
    
    assert!(auth_score > 0.0 && data_score > 0.0, "Both docs should have positive scores");
    // Auth has both "user" and "validate", data has only "validate"
    assert!(auth_score >= data_score, "Auth service should score >= data processor for 'user validate'");
    
    println!("\n=== BM25 Algorithm Verification ===");
    
    // Test 7: IDF verification
    println!("\n🧮 IDF Calculations:");
    let terms = vec!["user", "authentication", "jwt", "nonexistent"];
    for term in &terms {
        let idf = engine.calculate_idf(term);
        println!("  IDF('{}') = {:.4}", term, idf);
    }
    
    let idf_user = engine.calculate_idf("user");
    let idf_jwt = engine.calculate_idf("jwt");
    let idf_none = engine.calculate_idf("nonexistent");
    
    // Rare terms should have higher IDF
    assert!(idf_jwt > idf_user, "Rare term 'jwt' should have higher IDF than common 'user'");
    assert!(idf_none > idf_jwt, "Non-existent term should have highest IDF");
    
    // Test 8: Term frequency saturation
    println!("\n⚖️ Testing BM25 saturation with repetitive document");
    let repetitive_doc = BM25Document {
        id: "repetitive".to_string(),
        file_path: "test/repetitive.rs".to_string(),
        chunk_index: 0,
        tokens: (0..20).map(|i| Token { 
            text: "function".to_string(), 
            position: i, 
            importance_weight: 1.0 
        }).collect(),
        start_line: 1,
        end_line: 5,
        language: Some("rust".to_string()),
    };
    
    let normal_doc = BM25Document {
        id: "normal".to_string(),
        file_path: "test/normal.rs".to_string(),
        chunk_index: 0,
        tokens: vec![
            Token { text: "function".to_string(), position: 0, importance_weight: 1.0 },
            Token { text: "calculate".to_string(), position: 1, importance_weight: 1.0 },
            Token { text: "result".to_string(), position: 2, importance_weight: 1.0 },
        ],
        start_line: 1,
        end_line: 3,
        language: Some("rust".to_string()),
    };
    
    engine.add_document(repetitive_doc)?;
    engine.add_document(normal_doc)?;
    
    let results = engine.search("function", 10)?;
    println!("Results for 'function' after adding repetitive docs:");
    for (i, result) in results.iter().enumerate() {
        println!("  {}. {} (score: {:.4})", i+1, result.doc_id, result.score);
    }
    
    // Find scores for comparison
    let rep_score = results.iter().find(|r| r.doc_id == "repetitive").map(|r| r.score).unwrap_or(0.0);
    let norm_score = results.iter().find(|r| r.doc_id == "normal").map(|r| r.score).unwrap_or(0.0);
    
    // BM25 saturation should prevent extreme score differences
    if rep_score > 0.0 && norm_score > 0.0 {
        let ratio = rep_score / norm_score;
        println!("  Score ratio (repetitive/normal): {:.2}", ratio);
        assert!(ratio < 5.0, "BM25 saturation should prevent extreme score differences, got ratio: {:.2}", ratio);
    }
    
    println!("\n✅ All BM25 functionality tests PASSED!");
    println!("✅ BM25 engine is FULLY FUNCTIONAL and implements correct algorithms");
    
    Ok(())
}

#[tokio::test]
async fn test_bm25_mathematical_correctness() -> Result<()> {
    println!("=== BM25 Mathematical Correctness Test ===");
    
    let mut engine = BM25Engine::with_params(1.2, 0.75);
    
    // Create documents with known term distributions
    let doc1 = BM25Document {
        id: "doc1".to_string(),
        file_path: "test1.txt".to_string(),
        chunk_index: 0,
        tokens: vec![
            Token { text: "cat".to_string(), position: 0, importance_weight: 1.0 },
            Token { text: "dog".to_string(), position: 1, importance_weight: 1.0 },
        ],
        start_line: 1,
        end_line: 1,
        language: None,
    };
    
    let doc2 = BM25Document {
        id: "doc2".to_string(),
        file_path: "test2.txt".to_string(),
        chunk_index: 0,
        tokens: vec![
            Token { text: "cat".to_string(), position: 0, importance_weight: 1.0 },
            Token { text: "cat".to_string(), position: 1, importance_weight: 1.0 },
            Token { text: "mouse".to_string(), position: 2, importance_weight: 1.0 },
        ],
        start_line: 1,
        end_line: 1,
        language: None,
    };
    
    engine.add_document(doc1)?;
    engine.add_document(doc2)?;
    
    // Test mathematical properties
    let stats = engine.get_stats();
    println!("Documents: {}, Terms: {}, Avg length: {:.2}", 
             stats.total_documents, stats.total_terms, stats.avg_document_length);
    
    // Expected: doc1 length = 2, doc2 length = 3, avg = 2.5
    assert!((stats.avg_document_length - 2.5).abs() < 0.001, "Average document length should be 2.5");
    
    // Test IDF calculations
    let idf_cat = engine.calculate_idf("cat");      // appears in 2/2 docs
    let idf_dog = engine.calculate_idf("dog");      // appears in 1/2 docs  
    let idf_mouse = engine.calculate_idf("mouse");  // appears in 1/2 docs
    
    println!("IDF values: cat={:.4}, dog={:.4}, mouse={:.4}", idf_cat, idf_dog, idf_mouse);
    
    // Dog and mouse should have equal IDF (both appear in 1 document)
    assert!((idf_dog - idf_mouse).abs() < 0.001, "Dog and mouse should have equal IDF");
    
    // Cat appears in all documents, so should have lower IDF
    assert!(idf_dog > idf_cat, "Rare terms should have higher IDF than common terms");
    
    // Test BM25 scoring
    let score1_cat = engine.calculate_bm25_score(&["cat".to_string()], "doc1")?;
    let score2_cat = engine.calculate_bm25_score(&["cat".to_string()], "doc2")?;
    
    println!("BM25 scores for 'cat': doc1={:.4}, doc2={:.4}", score1_cat, score2_cat);
    
    // Doc2 has higher term frequency for "cat" (2 vs 1), so should score higher
    assert!(score2_cat > score1_cat, "Document with higher term frequency should score higher");
    
    // Test search ranking
    let results = engine.search("cat", 10)?;
    assert_eq!(results.len(), 2, "Should find both documents");
    assert_eq!(results[0].doc_id, "doc2", "Doc2 should rank higher due to higher TF");
    assert_eq!(results[1].doc_id, "doc1", "Doc1 should rank lower");
    
    println!("✅ BM25 mathematical correctness verified!");
    
    Ok(())
}

#[tokio::test]
async fn test_bm25_edge_cases_and_robustness() -> Result<()> {
    println!("=== BM25 Edge Cases and Robustness Test ===");
    
    let mut engine = BM25Engine::new();
    
    // Test 1: Empty query
    let result = engine.search("", 10);
    assert!(result.is_err(), "Empty query should return error");
    println!("✅ Empty query correctly rejected");
    
    // Test 2: Whitespace-only query
    let result = engine.search("   \t\n   ", 10);
    assert!(result.is_err(), "Whitespace-only query should return error");
    println!("✅ Whitespace-only query correctly rejected");
    
    // Test 3: Query with no matches
    let doc = BM25Document {
        id: "test".to_string(),
        file_path: "test.txt".to_string(),
        chunk_index: 0,
        tokens: vec![Token { text: "hello".to_string(), position: 0, importance_weight: 1.0 }],
        start_line: 1,
        end_line: 1,
        language: None,
    };
    engine.add_document(doc)?;
    
    let results = engine.search("goodbye", 10)?;
    assert!(results.is_empty(), "Query with no matches should return empty results");
    println!("✅ No-match query returns empty results");
    
    // Test 4: Case insensitivity
    let results_lower = engine.search("hello", 10)?;
    let results_upper = engine.search("HELLO", 10)?;
    let results_mixed = engine.search("HeLLo", 10)?;
    
    assert_eq!(results_lower.len(), results_upper.len(), "Search should be case insensitive");
    assert_eq!(results_lower.len(), results_mixed.len(), "Search should be case insensitive");
    println!("✅ Case insensitive search verified");
    
    // Test 5: Very long document
    let long_tokens: Vec<Token> = (0..1000).map(|i| Token {
        text: if i % 100 == 0 { "special".to_string() } else { format!("word{}", i) },
        position: i,
        importance_weight: 1.0,
    }).collect();
    
    let long_doc = BM25Document {
        id: "long_doc".to_string(),
        file_path: "long.txt".to_string(),
        chunk_index: 0,
        tokens: long_tokens,
        start_line: 1,
        end_line: 100,
        language: None,
    };
    
    engine.add_document(long_doc)?;
    let results = engine.search("special", 10)?;
    assert!(!results.is_empty(), "Should find results in long document");
    println!("✅ Long document handling verified");
    
    // Test 6: Empty document
    let empty_doc = BM25Document {
        id: "empty".to_string(),
        file_path: "empty.txt".to_string(),
        chunk_index: 0,
        tokens: vec![],
        start_line: 1,
        end_line: 1,
        language: None,
    };
    
    let result = engine.add_document(empty_doc);
    // Empty documents might be allowed but shouldn't break the engine
    if result.is_ok() {
        println!("✅ Empty document handled gracefully");
    }
    
    // Test 7: Score validation
    let results = engine.search("hello", 10)?;
    for result in &results {
        assert!(result.score.is_finite(), "All scores should be finite");
        assert!(result.score >= 0.0, "Scores should be non-negative");
    }
    println!("✅ All scores are valid and finite");
    
    println!("✅ All edge cases handled robustly!");
    
    Ok(())
}