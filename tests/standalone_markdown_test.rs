// Standalone test for markdown metadata extraction functionality
// This test can be run independently to verify the implementation

use embed_search::markdown_metadata_extractor::{MarkdownMetadataExtractor, SymbolType as MarkdownSymbolType};

#[test]
fn test_basic_metadata_extraction() {
    let result = std::panic::catch_unwind(|| {
        let mut extractor = MarkdownMetadataExtractor::new().expect("Failed to create extractor");
        
        let content = r#"# Test Header
This is some content.

## Second Header

Here's a [link](http://example.com) and ![image](test.png).

```rust
fn main() {
    println!("Hello");
}
```

### Third Header

Final content.
"#;
        
        // Test symbol extraction
        let symbols = extractor.extract_symbols(content, 0).expect("Failed to extract symbols");
        
        // Count headers
        let headers: Vec<_> = symbols.iter()
            .filter(|s| matches!(s.symbol_type, MarkdownSymbolType::Header(_)))
            .collect();
        
        println!("Found {} headers", headers.len());
        
        // Should find at least 3 headers
        assert!(headers.len() >= 3, "Should find at least 3 headers");
        
        // Should find different header levels
        let h1_count = symbols.iter()
            .filter(|s| matches!(s.symbol_type, MarkdownSymbolType::Header(1)))
            .count();
        let h2_count = symbols.iter()
            .filter(|s| matches!(s.symbol_type, MarkdownSymbolType::Header(2)))
            .count();
        let h3_count = symbols.iter()
            .filter(|s| matches!(s.symbol_type, MarkdownSymbolType::Header(3)))
            .count();
        
        println!("H1: {}, H2: {}, H3: {}", h1_count, h2_count, h3_count);
        assert!(h1_count >= 1, "Should find at least 1 H1");
        assert!(h2_count >= 1, "Should find at least 1 H2");
        assert!(h3_count >= 1, "Should find at least 1 H3");
        
        // Should find links
        let links: Vec<_> = symbols.iter()
            .filter(|s| matches!(s.symbol_type, MarkdownSymbolType::Link))
            .collect();
        println!("Found {} links", links.len());
        assert!(!links.is_empty(), "Should find links");
        
        // Should find images
        let images: Vec<_> = symbols.iter()
            .filter(|s| matches!(s.symbol_type, MarkdownSymbolType::Image))
            .collect();
        println!("Found {} images", images.len());
        assert!(!images.is_empty(), "Should find images");
        
        // Should find code language
        let code_langs: Vec<_> = symbols.iter()
            .filter(|s| matches!(s.symbol_type, MarkdownSymbolType::CodeLanguage(_)))
            .collect();
        println!("Found {} code language hints", code_langs.len());
        assert!(!code_langs.is_empty(), "Should find code language hints");
        
        // Test boundary detection
        let boundaries = extractor.detect_intelligent_boundaries(content)
            .expect("Failed to detect boundaries");
        println!("Found {} boundaries", boundaries.len());
        assert!(!boundaries.is_empty(), "Should detect boundaries");
        
        println!("✅ Basic metadata extraction test passed!");
    });
    
    match result {
        Ok(()) => println!("Test completed successfully"),
        Err(e) => {
            println!("Test panicked: {:?}", e);
            panic!("Test failed");
        }
    }
}

#[test]
fn test_element_extraction() {
    let result = std::panic::catch_unwind(|| {
        let extractor = MarkdownMetadataExtractor::new().expect("Failed to create extractor");
        
        let content = r#"# Main Header
Some content.

## Sub Header {#custom-id}
More content.

```python
def hello():
    print("world")
```
"#;
        
        let elements = extractor.extract_elements(content, 0)
            .expect("Failed to extract elements");
        
        println!("Found {} elements", elements.len());
        assert!(!elements.is_empty(), "Should extract elements");
        
        // Check for headers with different levels
        let headers: Vec<_> = elements.iter()
            .filter(|e| matches!(e.element_type, embed_search::markdown_metadata_extractor::ElementType::Header))
            .collect();
        
        println!("Found {} header elements", headers.len());
        assert!(headers.len() >= 2, "Should find at least 2 header elements");
        
        // Check header levels
        let h1_elements = headers.iter().filter(|h| h.level == Some(1)).count();
        let h2_elements = headers.iter().filter(|h| h.level == Some(2)).count();
        
        println!("H1 elements: {}, H2 elements: {}", h1_elements, h2_elements);
        assert!(h1_elements >= 1, "Should have H1 element");
        assert!(h2_elements >= 1, "Should have H2 element");
        
        // Check for anchor attribute
        let has_anchor = headers.iter().any(|h| h.attributes.contains_key("anchor"));
        assert!(has_anchor, "Should detect custom anchor");
        
        // Check for code block element
        let code_blocks: Vec<_> = elements.iter()
            .filter(|e| matches!(e.element_type, embed_search::markdown_metadata_extractor::ElementType::CodeBlock))
            .collect();
        
        println!("Found {} code block elements", code_blocks.len());
        assert!(!code_blocks.is_empty(), "Should find code block elements");
        
        // Check code block language attribute
        if let Some(code_block) = code_blocks.first() {
            if let Some(lang) = code_block.attributes.get("language") {
                assert_eq!(lang, "python", "Should detect Python language");
                println!("✅ Detected Python language correctly");
            }
        }
        
        println!("✅ Element extraction test passed!");
    });
    
    match result {
        Ok(()) => println!("Element test completed successfully"),
        Err(e) => {
            println!("Element test panicked: {:?}", e);
            panic!("Element test failed");
        }
    }
}

