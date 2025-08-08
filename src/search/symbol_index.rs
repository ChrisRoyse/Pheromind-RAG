use rustc_hash::FxHashMap;
use std::path::Path;
use anyhow::Result;
use tree_sitter::{Parser, Query, QueryCursor, Node};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub file_path: String,
    pub line_start: usize,
    pub line_end: usize,
    pub signature: Option<String>,
    pub parent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SymbolKind {
    Function,
    Method,
    Class,
    Struct,
    Interface,
    Enum,
    Variable,
    Constant,
    Type,
    Module,
    Namespace,
    Property,
    Field,
    Constructor,
    Parameter,
    TypeParameter,
    Label,
    Tag,  // For HTML/XML
    Selector,  // For CSS
    Key,  // For JSON/YAML/TOML
}

pub struct SymbolIndexer {
    parsers: FxHashMap<String, Parser>,
    queries: FxHashMap<String, Query>,
}

impl SymbolIndexer {
    pub fn new() -> Result<Self> {
        let mut parsers = FxHashMap::default();
        let mut queries = FxHashMap::default();
        
        // Initialize each language with error logging
        if let Err(e) = Self::init_rust(&mut parsers, &mut queries) {
            eprintln!("Warning: Failed to initialize Rust parser: {}", e);
        }
        
        if let Err(e) = Self::init_python(&mut parsers, &mut queries) {
            eprintln!("Warning: Failed to initialize Python parser: {}", e);
        }
        
        if let Err(e) = Self::init_javascript(&mut parsers, &mut queries) {
            eprintln!("Warning: Failed to initialize JavaScript parser: {}", e);
        }
        
        if let Err(e) = Self::init_typescript(&mut parsers, &mut queries) {
            eprintln!("Warning: Failed to initialize TypeScript parser: {}", e);
        }
        
        if let Err(e) = Self::init_go(&mut parsers, &mut queries) {
            eprintln!("Warning: Failed to initialize Go parser: {}", e);
        }
        
        if let Err(e) = Self::init_java(&mut parsers, &mut queries) {
            eprintln!("Warning: Failed to initialize Java parser: {}", e);
        }
        
        if let Err(e) = Self::init_c(&mut parsers, &mut queries) {
            eprintln!("Warning: Failed to initialize C parser: {}", e);
        }
        
        if let Err(e) = Self::init_cpp(&mut parsers, &mut queries) {
            eprintln!("Warning: Failed to initialize C++ parser: {}", e);
        }
        
        if let Err(e) = Self::init_html(&mut parsers, &mut queries) {
            eprintln!("Warning: Failed to initialize HTML parser: {}", e);
        }
        
        if let Err(e) = Self::init_css(&mut parsers, &mut queries) {
            eprintln!("Warning: Failed to initialize CSS parser: {}", e);
        }
        
        if let Err(e) = Self::init_json(&mut parsers, &mut queries) {
            eprintln!("Warning: Failed to initialize JSON parser: {}", e);
        }
        
        if let Err(e) = Self::init_bash(&mut parsers, &mut queries) {
            eprintln!("Warning: Failed to initialize Bash parser: {}", e);
        }
        
        Ok(Self { parsers, queries })
    }
    
    fn init_rust(parsers: &mut FxHashMap<String, Parser>, queries: &mut FxHashMap<String, Query>) -> Result<()> {
        let lang = tree_sitter_rust::LANGUAGE.into();
        let mut parser = Parser::new();
        parser.set_language(&lang)?;
        parsers.insert("rust".to_string(), parser);
        
        let query = Query::new(
            &lang,
            r#"
            (function_item name: (identifier) @function.name)
            (struct_item name: (type_identifier) @struct.name)
            (enum_item name: (type_identifier) @enum.name)
            (impl_item type: (type_identifier) @impl.type)
            (const_item name: (identifier) @const.name)
            (static_item name: (identifier) @static.name)
            (trait_item name: (type_identifier) @trait.name)
            (mod_item name: (identifier) @module.name)
            (macro_definition name: (identifier) @macro.name)
            (type_item name: (type_identifier) @type.name)
            (field_declaration name: (field_identifier) @field.name)
            "#
        )?;
        queries.insert("rust".to_string(), query);
        Ok(())
    }
    
