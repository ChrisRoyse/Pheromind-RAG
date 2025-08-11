// Tree-sitter based symbol extraction - sophisticated but focused

use anyhow::Result;
use tree_sitter::{Parser, Query, QueryCursor};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub line: usize,
    pub definition: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SymbolKind {
    Function,
    Class,
    Method,
    Variable,
    Constant,
    Module,
    Interface,
    Enum,
    Struct,
}

pub struct SymbolExtractor {
    parsers: HashMap<String, Parser>,
    queries: HashMap<String, Query>,
}

impl SymbolExtractor {
    pub fn new() -> Result<Self> {
        let mut parsers = HashMap::new();
        let mut queries = HashMap::new();
        
        // Initialize Rust parser and query
        let mut rust_parser = Parser::new();
        rust_parser.set_language(tree_sitter_rust::language())?;
        
        let rust_query = Query::new(
            tree_sitter_rust::language(),
            r#"
            (function_item name: (identifier) @function.name)
            (struct_item name: (type_identifier) @struct.name)
            (enum_item name: (type_identifier) @enum.name)
            (impl_item type: (type_identifier) @impl.name)
            (trait_item name: (type_identifier) @trait.name)
            (const_item name: (identifier) @const.name)
            "#
        )?;
        
        parsers.insert("rs".to_string(), rust_parser);
        queries.insert("rs".to_string(), rust_query);
        
        // Initialize Python parser and query
        let mut python_parser = Parser::new();
        python_parser.set_language(tree_sitter_python::language())?;
        
        let python_query = Query::new(
            tree_sitter_python::language(),
            r#"
            (function_definition name: (identifier) @function.name)
            (class_definition name: (identifier) @class.name)
            (assignment left: (identifier) @variable.name)
            "#
        )?;
        
        parsers.insert("py".to_string(), python_parser);
        queries.insert("py".to_string(), python_query);
        
        // Initialize JavaScript/TypeScript parser and query
        let mut js_parser = Parser::new();
        js_parser.set_language(tree_sitter_javascript::language())?;
        
        let js_query = Query::new(
            tree_sitter_javascript::language(),
            r#"
            (function_declaration name: (identifier) @function.name)
            (class_declaration name: (identifier) @class.name)
            (method_definition name: (property_identifier) @method.name)
            (variable_declarator name: (identifier) @variable.name)
            "#
        )?;
        
        // Create separate parser for TypeScript (can't clone Parser)
        let mut ts_parser = Parser::new();
        ts_parser.set_language(tree_sitter_javascript::language())?;
        
        // Create separate query for TypeScript (can't clone Query)
        let ts_query = Query::new(
            tree_sitter_javascript::language(),
            r#"
            (function_declaration name: (identifier) @function.name)
            (class_declaration name: (identifier) @class.name)
            (method_definition name: (property_identifier) @method.name)
            (variable_declarator name: (identifier) @variable.name)
            "#
        )?;
        
        parsers.insert("js".to_string(), js_parser);
        parsers.insert("ts".to_string(), ts_parser);
        queries.insert("js".to_string(), js_query);
        queries.insert("ts".to_string(), ts_query);
        
        Ok(Self { parsers, queries })
    }
    
    /// Extract symbols from source code
    pub fn extract(&mut self, code: &str, extension: &str) -> Result<Vec<Symbol>> {
        let parser = self.parsers.get_mut(extension)
            .ok_or_else(|| anyhow::anyhow!("Unsupported file extension: {}", extension))?;
        
        let query = self.queries.get(extension)
            .ok_or_else(|| anyhow::anyhow!("No query for extension: {}", extension))?;
        
        let tree = parser.parse(code, None)
            .ok_or_else(|| anyhow::anyhow!("Failed to parse code"))?;
        
        let root_node = tree.root_node();
        let mut cursor = QueryCursor::new();
        let matches = cursor.matches(query, root_node, code.as_bytes());
        
        let mut symbols = Vec::new();
        
        for match_ in matches {
            for capture in match_.captures {
                let node = capture.node;
                let name = node.utf8_text(code.as_bytes())?;
                let kind = self.determine_kind(&query.capture_names()[capture.index as usize]);
                
                // Get the full definition line
                let start_byte = node.start_byte();
                let end_byte = self.find_line_end(code, node.end_byte());
                let definition = &code[start_byte..end_byte];
                
                symbols.push(Symbol {
                    name: name.to_string(),
                    kind,
                    line: node.start_position().row + 1,
                    definition: definition.to_string(),
                });
            }
        }
        
        symbols.sort_by_key(|s| s.line);
        Ok(symbols)
    }
    
    fn determine_kind(&self, capture_name: &str) -> SymbolKind {
        match capture_name.split('.').next().unwrap_or("") {
            "function" => SymbolKind::Function,
            "class" => SymbolKind::Class,
            "method" => SymbolKind::Method,
            "struct" => SymbolKind::Struct,
            "enum" => SymbolKind::Enum,
            "trait" | "interface" => SymbolKind::Interface,
            "const" => SymbolKind::Constant,
            "variable" => SymbolKind::Variable,
            _ => SymbolKind::Variable,
        }
    }
    
    fn find_line_end(&self, code: &str, start: usize) -> usize {
        code[start..]
            .find('\n')
            .map(|i| start + i)
            .unwrap_or(code.len())
    }
    
    /// Extract and index symbols for faster searching
    pub fn extract_and_index(&mut self, code: &str, extension: &str, _file_path: &str) -> Result<HashMap<String, Vec<Symbol>>> {
        let symbols = self.extract(code, extension)?;
        let mut index = HashMap::new();
        
        for symbol in symbols {
            index.entry(symbol.name.clone())
                .or_insert_with(Vec::new)
                .push(symbol);
        }
        
        Ok(index)
    }
    
    /// Extract symbols from Rust code
    pub fn extract_rust(&mut self, code: &str) -> Result<Vec<Symbol>> {
        self.extract(code, "rs")
    }
    
    /// Extract symbols from Python code
    pub fn extract_python(&mut self, code: &str) -> Result<Vec<Symbol>> {
        self.extract(code, "py")
    }
    
    /// Extract symbols from JavaScript code
    pub fn extract_javascript(&mut self, code: &str) -> Result<Vec<Symbol>> {
        self.extract(code, "js")
    }
    
    /// Extract symbols from TypeScript code
    pub fn extract_typescript(&mut self, code: &str) -> Result<Vec<Symbol>> {
        self.extract(code, "ts")
    }
}