#[test]
fn test_link_and_image_extraction() {
    let result = std::panic::catch_unwind(|| {
        let extractor = MarkdownMetadataExtractor::new().expect("Failed to create extractor");
        
        let content = r#"# Links and Images Test

Here's an [external link](https://github.com/rust-lang/rust "Rust repository").
And an [internal link](#section-2).
Plus a [relative link](./other.md "Other document").

![Example Image](./images/example.png "Example image title")
![Remote Image](https://example.com/image.jpg)

## Section 2
More content here.
"#;
        
        let links = extractor.extract_links(content, 0);
        let images = extractor.extract_images(content, 0);
        
        println!("Found {} links", links.len());
        println!("Found {} images", images.len());
        
        assert!(links.len() >= 3, "Should extract at least 3 links");
        assert!(images.len() >= 2, "Should extract at least 2 images");
        
        // Test internal vs external link detection
        let internal_links = links.iter().filter(|l| l.is_internal).count();
        let external_links = links.iter().filter(|l| !l.is_internal).count();
        
        println!("Internal links: {}, External links: {}", internal_links, external_links);
        assert!(internal_links > 0, "Should detect internal links");
        assert!(external_links > 0, "Should detect external links");
        
        // Test link titles
        let links_with_titles = links.iter().filter(|l| l.title.is_some()).count();
        println!("Links with titles: {}", links_with_titles);
        assert!(links_with_titles > 0, "Should extract link titles");
        
        // Test image titles
        let images_with_titles = images.iter().filter(|i| i.title.is_some()).count();
        println!("Images with titles: {}", images_with_titles);
        assert!(images_with_titles > 0, "Should extract image titles");
        
        println!("✅ Link and image extraction test passed!");
    });
    
    match result {
        Ok(()) => println!("Link/image test completed successfully"),
        Err(e) => {
            println!("Link/image test panicked: {:?}", e);
            panic!("Link/image test failed");
        }
    }
}

#[test]
fn test_language_hints_extraction() {
    let result = std::panic::catch_unwind(|| {
        let extractor = MarkdownMetadataExtractor::new().expect("Failed to create extractor");
        
        let content = r#"# Code Examples

```rust
fn main() {
    println!("Hello, Rust!");
}
```

```python
def greet(name):
    print(f"Hello, {name}!")
```

```javascript
function hello() {
    console.log("Hello, JavaScript!");
}
```

```bash
echo "Hello from bash!"
ls -la
```

Some regular text here.

```
// Code without language specified
let x = 42;
```
"#;
        
        let language_hints = extractor.extract_language_hints(content);
        
        println!("Found language hints: {:?}", language_hints);
        assert!(language_hints.len() >= 4, "Should extract at least 4 language hints");
        
        // Check for specific languages
        assert!(language_hints.contains(&"rust".to_string()), "Should detect Rust");
        assert!(language_hints.contains(&"python".to_string()), "Should detect Python");
        assert!(language_hints.contains(&"javascript".to_string()), "Should detect JavaScript");
        assert!(language_hints.contains(&"bash".to_string()), "Should detect Bash");
        
        println!("✅ Language hints extraction test passed!");
    });
    
    match result {
        Ok(()) => println!("Language hints test completed successfully"),
        Err(e) => {
            println!("Language hints test panicked: {:?}", e);
            panic!("Language hints test failed");
        }
    }
}