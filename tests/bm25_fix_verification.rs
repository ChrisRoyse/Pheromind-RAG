use embed_search::search::bm25::{BM25Engine, BM25Document, Token};

#[test]
fn test_idf_calculation_mathematical_correctness() {
    let mut engine = BM25Engine::new();
    
    // Create test scenario where we have very common terms that would produce negative IDF
    // We'll create 10 documents, with "common" appearing in 8 documents (80% - very common)
    // and "rare" appearing in only 1 document (10% - rare)
    
    for i in 0..10 {
        let mut tokens = vec![
            Token { text: "common".to_string(), position: 0, importance_weight: 1.0 },
        ];
        
        // Only doc0 gets the rare term
        if i == 0 {
            tokens.push(Token { text: "rare".to_string(), position: 1, importance_weight: 1.0 });
        }
        
        let doc = BM25Document {
            id: format!("doc{}", i),
            file_path: format!("test{}.rs", i),
            chunk_index: 0,
            tokens,
            start_line: 0,
            end_line: 10,
            language: Some("rust".to_string()),
        };
        
        engine.add_document(doc).expect("Document addition must succeed");
    }
    
    // Calculate IDFs - the fix should ensure proper ordering
    let idf_common = engine.calculate_idf("common");  // Very common term (appears in 8/10 docs)
    let idf_rare = engine.calculate_idf("rare");      // Rare term (appears in 1/10 docs)
    
    // Verify mathematical correctness:
    // 1. Both IDFs should be positive (no more negative values)
    assert!(idf_common > 0.0, "Common term IDF should be positive, got: {}", idf_common);
    assert!(idf_rare > 0.0, "Rare term IDF should be positive, got: {}", idf_rare);
    
    // 2. Rare terms should have higher IDF than common terms (fundamental BM25 principle)
    assert!(idf_rare > idf_common, "Rare term IDF ({}) should be higher than common term IDF ({})", idf_rare, idf_common);
    
    // 3. The difference should be significant for such different term frequencies
    let ratio = idf_rare / idf_common;
    assert!(ratio > 2.0, "IDF ratio should be significant (>2.0), got: {}", ratio);
    
    println!("IDF Fix Verification SUCCESS:");
    println!("  Common term IDF: {:.6}", idf_common);
    println!("  Rare term IDF: {:.6}", idf_rare);
    println!("  Ratio (rare/common): {:.2}", ratio);
}

#[test]
fn test_document_length_edge_cases() {
    let mut engine = BM25Engine::new();
    
    // Test adding documents with various lengths
    let doc1 = BM25Document {
        id: "doc1".to_string(),
        file_path: "test1.rs".to_string(),
        chunk_index: 0,
        tokens: vec![
            Token { text: "test".to_string(), position: 0, importance_weight: 1.0 },
        ],
        start_line: 0,
        end_line: 10,
        language: Some("rust".to_string()),
    };
    
    engine.add_document(doc1).expect("Document addition must succeed");
    let stats1 = engine.get_stats();
    assert_eq!(stats1.total_documents, 1);
    assert_eq!(stats1.avg_document_length, 1.0);
    
    // Add a longer document
    let doc2 = BM25Document {
        id: "doc2".to_string(),
        file_path: "test2.rs".to_string(),
        chunk_index: 0,
        tokens: vec![
            Token { text: "test".to_string(), position: 0, importance_weight: 1.0 },
            Token { text: "longer".to_string(), position: 1, importance_weight: 1.0 },
            Token { text: "document".to_string(), position: 2, importance_weight: 1.0 },
        ],
        start_line: 0,
        end_line: 10,
        language: Some("rust".to_string()),
    };
    
    engine.add_document(doc2).expect("Document addition must succeed");
    let stats2 = engine.get_stats();
    assert_eq!(stats2.total_documents, 2);
    assert_eq!(stats2.avg_document_length, 2.0); // (1 + 3) / 2 = 2.0
    
    // Verify the fixed edge case handling doesn't cause division by zero
    engine.clear();
    let stats3 = engine.get_stats();
    assert_eq!(stats3.total_documents, 0);
    assert_eq!(stats3.avg_document_length, 0.0);
    
    println!("Document length edge cases handled correctly");
}