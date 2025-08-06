// Level 1: Core Tests (No heavy dependencies)
// Memory usage: ~50MB, Runtime: <5 seconds
// Run with: cargo test --no-default-features

use embed_search::search::{BM25Engine, BM25Document, BM25Token};
use embed_search::search::{CodeTextProcessor, ProcessedToken};
use embed_search::chunking::SimpleRegexChunker;

/// Fast unit tests for core BM25 functionality
mod bm25_core {
    use super::*;

    #[test]
    fn bm25_basic_functionality() {
        let mut engine = BM25Engine::new();
        
        // Test term frequency and IDF calculations
        let doc = BM25Document {
            id: "test".to_string(),
            file_path: "test.rs".to_string(),
            chunk_index: 0,
            tokens: vec![
                BM25Token { text: "function".to_string(), position: 0, importance_weight: 1.0 },
                BM25Token { text: "calculate".to_string(), position: 1, importance_weight: 1.0 },
            ],
            start_line: 0,
            end_line: 1,
            language: Some("rust".to_string()),
        };
        
        engine.add_document(doc).unwrap();
        
        // Verify search works
        let results = engine.search("function", 5);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].doc_id, "test");
        
        // Verify empty query handling
        assert_eq!(engine.search("", 5).len(), 0);
        assert_eq!(engine.search("   ", 5).len(), 0);
    }

    #[test]
    fn bm25_idf_calculation() {
        let mut engine = BM25Engine::new();
        
        // Add multiple docs to test IDF
        for i in 0..5 {
            let mut tokens = vec![
                BM25Token { text: "common".to_string(), position: 0, importance_weight: 1.0 },
            ];
            
            if i < 2 {
                tokens.push(BM25Token { text: "rare".to_string(), position: 1, importance_weight: 1.0 });
            }
            
            let doc = BM25Document {
                id: format!("doc{}", i),
                file_path: format!("test{}.rs", i),
                chunk_index: 0,
                tokens,
                start_line: 0,
                end_line: 1,
                language: Some("rust".to_string()),
            };
            
            engine.add_document(doc).unwrap();
        }
        
        // Rare terms should have higher IDF
        assert!(engine.calculate_idf("rare") > engine.calculate_idf("common"));
    }

    #[test]
    fn bm25_scoring() {
        let mut engine = BM25Engine::new();
        
        // Document with repeated term
        let doc1 = BM25Document {
            id: "repeated".to_string(),
            file_path: "test1.rs".to_string(),
            chunk_index: 0,
            tokens: vec![
                BM25Token { text: "search".to_string(), position: 0, importance_weight: 1.0 },
                BM25Token { text: "search".to_string(), position: 1, importance_weight: 1.0 },
                BM25Token { text: "search".to_string(), position: 2, importance_weight: 1.0 },
            ],
            start_line: 0,
            end_line: 1,
            language: Some("rust".to_string()),
        };
        
        // Document with single term
        let doc2 = BM25Document {
            id: "single".to_string(),
            file_path: "test2.rs".to_string(),
            chunk_index: 0,
            tokens: vec![
                BM25Token { text: "search".to_string(), position: 0, importance_weight: 1.0 },
            ],
            start_line: 0,
            end_line: 1,
            language: Some("rust".to_string()),
        };
        
        engine.add_document(doc1).unwrap();
        engine.add_document(doc2).unwrap();
        
        let results = engine.search("search", 5);
        assert_eq!(results.len(), 2);
        
        // Due to term frequency saturation (BM25 k1 parameter), the document with
        // repeated terms should score higher, but not linearly
        let repeated_score = results.iter().find(|r| r.doc_id == "repeated").unwrap().score;
        let single_score = results.iter().find(|r| r.doc_id == "single").unwrap().score;
        
        assert!(repeated_score > single_score);
    }
}

/// Text processing unit tests
mod text_processor_core {
    use super::*;

    #[test]
    fn tokenization_basic() {
        let processor = CodeTextProcessor::new();
        let tokens = processor.process_text("hello world", "rust");
        
        assert!(tokens.len() >= 2);
        assert!(tokens.iter().any(|t| t.text == "hello"));
        assert!(tokens.iter().any(|t| t.text == "world"));
    }

    #[test]
    fn camel_case_splitting() {
        let processor = CodeTextProcessor::new();
        let tokens = processor.process_text("getUserById", "rust");
        
        // Should split camelCase
        let token_texts: Vec<&str> = tokens.iter().map(|t| t.text.as_str()).collect();
        assert!(token_texts.contains(&"get") || token_texts.contains(&"user") || token_texts.contains(&"id"));
    }

    #[test]
    fn code_specific_tokens() {
        let processor = CodeTextProcessor::new();
        let tokens = processor.process_text("fn main() {}", "rust");
        
        let token_texts: Vec<&str> = tokens.iter().map(|t| t.text.as_str()).collect();
        assert!(token_texts.contains(&"fn"));
        assert!(token_texts.contains(&"main"));
    }
}

/// Chunking algorithm tests
mod chunking_core {
    use super::*;

    #[test]
    fn regex_chunking() {
        let chunker = SimpleRegexChunker::new(100, 20);
        let content = "fn main() {\n    println!(\"hello\");\n}\n\nfn test() {\n    assert_eq!(1, 1);\n}";
        
        let chunks = chunker.chunk_file(content);
        assert!(!chunks.is_empty());
        
        for chunk in &chunks {
            assert!(chunk.content.len() <= 100);
            assert!(chunk.start_line <= chunk.end_line);
        }
    }

    #[test]
    fn chunking_overlap() {
        let chunker = SimpleRegexChunker::new(50, 10);
        let content = "a".repeat(150); // 150 chars
        
        let chunks = chunker.chunk_file(&content);
        assert!(chunks.len() >= 3); // Should need multiple chunks
        
        // Verify chunks don't exceed max size
        for chunk in &chunks {
            assert!(chunk.content.len() <= 50);
        }
    }
}

/// Configuration and utility tests
mod config_core {
    use embed_search::config::Config;

    #[test]
    fn config_default() {
        let config = Config::default();
        // Basic config validation
        assert!(!config.project_path.as_os_str().is_empty());
    }
}