// Enhanced metadata extraction for markdown documents
// Implements Phase 4 requirements: enhanced metadata and context extraction

use anyhow::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use crate::chunking::{MarkdownChunk, MarkdownChunkType};

/// Enhanced markdown element with extracted metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkdownElement {
    pub element_type: ElementType,
    pub content: String,
    pub line_number: usize,
    pub level: Option<usize>, // For headers
    pub attributes: HashMap<String, String>,
    pub children: Vec<MarkdownElement>,
}

/// Types of markdown elements we can extract
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ElementType {
    Header,
    Link,
    Image,
    CodeBlock,
    InlineCode,
    List,
    ListItem,
    Table,
    TableRow,
    TableCell,
    Blockquote,
    HorizontalRule,
    Text,
}

/// Enhanced chunk metadata structure with document hierarchy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedChunkMetadata {
    pub chunk_id: String,
    pub content: String,
    pub start_line: usize,
    pub end_line: usize,
    pub chunk_type: MarkdownChunkType,
    
    // Enhanced metadata
    pub symbols: Vec<MarkdownSymbol>,
    pub document_outline: DocumentOutline,
    pub parent_sections: Vec<String>,
    pub child_sections: Vec<String>,
    pub related_chunks: Vec<String>,
    pub extracted_elements: Vec<MarkdownElement>,
    pub context_hints: Vec<String>,
    pub language_hints: Vec<String>, // For code blocks
    pub links: Vec<LinkInfo>,
    pub images: Vec<ImageInfo>,
}

/// Document outline for context preservation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentOutline {
    pub document_title: Option<String>,
    pub current_path: Vec<String>, // Path from root to current section
    pub sibling_sections: Vec<String>,
    pub depth: usize,
    pub section_number: Option<String>,
}

/// Markdown symbols (headers, links, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkdownSymbol {
    pub name: String,
    pub symbol_type: SymbolType,
    pub line: usize,
    pub anchor: Option<String>, // For headers
    pub context: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SymbolType {
    Header(usize), // Level
    Link,
    Image,
    CodeLanguage(String),
    ListItem,
    TableHeader,
    Anchor,
}

/// Link information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkInfo {
    pub text: String,
    pub url: String,
    pub title: Option<String>,
    pub line: usize,
    pub is_internal: bool,
}

/// Image information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageInfo {
    pub alt_text: String,
    pub url: String,
    pub title: Option<String>,
    pub line: usize,
}

/// Enhanced metadata extractor for markdown
pub struct MarkdownMetadataExtractor {
    // Compiled regex patterns for efficient matching
    header_pattern: Regex,
    link_pattern: Regex,
    image_pattern: Regex,
    code_block_pattern: Regex,
    inline_code_pattern: Regex,
    list_item_pattern: Regex,
    table_pattern: Regex,
    
    // Context tracking
    document_outline: DocumentOutline,
    extracted_headers: Vec<(String, usize, usize)>, // (text, level, line)
}

impl MarkdownMetadataExtractor {
    /// Create a new markdown metadata extractor
    pub fn new() -> Result<Self> {
        let header_pattern = Regex::new(r"^(#{1,6})\s+(.+?)(?:\s*\{#([^}]+)\})?\s*$")?;
        let link_pattern = Regex::new(r"\[([^\]]*)\]\(([^)]+)\)")?;
        let image_pattern = Regex::new(r"!\[([^\]]*)\]\(([^)]+)\)")?;
        let code_block_pattern = Regex::new(r"^```(\w+)?\s*$")?;
        let inline_code_pattern = Regex::new(r"`([^`]+)`")?;
        let list_item_pattern = Regex::new(r"^(\s*)[-*+]\s+(.+)$")?;
        let table_pattern = Regex::new(r"^\s*\|(.+)\|\s*$")?;
        
        Ok(Self {
            header_pattern,
            link_pattern,
            image_pattern,
            code_block_pattern,
            inline_code_pattern,
            list_item_pattern,
            table_pattern,
            document_outline: DocumentOutline {
                document_title: None,
                current_path: Vec::new(),
                sibling_sections: Vec::new(),
                depth: 0,
                section_number: None,
            },
            extracted_headers: Vec::new(),
        })
    }
    
