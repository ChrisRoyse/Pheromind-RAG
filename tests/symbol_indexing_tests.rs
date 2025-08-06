#[cfg(feature = "tree-sitter")]
use std::path::PathBuf;
#[cfg(feature = "tree-sitter")]
use embed_search::search::symbol_index::{SymbolIndexer, SymbolDatabase, SymbolKind};

#[cfg(feature = "tree-sitter")]
#[test]
fn test_rust_symbol_extraction() {
    let mut indexer = SymbolIndexer::new().unwrap();
    let file_path = PathBuf::from("vectortest/memory_cache.rs");
    let content = std::fs::read_to_string(&file_path).unwrap();
    
    let symbols = indexer.extract_symbols(&content, "rust", file_path.to_str().unwrap()).unwrap();
    
    // Verify we found the main struct
    assert!(symbols.iter().any(|s| s.name == "MemoryCache" && s.kind == SymbolKind::Struct));
    
    // Verify we found the CacheEntry struct
    assert!(symbols.iter().any(|s| s.name == "CacheEntry" && s.kind == SymbolKind::Struct));
    
    // Count total symbols
    assert!(symbols.len() > 5, "Should find multiple symbols in Rust file");
    
    println!("Found {} Rust symbols", symbols.len());
    for symbol in &symbols {
        println!("  - {} ({:?}) at line {}", symbol.name, symbol.kind, symbol.line_start);
    }
}

#[cfg(feature = "tree-sitter")]
#[test]
fn test_python_symbol_extraction() {
    let mut indexer = SymbolIndexer::new().unwrap();
    let file_path = PathBuf::from("vectortest/auth_service.py");
    let content = std::fs::read_to_string(&file_path).unwrap();
    
    let symbols = indexer.extract_symbols(&content, "python", file_path.to_str().unwrap()).unwrap();
    
    // Python files should have classes and functions
    let classes: Vec<_> = symbols.iter().filter(|s| s.kind == SymbolKind::Class).collect();
    let functions: Vec<_> = symbols.iter().filter(|s| s.kind == SymbolKind::Function).collect();
    
    assert!(!classes.is_empty(), "Should find classes in Python file");
    assert!(!functions.is_empty(), "Should find functions in Python file");
    
    println!("Found {} Python symbols ({} classes, {} functions)", 
             symbols.len(), classes.len(), functions.len());
}

#[cfg(feature = "tree-sitter")]
#[test]
fn test_javascript_symbol_extraction() {
    let mut indexer = SymbolIndexer::new().unwrap();
    let file_path = PathBuf::from("vectortest/user_controller.js");
    let content = std::fs::read_to_string(&file_path).unwrap();
    
    let symbols = indexer.extract_symbols(&content, "javascript", file_path.to_str().unwrap()).unwrap();
    
    // JavaScript should have functions and variables
    assert!(!symbols.is_empty(), "Should find symbols in JavaScript file");
    
    println!("Found {} JavaScript symbols", symbols.len());
    for symbol in &symbols {
        println!("  - {} ({:?})", symbol.name, symbol.kind);
    }
}

#[cfg(feature = "tree-sitter")]
#[test]
fn test_typescript_symbol_extraction() {
    let mut indexer = SymbolIndexer::new().unwrap();
    let file_path = PathBuf::from("vectortest/payment_gateway.ts");
    let content = std::fs::read_to_string(&file_path).unwrap();
    
    let symbols = indexer.extract_symbols(&content, "typescript", file_path.to_str().unwrap()).unwrap();
    
    // TypeScript should have classes, interfaces, and types
    let has_classes = symbols.iter().any(|s| s.kind == SymbolKind::Class);
    let has_interfaces = symbols.iter().any(|s| s.kind == SymbolKind::Interface);
    
    assert!(has_classes || has_interfaces, "Should find classes or interfaces in TypeScript");
    
    println!("Found {} TypeScript symbols", symbols.len());
}

#[cfg(feature = "tree-sitter")]
#[test]
fn test_go_symbol_extraction() {
    let mut indexer = SymbolIndexer::new().unwrap();
    let file_path = PathBuf::from("vectortest/analytics_dashboard.go");
    let content = std::fs::read_to_string(&file_path).unwrap();
    
    let symbols = indexer.extract_symbols(&content, "go", file_path.to_str().unwrap()).unwrap();
    
    // Go should have functions and types
    assert!(!symbols.is_empty(), "Should find symbols in Go file");
    
    println!("Found {} Go symbols", symbols.len());
    for symbol in symbols.iter().take(5) {
        println!("  - {} ({:?})", symbol.name, symbol.kind);
    }
}

