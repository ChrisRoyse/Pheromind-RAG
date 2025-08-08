use embed_search::search::bm25::{BM25Engine, BM25Document, Token};
use embed_search::search::text_processor::CodeTextProcessor;
use embed_search::search::preprocessing::QueryPreprocessor;
use embed_search::config::Config;
use anyhow::Result;

/// TRUTH VERIFICATION TEST: Baseline functionality verification
/// This test provides independent verification of claimed component functionality
/// without trusting any previous agent claims or outputs.

#[test]
fn baseline_bm25_functionality_verification() -> Result<()> {
    println!("=== BM25 ENGINE BASELINE VERIFICATION ===");
    
    // Test 1: Engine Creation
    let mut engine = BM25Engine::new();
    println!("✓ BM25Engine creation: SUCCESS");
    
    // Test 2: Document Addition
    let doc1 = BM25Document {
        id: "test_doc_1".to_string(),
        file_path: "test.rs".to_string(),
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
    println!("✓ Document addition: SUCCESS (2 documents indexed)");
    
    // Test 3: Search Functionality
    let results = engine.search("function", 5)?;
    
    if results.is_empty() {
        println!("✗ Search functionality: FAILED - No results returned");
        return Err(anyhow::anyhow!("BM25 search returned no results for 'function'"));
    }
    
    println!("✓ Search functionality: SUCCESS ({} results)", results.len());
    
    // Test 4: Score Validation
    if results[0].score <= 0.0 || !results[0].score.is_finite() {
        println!("✗ Score calculation: FAILED - Invalid score: {}", results[0].score);
        return Err(anyhow::anyhow!("BM25 produced invalid score"));
    }
    
    println!("✓ Score calculation: SUCCESS (score: {:.4})", results[0].score);
    
    // Test 5: Ranking Verification
    if results.len() >= 2 && results[0].score <= results[1].score {
        println!("✗ Ranking logic: FAILED - Results not properly ranked");
        return Err(anyhow::anyhow!("BM25 ranking failed"));
    }
    
    if results.len() >= 2 {
        println!("✓ Ranking logic: SUCCESS (top score: {:.4}, second: {:.4})", 
                results[0].score, results[1].score);
    } else {
        println!("✓ Ranking logic: SUCCESS (single result)");
    }
    
    // Test 6: IDF Calculation
    let idf_common = engine.calculate_idf("function"); // appears in both docs
    let idf_rare = engine.calculate_idf("calculate"); // appears in one doc
    
    if idf_rare <= idf_common {
        println!("✗ IDF calculation: FAILED - Rare terms should have higher IDF");
        return Err(anyhow::anyhow!("IDF calculation logic failed"));
    }
    
    println!("✓ IDF calculation: SUCCESS (rare: {:.4} > common: {:.4})", idf_rare, idf_common);
    
    println!("=== BM25 ENGINE VERIFICATION COMPLETE: ALL TESTS PASSED ===\n");
    Ok(())
}

#[test]
fn baseline_text_processor_functionality_verification() -> Result<()> {
    println!("=== TEXT PROCESSOR BASELINE VERIFICATION ===");
    
    // Test 1: Processor Creation
    let processor = CodeTextProcessor::new();
    println!("✓ CodeTextProcessor creation: SUCCESS");
    
    // Test 2: Basic Tokenization
    let code = "function calculateTotal(items) { return sum; }";
    let tokens = processor.tokenize_code(code, Some("javascript"));
    
    if tokens.is_empty() {
        println!("✗ Tokenization: FAILED - No tokens produced");
        return Err(anyhow::anyhow!("Text processor produced no tokens"));
    }
    
    println!("✓ Tokenization: SUCCESS ({} tokens)", tokens.len());
    
    // Test 3: Token Content Verification
    let token_texts: Vec<String> = tokens.iter().map(|t| t.text.clone()).collect();
    let has_function_related = token_texts.iter().any(|t| t.contains("function") || t.contains("calculate"));
    
    if !has_function_related {
        println!("✗ Token content: FAILED - Missing expected tokens");
        println!("  Tokens found: {:?}", token_texts);
        return Err(anyhow::anyhow!("Text processor missing expected content"));
    }
    
    println!("✓ Token content: SUCCESS (found relevant tokens)");
    
    // Test 4: CamelCase Splitting
    let camel_tokens = processor.process_text("getUserName", "javascript");
    let camel_texts: Vec<String> = camel_tokens.iter().map(|t| t.text.clone()).collect();
    let has_split = camel_texts.iter().any(|t| t == "get" || t == "user" || t == "name");
    
    if !has_split {
        println!("✗ CamelCase splitting: FAILED - No splitting detected");
        return Err(anyhow::anyhow!("CamelCase splitting not working"));
    }
    
    println!("✓ CamelCase splitting: SUCCESS");
    
    println!("=== TEXT PROCESSOR VERIFICATION COMPLETE: ALL TESTS PASSED ===\n");
    Ok(())
}

#[test]
fn baseline_query_preprocessor_functionality_verification() -> Result<()> {
    println!("=== QUERY PREPROCESSOR BASELINE VERIFICATION ===");
    
    // Test 1: Preprocessor Creation
    let preprocessor = QueryPreprocessor::new();
    println!("✓ QueryPreprocessor creation: SUCCESS");
    
    // Test 2: Noise Word Removal
    let result = preprocessor.preprocess("find the function in the database");
    if result.contains("the") || result.contains(" in ") {
        println!("✗ Noise word removal: FAILED - Still contains noise words");
        return Err(anyhow::anyhow!("Query preprocessor failed to remove noise words"));
    }
    
    println!("✓ Noise word removal: SUCCESS ('{}')", result);
    
    // Test 3: Abbreviation Expansion
    let result = preprocessor.preprocess("fn auth db");
    if !result.contains("function") || !result.contains("authentication") || !result.contains("database") {
        println!("✗ Abbreviation expansion: FAILED - Abbreviations not expanded");
        return Err(anyhow::anyhow!("Abbreviation expansion failed"));
    }
    
    println!("✓ Abbreviation expansion: SUCCESS ('{}')", result);
    
    // Test 4: Whitespace Normalization
    let result = preprocessor.preprocess("  multiple   spaces   here  ");
    if result.contains("  ") || result.starts_with(' ') || result.ends_with(' ') {
        println!("✗ Whitespace normalization: FAILED - Extra spaces remain");
        return Err(anyhow::anyhow!("Whitespace normalization failed"));
    }
    
    println!("✓ Whitespace normalization: SUCCESS ('{}')", result);
    
    // Test 5: Keyword Extraction
    let keywords = preprocessor.extract_keywords("search for function definitions");
    if keywords.is_empty() {
        println!("✗ Keyword extraction: FAILED - No keywords extracted");
        return Err(anyhow::anyhow!("Keyword extraction failed"));
    }
    
    println!("✓ Keyword extraction: SUCCESS ({} keywords)", keywords.len());
    
    println!("=== QUERY PREPROCESSOR VERIFICATION COMPLETE: ALL TESTS PASSED ===\n");
    Ok(())
}

#[test]
fn baseline_config_functionality_verification() -> Result<()> {
    println!("=== CONFIG SYSTEM BASELINE VERIFICATION ===");
    
    // Test 1: Test Config Creation
    #[cfg(any(test, debug_assertions))]
    {
        let config = Config::new_test_config();
        println!("✓ Config creation: SUCCESS");
        
        // Test 2: Config Validation
        let validation_result = config.validate();
        if validation_result.is_err() {
            println!("✗ Config validation: FAILED - {:?}", validation_result);
            return Err(anyhow::anyhow!("Config validation failed"));
        }
        
        println!("✓ Config validation: SUCCESS");
        
        // Test 3: Basic Field Access
        if config.chunk_size == 0 {
            println!("✗ Config field access: FAILED - Invalid chunk_size");
            return Err(anyhow::anyhow!("Config field access failed"));
        }
        
        println!("✓ Config field access: SUCCESS (chunk_size: {})", config.chunk_size);
        
        // Test 4: Global Config Initialization
        let init_result = Config::init_test();
        if init_result.is_err() {
            println!("✗ Global config init: FAILED - {:?}", init_result);
            return Err(anyhow::anyhow!("Global config initialization failed"));
        }
        
        println!("✓ Global config init: SUCCESS");
        
        // Test 5: Global Config Access
        let retrieved_config = Config::get();
        if retrieved_config.is_err() {
            println!("✗ Global config access: FAILED - {:?}", retrieved_config);
            return Err(anyhow::anyhow!("Global config access failed"));
        }
        
        println!("✓ Global config access: SUCCESS");
    }
    
    #[cfg(not(any(test, debug_assertions)))]
    {
        println!("⚠ Config testing skipped: Not in test/debug mode");
    }
    
    println!("=== CONFIG SYSTEM VERIFICATION COMPLETE: ALL TESTS PASSED ===\n");
    Ok(())
}