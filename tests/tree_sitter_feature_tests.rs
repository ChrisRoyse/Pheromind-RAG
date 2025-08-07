// Level 2: Tree-sitter Feature Tests
// Compile with: cargo test --features tree-sitter
// Memory usage: ~200MB, Runtime: 30-60 seconds

#[cfg(feature = "tree-sitter")]
mod tree_sitter_tests {
    use embed_search::search::{SymbolIndexer, Symbol, SymbolKind};

    #[test]
    fn symbol_extraction_rust() {
        let mut indexer = SymbolIndexer::new().expect("Failed to create SymbolIndexer");
        
        let rust_code = r#"
        pub struct TestStruct {
            field: i32,
        }
        
        impl TestStruct {
            pub fn test_method(&self) -> i32 {
                self.field
            }
        }
        
        pub fn test_function() -> bool {
            true
        }
        "#;
        
        let symbols = indexer.extract_symbols(rust_code, "rust", "test.rs")
            .expect("Should extract symbols");
        
        // Verify we found the expected symbols
        assert!(!symbols.is_empty());
        
        let symbol_names: Vec<&str> = symbols.iter().map(|s| s.name.as_str()).collect();
        assert!(symbol_names.contains(&"TestStruct"));
        assert!(symbol_names.contains(&"test_method"));
        assert!(symbol_names.contains(&"test_function"));
    }

    #[test]
    fn symbol_extraction_javascript() {
        let mut indexer = SymbolIndexer::new().expect("Failed to create SymbolIndexer");
        
        let js_code = r#"
        class UserController {
            constructor() {
                this.users = [];
            }
            
            async getUser(id) {
                return this.users.find(u => u.id === id);
            }
            
            createUser(userData) {
                const user = { ...userData, id: Date.now() };
                this.users.push(user);
                return user;
            }
        }
        
        function validateEmail(email) {
            return email.includes('@');
        }
        "#;
        
        let symbols = indexer.extract_symbols(js_code, "javascript", "controller.js")
            .expect("Should extract symbols");
        
        let symbol_names: Vec<&str> = symbols.iter().map(|s| s.name.as_str()).collect();
        assert!(symbol_names.contains(&"UserController"));
        assert!(symbol_names.contains(&"getUser"));
        assert!(symbol_names.contains(&"validateEmail"));
    }

    #[test]
    fn symbol_extraction_python() {
        let mut indexer = SymbolIndexer::new().expect("Failed to create SymbolIndexer");
        
        let py_code = r#"
        class DataProcessor:
            def __init__(self, config):
                self.config = config
            
            def process_data(self, data):
                return self.transform(data)
            
            def transform(self, data):
                return data.upper()
        
        def utility_function():
            return "helper"
        "#;
        
        let symbols = indexer.extract_symbols(py_code, "python", "processor.py")
            .expect("Should extract symbols");
        
        let symbol_names: Vec<&str> = symbols.iter().map(|s| s.name.as_str()).collect();
        assert!(symbol_names.contains(&"DataProcessor"));
        assert!(symbol_names.contains(&"process_data"));
        assert!(symbol_names.contains(&"utility_function"));
    }

    #[test]
    fn symbol_database_operations() {
        let mut db = embed_search::search::SymbolDatabase::new();
        
        let test_symbols = vec![
            Symbol {
                name: "TestClass".to_string(),
                kind: SymbolKind::Class,
                file_path: "test.rs".to_string(),
                line_start: 1,
                line_end: 1,
                signature: Some("class TestClass".to_string()),
                parent: None,
            },
            Symbol {
                name: "test_method".to_string(),
                kind: SymbolKind::Function,
                file_path: "test.rs".to_string(),
                line_start: 5,
                line_end: 5,
                signature: Some("fn test_method()".to_string()),
                parent: Some("TestClass".to_string()),
            },
        ];
        
        // Add symbols
        db.add_symbols(test_symbols.clone());
        
        // Search for symbols
        let result = db.find_definition("TestClass");
        assert!(result.is_some());
        
        let references = db.find_all_references("test_method");
        assert!(!references.is_empty());
    }
}

#[cfg(not(feature = "tree-sitter"))]
mod tree_sitter_disabled {
    #[test]
    fn tree_sitter_feature_disabled() {
        // This test runs when tree-sitter feature is disabled
        // Ensures we handle missing features gracefully
        println!("Tree-sitter feature tests skipped (feature not enabled)");
    }
}