use embed_search::search::symbol_index::{SymbolIndexer, SymbolKind};
use std::path::PathBuf;

fn main() {
    println!("Creating SymbolIndexer...");
    let mut indexer = match SymbolIndexer::new() {
        Ok(i) => {
            println!("✓ SymbolIndexer created successfully");
            i
        }
        Err(e) => {
            println!("✗ Failed to create SymbolIndexer: {}", e);
            return;
        }
    };
    
    // Test 1: Read and parse memory_cache.rs
    println!("\n=== Testing Rust file: vectortest/memory_cache.rs ===");
    let rust_file = PathBuf::from("vectortest/memory_cache.rs");
    match std::fs::read_to_string(&rust_file) {
        Ok(content) => {
            println!("✓ File read successfully ({} bytes)", content.len());
            
            match indexer.extract_symbols(&content, "rust", rust_file.to_str().unwrap()) {
                Ok(symbols) => {
                    println!("✓ Extracted {} symbols", symbols.len());
                    
                    // Check for specific symbols we know should exist
                    let has_memory_cache = symbols.iter().any(|s| s.name == "MemoryCache");
                    let has_cache_entry = symbols.iter().any(|s| s.name == "CacheEntry");
                    
                    println!("  MemoryCache struct found: {}", if has_memory_cache { "✓" } else { "✗" });
                    println!("  CacheEntry struct found: {}", if has_cache_entry { "✓" } else { "✗" });
                    
                    // List first 10 symbols
                    println!("\n  First 10 symbols found:");
                    for (i, symbol) in symbols.iter().take(10).enumerate() {
                        println!("    {}. {} ({:?}) at line {}", 
                                 i+1, symbol.name, symbol.kind, symbol.line_start);
                    }
                }
                Err(e) => println!("✗ Failed to extract symbols: {}", e),
            }
        }
        Err(e) => println!("✗ Failed to read file: {}", e),
    }
    
    // Test 2: Read and parse auth_service.py
    println!("\n=== Testing Python file: vectortest/auth_service.py ===");
    let python_file = PathBuf::from("vectortest/auth_service.py");
    match std::fs::read_to_string(&python_file) {
        Ok(content) => {
            println!("✓ File read successfully ({} bytes)", content.len());
            
            match indexer.extract_symbols(&content, "python", python_file.to_str().unwrap()) {
                Ok(symbols) => {
                    println!("✓ Extracted {} symbols", symbols.len());
                    
                    let classes = symbols.iter().filter(|s| s.kind == SymbolKind::Class).count();
                    let functions = symbols.iter().filter(|s| s.kind == SymbolKind::Function).count();
                    
                    println!("  Classes found: {}", classes);
                    println!("  Functions found: {}", functions);
                    
                    // List first 10 symbols
                    println!("\n  First 10 symbols found:");
                    for (i, symbol) in symbols.iter().take(10).enumerate() {
                        println!("    {}. {} ({:?}) at line {}", 
                                 i+1, symbol.name, symbol.kind, symbol.line_start);
                    }
                }
                Err(e) => println!("✗ Failed to extract symbols: {}", e),
            }
        }
        Err(e) => println!("✗ Failed to read file: {}", e),
    }
    
    // Test 3: Read and parse OrderService.java
    println!("\n=== Testing Java file: vectortest/OrderService.java ===");
    let java_file = PathBuf::from("vectortest/OrderService.java");
    match std::fs::read_to_string(&java_file) {
        Ok(content) => {
            println!("✓ File read successfully ({} bytes)", content.len());
            
            match indexer.extract_symbols(&content, "java", java_file.to_str().unwrap()) {
                Ok(symbols) => {
                    println!("✓ Extracted {} symbols", symbols.len());
                    
                    let has_order_service = symbols.iter().any(|s| s.name == "OrderService");
                    println!("  OrderService class found: {}", if has_order_service { "✓" } else { "✗" });
                    
                    // List first 10 symbols
                    println!("\n  First 10 symbols found:");
                    for (i, symbol) in symbols.iter().take(10).enumerate() {
                        println!("    {}. {} ({:?}) at line {}", 
                                 i+1, symbol.name, symbol.kind, symbol.line_start);
                    }
                }
                Err(e) => println!("✗ Failed to extract symbols: {}", e),
            }
        }
        Err(e) => println!("✗ Failed to read file: {}", e),
    }
    
    println!("\n=== Verification Complete ===");
}