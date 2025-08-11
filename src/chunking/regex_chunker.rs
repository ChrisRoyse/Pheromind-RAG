use regex::Regex;
use std::path::Path;
// Config temporarily removed

// Markdown-specific patterns for structure detection
const MARKDOWN_HEADER_ATX: &[&str] = &[
    r"^#{1,6}\s+.+$",  // ATX headers: # Header, ## Header, etc.
];

const MARKDOWN_HEADER_SETEXT: &[&str] = &[
    r"^.+\n=+\s*$",     // Setext H1: underlined with =
    r"^.+\n-+\s*$",     // Setext H2: underlined with -
];

const MARKDOWN_CODE_BLOCKS: &[&str] = &[
    r"^```[a-zA-Z0-9_+-]*\s*$",  // Fenced code blocks start/end
    r"^~~~[a-zA-Z0-9_+-]*\s*$",  // Alternative fenced code blocks
    r"^    .+$",                 // Indented code blocks (4 spaces)
    r"^\t.+$",                   // Indented code blocks (tab)
];

const MARKDOWN_LISTS: &[&str] = &[
    r"^\s*[-*+]\s+.+$",          // Bullet lists: -, *, +
    r"^\s*\d+\.\s+.+$",         // Numbered lists: 1., 2., etc.
    r"^\s*[-*+]\s*$",            // Empty bullet list items
    r"^\s*\d+\.\s*$",           // Empty numbered list items
];

const MARKDOWN_TABLES: &[&str] = &[
    r"^\s*\|.+\|\s*$",           // Table rows with pipes
    r"^\s*\|?\s*:?-+:?\s*\|?\s*(\|\s*:?-+:?\s*)*\|?\s*$", // Table separator rows
];

const MARKDOWN_BLOCKQUOTES: &[&str] = &[
    r"^\s*>.*$",                 // Blockquotes
];

const MARKDOWN_HORIZONTAL_RULES: &[&str] = &[
    r"^\s*[-*_]{3,}\s*$",        // Horizontal rules: ---, ***, ___
];

const MARKDOWN_TASK_LISTS: &[&str] = &[
    r"^\s*[-*+]\s+\[[ xX]\]\s+.+$", // Task lists: - [x] Done, - [ ] Todo
];

// Language-specific patterns
const FUNCTION_PATTERNS: &[&str] = &[
    r"^\s*(pub|public|private|protected|static|async)?\s*(fn|func|function|def)\s+\w+",  // Rust, Go, Python, JS
    r"^\s*(public|private|protected|static)?\s*\w+\s+\w+\s*\([^)]*\)\s*\{",  // Java, C#, C++
    r"^\s*def\s+\w+\s*\(",  // Python
    r"^\s*(async\s+)?function\s+\w+",  // JavaScript
    r"^\s*func\s+(\(\w+\s+\*?\w+\)\s+)?\w+\s*\(",  // Go
];

const CLASS_PATTERNS: &[&str] = &[
    r"^\s*(pub|public|private|protected)?\s*(class|struct|interface|enum|trait)\s+\w+",
    r"^\s*type\s+\w+\s+(struct|interface)",  // Go
    r"^\s*CREATE\s+TABLE",  // SQL
];

pub struct SimpleRegexChunker {
    function_patterns: Vec<Regex>,
    class_patterns: Vec<Regex>,
    chunk_size_target: usize,
}

impl SimpleRegexChunker {
    /// Create a new regex chunker using configured chunk size
    /// Returns an error if configuration is not properly initialized  
    pub fn new() -> Result<Self, crate::error::EmbedError> {
        // Use default chunk size of 1500 chars
        Self::with_chunk_size(1500)
    }
    
    pub fn with_chunk_size(chunk_size: usize) -> Result<Self, crate::error::EmbedError> {
        let function_patterns = FUNCTION_PATTERNS
            .iter()
            .map(|p| Regex::new(p).map_err(|e| crate::error::EmbedError::Internal {
                message: format!("Invalid regex pattern '{}': {}", p, e),
                backtrace: None,
            }))
            .collect::<Result<Vec<_>, _>>()?;
            
        let class_patterns = CLASS_PATTERNS
            .iter()
            .map(|p| Regex::new(p).map_err(|e| crate::error::EmbedError::Internal {
                message: format!("Invalid regex pattern '{}': {}", p, e),
                backtrace: None,
            }))
            .collect::<Result<Vec<_>, _>>()?;
            
        Ok(Self {
            function_patterns,
            class_patterns,
            chunk_size_target: chunk_size,
        })
    }
    
