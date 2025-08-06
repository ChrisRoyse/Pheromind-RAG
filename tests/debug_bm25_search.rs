use embed_search::search::bm25::{BM25Engine, BM25Document, Token as BM25Token};

#[test]
fn debug_bm25_search() {
    // Create a simple BM25 engine
    let mut engine = BM25Engine::new();
    
    // Create a simple document
    let tokens = vec![
        BM25Token { text: "function".to_string(), position: 0, importance_weight: 1.0 },
        BM25Token { text: "calculate".to_string(), position: 1, importance_weight: 1.0 },
        BM25Token { text: "total".to_string(), position: 2, importance_weight: 1.0 },
    ];
    
    let doc = BM25Document {
        id: "test.js-0".to_string(),
        file_path: "test.js".to_string(),
        chunk_index: 0,
        tokens,
        start_line: 0,
        end_line: 10,
        language: Some("javascript".to_string()),
    };
    
    // Add document
    engine.add_document(doc).unwrap();
    
    // Get stats
    let stats = engine.get_stats();
    println!("Index stats: {:?}", stats);
    
    // Debug: Check internal state
    println!("\nDEBUG: Checking engine internal state...");
    
    // Search for "function"
    println!("\nSearching for 'function':");
    let results = engine.search("function", 10);
    println!("Found {} results", results.len());
    
    if results.is_empty() {
        println!("\nERROR: No results found for 'function'!");
        println!("This indicates a bug in the search implementation.");
        
        // Let's manually check if the term exists
        println!("\nManual check: Looking for 'function' in the index...");
        
        // The search method normalizes to lowercase, so let's check both
        let test_queries = vec!["function", "Function", "FUNCTION"];
        for query in test_queries {
            println!("  Testing query: '{}'", query);
            let results = engine.search(query, 10);
            println!("    Results: {}", results.len());
        }
    } else {
        for result in &results {
            println!("  - Doc: {}, Score: {}", result.doc_id, result.score);
        }
    }
    
    assert!(results.len() > 0, "Should find at least one result for 'function'");
}