    fn init_python(parsers: &mut FxHashMap<String, Parser>, queries: &mut FxHashMap<String, Query>) -> Result<()> {
        let lang = tree_sitter_python::LANGUAGE.into();
        let mut parser = Parser::new();
        parser.set_language(&lang)?;
        parsers.insert("python".to_string(), parser);
        
        let query = Query::new(
            &lang,
            r#"
            (function_definition name: (identifier) @function.name)
            (class_definition name: (identifier) @class.name)
            (decorated_definition) @decorator
            (assignment left: (identifier) @variable.name)
            (import_statement) @import
            (import_from_statement) @import
            "#
        )?;
        queries.insert("python".to_string(), query);
        Ok(())
    }
    
    fn init_javascript(parsers: &mut FxHashMap<String, Parser>, queries: &mut FxHashMap<String, Query>) -> Result<()> {
        let lang = tree_sitter_javascript::LANGUAGE.into();
        let mut parser = Parser::new();
        parser.set_language(&lang)?;
        parsers.insert("javascript".to_string(), parser);
        parsers.insert("js".to_string(), {
            let mut p = Parser::new();
            p.set_language(&lang)?;
            p
        });
        
        let query_str = r#"
            (function_declaration name: (identifier) @function.name)
            (class_declaration name: (identifier) @class.name)
            (method_definition name: (property_identifier) @method.name)
            (variable_declarator name: (identifier) @variable.name)
            (arrow_function) @arrow
            (generator_function_declaration name: (identifier) @generator.name)
            (lexical_declaration) @declaration
            "#;
        
        queries.insert("javascript".to_string(), Query::new(&lang, query_str)?);
        queries.insert("js".to_string(), Query::new(&lang, query_str)?);
        Ok(())
    }
    
    fn init_typescript(parsers: &mut FxHashMap<String, Parser>, queries: &mut FxHashMap<String, Query>) -> Result<()> {
        let lang = tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into();
        let mut parser = Parser::new();
        parser.set_language(&lang)?;
        parsers.insert("typescript".to_string(), parser);
        parsers.insert("ts".to_string(), {
            let mut p = Parser::new();
            p.set_language(&lang)?;
            p
        });
        
        // TypeScript TSX
        let tsx_lang = tree_sitter_typescript::LANGUAGE_TSX.into();
        let mut tsx_parser = Parser::new();
        tsx_parser.set_language(&tsx_lang)?;
        parsers.insert("tsx".to_string(), tsx_parser);
        
        let query_str = r#"
            (function_declaration name: (identifier) @function.name)
            (class_declaration name: (type_identifier) @class.name)
            (interface_declaration name: (type_identifier) @interface.name)
            (enum_declaration name: (identifier) @enum.name)
            (type_alias_declaration name: (type_identifier) @type.name)
            (method_signature name: (property_identifier) @method.name)
            "#;
        
        queries.insert("typescript".to_string(), Query::new(&lang, query_str)?);
        queries.insert("ts".to_string(), Query::new(&lang, query_str)?);
        queries.insert("tsx".to_string(), Query::new(&tsx_lang, query_str)?);
        Ok(())
    }
    
    fn init_go(parsers: &mut FxHashMap<String, Parser>, queries: &mut FxHashMap<String, Query>) -> Result<()> {
        let lang = tree_sitter_go::LANGUAGE.into();
        let mut parser = Parser::new();
        parser.set_language(&lang)?;
        parsers.insert("go".to_string(), parser);
        
        let query = Query::new(
            &lang,
            r#"
            (function_declaration name: (identifier) @function.name)
            (method_declaration name: (field_identifier) @method.name)
            (type_declaration (type_spec name: (type_identifier) @type.name))
            (const_declaration (const_spec name: (identifier) @const.name))
            (var_declaration (var_spec name: (identifier) @variable.name))
            (package_clause (package_identifier) @package.name)
            "#
        )?;
        queries.insert("go".to_string(), query);
        Ok(())
    }
    
