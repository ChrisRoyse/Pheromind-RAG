use embed_search::search::bm25::{BM25Engine, BM25Document, Token as BM25Token};
use embed_search::search::text_processor::CodeTextProcessor;

fn main() {
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
    
    // Search for "function"
    println!("\nSearching for 'function':");
    let results = engine.search("function", 10);
    println!("Found {} results", results.len());
    for result in &results {
        println!("  - Doc: {}, Score: {}", result.doc_id, result.score);
    }
    
    // Search for "calculate"
    println!("\nSearching for 'calculate':");
    let results = engine.search("calculate", 10);
    println!("Found {} results", results.len());
    for result in &results {
        println!("  - Doc: {}, Score: {}", result.doc_id, result.score);
    }
    
    // Test text processor
    println!("\n\nTesting text processor:");
    let processor = CodeTextProcessor::new();
    let code = "function calculateTotal() { return sum; }";
    let tokens = processor.tokenize_code(code, Some("javascript"));
    println!("Tokenized '{}' into {} tokens:", code, tokens.len());
    for token in &tokens[..5.min(tokens.len())] {
        println!("  - '{}' (type: {:?}, weight: {})", token.text, token.token_type, token.importance_weight);
    }
}