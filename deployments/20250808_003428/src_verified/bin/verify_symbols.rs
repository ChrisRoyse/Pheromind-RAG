#[cfg(feature = "tree-sitter")]
use embed_search::search::SymbolIndexer;
#[cfg(feature = "tree-sitter")]
use std::path::PathBuf;

#[cfg(feature = "tree-sitter")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Symbol Extraction Verification ===\n");
    
    println!("Creating SymbolIndexer...");
    let mut indexer = match SymbolIndexer::new() {
        Ok(i) => {
            println!("✓ SymbolIndexer created\n");
            i
        }
        Err(e) => {
            println!("✗ Failed to create SymbolIndexer: {}", e);
            return Ok(());
        }
    };
    
    // Test Rust file
    println!("Testing: vectortest/memory_cache.rs");
    println!("{}", "-".repeat(40));
    let rust_file = PathBuf::from("vectortest/memory_cache.rs");
    match std::fs::read_to_string(&rust_file) {
        Ok(content) => {
            println!("File size: {} bytes", content.len());
            
            let file_path_str = rust_file.to_str()
                .ok_or_else(|| format!("Path contains invalid UTF-8 characters: {:?}", rust_file))?;
            
            match indexer.extract_symbols(&content, "rust", file_path_str) {
                Ok(symbols) => {
                    println!("Symbols extracted: {}", symbols.len());
                    
                    if symbols.is_empty() {
                        println!("⚠️ WARNING: No symbols were extracted!");
                    } else {
                        println!("\nSymbols found:");
                        for symbol in symbols.iter().take(20) {
                            println!("  • {} ({:?}) at line {}", 
                                     symbol.name, symbol.kind, symbol.line_start);
                        }
                        if symbols.len() > 20 {
                            println!("  ... and {} more", symbols.len() - 20);
                        }
                    }
                }
                Err(e) => println!("✗ Extraction failed: {}", e),
            }
        }
        Err(e) => println!("✗ File read failed: {}", e),
    }
    
    println!("\n");
    
    // Test Python file
    println!("Testing: vectortest/auth_service.py");
    println!("{}", "-".repeat(40));
    let python_file = PathBuf::from("vectortest/auth_service.py");
    match std::fs::read_to_string(&python_file) {
        Ok(content) => {
            println!("File size: {} bytes", content.len());
            
            let file_path_str = python_file.to_str()
                .ok_or_else(|| format!("Path contains invalid UTF-8 characters: {:?}", python_file))?;
            
            match indexer.extract_symbols(&content, "python", file_path_str) {
                Ok(symbols) => {
                    println!("Symbols extracted: {}", symbols.len());
                    
                    if symbols.is_empty() {
                        println!("⚠️ WARNING: No symbols were extracted!");
                    } else {
                        println!("\nSymbols found:");
                        for symbol in symbols.iter().take(20) {
                            println!("  • {} ({:?}) at line {}", 
                                     symbol.name, symbol.kind, symbol.line_start);
                        }
                        if symbols.len() > 20 {
                            println!("  ... and {} more", symbols.len() - 20);
                        }
                    }
                }
                Err(e) => println!("✗ Extraction failed: {}", e),
            }
        }
        Err(e) => println!("✗ File read failed: {}", e),
    }
    
    println!("\n");
    
    // Test Java file  
    println!("Testing: vectortest/OrderService.java");
    println!("{}", "-".repeat(40));
    let java_file = PathBuf::from("vectortest/OrderService.java");
    match std::fs::read_to_string(&java_file) {
        Ok(content) => {
            println!("File size: {} bytes", content.len());
            
            let file_path_str = java_file.to_str()
                .ok_or_else(|| format!("Path contains invalid UTF-8 characters: {:?}", java_file))?;
            
            match indexer.extract_symbols(&content, "java", file_path_str) {
                Ok(symbols) => {
                    println!("Symbols extracted: {}", symbols.len());
                    
                    if symbols.is_empty() {
                        println!("⚠️ WARNING: No symbols were extracted!");  
                    } else {
                        println!("\nSymbols found:");
                        for symbol in symbols.iter().take(20) {
                            println!("  • {} ({:?}) at line {}", 
                                     symbol.name, symbol.kind, symbol.line_start);
                        }
                        if symbols.len() > 20 {
                            println!("  ... and {} more", symbols.len() - 20);
                        }
                    }
                }
                Err(e) => println!("✗ Extraction failed: {}", e),
            }
        }
        Err(e) => println!("✗ File read failed: {}", e),
    }
    
    Ok(())
}

#[cfg(not(feature = "tree-sitter"))]
fn main() {
    println!("❌ verify_symbols requires 'tree-sitter' feature to be enabled");
    std::process::exit(1);
}