    fn init_java(parsers: &mut FxHashMap<String, Parser>, queries: &mut FxHashMap<String, Query>) -> Result<()> {
        let lang = tree_sitter_java::LANGUAGE.into();
        let mut parser = Parser::new();
        parser.set_language(&lang)?;
        parsers.insert("java".to_string(), parser);
        
        let query = Query::new(
            &lang,
            r#"
            (class_declaration name: (identifier) @class.name)
            (interface_declaration name: (identifier) @interface.name)
            (enum_declaration name: (identifier) @enum.name)
            (method_declaration name: (identifier) @method.name)
            (constructor_declaration name: (identifier) @constructor.name)
            (field_declaration declarator: (variable_declarator name: (identifier) @field.name))
            (package_declaration (scoped_identifier) @package)
            (import_declaration) @import
            "#
        )?;
        queries.insert("java".to_string(), query);
        Ok(())
    }
    
    fn init_c(parsers: &mut FxHashMap<String, Parser>, queries: &mut FxHashMap<String, Query>) -> Result<()> {
        let lang = tree_sitter_c::LANGUAGE.into();
        let mut parser = Parser::new();
        parser.set_language(&lang)?;
        parsers.insert("c".to_string(), parser);
        parsers.insert("h".to_string(), {
            let mut p = Parser::new();
            p.set_language(&lang)?;
            p
        });
        
        let query_str = r#"
            (function_definition declarator: (function_declarator declarator: (identifier) @function.name))
            (declaration declarator: (function_declarator declarator: (identifier) @function.declaration))
            (struct_specifier name: (type_identifier) @struct.name)
            (enum_specifier name: (type_identifier) @enum.name)
            (type_definition declarator: (type_identifier) @typedef)
            (declaration (init_declarator declarator: (identifier) @variable.name))
            "#;
        
        queries.insert("c".to_string(), Query::new(&lang, query_str)?);
        queries.insert("h".to_string(), Query::new(&lang, query_str)?);
        Ok(())
    }
    
    fn init_cpp(parsers: &mut FxHashMap<String, Parser>, queries: &mut FxHashMap<String, Query>) -> Result<()> {
        let lang = tree_sitter_cpp::LANGUAGE.into();
        let mut parser = Parser::new();
        parser.set_language(&lang)?;
        parsers.insert("cpp".to_string(), parser);
        parsers.insert("cc".to_string(), {
            let mut p = Parser::new();
            p.set_language(&lang)?;
            p
        });
        parsers.insert("cxx".to_string(), {
            let mut p = Parser::new();
            p.set_language(&lang)?;
            p
        });
        parsers.insert("hpp".to_string(), {
            let mut p = Parser::new();
            p.set_language(&lang)?;
            p
        });
        
        let query_str = r#"
            (function_definition declarator: (function_declarator declarator: (identifier) @function.name))
            (class_specifier name: (type_identifier) @class.name)
            (struct_specifier name: (type_identifier) @struct.name)
            (enum_specifier name: (type_identifier) @enum.name)
            (template_declaration) @template
            "#;
        
        queries.insert("cpp".to_string(), Query::new(&lang, query_str)?);
        queries.insert("cc".to_string(), Query::new(&lang, query_str)?);
        queries.insert("cxx".to_string(), Query::new(&lang, query_str)?);
        queries.insert("hpp".to_string(), Query::new(&lang, query_str)?);
        Ok(())
    }
    
    fn init_html(parsers: &mut FxHashMap<String, Parser>, queries: &mut FxHashMap<String, Query>) -> Result<()> {
        let lang = tree_sitter_html::LANGUAGE.into();
        let mut parser = Parser::new();
        parser.set_language(&lang)?;
        parsers.insert("html".to_string(), parser);
        parsers.insert("htm".to_string(), {
            let mut p = Parser::new();
            p.set_language(&lang)?;
            p
        });
        
        let query_str = r#"
            (element (start_tag (tag_name) @tag.name))
            (element (self_closing_tag (tag_name) @tag.name))
            (attribute (attribute_name) @attribute.name)
            "#;
        
        queries.insert("html".to_string(), Query::new(&lang, query_str)?);
        queries.insert("htm".to_string(), Query::new(&lang, query_str)?);
        Ok(())
    }
    
