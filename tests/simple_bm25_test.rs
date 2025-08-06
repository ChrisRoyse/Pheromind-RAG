#[cfg(test)]
mod tests {
    use embed_search::search::bm25::{BM25Engine, BM25Document, Token as BM25Token};
    
    #[test]
    fn test_bm25_engine_directly() {
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
        assert_eq!(stats.total_documents, 1);
        
        // Search for "function"
        println!("\nSearching for 'function':");
        let results = engine.search("function", 10);
        println!("Found {} results", results.len());
        assert_eq!(results.len(), 1, "Should find 1 result for 'function'");
        assert_eq!(results[0].doc_id, "test.js-0");
        
        // Search for "calculate"
        println!("\nSearching for 'calculate':");
        let results = engine.search("calculate", 10);
        println!("Found {} results", results.len());
        assert_eq!(results.len(), 1, "Should find 1 result for 'calculate'");
        
        // Search for multi-word query
        println!("\nSearching for 'function calculate':");
        let results = engine.search("function calculate", 10);
        println!("Found {} results", results.len());
        assert_eq!(results.len(), 1, "Should find 1 result for 'function calculate'");
        assert!(results[0].score > 0.0, "Score should be positive");
    }
}