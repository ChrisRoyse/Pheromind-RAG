// Simple markdown test without tree-sitter dependency
use anyhow::Result;

fn test_simple_markdown() -> Result<()> {
    println!("Testing basic markdown functionality without tree-sitter...");
    
    let markdown_content = r#"# Introduction

This is the introduction section.

## Getting Started

Here's how to get started.

### Prerequisites

- Rust installed
- Git configured

```rust
fn main() {
    println!("Hello, world!");
}
```

## Advanced Topics

More advanced content here.
"#;

    // Simple parsing without tree-sitter
    let lines: Vec<&str> = markdown_content.lines().collect();
    let mut header_count = 0;
    let mut code_block_count = 0;
    let mut list_count = 0;
    
    let mut in_code_block = false;
    
    for line in lines {
        let line = line.trim();
        if line.starts_with("```") {
            if in_code_block {
                code_block_count += 1;
            }
            in_code_block = !in_code_block;
        } else if line.starts_with("#") {
            header_count += 1;
        } else if line.starts_with("-") || line.starts_with("*") {
            list_count += 1;
        }
    }
    
    println!("Found:");
    println!("- Headers: {}", header_count);
    println!("- Code blocks: {}", code_block_count);
    println!("- Lists: {}", list_count);
    
    // Test basic functionality is working
    assert!(header_count > 0, "Should find headers");
    assert!(code_block_count > 0, "Should find code blocks");
    assert!(list_count > 0, "Should find lists");
    
    println!("✓ Basic markdown parsing working correctly!");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_markdown_parsing() -> Result<()> {
        test_simple_markdown()
    }
}

fn main() -> Result<()> {
    test_simple_markdown()
}