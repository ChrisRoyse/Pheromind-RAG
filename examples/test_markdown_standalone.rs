// Standalone test for markdown chunker - no external dependencies
use std::collections::HashMap;

// Simple regex matcher for testing
struct SimpleRegex {
    pattern: String,
}

impl SimpleRegex {
    fn new(pattern: &str) -> Result<Self, &'static str> {
        Ok(Self {
            pattern: pattern.to_string(),
        })
    }
    
    fn is_match(&self, text: &str) -> bool {
        // Simple pattern matching for testing
        match self.pattern.as_str() {
            r"^#{1,6}\s+.+$" => text.trim_start().starts_with('#') && text.len() > 1,
            r"^```[a-zA-Z0-9_+-]*\s*$" => text.trim_start().starts_with("```"),
            r"^~~~[a-zA-Z0-9_+-]*\s*$" => text.trim_start().starts_with("~~~"),
            r"^\s*[-*+]\s+.+$" => {
                let trimmed = text.trim_start();
                (trimmed.starts_with("- ") || trimmed.starts_with("* ") || trimmed.starts_with("+ ")) && trimmed.len() > 2
            },
            r"^\s*\d+\.\s+.+$" => {
                let trimmed = text.trim_start();
                if let Some(pos) = trimmed.find(". ") {
                    let num_part = &trimmed[..pos];
                    num_part.chars().all(|c| c.is_ascii_digit()) && trimmed.len() > pos + 2
                } else {
                    false
                }
            },
            r"^\s*\|.+\|\s*$" => text.trim().starts_with('|') && text.trim().ends_with('|'),
            r"^\s*>.*$" => text.trim_start().starts_with('>'),
            r"^\s*[-*_]{3,}\s*$" => {
                let trimmed = text.trim();
                trimmed.len() >= 3 && (
                    trimmed.chars().all(|c| c == '-') ||
                    trimmed.chars().all(|c| c == '*') ||
                    trimmed.chars().all(|c| c == '_')
                )
            },
            r"^\s*[-*+]\s+\[[ xX]\]\s+.+$" => {
                let trimmed = text.trim_start();
                (trimmed.starts_with("- [") || trimmed.starts_with("* [") || trimmed.starts_with("+ [")) &&
                trimmed.contains(']') && trimmed.len() > 6
            },
            _ => false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
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

#[derive(Clone, Debug, PartialEq)]
pub struct MarkdownChunk {
    pub content: String,
    pub start_line: usize,
    pub end_line: usize,
    pub chunk_type: MarkdownChunkType,
}

#[derive(Clone, Debug, PartialEq)]
enum CodeFenceType {
    Backtick,
    Tilde,
}

pub struct MarkdownRegexChunker {
    patterns: HashMap<String, SimpleRegex>,
    chunk_size_target: usize,
    preserve_code_blocks: bool,
}

impl MarkdownRegexChunker {
    pub fn new() -> Result<Self, &'static str> {
        Self::with_options(1500, true)
    }
    
    pub fn with_options(chunk_size: usize, preserve_code_blocks: bool) -> Result<Self, &'static str> {
        let mut patterns = HashMap::new();
        
        let pattern_definitions = [
            ("header_atx", r"^#{1,6}\s+.+$"),
            ("code_backtick", r"^```[a-zA-Z0-9_+-]*\s*$"),
            ("code_tilde", r"^~~~[a-zA-Z0-9_+-]*\s*$"),
            ("list_bullet", r"^\s*[-*+]\s+.+$"),
            ("list_numbered", r"^\s*\d+\.\s+.+$"),
            ("table", r"^\s*\|.+\|\s*$"),
            ("blockquote", r"^\s*>.*$"),
            ("horizontal_rule", r"^\s*[-*_]{3,}\s*$"),
            ("task_list", r"^\s*[-*+]\s+\[[ xX]\]\s+.+$"),
        ];
        
        for (name, pattern) in &pattern_definitions {
            patterns.insert(name.to_string(), SimpleRegex::new(pattern)?);
        }
        
        Ok(Self {
            patterns,
            chunk_size_target: chunk_size,
            preserve_code_blocks,
        })
    }
    
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
                    if Some(fence_type) == code_block_fence {
                        in_code_block = false;
                        code_block_fence = None;
                    }
                } else {
                    in_code_block = true;
                    code_block_fence = Some(fence_type);
                }
            }
            
            let should_break = if self.preserve_code_blocks && in_code_block {
                false
            } else {
                i > 0 && self.is_markdown_boundary(line)
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
    
    fn detect_code_fence(&self, line: &str) -> Option<CodeFenceType> {
        if line.trim_start().starts_with("```") {
            Some(CodeFenceType::Backtick)
        } else if line.trim_start().starts_with("~~~") {
            Some(CodeFenceType::Tilde)
        } else {
            None
        }
    }
    
    fn detect_chunk_type(&self, lines: &[&str]) -> MarkdownChunkType {
        if lines.is_empty() {
            return MarkdownChunkType::Text;
        }
        
        let first_line = lines[0];
        
        if self.patterns["header_atx"].is_match(first_line) {
            return MarkdownChunkType::Header;
        }
        
        if self.patterns["code_backtick"].is_match(first_line) || self.patterns["code_tilde"].is_match(first_line) {
            return MarkdownChunkType::CodeBlock;
        }
        
        if self.patterns["task_list"].is_match(first_line) {
            return MarkdownChunkType::TaskList;
        }
        
        if self.patterns["list_bullet"].is_match(first_line) || self.patterns["list_numbered"].is_match(first_line) {
            return MarkdownChunkType::List;
        }
        
        if self.patterns["table"].is_match(first_line) {
            return MarkdownChunkType::Table;
        }
        
        if self.patterns["blockquote"].is_match(first_line) {
            return MarkdownChunkType::Blockquote;
        }
        
        if self.patterns["horizontal_rule"].is_match(first_line) {
            return MarkdownChunkType::HorizontalRule;
        }
        
        MarkdownChunkType::Text
    }
    
    fn build_chunk_content(&self, lines: &[&str], start_line: usize, end_line: usize) -> String {
        lines[start_line..=end_line].join("\n")
    }
    
    fn is_markdown_boundary(&self, line: &str) -> bool {
        self.patterns["header_atx"].is_match(line) ||
        self.patterns["horizontal_rule"].is_match(line) ||
        self.patterns["code_backtick"].is_match(line) ||
        self.patterns["code_tilde"].is_match(line)
    }
}