    /// Extract enhanced metadata from markdown chunks
    pub fn extract_enhanced_metadata(
        &mut self,
        chunks: Vec<MarkdownChunk>,
        file_path: &str,
    ) -> Result<Vec<EnhancedChunkMetadata>> {
        // First pass: extract document structure
        self.build_document_outline(&chunks)?;
        
        let mut enhanced_chunks = Vec::new();
        
        for (idx, chunk) in chunks.iter().enumerate() {
            let chunk_id = format!("{}#{}", file_path, idx);
            
            // Extract symbols from this chunk
            let symbols = self.extract_symbols(&chunk.content, chunk.start_line)?;
            
            // Extract elements
            let elements = self.extract_elements(&chunk.content, chunk.start_line)?;
            
            // Build document outline for this chunk
            let outline = self.build_chunk_outline(chunk, &chunks, idx)?;
            
            // Find parent and child sections
            let (parent_sections, child_sections) = self.find_section_relationships(chunk, &chunks, idx);
            
            // Find related chunks
            let related_chunks = self.find_related_chunks(chunk, &chunks, idx);
            
            // Extract context hints
            let context_hints = self.extract_context_hints(&chunk.content, &symbols);
            
            // Extract language hints from code blocks
            let language_hints = self.extract_language_hints(&chunk.content);
            
            // Extract links and images
            let links = self.extract_links(&chunk.content, chunk.start_line);
            let images = self.extract_images(&chunk.content, chunk.start_line);
            
            enhanced_chunks.push(EnhancedChunkMetadata {
                chunk_id,
                content: chunk.content.clone(),
                start_line: chunk.start_line,
                end_line: chunk.end_line,
                chunk_type: chunk.chunk_type.clone(),
                symbols,
                document_outline: outline,
                parent_sections,
                child_sections,
                related_chunks,
                extracted_elements: elements,
                context_hints,
                language_hints,
                links,
                images,
            });
        }
        
        Ok(enhanced_chunks)
    }
    
