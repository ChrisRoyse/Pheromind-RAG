use super::*;
use anyhow::Result;

#[test]
fn test_bm25_incremental_updates() -> Result<()> {
    let mut engine = BM25Engine::new();
    
    // Add initial document
    let doc1 = BM25Document {
        id: "doc1".to_string(),
        file_path: "test.rs".to_string(),
        chunk_index: 0,
        tokens: vec![
            Token { text: "hello".to_string(), position: 0, importance_weight: 1.0 },
            Token { text: "world".to_string(), position: 1, importance_weight: 1.0 },
        ],
        start_line: 1,
        end_line: 10,
        language: Some("rust".to_string()),
    };
    
    engine.add_document(doc1)?;
    
    // Verify initial state
    let results = engine.search("hello", 10)?;
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].doc_id, "doc1");
    
    let stats = engine.get_stats();
    assert_eq!(stats.total_documents, 1);
    
    // Update the document
    let updated_doc = BM25Document {
        id: "doc1".to_string(),
        file_path: "test.rs".to_string(),
        chunk_index: 0,
        tokens: vec![
            Token { text: "hello".to_string(), position: 0, importance_weight: 1.0 },
            Token { text: "rust".to_string(), position: 1, importance_weight: 1.0 },
        ],
        start_line: 1,
        end_line: 10,
        language: Some("rust".to_string()),
    };
    
    engine.update_document(updated_doc)?;
    
    // Verify update worked
    let results = engine.search("rust", 10)?;
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].doc_id, "doc1");
    
    // Old term "world" should not be found
    let results = engine.search("world", 10)?;
    assert_eq!(results.len(), 0);
    
    // Document count should remain the same
    let stats = engine.get_stats();
    assert_eq!(stats.total_documents, 1);
    
    // Remove the document
    engine.remove_document("doc1")?;
    
    // Verify removal worked
    let results = engine.search("hello", 10)?;
    assert_eq!(results.len(), 0);
    
    let stats = engine.get_stats();
    assert_eq!(stats.total_documents, 0);
    
    Ok(())
}

#[test]
fn test_bm25_statistics_update() -> Result<()> {
    let mut engine = BM25Engine::new();
    
    // Add documents with overlapping terms
    let doc1 = BM25Document {
        id: "doc1".to_string(),
        file_path: "test1.rs".to_string(),
        chunk_index: 0,
        tokens: vec![
            Token { text: "hello".to_string(), position: 0, importance_weight: 1.0 },
            Token { text: "world".to_string(), position: 1, importance_weight: 1.0 },
        ],
        start_line: 1,
        end_line: 10,
        language: Some("rust".to_string()),
    };
    
    let doc2 = BM25Document {
        id: "doc2".to_string(),
        file_path: "test2.rs".to_string(),
        chunk_index: 0,
        tokens: vec![
            Token { text: "hello".to_string(), position: 0, importance_weight: 1.0 },
            Token { text: "rust".to_string(), position: 1, importance_weight: 1.0 },
        ],
        start_line: 1,
        end_line: 10,
        language: Some("rust".to_string()),
    };
    
    engine.add_document(doc1)?;
    engine.add_document(doc2)?;
    
    // Verify initial state
    let stats_before = engine.get_stats();
    assert_eq!(stats_before.total_documents, 2);
    
    // Remove one document
    engine.remove_document("doc1")?;
    
    // Update statistics
    engine.update_statistics()?;
    
    // Verify statistics are correct
    let stats_after = engine.get_stats();
    assert_eq!(stats_after.total_documents, 1);
    
    // "hello" should still be found in remaining document
    let results = engine.search("hello", 10)?;
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].doc_id, "doc2");
    
    // "world" should not be found (was only in removed document)
    let results = engine.search("world", 10)?;
    assert_eq!(results.len(), 0);
    
    Ok(())
}

#[test]
fn test_bm25_remove_multiple_documents() -> Result<()> {
    let mut engine = BM25Engine::new();
    
    // Add multiple documents
    for i in 0..5 {
        let doc = BM25Document {
            id: format!("doc{}", i),
            file_path: format!("test{}.rs", i),
            chunk_index: 0,
            tokens: vec![
                Token { text: "common".to_string(), position: 0, importance_weight: 1.0 },
                Token { text: format!("term{}", i), position: 1, importance_weight: 1.0 },
            ],
            start_line: 1,
            end_line: 10,
            language: Some("rust".to_string()),
        };
        engine.add_document(doc)?;
    }
    
    // Verify all documents are present
    let stats = engine.get_stats();
    assert_eq!(stats.total_documents, 5);
    
    let results = engine.search("common", 10)?;
    assert_eq!(results.len(), 5);
    
    // Remove documents one by one
    for i in 0..3 {
        engine.remove_document(&format!("doc{}", i))?;
    }
    
    // Update statistics
    engine.update_statistics()?;
    
    // Verify remaining documents
    let stats = engine.get_stats();
    assert_eq!(stats.total_documents, 2);
    
    let results = engine.search("common", 10)?;
    assert_eq!(results.len(), 2);
    
    // Verify specific terms are gone
    let results = engine.search("term0", 10)?;
    assert_eq!(results.len(), 0);
    
    let results = engine.search("term1", 10)?;
    assert_eq!(results.len(), 0);
    
    // But remaining terms should still work
    let results = engine.search("term3", 10)?;
    assert_eq!(results.len(), 1);
    
    Ok(())
}