#[cfg(feature = "tree-sitter")]
#[test]
fn test_java_symbol_extraction() {
    let mut indexer = SymbolIndexer::new().unwrap();
    let file_path = PathBuf::from("vectortest/OrderService.java");
    let content = std::fs::read_to_string(&file_path).unwrap();
    
    let symbols = indexer.extract_symbols(&content, "java", file_path.to_str().unwrap()).unwrap();
    
    // Verify we found the OrderService class
    assert!(symbols.iter().any(|s| s.name == "OrderService" && s.kind == SymbolKind::Class));
    
    // Should have constructor and fields
    let fields: Vec<_> = symbols.iter().filter(|s| s.kind == SymbolKind::Field).collect();
    assert!(!fields.is_empty(), "Should find fields in Java class");
    
    println!("Found {} Java symbols", symbols.len());
}

#[cfg(feature = "tree-sitter")]
#[test]
fn test_cpp_symbol_extraction() {
    let mut indexer = SymbolIndexer::new().unwrap();
    let file_path = PathBuf::from("vectortest/websocket_server.cpp");
    let content = std::fs::read_to_string(&file_path).unwrap();
    
    let symbols = indexer.extract_symbols(&content, "cpp", file_path.to_str().unwrap()).unwrap();
    
    // C++ should have classes and functions
    assert!(!symbols.is_empty(), "Should find symbols in C++ file");
    
    println!("Found {} C++ symbols", symbols.len());
}

#[cfg(feature = "tree-sitter")]
#[test]
fn test_c_symbol_extraction() {
    let mut indexer = SymbolIndexer::new().unwrap();
    
    // Test with a simple C code snippet since we don't have a .c file
    let code = r#"
    #include <stdio.h>
    
    typedef struct Node {
        int data;
        struct Node* next;
    } Node;
    
    int add(int a, int b) {
        return a + b;
    }
    
    void print_message(const char* msg) {
        printf("%s\n", msg);
    }
    
    int main() {
        return 0;
    }
    "#;
    
    let symbols = indexer.extract_symbols(code, "c", "test.c").unwrap();
    
    // Should find struct and functions
    assert!(symbols.iter().any(|s| s.name == "Node" && s.kind == SymbolKind::Struct));
    assert!(symbols.iter().any(|s| s.name == "add" && s.kind == SymbolKind::Function));
    assert!(symbols.iter().any(|s| s.name == "main" && s.kind == SymbolKind::Function));
    
    println!("Found {} C symbols", symbols.len());
}

#[cfg(feature = "tree-sitter")]
#[test]
fn test_symbol_database_operations() {
    let mut indexer = SymbolIndexer::new().unwrap();
    let mut db = SymbolDatabase::new();
    
    // Index multiple files
    let test_files = vec![
        ("vectortest/memory_cache.rs", "rust"),
        ("vectortest/auth_service.py", "python"),
        ("vectortest/user_controller.js", "javascript"),
        ("vectortest/payment_gateway.ts", "typescript"),
    ];
    
    for (file_path, lang) in test_files {
        let content = std::fs::read_to_string(file_path).unwrap();
        let symbols = indexer.extract_symbols(&content, lang, file_path).unwrap();
        db.add_symbols(symbols);
    }
    
    // Test database queries
    assert!(db.total_symbols() > 0, "Database should contain symbols");
    assert!(db.files_indexed() >= 4, "Should have indexed at least 4 files");
    
    // Test finding by kind
    let functions = db.find_by_kind(SymbolKind::Function);
    assert!(!functions.is_empty(), "Should find functions in database");
    
    let classes = db.find_by_kind(SymbolKind::Class);
    assert!(!classes.is_empty(), "Should find classes in database");
    
    println!("Database stats:");
    println!("  Total symbols: {}", db.total_symbols());
    println!("  Files indexed: {}", db.files_indexed());
    println!("  Functions: {}", functions.len());
    println!("  Classes: {}", classes.len());
}

