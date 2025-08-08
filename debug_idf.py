#!/usr/bin/env python3
"""Debug the IDF calculation issue."""

import subprocess
import json
import sys

def main():
    print("=== DEBUGGING BM25 IDF CALCULATION ===")
    
    # Create a simple Rust test program
    rust_code = '''
use embed_search::search::{BM25Engine, Token, BM25Document};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut engine = BM25Engine::new();
    
    // Create test documents exactly like the test
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
    
    engine.add_document(doc1)?;
    engine.add_document(doc2)?;
    
    println!("Total docs: {}", engine.total_docs);
    println!("Doc frequencies: {:?}", engine.doc_frequencies);
    
    let idf_common = engine.calculate_idf("function");
    let idf_rare = engine.calculate_idf("calculate");
    
    println!("IDF function (common): {:.6}", idf_common);
    println!("IDF calculate (rare): {:.6}", idf_rare);
    println!("Rare > Common: {}", idf_rare > idf_common);
    
    Ok(())
}
'''
    
    # Write the test program
    with open('debug_bm25.rs', 'w') as f:
        f.write(rust_code)
    
    print("Created debug_bm25.rs - add it to Cargo.toml [[bin]] section to run")

if __name__ == "__main__":
    main()