    pub fn chunk_file(&self, content: &str) -> Vec<Chunk> {
        let lines: Vec<&str> = content.lines().collect();
        let mut chunks = Vec::new();
        let mut current_chunk_lines = Vec::new();
        let mut start_line = 0;
        
        for (i, line) in lines.iter().enumerate() {
            if i > 0 && self.is_chunk_boundary(line) && !current_chunk_lines.is_empty() {
                let chunk_content = self.build_chunk_content(&lines, start_line, i - 1);
                chunks.push(Chunk {
                    content: chunk_content,
                    start_line,
                    end_line: i - 1,
                });
                current_chunk_lines.clear();
                start_line = i;
            }
            
            current_chunk_lines.push(*line);
            
            if current_chunk_lines.len() >= self.chunk_size_target {
                let chunk_content = self.build_chunk_content(&lines, start_line, i);
                chunks.push(Chunk {
                    content: chunk_content,
                    start_line,
                    end_line: i,
                });
                current_chunk_lines.clear();
                start_line = i + 1;
            }
        }
        
        if !current_chunk_lines.is_empty() {
            let end_line = lines.len() - 1;
            let chunk_content = self.build_chunk_content(&lines, start_line, end_line);
            chunks.push(Chunk {
                content: chunk_content,
                start_line,
                end_line,
            });
        }
        
        chunks
    }
    
    /// Build chunk content that exactly matches the original file's line structure
    fn build_chunk_content(&self, lines: &[&str], start_line: usize, end_line: usize) -> String {
        lines[start_line..=end_line].join("\n")
    }
    
    fn is_chunk_boundary(&self, line: &str) -> bool {
        self.function_patterns.iter().any(|p| p.is_match(line)) || 
        self.class_patterns.iter().any(|p| p.is_match(line))
    }
    
    pub fn chunk_file_from_path(&self, path: &Path) -> std::io::Result<Vec<Chunk>> {
        let content = std::fs::read_to_string(path)?;
        Ok(self.chunk_file(&content))
    }
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Chunk {
    pub content: String,
    pub start_line: usize,
    pub end_line: usize,
}

/// Specialized chunker for Markdown files that preserves document structure
pub struct MarkdownRegexChunker {
    header_atx_patterns: Vec<Regex>,
    header_setext_patterns: Vec<Regex>,
    code_block_patterns: Vec<Regex>,
    list_patterns: Vec<Regex>,
    table_patterns: Vec<Regex>,
    blockquote_patterns: Vec<Regex>,
    horizontal_rule_patterns: Vec<Regex>,
    task_list_patterns: Vec<Regex>,
    chunk_size_target: usize,
    preserve_code_blocks: bool,
}

impl MarkdownRegexChunker {
    /// Create a new markdown-specific regex chunker
    pub fn new() -> Result<Self, crate::error::EmbedError> {
        Self::with_options(1500, true)
    }
    
    /// Create a new markdown chunker with custom options
    pub fn with_options(chunk_size: usize, preserve_code_blocks: bool) -> Result<Self, crate::error::EmbedError> {
        let header_atx_patterns = Self::compile_patterns(MARKDOWN_HEADER_ATX)?;
        let header_setext_patterns = Self::compile_patterns(MARKDOWN_HEADER_SETEXT)?;
        let code_block_patterns = Self::compile_patterns(MARKDOWN_CODE_BLOCKS)?;
        let list_patterns = Self::compile_patterns(MARKDOWN_LISTS)?;
        let table_patterns = Self::compile_patterns(MARKDOWN_TABLES)?;
        let blockquote_patterns = Self::compile_patterns(MARKDOWN_BLOCKQUOTES)?;
        let horizontal_rule_patterns = Self::compile_patterns(MARKDOWN_HORIZONTAL_RULES)?;
        let task_list_patterns = Self::compile_patterns(MARKDOWN_TASK_LISTS)?;
        
        Ok(Self {
            header_atx_patterns,
            header_setext_patterns,
            code_block_patterns,
            list_patterns,
            table_patterns,
            blockquote_patterns,
            horizontal_rule_patterns,
            task_list_patterns,
            chunk_size_target: chunk_size,
            preserve_code_blocks,
        })
    }
    
    fn compile_patterns(patterns: &[&str]) -> Result<Vec<Regex>, crate::error::EmbedError> {
        patterns
            .iter()
            .map(|p| Regex::new(p).map_err(|e| crate::error::EmbedError::Internal {
                message: format!("Invalid markdown regex pattern '{}': {}", p, e),
                backtrace: None,
            }))
            .collect::<Result<Vec<_>, _>>()
    }
    
