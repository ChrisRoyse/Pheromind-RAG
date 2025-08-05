use embed_search::chunking::{SimpleRegexChunker, Chunk};
use std::path::Path;

fn main() {
    let chunker = SimpleRegexChunker::new();
    
    // Test with Python file
    println!("Testing Python file chunking:");
    let python_path = Path::new("vectortest/auth_service.py");
    if let Ok(chunks) = chunker.chunk_file_from_path(python_path) {
        println!("Total chunks: {}", chunks.len());
        println!("\nFirst 3 chunks:");
        for (i, chunk) in chunks.iter().take(3).enumerate() {
            println!("\n--- Chunk {} (lines {}-{}) ---", i + 1, chunk.start_line, chunk.end_line);
            println!("{}", chunk.content.lines().take(10).collect::<Vec<_>>().join("\n"));
            if chunk.content.lines().count() > 10 {
                println!("... ({} more lines)", chunk.content.lines().count() - 10);
            }
        }
    }
    
    // Test with JavaScript file
    println!("\n\nTesting JavaScript file chunking:");
    let js_path = Path::new("vectortest/user_controller.js");
    if let Ok(chunks) = chunker.chunk_file_from_path(js_path) {
        println!("Total chunks: {}", chunks.len());
        
        // Find chunks with async functions
        let async_chunks: Vec<&Chunk> = chunks.iter()
            .filter(|c| c.content.contains("async "))
            .collect();
        println!("Chunks with async functions: {}", async_chunks.len());
    }
    
    // Test chunk size distribution
    println!("\n\nChunk size analysis across all test files:");
    let test_files = vec![
        "vectortest/auth_service.py",
        "vectortest/user_controller.js",
        "vectortest/OrderService.java",
        "vectortest/analytics_dashboard.go",
        "vectortest/memory_cache.rs",
    ];
    
    for file_path in test_files {
        let path = Path::new(file_path);
        if let Ok(chunks) = chunker.chunk_file_from_path(path) {
            let sizes: Vec<usize> = chunks.iter()
                .map(|c| c.content.lines().count())
                .collect();
            let avg_size = sizes.iter().sum::<usize>() as f32 / sizes.len() as f32;
            let max_size = sizes.iter().max().unwrap_or(&0);
            let min_size = sizes.iter().min().unwrap_or(&0);
            
            println!("\n{}: {} chunks", file_path, chunks.len());
            println!("  Avg lines: {:.1}, Min: {}, Max: {}", avg_size, min_size, max_size);
        }
    }
}