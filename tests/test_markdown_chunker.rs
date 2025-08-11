// Standalone test for markdown chunking functionality
use std::path::Path;

// Simple test of semantic chunker with just markdown functionality
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing markdown chunking implementation...");
    
    // Test tree-sitter-markdown integration
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(tree_sitter_markdown::language())?;
    
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

| Feature | Status |
|---------|---------|
| Basic   | ✓       |
| Advanced| ✗       |
"#;

    let tree = parser.parse(markdown_content, None)
        .ok_or("Failed to parse markdown")?;
    
    println!("Successfully parsed markdown with {} nodes", tree.root_node().child_count());
    
    // Walk the tree and identify node types
    let mut cursor = tree.root_node().walk();
    let mut header_count = 0;
    let mut code_block_count = 0;
    let mut list_count = 0;
    let mut table_count = 0;
    
    walk_node(&mut cursor, &mut header_count, &mut code_block_count, &mut list_count, &mut table_count);
    
    println!("Found:");
    println!("- Headers: {}", header_count);
    println!("- Code blocks: {}", code_block_count);
    println!("- Lists: {}", list_count);
    println!("- Tables: {}", table_count);
    
    // Test basic functionality is working
    assert!(header_count > 0, "Should find headers");
    assert!(code_block_count > 0, "Should find code blocks");
    assert!(list_count > 0, "Should find lists");
    assert!(table_count > 0, "Should find tables");
    
    println!("✓ Markdown tree-sitter integration working correctly!");
    
    Ok(())
}

fn walk_node(cursor: &mut tree_sitter::TreeCursor, headers: &mut i32, code_blocks: &mut i32, lists: &mut i32, tables: &mut i32) {
    let node = cursor.node();
    
    match node.kind() {
        "atx_heading" => *headers += 1,
        "fenced_code_block" | "indented_code_block" => *code_blocks += 1,
        "list" => *lists += 1,
        "table" => *tables += 1,
        _ => {}
    }
    
    if cursor.goto_first_child() {
        loop {
            walk_node(cursor, headers, code_blocks, lists, tables);
            if !cursor.goto_next_sibling() {
                break;
            }
        }
        cursor.goto_parent();
    }
}