    /// Chunk markdown content preserving document structure
    pub fn chunk_markdown(&self, content: &str) -> Vec<MarkdownChunk> {
        let lines: Vec<&str> = content.lines().collect();
        let mut chunks = Vec::new();
        let mut current_chunk_lines = Vec::new();
        let mut start_line = 0;
        let mut in_code_block = false;
        let mut code_block_fence = None;
        
        for (i, line) in lines.iter().enumerate() {
            // Track code block state
            if let Some(fence_type) = self.detect_code_fence(line) {
                if in_code_block {
                    // Check if this closes the current code block
                    if Some(fence_type) == code_block_fence {
                        in_code_block = false;
                        code_block_fence = None;
                    }
                } else {
                    // Start new code block
                    in_code_block = true;
                    code_block_fence = Some(fence_type);
                }
            }
            
            // Don't break chunks inside code blocks if preserve_code_blocks is true
            let should_break = if self.preserve_code_blocks && in_code_block {
                false
            } else {
                i > 0 && self.is_markdown_boundary(line, i, &lines)
            };
            
            if should_break && !current_chunk_lines.is_empty() {
                let chunk_content = self.build_chunk_content(&lines, start_line, i - 1);
                let chunk_type = self.detect_chunk_type(&lines[start_line..i]);
                chunks.push(MarkdownChunk {
                    content: chunk_content,
                    start_line,
                    end_line: i - 1,
                    chunk_type,
                });
                current_chunk_lines.clear();
                start_line = i;
            }
            
            current_chunk_lines.push(*line);
            
            // Check size limit (character-based)
            let current_content = self.build_chunk_content(&lines, start_line, i);
            if current_content.len() >= self.chunk_size_target && !in_code_block {
                let chunk_type = self.detect_chunk_type(&lines[start_line..=i]);
                chunks.push(MarkdownChunk {
                    content: current_content,
                    start_line,
                    end_line: i,
                    chunk_type,
                });
                current_chunk_lines.clear();
                start_line = i + 1;
            }
        }
        
        // Add final chunk if any content remains
        if !current_chunk_lines.is_empty() {
            let end_line = lines.len() - 1;
            let chunk_content = self.build_chunk_content(&lines, start_line, end_line);
            let chunk_type = self.detect_chunk_type(&lines[start_line..]);
            chunks.push(MarkdownChunk {
                content: chunk_content,
                start_line,
                end_line,
                chunk_type,
            });
        }
        
        chunks
    }
    
    /// Detect if a line represents a code fence and return the fence type
    fn detect_code_fence(&self, line: &str) -> Option<CodeFenceType> {
        if line.trim_start().starts_with("```") {
            Some(CodeFenceType::Backtick)
        } else if line.trim_start().starts_with("~~~") {
            Some(CodeFenceType::Tilde)
        } else {
            None
        }
    }
    
    /// Detect chunk type based on the content
    fn detect_chunk_type(&self, lines: &[&str]) -> MarkdownChunkType {
        if lines.is_empty() {
            return MarkdownChunkType::Text;
        }
        
        let first_line = lines[0];
        
        // Check for headers
        if self.header_atx_patterns.iter().any(|p| p.is_match(first_line)) {
            return MarkdownChunkType::Header;
        }
        
        // Check for Setext headers (need at least 2 lines)
        if lines.len() >= 2 {
            let two_line_content = format!("{}\n{}", lines[0], lines[1]);
            if self.header_setext_patterns.iter().any(|p| p.is_match(&two_line_content)) {
                return MarkdownChunkType::Header;
            }
        }
        
        // Check for code blocks
        if self.code_block_patterns.iter().any(|p| p.is_match(first_line)) {
            return MarkdownChunkType::CodeBlock;
        }
        
        // Check for lists
        if self.list_patterns.iter().any(|p| p.is_match(first_line)) {
            return MarkdownChunkType::List;
        }
        
        // Check for task lists
        if self.task_list_patterns.iter().any(|p| p.is_match(first_line)) {
            return MarkdownChunkType::TaskList;
        }
        
        // Check for tables
        if self.table_patterns.iter().any(|p| p.is_match(first_line)) {
            return MarkdownChunkType::Table;
        }
        
        // Check for blockquotes
        if self.blockquote_patterns.iter().any(|p| p.is_match(first_line)) {
            return MarkdownChunkType::Blockquote;
        }
        
        // Check for horizontal rules
        if self.horizontal_rule_patterns.iter().any(|p| p.is_match(first_line)) {
            return MarkdownChunkType::HorizontalRule;
        }
        
        MarkdownChunkType::Text
    }
    
