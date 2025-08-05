use embed_search::chunking::SimpleRegexChunker;
use std::time::Instant;
use std::path::Path;

fn main() {
    let chunker = SimpleRegexChunker::new();
    
    // Test files
    let test_files = vec![
        ("Python", "vectortest/auth_service.py"),
        ("JavaScript", "vectortest/user_controller.js"),
        ("Java", "vectortest/OrderService.java"),
        ("Go", "vectortest/analytics_dashboard.go"),
        ("Rust", "vectortest/memory_cache.rs"),
        ("TypeScript", "vectortest/payment_gateway.ts"),
        ("C#", "vectortest/DataProcessor.cs"),
        ("Ruby", "vectortest/product_catalog.rb"),
        ("SQL", "vectortest/database_migration.sql"),
        ("C++", "vectortest/websocket_server.cpp"),
    ];
    
    println!("Performance Benchmark - Chunking Speed\n");
    println!("{:<15} {:<20} {:<15} {:<15} {:<15}", "Language", "File Size (KB)", "Chunks", "Time (ms)", "Lines/ms");
    println!("{}", "-".repeat(80));
    
    let mut total_lines = 0;
    let mut total_time = 0.0;
    
    for (lang, file_path) in test_files {
        let path = Path::new(file_path);
        
        if path.exists() {
            // Read file to get size and line count
            let content = std::fs::read_to_string(path).unwrap();
            let file_size = content.len() as f64 / 1024.0;
            let line_count = content.lines().count();
            
            // Benchmark chunking
            let start = Instant::now();
            let chunks = chunker.chunk_file(&content);
            let elapsed = start.elapsed().as_secs_f64() * 1000.0; // Convert to milliseconds
            
            let lines_per_ms = if elapsed > 0.0 { 
                line_count as f64 / elapsed 
            } else { 
                line_count as f64 
            };
            
            println!("{:<15} {:<20.2} {:<15} {:<15.3} {:<15.0}", 
                     lang, file_size, chunks.len(), elapsed, lines_per_ms);
            
            total_lines += line_count;
            total_time += elapsed;
        }
    }
    
    println!("{}", "-".repeat(80));
    println!("Total lines processed: {}", total_lines);
    println!("Total time: {:.3} ms", total_time);
    println!("Average throughput: {:.0} lines/ms", total_lines as f64 / total_time);
    
    // Test with a very large file
    println!("\nLarge file test:");
    let mut large_content = String::new();
    for i in 0..10000 {
        large_content.push_str(&format!("fn function_{:04}() {{\n", i));
        large_content.push_str("    // Some implementation\n");
        large_content.push_str("    let result = calculate_something();\n");
        large_content.push_str("    return result;\n");
        large_content.push_str("}\n\n");
    }
    
    let line_count = large_content.lines().count();
    let file_size = large_content.len() as f64 / 1024.0;
    
    let start = Instant::now();
    let chunks = chunker.chunk_file(&large_content);
    let elapsed = start.elapsed().as_secs_f64() * 1000.0;
    
    println!("Synthetic large file: {:.1} KB, {} lines", file_size, line_count);
    println!("Chunks created: {}", chunks.len());
    println!("Time taken: {:.3} ms", elapsed);
    println!("Throughput: {:.0} lines/ms", line_count as f64 / elapsed);
    
    // Performance requirement check
    println!("\nPerformance Requirements Check:");
    println!("✓ Target: <50ms per file");
    let all_under_50ms = total_time / 10.0 < 50.0; // Average per file
    if all_under_50ms {
        println!("✓ PASSED: Average {:.1}ms per file", total_time / 10.0);
    } else {
        println!("✗ FAILED: Average {:.1}ms per file", total_time / 10.0);
    }
}