    fn init_css(parsers: &mut FxHashMap<String, Parser>, queries: &mut FxHashMap<String, Query>) -> Result<()> {
        let lang = tree_sitter_css::LANGUAGE.into();
        let mut parser = Parser::new();
        parser.set_language(&lang)?;
        parsers.insert("css".to_string(), parser);
        parsers.insert("scss".to_string(), {
            let mut p = Parser::new();
            p.set_language(&lang)?;
            p
        });
        
        let query_str = r#"
            (rule_set (selectors) @selector)
            (class_selector) @class.selector
            (id_selector) @id.selector
            (media_statement) @media
            (keyframes_statement (keyframes_name) @keyframes.name)
            "#;
        
        queries.insert("css".to_string(), Query::new(&lang, query_str)?);
        queries.insert("scss".to_string(), Query::new(&lang, query_str)?);
        Ok(())
    }
    
    fn init_json(parsers: &mut FxHashMap<String, Parser>, queries: &mut FxHashMap<String, Query>) -> Result<()> {
        let lang = tree_sitter_json::LANGUAGE.into();
        let mut parser = Parser::new();
        parser.set_language(&lang)?;
        parsers.insert("json".to_string(), parser);
        
        let query = Query::new(
            &lang,
            r#"
            (pair key: (string (string_content) @key.name))
            "#
        )?;
        queries.insert("json".to_string(), query);
        Ok(())
    }
    
    fn init_bash(parsers: &mut FxHashMap<String, Parser>, queries: &mut FxHashMap<String, Query>) -> Result<()> {
        let lang = tree_sitter_bash::LANGUAGE.into();
        let mut parser = Parser::new();
        parser.set_language(&lang)?;
        parsers.insert("bash".to_string(), parser);
        parsers.insert("sh".to_string(), {
            let mut p = Parser::new();
            p.set_language(&lang)?;
            p
        });
        
        let query_str = r#"
            (function_definition name: (word) @function.name)
            (variable_assignment name: (variable_name) @variable.name)
            "#;
        
        queries.insert("bash".to_string(), Query::new(&lang, query_str)?);
        queries.insert("sh".to_string(), Query::new(&lang, query_str)?);
        Ok(())
    }
    
    pub fn extract_symbols(&mut self, code: &str, language: &str, file_path: &str) -> Result<Vec<Symbol>> {
        let lang = language.to_lowercase();
        
        let parser = self.parsers.get_mut(&lang)
            .ok_or_else(|| anyhow::anyhow!("Unsupported language: {}", language))?;
        
        let tree = parser.parse(code, None)
            .ok_or_else(|| anyhow::anyhow!("Failed to parse code"))?;
        
        let query = self.queries.get(&lang)
            .ok_or_else(|| anyhow::anyhow!("No query for language: {}", language))?;
        
        let mut cursor = QueryCursor::new();
        let matches = cursor.matches(query, tree.root_node(), code.as_bytes());
        
        let mut symbols = Vec::new();
        let mut seen = std::collections::HashSet::new();
        
        for match_ in matches {
            for capture in match_.captures {
                let node = capture.node;
                let capture_name = query.capture_names()[capture.index as usize];
                
                // Only process name captures (skip the parent captures)
                if !capture_name.contains(".name") && !capture_name.contains(".selector") {
                    continue;
                }
                
                if let Some(symbol) = self.node_to_symbol(node, capture_name, code, file_path) {
                    // Deduplicate symbols
                    let key = format!("{}:{}:{}", symbol.file_path, symbol.name, symbol.line_start);
                    if seen.insert(key) {
                        symbols.push(symbol);
                    }
                }
            }
        }
        
        Ok(symbols)
    }
    
