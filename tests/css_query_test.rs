use embed_search::search::symbol_index::SymbolIndexer;

#[test]
fn test_css_parser_initialization() {
    // This test will help us identify the exact CSS query error
    let result = SymbolIndexer::new();
    
    match result {
        Ok(_) => println!("CSS parser initialized successfully"),
        Err(e) => {
            println!("CSS parser failed to initialize: {}", e);
            // The error should be caught in init_css function
        }
    }
}

#[test]
fn test_css_symbol_extraction() {
    let css_code = r#"
    .button {
        background-color: blue;
        color: white;
    }
    
    #header {
        font-size: 24px;
    }
    
    @keyframes slideIn {
        from { transform: translateX(-100%); }
        to { transform: translateX(0); }
    }
    
    @media screen and (max-width: 768px) {
        .button { padding: 10px; }
    }
    "#;
    
    let mut indexer = match SymbolIndexer::new() {
        Ok(idx) => idx,
        Err(e) => {
            println!("Failed to create indexer: {}", e);
            return;
        }
    };
    
    let result = indexer.extract_symbols(css_code, "css", "test.css");
    match result {
        Ok(symbols) => {
            println!("Extracted {} symbols from CSS", symbols.len());
            for symbol in symbols {
                println!("Symbol: {} ({:?})", symbol.name, symbol.kind);
            }
        }
        Err(e) => {
            println!("Failed to extract CSS symbols: {}", e);
        }
    }
}