    /// Build chunk content preserving original formatting
    fn build_chunk_content(&self, lines: &[&str], start_line: usize, end_line: usize) -> String {
        lines[start_line..=end_line].join("\n")
    }
    
    /// Determine if a line represents a markdown boundary
    fn is_markdown_boundary(&self, line: &str, line_index: usize, all_lines: &[&str]) -> bool {
        // ATX headers are always boundaries
        if self.header_atx_patterns.iter().any(|p| p.is_match(line)) {
            return true;
        }
        
        // Check for Setext headers (current line + next line)
        if line_index + 1 < all_lines.len() {
            let two_line_content = format!("{}\n{}", line, all_lines[line_index + 1]);
            if self.header_setext_patterns.iter().any(|p| p.is_match(&two_line_content)) {
                return true;
            }
        }
        
        // Horizontal rules are boundaries
        if self.horizontal_rule_patterns.iter().any(|p| p.is_match(line)) {
            return true;
        }
        
        // Code block fences are boundaries
        if self.code_block_patterns.iter().take(2).any(|p| p.is_match(line)) {
            return true;
        }
        
        false
    }
    
    /// Chunk file from filesystem path
    pub fn chunk_file_from_path(&self, path: &Path) -> std::io::Result<Vec<MarkdownChunk>> {
        let content = std::fs::read_to_string(path)?;
        Ok(self.chunk_markdown(&content))
    }
}

/// Enum representing different types of code fences
#[derive(Clone, Debug, PartialEq)]
enum CodeFenceType {
    Backtick,  // ```
    Tilde,     // ~~~
}

/// Specialized chunk type for markdown with semantic information
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MarkdownChunk {
    pub content: String,
    pub start_line: usize,
    pub end_line: usize,
    pub chunk_type: MarkdownChunkType,
}

/// Enum representing different types of markdown content
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum MarkdownChunkType {
    Header,
    CodeBlock,
    List,
    TaskList,
    Table,
    Blockquote,
    HorizontalRule,
    Text,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Once;

    static INIT: Once = Once::new();

    fn init() {
        INIT.call_once(|| {
            // Config initialization for tests - simplified since we don't have global CONFIG
            // Tests will create their own config instances
        });
    }

    #[test]
    fn test_basic_chunking() {
        init();
        let chunker = SimpleRegexChunker::new().expect("Failed to create chunker");
        let content = "line1\nline2\nfn test() {\n    body\n}\nline6";
        let chunks = chunker.chunk_file(content);
        
        assert!(!chunks.is_empty());
        assert_eq!(chunks[0].start_line, 0);
    }
    
    #[test]
    fn test_chunk_size_limit() {
        init();
        let chunker = SimpleRegexChunker::new().expect("Failed to create chunker");
        let mut content = String::new();
        for i in 0..150 {
            content.push_str(&format!("line {}\n", i));
        }
        
        let chunks = chunker.chunk_file(&content);
        assert!(chunks.len() > 1);
        assert!(chunks[0].content.lines().count() <= 100);
    }
    
    #[test]
    fn test_function_boundary_detection() {
        init();
        let chunker = SimpleRegexChunker::new().expect("Failed to create chunker");
        let content = "// comment\nfn first() {\n}\nfn second() {\n}";
        let chunks = chunker.chunk_file(content);
        
        // The chunker creates chunks at boundaries - this is correct behavior
        // The 3-chunk context expansion happens during search, not during chunking
        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0].content, "// comment");
        assert!(chunks[1].content.contains("fn first"));
        assert!(chunks[2].content.contains("fn second"));
    }
    
    #[test]
    fn test_markdown_atx_headers() {
        let chunker = MarkdownRegexChunker::new().expect("Failed to create markdown chunker");
        let content = "# Header 1\nSome text\n## Header 2\nMore text\n### Header 3\nFinal text";
        let chunks = chunker.chunk_markdown(content);
        
        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0].chunk_type, MarkdownChunkType::Header);
        assert!(chunks[0].content.contains("# Header 1"));
        assert_eq!(chunks[1].chunk_type, MarkdownChunkType::Header);
        assert!(chunks[1].content.contains("## Header 2"));
        assert_eq!(chunks[2].chunk_type, MarkdownChunkType::Header);
        assert!(chunks[2].content.contains("### Header 3"));
    }
    
    #[test]
    fn test_markdown_setext_headers() {
        let chunker = MarkdownRegexChunker::new().expect("Failed to create markdown chunker");
        let content = "Header 1\n========\nSome text\nHeader 2\n--------\nMore text";
        let chunks = chunker.chunk_markdown(content);
        
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0].chunk_type, MarkdownChunkType::Header);
        assert!(chunks[0].content.contains("Header 1"));
        assert!(chunks[0].content.contains("========"));
        assert_eq!(chunks[1].chunk_type, MarkdownChunkType::Header);
        assert!(chunks[1].content.contains("Header 2"));
        assert!(chunks[1].content.contains("--------"));
    }
    
    #[test]
    fn test_markdown_code_blocks() {
        let chunker = MarkdownRegexChunker::new().expect("Failed to create markdown chunker");
        let content = "```rust\nfn main() {\n    println!(\"Hello\");\n}\n```\nSome text after";
        let chunks = chunker.chunk_markdown(content);
        
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0].chunk_type, MarkdownChunkType::CodeBlock);
        assert!(chunks[0].content.contains("```rust"));
        assert!(chunks[0].content.contains("fn main"));
        assert_eq!(chunks[1].chunk_type, MarkdownChunkType::Text);
    }
    
    #[test]
    fn test_markdown_lists() {
        let chunker = MarkdownRegexChunker::new().expect("Failed to create markdown chunker");
        let content = "- Item 1\n- Item 2\n  - Nested item\n1. Numbered item\n2. Another numbered";
        let chunks = chunker.chunk_markdown(content);
        
        assert!(!chunks.is_empty());
        assert_eq!(chunks[0].chunk_type, MarkdownChunkType::List);
        assert!(chunks[0].content.contains("- Item 1"));
    }
    
    #[test]
    fn test_markdown_task_lists() {
        let chunker = MarkdownRegexChunker::new().expect("Failed to create markdown chunker");
        let content = "- [x] Completed task\n- [ ] Incomplete task\n- [X] Another completed";
        let chunks = chunker.chunk_markdown(content);
        
        assert!(!chunks.is_empty());
        assert_eq!(chunks[0].chunk_type, MarkdownChunkType::TaskList);
        assert!(chunks[0].content.contains("- [x] Completed"));
    }
    
    #[test]
    fn test_markdown_tables() {
        let chunker = MarkdownRegexChunker::new().expect("Failed to create markdown chunker");
        let content = "| Column 1 | Column 2 |\n|----------|----------|\n| Data 1   | Data 2   |";
        let chunks = chunker.chunk_markdown(content);
        
        assert!(!chunks.is_empty());
        assert_eq!(chunks[0].chunk_type, MarkdownChunkType::Table);
        assert!(chunks[0].content.contains("| Column 1"));
    }
    
    #[test]
    fn test_markdown_blockquotes() {
        let chunker = MarkdownRegexChunker::new().expect("Failed to create markdown chunker");
        let content = "> This is a quote\n> Continued quote\nNormal text";
        let chunks = chunker.chunk_markdown(content);
        
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0].chunk_type, MarkdownChunkType::Blockquote);
        assert!(chunks[0].content.contains("> This is a quote"));
    }
    
    #[test]
    fn test_markdown_horizontal_rules() {
        let chunker = MarkdownRegexChunker::new().expect("Failed to create markdown chunker");
        let content = "Text before\n---\nText after\n***\nMore text";
        let chunks = chunker.chunk_markdown(content);
        
        assert!(chunks.len() >= 3);
        // Find the horizontal rule chunk
        let hr_chunk = chunks.iter().find(|c| c.chunk_type == MarkdownChunkType::HorizontalRule);
        assert!(hr_chunk.is_some());
    }
    
    #[test]
    fn test_markdown_code_block_preservation() {
        let chunker = MarkdownRegexChunker::with_options(50, true).expect("Failed to create chunker");
        let content = "```\nThis is a very long code block that would normally be split\nbut should be preserved as a single chunk when preserve_code_blocks is true\n```";
        let chunks = chunker.chunk_markdown(content);
        
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].chunk_type, MarkdownChunkType::CodeBlock);
        assert!(chunks[0].content.contains("very long code block"));
    }
    
    #[test]
    fn test_markdown_mixed_content() {
        let chunker = MarkdownRegexChunker::new().expect("Failed to create markdown chunker");
        let content = "# Main Header\nIntro text\n## Sub Header\n- List item 1\n- List item 2\n```code\nSome code\n```\nFinal text";
        let chunks = chunker.chunk_markdown(content);
        
        assert!(chunks.len() >= 4);
        assert_eq!(chunks[0].chunk_type, MarkdownChunkType::Header);
        assert_eq!(chunks[1].chunk_type, MarkdownChunkType::Header);
        assert_eq!(chunks[2].chunk_type, MarkdownChunkType::List);
        assert_eq!(chunks[3].chunk_type, MarkdownChunkType::CodeBlock);
    }
}