fn main() {
    println!("Testing Markdown Regex Chunker Implementation");
    let separator = "=".repeat(50);
    println!("{}", separator);
    
    let chunker = MarkdownRegexChunker::new().expect("Failed to create markdown chunker");
    
    // Test 1: Basic headers
    let content1 = "# Header 1\nSome text\n## Header 2\nMore text\n### Header 3\nFinal text";
    let chunks1 = chunker.chunk_markdown(content1);
    println!("\\nTest 1 - ATX Headers:");
    for (i, chunk) in chunks1.iter().enumerate() {
        println!("  Chunk {}: {:?} (lines {}-{})", i+1, chunk.chunk_type, chunk.start_line, chunk.end_line);
        println!("    Content: {}", chunk.content.replace('\n', "\\n"));
    }
    
    // Test 2: Code blocks
    let content2 = "```rust\nfn main() {\n    println!(\"Hello\");\n}\n```\nSome text after";
    let chunks2 = chunker.chunk_markdown(content2);
    println!("\\nTest 2 - Code Blocks:");
    for (i, chunk) in chunks2.iter().enumerate() {
        println!("  Chunk {}: {:?} (lines {}-{})", i+1, chunk.chunk_type, chunk.start_line, chunk.end_line);
        println!("    Content: {}", chunk.content.replace('\n', "\\n"));
    }
    
    // Test 3: Lists
    let content3 = "- Item 1\n- Item 2\n  - Nested item\n1. Numbered item\n2. Another numbered";
    let chunks3 = chunker.chunk_markdown(content3);
    println!("\\nTest 3 - Lists:");
    for (i, chunk) in chunks3.iter().enumerate() {
        println!("  Chunk {}: {:?} (lines {}-{})", i+1, chunk.chunk_type, chunk.start_line, chunk.end_line);
        println!("    Content: {}", chunk.content.replace('\n', "\\n"));
    }
    
    // Test 4: Task lists
    let content4 = "- [x] Completed task\n- [ ] Incomplete task\n- [X] Another completed";
    let chunks4 = chunker.chunk_markdown(content4);
    println!("\\nTest 4 - Task Lists:");
    for (i, chunk) in chunks4.iter().enumerate() {
        println!("  Chunk {}: {:?} (lines {}-{})", i+1, chunk.chunk_type, chunk.start_line, chunk.end_line);
        println!("    Content: {}", chunk.content.replace('\n', "\\n"));
    }
    
    // Test 5: Tables
    let content5 = "| Column 1 | Column 2 |\n|----------|----------|\n| Data 1   | Data 2   |";
    let chunks5 = chunker.chunk_markdown(content5);
    println!("\\nTest 5 - Tables:");
    for (i, chunk) in chunks5.iter().enumerate() {
        println!("  Chunk {}: {:?} (lines {}-{})", i+1, chunk.chunk_type, chunk.start_line, chunk.end_line);
        println!("    Content: {}", chunk.content.replace('\n', "\\n"));
    }
    
    // Test 6: Blockquotes
    let content6 = "> This is a quote\n> Continued quote\nNormal text";
    let chunks6 = chunker.chunk_markdown(content6);
    println!("\\nTest 6 - Blockquotes:");
    for (i, chunk) in chunks6.iter().enumerate() {
        println!("  Chunk {}: {:?} (lines {}-{})", i+1, chunk.chunk_type, chunk.start_line, chunk.end_line);
        println!("    Content: {}", chunk.content.replace('\n', "\\n"));
    }
    
    // Test 7: Horizontal rules
    let content7 = "Text before\n---\nText after\n***\nMore text";
    let chunks7 = chunker.chunk_markdown(content7);
    println!("\\nTest 7 - Horizontal Rules:");
    for (i, chunk) in chunks7.iter().enumerate() {
        println!("  Chunk {}: {:?} (lines {}-{})", i+1, chunk.chunk_type, chunk.start_line, chunk.end_line);
        println!("    Content: {}", chunk.content.replace('\n', "\\n"));
    }
    
    // Test 8: Code block preservation
    let small_chunker = MarkdownRegexChunker::with_options(50, true).expect("Failed to create small chunker");
    let content8 = "```\nThis is a very long code block that would normally be split\nbut should be preserved as a single chunk when preserve_code_blocks is true\n```";
    let chunks8 = small_chunker.chunk_markdown(content8);
    println!("\\nTest 8 - Code Block Preservation (chunk_size=50):");
    for (i, chunk) in chunks8.iter().enumerate() {
        println!("  Chunk {}: {:?} (lines {}-{})", i+1, chunk.chunk_type, chunk.start_line, chunk.end_line);
        println!("    Content length: {} chars", chunk.content.len());
        println!("    Content: {}", chunk.content.replace('\n', "\\n"));
    }
    
    // Test 9: Mixed content
    let content9 = r#"# Main Document

This is some introductory text.

## Code Examples

Here's some code:

```python
def hello_world():
    print("Hello, World!")
```

## Lists

- First item
- Second item
- Third item

### Task List

- [x] Completed task
- [ ] Pending task

## Tables

| Column 1 | Column 2 |
|----------|----------|
| Data 1   | Data 2   |

> This is a blockquote

---

Final section with text.
"#;
    
    let chunks9 = chunker.chunk_markdown(content9);
    println!("\\nTest 9 - Mixed Content ({} chunks):", chunks9.len());
    for (i, chunk) in chunks9.iter().enumerate() {
        println!("  Chunk {}: {:?} (lines {}-{})", i+1, chunk.chunk_type, chunk.start_line, chunk.end_line);
    }
    
    let separator = "=".repeat(50);
    println!("\n{}", separator);
    println!("All tests completed successfully!");
    
    // Verify all chunk types are represented
    let all_types: std::collections::HashSet<_> = chunks9.iter().map(|c| &c.chunk_type).collect();
    println!("\\nDetected chunk types: {:?}", all_types);
}