    fn node_to_symbol(&self, node: Node, capture_name: &str, code: &str, file_path: &str) -> Option<Symbol> {
        let name = match node.utf8_text(code.as_bytes()) {
            Ok(text) => text.to_string(),
            Err(e) => {
                log::error!("Failed to extract UTF-8 text from node at byte range {:?}: {}", node.byte_range(), e);
                return None;
            }
        };
        
        // Skip empty or invalid names
        if name.is_empty() || name.contains('@') {
            return None;
        }
        
        let kind = match capture_name {
            s if s.contains("function") => SymbolKind::Function,
            s if s.contains("method") => SymbolKind::Method,
            s if s.contains("class") => SymbolKind::Class,
            s if s.contains("struct") => SymbolKind::Struct,
            s if s.contains("interface") => SymbolKind::Interface,
            s if s.contains("enum") => SymbolKind::Enum,
            s if s.contains("const") | s.contains("static") => SymbolKind::Constant,
            s if s.contains("type") | s.contains("trait") | s.contains("typedef") => SymbolKind::Type,
            s if s.contains("variable") | s.contains("var") => SymbolKind::Variable,
            s if s.contains("module") | s.contains("package") => SymbolKind::Module,
            s if s.contains("namespace") => SymbolKind::Namespace,
            s if s.contains("property") => SymbolKind::Property,
            s if s.contains("field") => SymbolKind::Field,
            s if s.contains("constructor") => SymbolKind::Constructor,
            s if s.contains("parameter") => SymbolKind::Parameter,
            s if s.contains("tag") => SymbolKind::Tag,
            s if s.contains("selector") => SymbolKind::Selector,
            // More specific matches must come before more general ones
            s if s.contains("keyframes") => SymbolKind::Function,
            s if s.contains("generator") => SymbolKind::Function,
            s if s.contains("arrow") => SymbolKind::Function,
            s if s.contains("macro") => SymbolKind::Function,
            s if s.contains("key") => SymbolKind::Key,  // Must come after keyframes
            _ => return None,
        };
        
        Some(Symbol {
            name,
            kind,
            file_path: file_path.to_string(),
            line_start: node.start_position().row + 1,
            line_end: node.end_position().row + 1,
            signature: None,
            parent: None,
        })
    }
    
    pub fn detect_language(path: &Path) -> Option<&'static str> {
        match path.extension()?.to_str()? {
            "rs" => Some("rust"),
            "py" => Some("python"),
            "js" | "mjs" | "cjs" => Some("javascript"),
            "ts" => Some("typescript"),
            "tsx" => Some("tsx"),
            "go" => Some("go"),
            "java" => Some("java"),
            "c" => Some("c"),
            "h" => Some("h"),
            "cpp" | "cc" | "cxx" | "c++" => Some("cpp"),
            "hpp" | "hh" | "hxx" | "h++" => Some("hpp"),
            "html" | "htm" => Some("html"),
            "css" => Some("css"),
            "scss" | "sass" => Some("scss"),
            "json" => Some("json"),
            "sh" => Some("sh"),
            "bash" => Some("bash"),
            _ => None,
        }
    }
}

// Quick symbol database for fast lookups
pub struct SymbolDatabase {
    pub symbols_by_name: FxHashMap<String, Vec<Symbol>>,
    pub symbols_by_file: FxHashMap<String, Vec<Symbol>>,
    pub symbols_by_kind: FxHashMap<SymbolKind, Vec<Symbol>>,
}

impl SymbolDatabase {
    pub fn new() -> Self {
        Self {
            symbols_by_name: FxHashMap::default(),
            symbols_by_file: FxHashMap::default(),
            symbols_by_kind: FxHashMap::default(),
        }
    }
    
    pub fn add_symbols(&mut self, symbols: Vec<Symbol>) {
        for symbol in symbols {
            // Index by name
            self.symbols_by_name
                .entry(symbol.name.clone())
                .or_insert_with(Vec::new)
                .push(symbol.clone());
            
            // Index by file
            self.symbols_by_file
                .entry(symbol.file_path.clone())
                .or_insert_with(Vec::new)
                .push(symbol.clone());
            
            // Index by kind
            self.symbols_by_kind
                .entry(symbol.kind.clone())
                .or_insert_with(Vec::new)
                .push(symbol);
        }
    }
    
    pub fn find_definition(&self, name: &str) -> Option<Symbol> {
        self.symbols_by_name.get(name)?.first().cloned()
    }
    
    pub fn find_all_references(&self, name: &str) -> Vec<Symbol> {
        match self.symbols_by_name.get(name) {
            Some(symbols) => symbols.iter().cloned().collect(),
            None => {
                // Symbol name not found in index - legitimate empty result
                Vec::new()
            }
        }
    }
    
    pub fn clear(&mut self) {
        self.symbols_by_name.clear();
        self.symbols_by_file.clear();
        self.symbols_by_kind.clear();
    }
    
