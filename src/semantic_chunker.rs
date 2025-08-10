// AST-based semantic code chunking using Tree-sitter
// This is the REAL implementation for production use

use anyhow::Result;
use tree_sitter::{Parser, Node, Tree, TreeCursor};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct SemanticChunk {
    pub content: String,
    pub file_path: String,
    pub start_line: usize,
    pub end_line: usize,
    pub chunk_type: ChunkType,
    pub symbols: Vec<String>,
    pub parent_context: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ChunkType {
    Function,
    Class,
    Method,
    Module,
    Block,
    Import,
}

pub struct SemanticChunker {
    parsers: HashMap<String, Parser>,
    max_chunk_size: usize, // in characters
    overlap_lines: usize,
}

impl SemanticChunker {
    pub fn new(max_chunk_size: usize) -> Result<Self> {
        let mut parsers = HashMap::new();
        
        // Initialize language parsers
        let mut rust_parser = Parser::new();
        rust_parser.set_language(&tree_sitter_rust::language())?;
        parsers.insert("rs".to_string(), rust_parser);
        
        let mut python_parser = Parser::new();
        python_parser.set_language(&tree_sitter_python::language())?;
        parsers.insert("py".to_string(), python_parser);
        
        let mut js_parser = Parser::new();
        js_parser.set_language(&tree_sitter_javascript::language())?;
        parsers.insert("js".to_string(), js_parser);
        
        // Create separate parser for TypeScript
        let mut ts_parser = Parser::new();
        ts_parser.set_language(&tree_sitter_javascript::language())?;
        parsers.insert("ts".to_string(), ts_parser);
        
        Ok(Self {
            parsers,
            max_chunk_size,
            overlap_lines: 2,
        })
    }
    
    /// Create semantic chunks from code based on AST structure
    pub fn chunk_code(&mut self, code: &str, file_path: &str, extension: &str) -> Result<Vec<SemanticChunk>> {
        let parser = self.parsers.get_mut(extension)
            .ok_or_else(|| anyhow::anyhow!("Unsupported language: {}", extension))?;
        
        let tree = parser.parse(code, None)
            .ok_or_else(|| anyhow::anyhow!("Failed to parse code"))?;
        
        let mut chunks = Vec::new();
        let lines: Vec<&str> = code.lines().collect();
        
        // Extract semantic units based on language
        match extension {
            "rs" => self.chunk_rust(&tree, &lines, file_path, code.as_bytes(), &mut chunks)?,
            "py" => self.chunk_python(&tree, &lines, file_path, code.as_bytes(), &mut chunks)?,
            "js" | "ts" => self.chunk_javascript(&tree, &lines, file_path, code.as_bytes(), &mut chunks)?,
            _ => self.chunk_generic(&tree, &lines, file_path, code.as_bytes(), &mut chunks)?,
        }
        
        // Post-process chunks to handle size constraints
        let processed_chunks = self.post_process_chunks(chunks);
        
        Ok(processed_chunks)
    }
    
    fn chunk_rust(&self, tree: &Tree, lines: &[&str], file_path: &str, source: &[u8], chunks: &mut Vec<SemanticChunk>) -> Result<()> {
        let root = tree.root_node();
        let mut cursor = root.walk();
        
        self.walk_rust_node(&mut cursor, lines, file_path, source, chunks, None)?;
        
        Ok(())
    }
    
    fn walk_rust_node(&self, cursor: &mut TreeCursor, lines: &[&str], file_path: &str, source: &[u8], chunks: &mut Vec<SemanticChunk>, parent: Option<String>) -> Result<()> {
        let node = cursor.node();
        
        match node.kind() {
            "function_item" | "impl_item" | "struct_item" | "enum_item" | "trait_item" | "mod_item" => {
                let chunk = self.create_chunk_from_node(node, lines, file_path, source, parent.clone())?;
                chunks.push(chunk);
                
                // Extract nested items
                if cursor.goto_first_child() {
                    loop {
                        let item_name = self.get_node_name(cursor.node(), source);
                        self.walk_rust_node(cursor, lines, file_path, source, chunks, item_name)?;
                        if !cursor.goto_next_sibling() {
                            break;
                        }
                    }
                    cursor.goto_parent();
                }
            }
            _ => {
                // Continue walking the tree
                if cursor.goto_first_child() {
                    loop {
                        self.walk_rust_node(cursor, lines, file_path, source, chunks, parent.clone())?;
                        if !cursor.goto_next_sibling() {
                            break;
                        }
                    }
                    cursor.goto_parent();
                }
            }
        }
        
        Ok(())
    }
    
    fn chunk_python(&self, tree: &Tree, lines: &[&str], file_path: &str, source: &[u8], chunks: &mut Vec<SemanticChunk>) -> Result<()> {
        let root = tree.root_node();
        let mut cursor = root.walk();
        
        self.walk_python_node(&mut cursor, lines, file_path, source, chunks, None)?;
        
        Ok(())
    }
    
    fn walk_python_node(&self, cursor: &mut TreeCursor, lines: &[&str], file_path: &str, source: &[u8], chunks: &mut Vec<SemanticChunk>, parent: Option<String>) -> Result<()> {
        let node = cursor.node();
        
        match node.kind() {
            "function_definition" | "class_definition" => {
                let chunk = self.create_chunk_from_node(node, lines, file_path, source, parent.clone())?;
                chunks.push(chunk);
                
                // Extract nested items
                if cursor.goto_first_child() {
                    let item_name = self.get_node_name(cursor.node(), source);
                    loop {
                        self.walk_python_node(cursor, lines, file_path, source, chunks, item_name.clone())?;
                        if !cursor.goto_next_sibling() {
                            break;
                        }
                    }
                    cursor.goto_parent();
                }
            }
            _ => {
                // Continue walking
                if cursor.goto_first_child() {
                    loop {
                        self.walk_python_node(cursor, lines, file_path, source, chunks, parent.clone())?;
                        if !cursor.goto_next_sibling() {
                            break;
                        }
                    }
                    cursor.goto_parent();
                }
            }
        }
        
        Ok(())
    }
    
    fn chunk_javascript(&self, tree: &Tree, lines: &[&str], file_path: &str, source: &[u8], chunks: &mut Vec<SemanticChunk>) -> Result<()> {
        let root = tree.root_node();
        let mut cursor = root.walk();
        
        self.walk_javascript_node(&mut cursor, lines, file_path, source, chunks, None)?;
        
        Ok(())
    }
    
    fn walk_javascript_node(&self, cursor: &mut TreeCursor, lines: &[&str], file_path: &str, source: &[u8], chunks: &mut Vec<SemanticChunk>, parent: Option<String>) -> Result<()> {
        let node = cursor.node();
        
        match node.kind() {
            "function_declaration" | "class_declaration" | "method_definition" | "arrow_function" => {
                let chunk = self.create_chunk_from_node(node, lines, file_path, source, parent.clone())?;
                chunks.push(chunk);
                
                // Extract nested items
                if cursor.goto_first_child() {
                    let item_name = self.get_node_name(cursor.node(), source);
                    loop {
                        self.walk_javascript_node(cursor, lines, file_path, source, chunks, item_name.clone())?;
                        if !cursor.goto_next_sibling() {
                            break;
                        }
                    }
                    cursor.goto_parent();
                }
            }
            _ => {
                // Continue walking
                if cursor.goto_first_child() {
                    loop {
                        self.walk_javascript_node(cursor, lines, file_path, source, chunks, parent.clone())?;
                        if !cursor.goto_next_sibling() {
                            break;
                        }
                    }
                    cursor.goto_parent();
                }
            }
        }
        
        Ok(())
    }
    
    fn chunk_generic(&self, _tree: &Tree, lines: &[&str], file_path: &str, _source: &[u8], chunks: &mut Vec<SemanticChunk>) -> Result<()> {
        // Fallback: create line-based chunks
        let mut current_chunk = Vec::new();
        let mut start_line = 0;
        
        for (i, line) in lines.iter().enumerate() {
            current_chunk.push(*line);
            
            let chunk_text = current_chunk.join("\n");
            if chunk_text.len() > self.max_chunk_size {
                // Create chunk
                chunks.push(SemanticChunk {
                    content: chunk_text,
                    file_path: file_path.to_string(),
                    start_line,
                    end_line: i,
                    chunk_type: ChunkType::Block,
                    symbols: vec![],
                    parent_context: None,
                });
                
                // Start new chunk with overlap
                current_chunk.clear();
                start_line = i.saturating_sub(self.overlap_lines);
                for j in start_line..=i {
                    if j < lines.len() {
                        current_chunk.push(lines[j]);
                    }
                }
            }
        }
        
        // Add remaining content
        if !current_chunk.is_empty() {
            chunks.push(SemanticChunk {
                content: current_chunk.join("\n"),
                file_path: file_path.to_string(),
                start_line,
                end_line: lines.len() - 1,
                chunk_type: ChunkType::Block,
                symbols: vec![],
                parent_context: None,
            });
        }
        
        Ok(())
    }
    
    fn create_chunk_from_node(&self, node: Node, lines: &[&str], file_path: &str, source: &[u8], parent: Option<String>) -> Result<SemanticChunk> {
        let start_line = node.start_position().row;
        let end_line = node.end_position().row;
        
        // Extract content with context
        let context_start = start_line.saturating_sub(self.overlap_lines);
        let context_end = (end_line + self.overlap_lines).min(lines.len() - 1);
        
        let content = lines[context_start..=context_end].join("\n");
        
        // Extract symbols
        let symbols = self.extract_symbols_from_node(node, source);
        
        // Determine chunk type
        let chunk_type = match node.kind() {
            "function_item" | "function_declaration" | "function_definition" => ChunkType::Function,
            "class_declaration" | "class_definition" => ChunkType::Class,
            "method_definition" => ChunkType::Method,
            "mod_item" | "module" => ChunkType::Module,
            _ => ChunkType::Block,
        };
        
        Ok(SemanticChunk {
            content,
            file_path: file_path.to_string(),
            start_line,
            end_line,
            chunk_type,
            symbols,
            parent_context: parent,
        })
    }
    
    fn get_node_name(&self, node: Node, source: &[u8]) -> Option<String> {
        // Try to find the name/identifier child
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                if child.kind() == "identifier" || child.kind() == "type_identifier" || child.kind() == "property_identifier" {
                    if let Ok(name) = child.utf8_text(source) {
                        return Some(name.to_string());
                    }
                }
            }
        }
        None
    }
    
    fn extract_symbols_from_node(&self, node: Node, source: &[u8]) -> Vec<String> {
        let mut symbols = Vec::new();
        let mut cursor = node.walk();
        
        self.collect_symbols(&mut cursor, source, &mut symbols);
        
        symbols
    }
    
    fn collect_symbols(&self, cursor: &mut TreeCursor, source: &[u8], symbols: &mut Vec<String>) {
        let node = cursor.node();
        
        if node.kind() == "identifier" || node.kind() == "type_identifier" || node.kind() == "property_identifier" {
            if let Ok(text) = node.utf8_text(source) {
                if !symbols.contains(&text.to_string()) {
                    symbols.push(text.to_string());
                }
            }
        }
        
        if cursor.goto_first_child() {
            loop {
                self.collect_symbols(cursor, source, symbols);
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
            cursor.goto_parent();
        }
    }
    
    fn post_process_chunks(&self, chunks: Vec<SemanticChunk>) -> Vec<SemanticChunk> {
        let mut processed = Vec::new();
        
        for chunk in chunks {
            if chunk.content.len() <= self.max_chunk_size {
                processed.push(chunk);
            } else {
                // Split large chunks while preserving semantic boundaries
                let lines: Vec<&str> = chunk.content.lines().collect();
                let mut current_content = Vec::new();
                let mut current_size = 0;
                let mut chunk_start = chunk.start_line;
                
                for (i, line) in lines.iter().enumerate() {
                    let line_size = line.len() + 1; // +1 for newline
                    
                    if current_size + line_size > self.max_chunk_size && !current_content.is_empty() {
                        // Create a sub-chunk
                        processed.push(SemanticChunk {
                            content: current_content.join("\n"),
                            file_path: chunk.file_path.clone(),
                            start_line: chunk_start,
                            end_line: chunk.start_line + i - 1,
                            chunk_type: chunk.chunk_type.clone(),
                            symbols: chunk.symbols.clone(),
                            parent_context: chunk.parent_context.clone(),
                        });
                        
                        // Start new chunk with overlap
                        current_content.clear();
                        current_size = 0;
                        chunk_start = chunk.start_line + i.saturating_sub(self.overlap_lines);
                        
                        // Add overlap lines
                        for j in (i.saturating_sub(self.overlap_lines))..i {
                            if j < lines.len() {
                                current_content.push(lines[j]);
                                current_size += lines[j].len() + 1;
                            }
                        }
                    }
                    
                    current_content.push(line);
                    current_size += line_size;
                }
                
                // Add remaining content
                if !current_content.is_empty() {
                    processed.push(SemanticChunk {
                        content: current_content.join("\n"),
                        file_path: chunk.file_path.clone(),
                        start_line: chunk_start,
                        end_line: chunk.end_line,
                        chunk_type: chunk.chunk_type,
                        symbols: chunk.symbols,
                        parent_context: chunk.parent_context,
                    });
                }
            }
        }
        
        processed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_rust_chunking() -> Result<()> {
        let mut chunker = SemanticChunker::new(1500)?;
        
        let code = r#"
fn main() {
    println!("Hello, world!");
}

fn process_data(input: &str) -> String {
    input.to_uppercase()
}

struct User {
    name: String,
    age: u32,
}

impl User {
    fn new(name: String, age: u32) -> Self {
        Self { name, age }
    }
}
"#;
        
        let chunks = chunker.chunk_code(code, "test.rs", "rs")?;
        
        assert!(!chunks.is_empty());
        assert!(chunks.iter().any(|c| c.chunk_type == ChunkType::Function));
        assert!(chunks.iter().any(|c| c.symbols.contains(&"main".to_string())));
        
        Ok(())
    }
    
    #[test]
    fn test_python_chunking() -> Result<()> {
        let mut chunker = SemanticChunker::new(1500)?;
        
        let code = r#"
def main():
    print("Hello, world!")

class User:
    def __init__(self, name, age):
        self.name = name
        self.age = age
    
    def greet(self):
        return f"Hello, {self.name}"
"#;
        
        let chunks = chunker.chunk_code(code, "test.py", "py")?;
        
        assert!(!chunks.is_empty());
        assert!(chunks.iter().any(|c| c.chunk_type == ChunkType::Function));
        assert!(chunks.iter().any(|c| c.chunk_type == ChunkType::Class));
        
        Ok(())
    }
}