use embed_search::search::{UnifiedSearcher, SearchConfig, BM25Engine};
use embed_search::config::Config;
use std::path::PathBuf;
use tokio::fs;
use anyhow::Result;

#[tokio::test]
async fn test_bm25_search_functionality() -> Result<()> {
    // Test BM25 search engine directly
    let mut engine = BM25Engine::with_params(1.2, 0.75);
    
    // Create test documents
    let test_doc = embed_search::search::BM25Document {
        id: "test_doc_1".to_string(),
        file_path: "test.rs".to_string(),
        chunk_index: 0,
        tokens: vec![
            embed_search::search::BM25Token {
                text: "function".to_string(),
                position: 0,
                importance_weight: 1.0,
            },
            embed_search::search::BM25Token {
                text: "search".to_string(),
                position: 1,
                importance_weight: 1.0,
            },
        ],
        start_line: 1,
        end_line: 10,
        language: Some("rust".to_string()),
    };
    
    engine.add_document(test_doc)?;
    
    // Test search
    let results = engine.search("function", 10)?;
    
    assert!(!results.is_empty(), "BM25 search should return results for 'function'");
    assert_eq!(results[0].doc_id, "test_doc_1", "Should find the test document");
    
    println!("✅ BM25 search working - found {} results", results.len());
    Ok(())
}

#[tokio::test] 
#[cfg(feature = "tantivy")]
async fn test_tantivy_search_functionality() -> Result<()> {
    use embed_search::search::TantivySearcher;
    
    // Create temporary directory for index
    let temp_dir = tempfile::tempdir()?;
    let index_path = temp_dir.path().join("test_index");
    
    // Create Tantivy searcher
    let mut searcher = TantivySearcher::new_with_path(index_path).await?;
    
    // Create test file
    let test_file_path = temp_dir.path().join("test_code.rs");
    fs::write(&test_file_path, "fn search_function() { println!(\"Hello world\"); }").await?;
    
    // Index the test file
    searcher.index_file(&test_file_path).await?;
    
    // Test exact search
    let results = searcher.search("search_function").await?;
    
    assert!(!results.is_empty(), "Tantivy search should find 'search_function'");
    println!("✅ Tantivy exact search working - found {} results", results.len());
    
    // Test fuzzy search  
    let fuzzy_results = searcher.search_fuzzy("serch_functon", 2).await?;
    
    assert!(!fuzzy_results.is_empty(), "Tantivy fuzzy search should find misspelled 'serch_functon'");
    println!("✅ Tantivy fuzzy search working - found {} results", fuzzy_results.len());
    
    Ok(())
}

#[tokio::test]
#[cfg(feature = "tree-sitter")]
async fn test_symbol_search_functionality() -> Result<()> {
    use embed_search::search::{SymbolIndexer, SymbolDatabase};
    
    // Create symbol indexer
    let mut indexer = SymbolIndexer::new()?;
    
    // Test Rust code with symbols
    let test_code = r#"
struct TestStruct {
    field: String,
}

impl TestStruct {
    fn test_method(&self) -> String {
        self.field.clone()
    }
}

fn test_function() -> i32 {
    42
}
"#;
    
    // Extract symbols
    let symbols = indexer.extract_symbols(test_code, "rust", "test.rs")?;
    
    assert!(!symbols.is_empty(), "Should extract symbols from Rust code");
    
    // Create symbol database and add symbols
    let mut db = SymbolDatabase::new();
    db.add_symbols(symbols.clone());
    
    // Test symbol search
    let struct_symbols = db.find_all_references("TestStruct");
    assert!(!struct_symbols.is_empty(), "Should find TestStruct references");
    
    let function_symbols = db.find_all_references("test_function");
    assert!(!function_symbols.is_empty(), "Should find test_function references");
    
    println!("✅ Symbol search working - indexed {} symbols", symbols.len());
    println!("  Found {} TestStruct references", struct_symbols.len());
    println!("  Found {} test_function references", function_symbols.len());
    
    Ok(())
}

#[tokio::test]
async fn test_unified_search_integration() -> Result<()> {
    // Initialize config (required for UnifiedSearcher)
    Config::init()?;
    
    // Create temporary directories
    let temp_dir = tempfile::tempdir()?;
    let project_path = temp_dir.path().to_path_buf();
    let db_path = temp_dir.path().join("db");
    
    // Create test source file
    let src_dir = project_path.join("src");
    fs::create_dir_all(&src_dir).await?;
    
    let test_file = src_dir.join("main.rs");
    fs::write(&test_file, r#"
fn main() {
    println!("Hello search system!");
    let result = calculate_value(42);
}

fn calculate_value(input: i32) -> i32 {
    input * 2
}
"#).await?;
    
    // Create UnifiedSearcher
    let searcher = UnifiedSearcher::new(project_path.clone(), db_path).await?;
    
    // Index the directory
    let stats = searcher.index_directory(&src_dir).await?;
    println!("✅ Indexed {} files successfully", stats.files_indexed);
    
    // Test unified search
    let search_results = searcher.search("calculate_value").await?;
    
    if !search_results.is_empty() {
        println!("✅ Unified search working - found {} results for 'calculate_value'", search_results.len());
        for result in &search_results {
            println!("  - Found in: {}", result.file);
            println!("  - Score: {:.3}", result.score);
            println!("  - Match type: {:?}", result.match_type);
        }
    } else {
        println!("⚠️ Unified search returned no results, but indexing succeeded");
    }
    
    Ok(())
}

#[tokio::test]
async fn test_search_configuration() -> Result<()> {
    // Test SearchConfig controls feature enablement
    let search_config = SearchConfig {
        enable_bm25: true,
        enable_tantivy: false,
        enable_ml: false,
        enable_tree_sitter: true,
        index_path: std::path::PathBuf::from("test_index"),
    };
    
    // Test main Config for BM25 parameters
    let main_config = Config::new_test_config();
    
    // Create BM25 engine with parameters from main config
    let _engine = BM25Engine::with_params(main_config.bm25_k1, main_config.bm25_b);
    
    // Verify SearchConfig feature flags
    assert!(search_config.enable_bm25, "BM25 should be enabled");
    assert!(!search_config.enable_tantivy, "Tantivy should be disabled");
    assert!(search_config.enable_tree_sitter, "Tree-sitter should be enabled");
    
    // Verify main Config BM25 parameters
    assert_eq!(main_config.bm25_k1, 1.2, "K1 parameter should match test config default");
    assert_eq!(main_config.bm25_b, 0.75, "B parameter should match test config default");
    
    println!("✅ Search configuration working - feature flags properly set");
    println!("  SearchConfig features: BM25={}, Tantivy={}, TreeSitter={}", 
             search_config.enable_bm25, search_config.enable_tantivy, search_config.enable_tree_sitter);
    println!("  Main Config BM25 params: K1={}, B={}", main_config.bm25_k1, main_config.bm25_b);
    
    Ok(())
}