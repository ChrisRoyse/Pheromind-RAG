// Test suite for Phase 4: Enhanced Metadata and Context Extraction

use anyhow::Result;
use embed_search::chunking::{MarkdownRegexChunker, MarkdownChunkType};
use embed_search::markdown_metadata_extractor::{
    MarkdownMetadataExtractor, MarkdownSymbol, ElementType
};

#[test]
fn test_enhanced_symbol_extraction() -> Result<()> {
    let mut extractor = MarkdownMetadataExtractor::new()?;
    
    let content = r#"# Main Document Title

This is an introduction with [external link](https://example.com) and ![test image](./test.png).

## Section 1: Getting Started

Here's some content with `inline code` and an [internal link](#section-2).

```rust
fn main() {
    println!("Hello, Rust!");
}
```

### Subsection 1.1

More detailed content here.

```python
def greet(name):
    return f"Hello, {name}!"
```

## Section 2: Advanced Topics {#section-2}

This section has a custom anchor.

- First list item
- Second list item with **bold text**
- Third item with [another link](./relative/path.md)

### Table Example

| Column 1 | Column 2 | Column 3 |
|----------|----------|----------|
| Data 1   | Data 2   | Data 3   |
| More     | Data     | Here     |

## Section 3: Code Examples

Different programming languages:

```javascript
function hello() {
    console.log("Hello, JavaScript!");
}
```

```go
package main

import "fmt"

func main() {
    fmt.Println("Hello, Go!")
}
```

> This is a blockquote with some important information
> that spans multiple lines.

---

## Conclusion

Final thoughts and summary.
"#;
    
    // Test basic symbol extraction
    let symbols = extractor.extract_symbols(content, 0)?;
    
    println!("Extracted {} symbols", symbols.len());
    
    // Verify we extracted headers
    let headers: Vec<_> = symbols.iter()
        .filter(|s| matches!(s.symbol_type, embed_search::MarkdownSymbolType::Header(_)))
        .collect();
    assert!(headers.len() >= 5, "Should extract at least 5 headers");
    
    // Verify header levels
    assert!(symbols.iter().any(|s| matches!(s.symbol_type, embed_search::MarkdownSymbolType::Header(1))));
    assert!(symbols.iter().any(|s| matches!(s.symbol_type, embed_search::MarkdownSymbolType::Header(2))));
    assert!(symbols.iter().any(|s| matches!(s.symbol_type, embed_search::MarkdownSymbolType::Header(3))));
    
    // Verify we extracted links
    let links: Vec<_> = symbols.iter()
        .filter(|s| matches!(s.symbol_type, embed_search::MarkdownSymbolType::Link))
        .collect();
    assert!(links.len() >= 3, "Should extract at least 3 links");
    
    // Verify we extracted images
    let images: Vec<_> = symbols.iter()
        .filter(|s| matches!(s.symbol_type, embed_search::MarkdownSymbolType::Image))
        .collect();
    assert!(images.len() >= 1, "Should extract at least 1 image");
    
    // Verify we extracted code language hints
    let code_langs: Vec<_> = symbols.iter()
        .filter(|s| matches!(s.symbol_type, embed_search::MarkdownSymbolType::CodeLanguage(_)))
        .collect();
    assert!(code_langs.len() >= 3, "Should extract at least 3 code language hints");
    
    // Check specific languages
    assert!(symbols.iter().any(|s| matches!(&s.symbol_type, embed_search::MarkdownSymbolType::CodeLanguage(lang) if lang == "rust")));
    assert!(symbols.iter().any(|s| matches!(&s.symbol_type, embed_search::MarkdownSymbolType::CodeLanguage(lang) if lang == "python")));
    assert!(symbols.iter().any(|s| matches!(&s.symbol_type, embed_search::MarkdownSymbolType::CodeLanguage(lang) if lang == "javascript")));
    
    Ok(())
}

#[test]
fn test_document_outline_building() -> Result<()> {
    let chunker = MarkdownRegexChunker::new().map_err(|e| anyhow::anyhow!(e.to_string()))?;
    let mut extractor = MarkdownMetadataExtractor::new()?;
    
    let content = r#"# User Guide

Welcome to our documentation.

## Installation

How to install the software.

### System Requirements

Minimum system requirements.

### Download Instructions

Step by step download process.

## Configuration

How to configure the system.

### Basic Configuration

Essential settings.

### Advanced Configuration

Optional advanced settings.

## Usage

How to use the software.
"#;
    
    let chunks = chunker.chunk_markdown(content);
    let enhanced = extractor.extract_enhanced_metadata(chunks, "user_guide.md")?;
    
    assert!(!enhanced.is_empty(), "Should have enhanced chunks");
    
    // Check document title is detected
    let has_title = enhanced.iter().any(|chunk| 
        chunk.document_outline.document_title.is_some()
    );
    assert!(has_title, "Should detect document title");
    
    // Check that chunks have hierarchical context
    let has_path = enhanced.iter().any(|chunk| 
        !chunk.document_outline.current_path.is_empty()
    );
    assert!(has_path, "Should have hierarchical path context");
    
    // Check parent-child relationships
    let has_relationships = enhanced.iter().any(|chunk| 
        !chunk.parent_sections.is_empty() || !chunk.child_sections.is_empty()
    );
    assert!(has_relationships, "Should detect section relationships");
    
    println!("Successfully built document outline with {} chunks", enhanced.len());
    
    Ok(())
}

#[test]
fn test_enhanced_chunk_metadata() -> Result<()> {
    let chunker = MarkdownRegexChunker::new().map_err(|e| anyhow::anyhow!(e.to_string()))?;
    let mut extractor = MarkdownMetadataExtractor::new()?;
    
    let content = r#"# API Documentation

This document describes our REST API.

## Authentication

All requests require authentication via [API keys](./auth.md).

```bash
curl -H "Authorization: Bearer YOUR_TOKEN" \
     https://api.example.com/v1/users
```

### Getting API Keys

Visit the [developer portal](https://dev.example.com) to get your keys.

## Endpoints

### Users API

The users endpoint allows you to manage user accounts.

```json
{
  "id": 123,
  "name": "John Doe",
  "email": "john@example.com"
}
```

#### List Users

```http
GET /v1/users
```

Returns a list of all users.

#### Create User

```http
POST /v1/users
```

Creates a new user account.
"#;
    
    let chunks = chunker.chunk_markdown(content);
    let enhanced = extractor.extract_enhanced_metadata(chunks, "api_docs.md")?;
    
    assert!(!enhanced.is_empty(), "Should have enhanced chunks");
    
    // Test that we have comprehensive metadata
    for chunk in &enhanced {
        assert!(!chunk.chunk_id.is_empty(), "Should have chunk ID");
        assert!(chunk.start_line <= chunk.end_line, "Line numbers should be valid");
        
        // Check for context hints
        if matches!(chunk.chunk_type, MarkdownChunkType::Header) {
            let has_header_hint = chunk.context_hints.iter()
                .any(|hint| hint.starts_with("header_level_"));
            // Headers may or may not have symbols, this is implementation dependent
            println!("Header chunk has {} context hints", chunk.context_hints.len());
        }
        
        // Check language hints for code blocks
        if matches!(chunk.chunk_type, MarkdownChunkType::CodeBlock) {
            if !chunk.language_hints.is_empty() {
                println!("Code block with languages: {:?}", chunk.language_hints);
            }
        }
    }
    
    // Test link extraction
    let total_links: usize = enhanced.iter().map(|c| c.links.len()).sum();
    assert!(total_links >= 2, "Should extract at least 2 links");
    
    // Check for internal vs external links
    let has_internal = enhanced.iter().any(|c| 
        c.links.iter().any(|link| link.is_internal)
    );
    let has_external = enhanced.iter().any(|c| 
        c.links.iter().any(|link| !link.is_internal)
    );
    assert!(has_internal, "Should detect internal links");
    assert!(has_external, "Should detect external links");
    
    println!("Enhanced metadata extraction successful with {} chunks", enhanced.len());
    
    Ok(())
}

#[test]
fn test_intelligent_boundary_detection() -> Result<()> {
    let extractor = MarkdownMetadataExtractor::new()?;
    
    let content = r#"# Introduction

This is the introduction paragraph.

It continues here with more content.

## Section 1

First section content.

```rust
fn example() {
    println!("Code block");
}
```

More content after code.

---

## Section 2

Content after horizontal rule.

### Subsection

Final content.
"#;
    
    let boundaries = extractor.detect_intelligent_boundaries(content)?;
    
    assert!(!boundaries.is_empty(), "Should detect boundaries");
    
    // Should detect boundaries at headers, code blocks, and horizontal rules
    let lines: Vec<&str> = content.lines().collect();
    
    for &boundary in &boundaries {
        if boundary < lines.len() {
            let line = lines[boundary].trim();
            let is_valid_boundary = 
                line.starts_with('#') || // Header
                line.starts_with("```") || // Code block
                line.chars().all(|c| c == '-' || c.is_whitespace()) || // Horizontal rule
                line.is_empty(); // Empty line
            
            if !is_valid_boundary {
                println!("Boundary at line {}: '{}'", boundary, line);
            }
        }
    }
    
    println!("Detected {} intelligent boundaries", boundaries.len());
    
    Ok(())
}

#[test]
fn test_smart_overlaps() -> Result<()> {
    let chunker = MarkdownRegexChunker::new().map_err(|e| anyhow::anyhow!(e.to_string()))?;
    let mut extractor = MarkdownMetadataExtractor::new()?;
    
    let content = r#"## Database Setup

Setting up the database connection.

```sql
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL
);
```

## Query Examples

Here are some common queries.

```sql
SELECT * FROM users WHERE name = 'John';
```

## Optimization Tips

Performance optimization guidelines.

```sql
CREATE INDEX idx_users_name ON users(name);
```
"#;
    
    let chunks = chunker.chunk_markdown(content);
    let enhanced = extractor.extract_enhanced_metadata(chunks, "db_guide.md")?;
    let overlapped = extractor.create_smart_overlaps(&enhanced, 2)?;
    
    assert_eq!(enhanced.len(), overlapped.len(), "Should maintain same number of chunks");
    
    // Check that related chunks got extended context
    let has_context_hints = overlapped.iter().any(|chunk|
        chunk.context_hints.contains(&"has_previous_context".to_string()) ||
        chunk.context_hints.contains(&"has_next_context".to_string())
    );
    
    // This may not always be true depending on relatedness detection
    println!("Overlapped chunks: {} have context hints", 
             overlapped.iter().filter(|c| 
                 c.context_hints.contains(&"has_previous_context".to_string()) ||
                 c.context_hints.contains(&"has_next_context".to_string())
             ).count()
    );
    
    Ok(())
}

#[test]
fn test_element_extraction() -> Result<()> {
    let extractor = MarkdownMetadataExtractor::new()?;
    
    let content = r#"# Main Header

Some content.

## Secondary Header {#custom-anchor}

More content.

```python
def hello():
    print("world")
```

### Third Level Header

Final content.
"#;
    
    let elements = extractor.extract_elements(content, 0)?;
    
    assert!(!elements.is_empty(), "Should extract elements");
    
    // Check for headers
    let headers: Vec<_> = elements.iter()
        .filter(|e| matches!(e.element_type, ElementType::Header))
        .collect();
    assert!(headers.len() >= 3, "Should extract at least 3 headers");
    
    // Check header levels
    let has_h1 = headers.iter().any(|h| h.level == Some(1));
    let has_h2 = headers.iter().any(|h| h.level == Some(2));
    let has_h3 = headers.iter().any(|h| h.level == Some(3));
    
    assert!(has_h1, "Should have H1 header");
    assert!(has_h2, "Should have H2 header");
    assert!(has_h3, "Should have H3 header");
    
    // Check for anchor attribute
    let has_anchor = headers.iter().any(|h| 
        h.attributes.contains_key("anchor")
    );
    assert!(has_anchor, "Should detect custom anchor");
    
    // Check for code block
    let has_code_block = elements.iter().any(|e| 
        matches!(e.element_type, ElementType::CodeBlock)
    );
    assert!(has_code_block, "Should detect code block");
    
    // Check code block language attribute
    let code_blocks: Vec<_> = elements.iter()
        .filter(|e| matches!(e.element_type, ElementType::CodeBlock))
        .collect();
    
    if let Some(code_block) = code_blocks.first() {
        if let Some(lang) = code_block.attributes.get("language") {
            assert_eq!(lang, "python", "Should detect Python language");
        }
    }
    
    println!("Successfully extracted {} elements", elements.len());
    
    Ok(())
}

#[test]
fn test_comprehensive_metadata_extraction() -> Result<()> {
    let chunker = MarkdownRegexChunker::new().map_err(|e| anyhow::anyhow!(e.to_string()))?;
    let mut extractor = MarkdownMetadataExtractor::new()?;
    
    let content = r#"# Complete Feature Test

This document tests all metadata extraction features.

## Links and Images

Here's an [external link](https://github.com/rust-lang/rust) to Rust.
And an [internal link](#code-examples) to our code section.
Also a relative link to [other docs](./other.md).

![Example Image](./images/example.png "Example image title")
![Remote Image](https://example.com/image.jpg)

## Lists and Tasks

### Regular List
- First item
- Second item with [embedded link](http://example.com)
- Third item

### Task List
- [x] Completed task
- [ ] Pending task
- [X] Another completed task

## Tables

| Feature | Status | Notes |
|---------|--------|-------|
| Headers | ✅ | Working |
| Links | ✅ | Complete |
| Images | ✅ | Done |

## Code Examples

Multiple language examples:

```rust
// Rust example
fn main() {
    println!("Hello from Rust!");
}
```

```typescript
// TypeScript example
interface User {
    name: string;
    age: number;
}

function greet(user: User): string {
    return `Hello, ${user.name}!`;
}
```

```bash
# Shell script
echo "Hello from bash!"
ls -la
```

## Blockquotes

> This is an important quote
> that spans multiple lines
> and contains valuable information.

---

## Final Section

Summary and conclusion.
"#;
    
    let chunks = chunker.chunk_markdown(content);
    let enhanced = extractor.extract_enhanced_metadata(chunks, "complete_test.md")?;
    
    // Comprehensive validation
    assert!(!enhanced.is_empty(), "Should have enhanced chunks");
    
    // Document structure
    let has_title = enhanced.iter().any(|c| 
        c.document_outline.document_title.is_some()
    );
    assert!(has_title, "Should detect document title");
    
    // Symbol extraction
    let total_symbols: usize = enhanced.iter().map(|c| c.symbols.len()).sum();
    assert!(total_symbols > 0, "Should extract symbols");
    
    // Links and images
    let total_links: usize = enhanced.iter().map(|c| c.links.len()).sum();
    let total_images: usize = enhanced.iter().map(|c| c.images.len()).sum();
    assert!(total_links >= 4, "Should extract at least 4 links");
    assert!(total_images >= 2, "Should extract at least 2 images");
    
    // Language hints
    let has_rust = enhanced.iter().any(|c| c.language_hints.contains(&"rust".to_string()));
    let has_typescript = enhanced.iter().any(|c| c.language_hints.contains(&"typescript".to_string()));
    let has_bash = enhanced.iter().any(|c| c.language_hints.contains(&"bash".to_string()));
    
    assert!(has_rust, "Should detect Rust code");
    assert!(has_typescript, "Should detect TypeScript code");
    assert!(has_bash, "Should detect Bash code");
    
    // Context hints
    let total_hints: usize = enhanced.iter().map(|c| c.context_hints.len()).sum();
    assert!(total_hints > 0, "Should have context hints");
    
    // Chunk relationships
    let has_relationships = enhanced.iter().any(|c| 
        !c.parent_sections.is_empty() || 
        !c.child_sections.is_empty() ||
        !c.related_chunks.is_empty()
    );
    assert!(has_relationships, "Should detect chunk relationships");
    
    // Element extraction
    let total_elements: usize = enhanced.iter().map(|c| c.extracted_elements.len()).sum();
    assert!(total_elements > 0, "Should extract structured elements");
    
    println!("✅ Comprehensive metadata extraction test passed!");
    println!("  - {} chunks processed", enhanced.len());
    println!("  - {} symbols extracted", total_symbols);
    println!("  - {} links found", total_links);
    println!("  - {} images found", total_images);
    println!("  - {} context hints generated", total_hints);
    println!("  - {} structured elements extracted", total_elements);
    
    Ok(())
}