    /// Build document outline from all chunks
    fn build_document_outline(&mut self, chunks: &[MarkdownChunk]) -> Result<()> {
        self.extracted_headers.clear();
        
        // Extract all headers to build outline
        for chunk in chunks {
            let lines: Vec<&str> = chunk.content.lines().collect();
            for (line_idx, line) in lines.iter().enumerate() {
                if let Some(captures) = self.header_pattern.captures(line) {
                    let level = captures.get(1).unwrap().as_str().len();
                    let text = captures.get(2).unwrap().as_str().trim().to_string();
                    let absolute_line = chunk.start_line + line_idx;
                    
                    self.extracted_headers.push((text.clone(), level, absolute_line));
                    
                    // Set document title if this is the first H1
                    if level == 1 && self.document_outline.document_title.is_none() {
                        self.document_outline.document_title = Some(text.clone());
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Extract symbols from markdown content
    pub fn extract_symbols(&self, content: &str, start_line: usize) -> Result<Vec<MarkdownSymbol>> {
        let mut symbols = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        
        for (line_idx, line) in lines.iter().enumerate() {
            let absolute_line = start_line + line_idx;
            
            // Extract headers
            if let Some(captures) = self.header_pattern.captures(line) {
                let level = captures.get(1).unwrap().as_str().len();
                let text = captures.get(2).unwrap().as_str().trim();
                let anchor = captures.get(3).map(|m| m.as_str().to_string());
                
                symbols.push(MarkdownSymbol {
                    name: text.to_string(),
                    symbol_type: SymbolType::Header(level),
                    line: absolute_line,
                    anchor,
                    context: line.to_string(),
                });
            }
            
            // Extract links
            for captures in self.link_pattern.captures_iter(line) {
                let text = captures.get(1).unwrap().as_str();
                symbols.push(MarkdownSymbol {
                    name: text.to_string(),
                    symbol_type: SymbolType::Link,
                    line: absolute_line,
                    anchor: None,
                    context: line.to_string(),
                });
            }
            
            // Extract images
            for captures in self.image_pattern.captures_iter(line) {
                let alt_text = captures.get(1).unwrap().as_str();
                symbols.push(MarkdownSymbol {
                    name: alt_text.to_string(),
                    symbol_type: SymbolType::Image,
                    line: absolute_line,
                    anchor: None,
                    context: line.to_string(),
                });
            }
            
            // Extract code block languages
            if let Some(captures) = self.code_block_pattern.captures(line) {
                if let Some(lang) = captures.get(1) {
                    symbols.push(MarkdownSymbol {
                        name: lang.as_str().to_string(),
                        symbol_type: SymbolType::CodeLanguage(lang.as_str().to_string()),
                        line: absolute_line,
                        anchor: None,
                        context: line.to_string(),
                    });
                }
            }
        }
        
        Ok(symbols)
    }
    
    /// Extract structured elements from markdown content
    pub fn extract_elements(&self, content: &str, start_line: usize) -> Result<Vec<MarkdownElement>> {
        let mut elements = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        
        for (line_idx, line) in lines.iter().enumerate() {
            let absolute_line = start_line + line_idx;
            
            // Headers
            if let Some(captures) = self.header_pattern.captures(line) {
                let level = captures.get(1).unwrap().as_str().len();
                let text = captures.get(2).unwrap().as_str().trim();
                let mut attributes = HashMap::new();
                attributes.insert("level".to_string(), level.to_string());
                
                if let Some(anchor) = captures.get(3) {
                    attributes.insert("anchor".to_string(), anchor.as_str().to_string());
                }
                
                elements.push(MarkdownElement {
                    element_type: ElementType::Header,
                    content: text.to_string(),
                    line_number: absolute_line,
                    level: Some(level),
                    attributes,
                    children: Vec::new(),
                });
            }
            
            // Code blocks
            if self.code_block_pattern.is_match(line) {
                let mut attributes = HashMap::new();
                if let Some(captures) = self.code_block_pattern.captures(line) {
                    if let Some(lang) = captures.get(1) {
                        attributes.insert("language".to_string(), lang.as_str().to_string());
                    }
                }
                
                elements.push(MarkdownElement {
                    element_type: ElementType::CodeBlock,
                    content: line.to_string(),
                    line_number: absolute_line,
                    level: None,
                    attributes,
                    children: Vec::new(),
                });
            }
        }
        
        Ok(elements)
    }
    
    /// Build outline for a specific chunk
    fn build_chunk_outline(
        &self,
        chunk: &MarkdownChunk,
        all_chunks: &[MarkdownChunk],
        chunk_idx: usize,
    ) -> Result<DocumentOutline> {
        // Find current path in document hierarchy
        let mut current_path = Vec::new();
        let mut depth = 0;
        
        // Look backwards through chunks to find parent headers
        for i in (0..chunk_idx).rev() {
            let chunk_content = &all_chunks[i].content;
            for line in chunk_content.lines() {
                if let Some(captures) = self.header_pattern.captures(line) {
                    let level = captures.get(1).unwrap().as_str().len();
                    let text = captures.get(2).unwrap().as_str().trim();
                    
                    // Insert at beginning to maintain order
                    current_path.insert(0, text.to_string());
                    depth = std::cmp::max(depth, level);
                    
                    // Stop at first level 1 header (document root)
                    if level == 1 {
                        break;
                    }
                }
            }
        }
        
        // Find sibling sections at same level
        let sibling_sections = self.find_sibling_sections(chunk_idx, all_chunks);
        
        Ok(DocumentOutline {
            document_title: self.document_outline.document_title.clone(),
            current_path,
            sibling_sections,
            depth,
            section_number: None, // Could be implemented for numbered sections
        })
    }
    
    /// Find sibling sections at the same hierarchy level
    fn find_sibling_sections(&self, chunk_idx: usize, all_chunks: &[MarkdownChunk]) -> Vec<String> {
        let mut siblings = Vec::new();
        
        // This is a simplified implementation
        // In a full implementation, you'd analyze the header hierarchy
        for (idx, chunk) in all_chunks.iter().enumerate() {
            if idx == chunk_idx {
                continue;
            }
            
            if matches!(chunk.chunk_type, MarkdownChunkType::Header) {
                if let Some(first_line) = chunk.content.lines().next() {
                    if let Some(captures) = self.header_pattern.captures(first_line) {
                        let text = captures.get(2).unwrap().as_str().trim();
                        siblings.push(text.to_string());
                    }
                }
            }
        }
        
        siblings
    }
    
    /// Find parent-child relationships between sections
    fn find_section_relationships(
        &self,
        _chunk: &MarkdownChunk,
        all_chunks: &[MarkdownChunk],
        chunk_idx: usize,
    ) -> (Vec<String>, Vec<String>) {
        let mut parent_sections = Vec::new();
        let mut child_sections = Vec::new();
        
        // Find parent sections (higher level headers before this chunk)
        for i in (0..chunk_idx).rev() {
            if matches!(all_chunks[i].chunk_type, MarkdownChunkType::Header) {
                if let Some(first_line) = all_chunks[i].content.lines().next() {
                    if let Some(captures) = self.header_pattern.captures(first_line) {
                        let text = captures.get(2).unwrap().as_str().trim();
                        parent_sections.push(text.to_string());
                    }
                }
            }
        }
        
        // Find child sections (lower level headers after this chunk)
        for i in (chunk_idx + 1)..all_chunks.len() {
            if matches!(all_chunks[i].chunk_type, MarkdownChunkType::Header) {
                if let Some(first_line) = all_chunks[i].content.lines().next() {
                    if let Some(captures) = self.header_pattern.captures(first_line) {
                        let text = captures.get(2).unwrap().as_str().trim();
                        child_sections.push(text.to_string());
                    }
                }
            }
        }
        
        (parent_sections, child_sections)
    }
    
    /// Find related chunks based on content similarity
    fn find_related_chunks(
        &self,
        current_chunk: &MarkdownChunk,
        all_chunks: &[MarkdownChunk],
        current_idx: usize,
    ) -> Vec<String> {
        let mut related = Vec::new();
        
        // Simple implementation: find chunks with similar type or shared keywords
        let current_words: HashSet<String> = current_chunk
            .content
            .split_whitespace()
            .map(|w| w.to_lowercase())
            .collect();
        
        for (idx, chunk) in all_chunks.iter().enumerate() {
            if idx == current_idx {
                continue;
            }
            
            // Same chunk type is related
            if chunk.chunk_type == current_chunk.chunk_type {
                related.push(format!("chunk_{}", idx));
                continue;
            }
            
            // Count shared words
            let chunk_words: HashSet<String> = chunk
                .content
                .split_whitespace()
                .map(|w| w.to_lowercase())
                .collect();
            
            let shared_words = current_words.intersection(&chunk_words).count();
            if shared_words > 3 { // Threshold for relatedness
                related.push(format!("chunk_{}", idx));
            }
        }
        
        related
    }
    
    /// Extract context hints from content and symbols
    fn extract_context_hints(&self, content: &str, symbols: &[MarkdownSymbol]) -> Vec<String> {
        let mut hints = Vec::new();
        
        // Add hints based on symbols
        for symbol in symbols {
            match &symbol.symbol_type {
                SymbolType::Header(level) => {
                    hints.push(format!("header_level_{}", level));
                }
                SymbolType::CodeLanguage(lang) => {
                    hints.push(format!("code_{}", lang));
                }
                SymbolType::Link => {
                    hints.push("contains_links".to_string());
                }
                SymbolType::Image => {
                    hints.push("contains_images".to_string());
                }
                _ => {}
            }
        }
        
        // Add structural hints
        if content.contains("|") && content.contains("-") {
            hints.push("table_content".to_string());
        }
        
        if content.lines().any(|l| l.trim_start().starts_with('-') || l.trim_start().starts_with('*')) {
            hints.push("list_content".to_string());
        }
        
        if content.contains("> ") {
            hints.push("quoted_content".to_string());
        }
        
        hints
    }
    
    /// Extract language hints from code blocks
    pub fn extract_language_hints(&self, content: &str) -> Vec<String> {
        let mut languages = Vec::new();
        
        for line in content.lines() {
            if let Some(captures) = self.code_block_pattern.captures(line) {
                if let Some(lang) = captures.get(1) {
                    languages.push(lang.as_str().to_string());
                }
            }
        }
        
        languages
    }
    
    /// Extract links from content
    pub fn extract_links(&self, content: &str, start_line: usize) -> Vec<LinkInfo> {
        let mut links = Vec::new();
        
        for (line_idx, line) in content.lines().enumerate() {
            for captures in self.link_pattern.captures_iter(line) {
                let text = captures.get(1).unwrap().as_str().to_string();
                let url = captures.get(2).unwrap().as_str().to_string();
                let title = captures.get(3).map(|m| m.as_str().to_string());
                let is_internal = url.starts_with('#') || url.starts_with("./") || url.starts_with("../");
                
                links.push(LinkInfo {
                    text,
                    url,
                    title,
                    line: start_line + line_idx,
                    is_internal,
                });
            }
        }
        
        links
    }
    
    /// Extract images from content
    pub fn extract_images(&self, content: &str, start_line: usize) -> Vec<ImageInfo> {
        let mut images = Vec::new();
        
        for (line_idx, line) in content.lines().enumerate() {
            for captures in self.image_pattern.captures_iter(line) {
                let alt_text = captures.get(1).unwrap().as_str().to_string();
                let url = captures.get(2).unwrap().as_str().to_string();
                let title = captures.get(3).map(|m| m.as_str().to_string());
                
                images.push(ImageInfo {
                    alt_text,
                    url,
                    title,
                    line: start_line + line_idx,
                });
            }
        }
        
        images
    }
    
    /// Implement intelligent boundary detection for better chunking
    pub fn detect_intelligent_boundaries(&self, content: &str) -> Result<Vec<usize>> {
        let mut boundaries = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        
        for (idx, line) in lines.iter().enumerate() {
            // Header boundaries
            if self.header_pattern.is_match(line) {
                boundaries.push(idx);
                continue;
            }
            
            // Code block boundaries
            if self.code_block_pattern.is_match(line) {
                boundaries.push(idx);
                continue;
            }
            
            // Horizontal rule boundaries
            if line.trim().len() >= 3 && (line.trim().chars().all(|c| c == '-') ||
                                       line.trim().chars().all(|c| c == '*') ||
                                       line.trim().chars().all(|c| c == '_')) {
                boundaries.push(idx);
                continue;
            }
            
            // Empty line after content (paragraph boundary)
            if line.trim().is_empty() && idx > 0 && !lines[idx - 1].trim().is_empty() {
                boundaries.push(idx);
            }
        }
        
        Ok(boundaries)
    }
    
    /// Create smart overlaps for related content
    pub fn create_smart_overlaps(
        &self,
        chunks: &[EnhancedChunkMetadata],
        overlap_size: usize,
    ) -> Result<Vec<EnhancedChunkMetadata>> {
        let mut overlapped_chunks = Vec::new();
        
        for (idx, chunk) in chunks.iter().enumerate() {
            let mut extended_chunk = chunk.clone();
            
            // Add context from previous chunk if related
            if idx > 0 && self.chunks_are_related(chunk, &chunks[idx - 1]) {
                let prev_lines: Vec<&str> = chunks[idx - 1].content.lines().collect();
                let context_lines = prev_lines.iter().rev().take(overlap_size).rev();
                let context = context_lines.cloned().collect::<Vec<_>>().join("\n");
                
                extended_chunk.content = format!("{}\n{}", context, chunk.content);
                extended_chunk.context_hints.push("has_previous_context".to_string());
            }
            
            // Add context from next chunk if related
            if idx + 1 < chunks.len() && self.chunks_are_related(chunk, &chunks[idx + 1]) {
                let next_lines: Vec<&str> = chunks[idx + 1].content.lines().collect();
                let context_lines = next_lines.iter().take(overlap_size);
                let context = context_lines.cloned().collect::<Vec<_>>().join("\n");
                
                extended_chunk.content = format!("{}\n{}", chunk.content, context);
                extended_chunk.context_hints.push("has_next_context".to_string());
            }
            
            overlapped_chunks.push(extended_chunk);
        }
        
        Ok(overlapped_chunks)
    }
    
    /// Check if two chunks are related and should have overlapping context
    fn chunks_are_related(&self, chunk1: &EnhancedChunkMetadata, chunk2: &EnhancedChunkMetadata) -> bool {
        // Same chunk type
        if chunk1.chunk_type == chunk2.chunk_type {
            return true;
        }
        
        // Both are headers in same section
        if matches!(chunk1.chunk_type, MarkdownChunkType::Header) && 
           matches!(chunk2.chunk_type, MarkdownChunkType::Header) {
            return true;
        }
        
        // Code blocks with same language
        let chunk1_langs: HashSet<_> = chunk1.language_hints.iter().collect();
        let chunk2_langs: HashSet<_> = chunk2.language_hints.iter().collect();
        if !chunk1_langs.is_empty() && !chunk1_langs.is_disjoint(&chunk2_langs) {
            return true;
        }
        
        // Shared links or images
        let chunk1_links: HashSet<_> = chunk1.links.iter().map(|l| &l.url).collect();
        let chunk2_links: HashSet<_> = chunk2.links.iter().map(|l| &l.url).collect();
        if !chunk1_links.is_empty() && !chunk1_links.is_disjoint(&chunk2_links) {
            return true;
        }
        
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunking::MarkdownRegexChunker;
    
    #[test]
    fn test_symbol_extraction() -> Result<()> {
        let mut extractor = MarkdownMetadataExtractor::new()?;
        
        let content = r#"# Main Header
        
Some text with [link](http://example.com) and ![image](image.png).

```rust
fn main() {
    println!("Hello");
}
```

## Sub Header

- List item 1
- List item 2
"#;
        
        let symbols = extractor.extract_symbols(content, 0)?;
        
        // Should find headers, links, images, and code language
        assert!(symbols.iter().any(|s| matches!(s.symbol_type, SymbolType::Header(1))));
        assert!(symbols.iter().any(|s| matches!(s.symbol_type, SymbolType::Header(2))));
        assert!(symbols.iter().any(|s| matches!(s.symbol_type, SymbolType::Link)));
        assert!(symbols.iter().any(|s| matches!(s.symbol_type, SymbolType::Image)));
        assert!(symbols.iter().any(|s| matches!(s.symbol_type, SymbolType::CodeLanguage(_))));
        
        Ok(())
    }
    
    #[test]
    fn test_enhanced_metadata_extraction() -> Result<()> {
        let chunker = MarkdownRegexChunker::new().map_err(|e| anyhow::anyhow!(e.to_string()))?;
        let mut extractor = MarkdownMetadataExtractor::new()?;
        
        let content = r#"# Document Title

Introduction paragraph.

## Section 1

Content with [external link](http://example.com) and [internal link](#section-2).

```python
def hello():
    print("Hello, world!")
```

## Section 2 {#section-2}

More content here.
"#;
        
        let chunks = chunker.chunk_markdown(content);
        let enhanced = extractor.extract_enhanced_metadata(chunks, "test.md")?;
        
        assert!(!enhanced.is_empty());
        
        // Check that we have document title
        assert!(enhanced.iter().any(|c| c.document_outline.document_title.is_some()));
        
        // Check that we extracted symbols
        assert!(enhanced.iter().any(|c| !c.symbols.is_empty()));
        
        // Check that we found links
        assert!(enhanced.iter().any(|c| !c.links.is_empty()));
        
        // Check language hints
        assert!(enhanced.iter().any(|c| c.language_hints.contains(&"python".to_string())));
        
        Ok(())
    }
    
    #[test]
    fn test_intelligent_boundary_detection() -> Result<()> {
        let extractor = MarkdownMetadataExtractor::new()?;
        
        let content = r#"# Header 1
Content here.

## Header 2
More content.

```rust
code here
```

---

Final section.
"#;
        
        let boundaries = extractor.detect_intelligent_boundaries(content)?;
        
        // Should detect boundaries at headers, code blocks, and horizontal rules
        assert!(boundaries.len() >= 4);
        
        Ok(())
    }
    
    #[test]
    fn test_smart_overlaps() -> Result<()> {
        let chunker = MarkdownRegexChunker::new().map_err(|e| anyhow::anyhow!(e.to_string()))?;
        let mut extractor = MarkdownMetadataExtractor::new()?;
        
        let content = r#"## Section A
Content A.

## Section B  
Content B.
"#;
        
        let chunks = chunker.chunk_markdown(content);
        let enhanced = extractor.extract_enhanced_metadata(chunks, "test.md")?;
        let overlapped = extractor.create_smart_overlaps(&enhanced, 2)?;
        
        assert!(!overlapped.is_empty());
        
        Ok(())
    }
}