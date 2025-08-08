use embed_search::search::bm25::{BM25Engine, BM25Document, Token};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== DEBUGGING BM25 IDF CALCULATION ===");
    
    let mut engine = BM25Engine::new();
    
    // Create test documents exactly like the baseline test
    let doc1 = BM25Document {
        id: "test_doc_1".to_string(),
        file_path: "test1.rs".to_string(), 
        chunk_index: 0,
        tokens: vec![
            Token { text: "function".to_string(), position: 0, importance_weight: 1.0 },
            Token { text: "calculate".to_string(), position: 1, importance_weight: 1.0 },
            Token { text: "total".to_string(), position: 2, importance_weight: 1.0 },
        ],
        start_line: 1,
        end_line: 10,
        language: Some("rust".to_string()),
    };
    
    let doc2 = BM25Document {
        id: "test_doc_2".to_string(),
        file_path: "test2.rs".to_string(),
        chunk_index: 0, 
        tokens: vec![
            Token { text: "function".to_string(), position: 0, importance_weight: 1.0 },
            Token { text: "function".to_string(), position: 1, importance_weight: 1.0 },
            Token { text: "process".to_string(), position: 2, importance_weight: 1.0 },
        ],
        start_line: 1,
        end_line: 10,
        language: Some("rust".to_string()),
    };
    
    println!("Adding doc1...");
    engine.add_document(doc1)?;
    println!("Adding doc2...");
    engine.add_document(doc2)?;
    
    println!("\nEngine state:");
    println!("Documents added successfully");
    
    let idf_common = engine.calculate_idf("function");
    let idf_rare = engine.calculate_idf("calculate");
    
    println!("\nDEBUG: Let's check what's wrong...");
    println!("Testing non-existent term: {:.6}", engine.calculate_idf("nonexistent"));
    
    println!("\nIDF Results:");
    println!("IDF function (common, in 2 docs): {:.6}", idf_common);
    println!("IDF calculate (rare, in 1 doc): {:.6}", idf_rare);
    println!("Rare > Common: {}", idf_rare > idf_common);
    
    if idf_rare <= idf_common {
        println!("\n❌ PROBLEM: Rare term has lower or equal IDF than common term!");
        println!("This indicates a bug in the IDF calculation or document frequency tracking.");
    } else {
        println!("\n✅ SUCCESS: IDF calculation working correctly!");
    }
    
    Ok(())
}