#[cfg(feature = "tree-sitter")]
#[test]
fn test_tsx_symbol_extraction() {
    let mut indexer = SymbolIndexer::new().unwrap();
    let file_path = PathBuf::from("realistic_test/frontend/components/Table.tsx");
    
    if let Ok(content) = std::fs::read_to_string(&file_path) {
        let symbols = indexer.extract_symbols(&content, "tsx", file_path.to_str().unwrap()).unwrap();
        
        // TSX files should have React components
        assert!(!symbols.is_empty(), "Should find symbols in TSX file");
        
        println!("Found {} TSX symbols", symbols.len());
        for symbol in symbols.iter().take(5) {
            println!("  - {} ({:?})", symbol.name, symbol.kind);
        }
    }
}

#[cfg(feature = "tree-sitter")]
#[test]
fn test_realistic_test_directory() {
    let mut indexer = SymbolIndexer::new().unwrap();
    let mut db = SymbolDatabase::new();
    
    // Test files from realistic_test directory
    let test_files = vec![
        ("realistic_test/backend/core/processor.rs", "rust"),
        ("realistic_test/backend/models/model.go", "go"),
        ("realistic_test/backend/services/data_service.js", "javascript"),
        ("realistic_test/backend/utils/helper.py", "python"),
        ("realistic_test/frontend/components/Table.tsx", "tsx"),
        ("realistic_test/legacy/old_code/utils.cpp", "cpp"),
    ];
    
    let mut total_indexed = 0;
    for (file_path, lang) in test_files {
        if let Ok(content) = std::fs::read_to_string(file_path) {
            let symbols = indexer.extract_symbols(&content, lang, file_path).unwrap();
            println!("{}: {} symbols", file_path, symbols.len());
            db.add_symbols(symbols);
            total_indexed += 1;
        }
    }
    
    assert!(total_indexed > 0, "Should index at least some files");
    assert!(db.total_symbols() > 0, "Should extract symbols from realistic test files");
    
    println!("\nRealistic test directory stats:");
    println!("  Files indexed: {}", db.files_indexed());
    println!("  Total symbols: {}", db.total_symbols());
}

#[cfg(feature = "tree-sitter")]
#[test]
fn test_language_detection() {
    use std::path::Path;
    #[cfg(feature = "tree-sitter")]
    use embed_search::search::symbol_index::SymbolIndexer;
    
    // Test file extension detection
    assert_eq!(SymbolIndexer::detect_language(Path::new("test.rs")), Some("rust"));
    assert_eq!(SymbolIndexer::detect_language(Path::new("test.py")), Some("python"));
    assert_eq!(SymbolIndexer::detect_language(Path::new("test.js")), Some("javascript"));
    assert_eq!(SymbolIndexer::detect_language(Path::new("test.ts")), Some("typescript"));
    assert_eq!(SymbolIndexer::detect_language(Path::new("test.tsx")), Some("tsx"));
    assert_eq!(SymbolIndexer::detect_language(Path::new("test.go")), Some("go"));
    assert_eq!(SymbolIndexer::detect_language(Path::new("test.java")), Some("java"));
    assert_eq!(SymbolIndexer::detect_language(Path::new("test.cpp")), Some("cpp"));
    assert_eq!(SymbolIndexer::detect_language(Path::new("test.c")), Some("c"));
    assert_eq!(SymbolIndexer::detect_language(Path::new("test.h")), Some("h"));
    assert_eq!(SymbolIndexer::detect_language(Path::new("test.html")), Some("html"));
    assert_eq!(SymbolIndexer::detect_language(Path::new("test.css")), Some("css"));
    assert_eq!(SymbolIndexer::detect_language(Path::new("test.json")), Some("json"));
    assert_eq!(SymbolIndexer::detect_language(Path::new("test.sh")), Some("sh"));
    
    // Test unknown extension
    assert_eq!(SymbolIndexer::detect_language(Path::new("test.xyz")), None);
}