    /// Remove all symbols from a specific file
    pub fn remove_file_symbols(&mut self, file_path: &str) {
        // Get symbols that need to be removed from this file
        let symbols_to_remove = match self.symbols_by_file.remove(file_path) {
            Some(symbols) => symbols,
            None => return, // File not indexed, nothing to remove
        };
        
        // Remove symbols from name-based index
        for symbol in &symbols_to_remove {
            if let Some(name_symbols) = self.symbols_by_name.get_mut(&symbol.name) {
                name_symbols.retain(|s| s.file_path != file_path);
                // Remove the name entry if no symbols left
                if name_symbols.is_empty() {
                    self.symbols_by_name.remove(&symbol.name);
                }
            }
        }
        
        // Remove symbols from kind-based index
        for symbol in &symbols_to_remove {
            if let Some(kind_symbols) = self.symbols_by_kind.get_mut(&symbol.kind) {
                kind_symbols.retain(|s| s.file_path != file_path);
                // Remove the kind entry if no symbols left
                if kind_symbols.is_empty() {
                    self.symbols_by_kind.remove(&symbol.kind);
                }
            }
        }
    }
    
    pub fn find_by_kind(&self, kind: SymbolKind) -> Vec<Symbol> {
        match self.symbols_by_kind.get(&kind) {
            Some(symbols) => symbols.iter().cloned().collect(),
            None => {
                // Symbol kind not found in index - legitimate empty result
                Vec::new()
            }
        }
    }
    
    pub fn symbols_in_file(&self, file_path: &str) -> Vec<&Symbol> {
        match self.symbols_by_file.get(file_path) {
            Some(symbols) => symbols.iter().collect(),
            None => {
                // File not found in index - legitimate empty result
                Vec::new()
            }
        }
    }
    
    pub fn total_symbols(&self) -> usize {
        self.symbols_by_file.values().map(|v| v.len()).sum()
    }
    
    pub fn files_indexed(&self) -> usize {
        self.symbols_by_file.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_rust_symbol_extraction() {
        let code = r#"
        pub struct UserAuth {
            username: String,
        }
        
        impl UserAuth {
            pub fn validate(&self) -> bool {
                true
            }
        }
        
        fn main() {
            println!("Hello");
        }
        "#;
        
        let mut indexer = SymbolIndexer::new().unwrap();
        let symbols = indexer.extract_symbols(code, "rust", "test.rs").unwrap();
        
        assert!(symbols.iter().any(|s| s.name == "UserAuth" && s.kind == SymbolKind::Struct));
        assert!(symbols.iter().any(|s| s.name == "main" && s.kind == SymbolKind::Function));
    }
    
    #[test]
    fn test_python_symbol_extraction() {
        let code = r#"
class AuthManager:
    def __init__(self):
        pass
    
    def validate_user(self, username, password):
        return True

def main():
    manager = AuthManager()
    print("Starting")
        "#;
        
        let mut indexer = SymbolIndexer::new().unwrap();
        let symbols = indexer.extract_symbols(code, "python", "test.py").unwrap();
        
        assert!(symbols.iter().any(|s| s.name == "AuthManager" && s.kind == SymbolKind::Class));
        assert!(symbols.iter().any(|s| s.name == "main" && s.kind == SymbolKind::Function));
    }
    
    #[test]
    fn test_java_symbol_extraction() {
        let code = r#"
        public class UserService {
            private String name;
            
            public UserService(String name) {
                this.name = name;
            }
            
            public void processUser() {
                System.out.println("Processing");
            }
        }
        "#;
        
        let mut indexer = SymbolIndexer::new().unwrap();
        let symbols = indexer.extract_symbols(code, "java", "UserService.java").unwrap();
        
        assert!(symbols.iter().any(|s| s.name == "UserService" && s.kind == SymbolKind::Class));
        assert!(symbols.iter().any(|s| s.name == "processUser" && s.kind == SymbolKind::Method));
    }
    
    #[test]
    fn test_c_symbol_extraction() {
        let code = r#"
        typedef struct {
            int x;
            int y;
        } Point;
        
        int add(int a, int b) {
            return a + b;
        }
        
        int main() {
            return 0;
        }
        "#;
        
        let mut indexer = SymbolIndexer::new().unwrap();
        let symbols = indexer.extract_symbols(code, "c", "test.c").unwrap();
        
        assert!(symbols.iter().any(|s| s.name == "add" && s.kind == SymbolKind::Function));
        assert!(symbols.iter().any(|s| s.name == "main" && s.kind == SymbolKind::Function));
    }
}