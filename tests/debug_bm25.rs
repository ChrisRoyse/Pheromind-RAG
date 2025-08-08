use embed_search::search::bm25::{BM25Engine, BM25Document, Token};

#[tokio::test]
async fn debug_bm25_basic() {
    let mut engine = BM25Engine::new();
    
    // Create a simple document
    let doc = BM25Document {
        id: "test-doc".to_string(),
        file_path: "test.rs".to_string(),
        chunk_index: 0,
        tokens: vec![
            Token { text: "persist".to_string(), position: 0, importance_weight: 1.0 },
            Token { text: "data".to_string(), position: 1, importance_weight: 1.0 },
        ],
        start_line: 1,
        end_line: 5,
        language: Some("rust".to_string()),
    };
    
    // Index the document
    println!("Adding document to BM25 engine...");
    engine.add_document(doc).expect("Failed to add document");
    
    // Search
    println!("Searching for 'persist data'...");
    let results = engine.search("persist data", 10).expect("Search failed");
    
    println!("Results: {:?}", results);
    assert!(!results.is_empty(), "Should find at least one result");
}