#[cfg(feature = "tree-sitter")]
#[test]
fn test_html_css_json_extraction() {
    let mut indexer = SymbolIndexer::new().unwrap();
    
    // Test HTML
    let html_code = r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>Test Page</title>
    </head>
    <body>
        <div id="main" class="container">
            <h1>Header</h1>
            <button onclick="handleClick()">Click</button>
        </div>
    </body>
    </html>
    "#;
    
    let html_symbols = indexer.extract_symbols(html_code, "html", "test.html").unwrap();
    assert!(!html_symbols.is_empty(), "Should find HTML tags");
    
    // Test CSS
    let css_code = r#"
    .container {
        width: 100%;
        padding: 20px;
    }
    
    #main {
        background: white;
    }
    
    @media (max-width: 768px) {
        .container {
            padding: 10px;
        }
    }
    "#;
    
    let css_symbols = indexer.extract_symbols(css_code, "css", "test.css").unwrap();
    assert!(!css_symbols.is_empty(), "Should find CSS selectors");
    
    // Test JSON
    let json_code = r#"
    {
        "name": "test-project",
        "version": "1.0.0",
        "dependencies": {
            "react": "^18.0.0",
            "typescript": "^5.0.0"
        }
    }
    "#;
    
    let json_symbols = indexer.extract_symbols(json_code, "json", "test.json").unwrap();
    assert!(!json_symbols.is_empty(), "Should find JSON keys");
    
    println!("HTML symbols: {}", html_symbols.len());
    println!("CSS symbols: {}", css_symbols.len());
    println!("JSON symbols: {}", json_symbols.len());
}

#[cfg(feature = "tree-sitter")]
#[test]
fn test_bash_script_extraction() {
    let mut indexer = SymbolIndexer::new().unwrap();
    
    let bash_code = r#"
    #!/bin/bash
    
    DATABASE_URL="postgresql://localhost/mydb"
    MAX_RETRIES=3
    
    function setup_environment() {
        echo "Setting up environment..."
        export PATH="/usr/local/bin:$PATH"
    }
    
    function run_migrations() {
        local retries=0
        while [ $retries -lt $MAX_RETRIES ]; do
            echo "Attempt $retries"
            retries=$((retries + 1))
        done
    }
    
    setup_environment
    run_migrations
    "#;
    
    let symbols = indexer.extract_symbols(bash_code, "bash", "test.sh").unwrap();
    
    // Should find functions and variables
    assert!(symbols.iter().any(|s| s.name == "setup_environment" && s.kind == SymbolKind::Function));
    assert!(symbols.iter().any(|s| s.name == "run_migrations" && s.kind == SymbolKind::Function));
    assert!(symbols.iter().any(|s| s.name == "DATABASE_URL" && s.kind == SymbolKind::Variable));
    
    println!("Found {} Bash symbols", symbols.len());
}

#[cfg(feature = "tree-sitter")]
#[test]
fn test_symbol_search_performance() {
    use std::time::Instant;
    
    let mut indexer = SymbolIndexer::new().unwrap();
    let mut db = SymbolDatabase::new();
    
    // Index all test files
    let test_files = vec![
        ("vectortest/memory_cache.rs", "rust"),
        ("vectortest/auth_service.py", "python"),
        ("vectortest/user_controller.js", "javascript"),
        ("vectortest/payment_gateway.ts", "typescript"),
        ("vectortest/analytics_dashboard.go", "go"),
        ("vectortest/OrderService.java", "java"),
        ("vectortest/websocket_server.cpp", "cpp"),
    ];
    
    // Measure indexing time
    let start = Instant::now();
    for (file_path, lang) in &test_files {
        if let Ok(content) = std::fs::read_to_string(file_path) {
            let symbols = indexer.extract_symbols(&content, lang, file_path).unwrap();
            db.add_symbols(symbols);
        }
    }
    let indexing_time = start.elapsed();
    
    // Measure search time
    let start = Instant::now();
    let _functions = db.find_by_kind(SymbolKind::Function);
    let search_by_kind_time = start.elapsed();
    
    // Measure lookup time
    let start = Instant::now();
    for _ in 0..1000 {
        let _ = db.find_definition("OrderService");
    }
    let lookup_time = start.elapsed();
    
    println!("\nPerformance metrics:");
    println!("  Indexing {} files: {:?}", test_files.len(), indexing_time);
    println!("  Search by kind: {:?}", search_by_kind_time);
    println!("  1000 lookups: {:?} (avg: {:?})", lookup_time, lookup_time / 1000);
    
    // Performance assertions
    assert!(indexing_time.as_millis() < 1000, "Indexing should be fast");
    assert!(search_by_kind_time.as_micros() < 1000, "Search by kind should be < 1ms");
    assert!(lookup_time.as_micros() / 1000 < 10, "Lookup should be < 10μs on average");
}