use embed_search::embedding::MiniLMEmbedder;
use embed_search::chunking::{SimpleRegexChunker, ThreeChunkExpander};
use std::fs;
use std::path::Path;
use std::collections::HashMap;

#[test]
fn test_vectortest_stress_all_files() {
    let embedder = MiniLMEmbedder::mock();
    let chunker = SimpleRegexChunker::new();
    
    let vectortest_dir = "vectortest";
    let files = get_all_code_files(vectortest_dir);
    
    assert!(!files.is_empty(), "Should find code files in vectortest directory");
    
    let mut total_chunks = 0;
    let mut total_embeddings = 0;
    let mut language_stats = HashMap::new();
    
    for file_path in &files {
        println!("Processing: {}", file_path.display());
        let content = fs::read_to_string(file_path).expect("Should be able to read file");
        
        // Track language statistics
        let extension = file_path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("unknown");
        *language_stats.entry(extension.to_string()).or_insert(0) += 1;
        
        // Test chunking
        let chunks = chunker.chunk_file(&content);
        assert!(!chunks.is_empty(), "Should create at least one chunk for {}", file_path.display());
        total_chunks += chunks.len();
        
        // Test embedding each chunk
        for (idx, chunk) in chunks.iter().enumerate() {
            let embedding = embedder.embed(&chunk.content).expect("Should create embedding");
            assert_eq!(embedding.len(), 384, "All embeddings should be 384-dimensional");
            
            // Test that embeddings are normalized
            let magnitude: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
            assert!((magnitude - 1.0).abs() < 0.1, "Embedding should be normalized");
            
            total_embeddings += 1;
            
            // Test three-chunk expansion for a few samples
            if idx % 5 == 0 { // Test every 5th chunk to avoid excessive testing
                let context = ThreeChunkExpander::expand(&chunks, idx).expect("Should expand chunk context");
                
                // Test that we can embed the full context
                let full_content = context.get_full_content();
                let context_embedding = embedder.embed(&full_content).expect("Should embed full context");
                assert_eq!(context_embedding.len(), 384);
            }
        }
        
        println!("  Created {} chunks and {} embeddings", chunks.len(), chunks.len());
    }
    
    println!("\n=== STRESS TEST RESULTS ===");
    println!("Files processed: {}", files.len());
    println!("Total chunks: {}", total_chunks);
    println!("Total embeddings: {}", total_embeddings);
    println!("Languages tested: {:?}", language_stats);
    
    // Assert minimum performance expectations
    assert!(files.len() >= 5, "Should test at least 5 files");
    assert!(total_chunks >= 20, "Should create at least 20 chunks total");
    assert_eq!(total_embeddings, total_chunks, "Should create one embedding per chunk");
}

#[test]
fn test_batch_embedding_performance() {
    let embedder = MiniLMEmbedder::mock();
    let chunker = SimpleRegexChunker::new();
    
    // Collect all text chunks from vectortest files
    let vectortest_dir = "vectortest";
    let files = get_all_code_files(vectortest_dir);
    
    let mut all_chunks = Vec::new();
    for file_path in files {
        let content = fs::read_to_string(file_path).expect("Should read file");
        let chunks = chunker.chunk_file(&content);
        for chunk in chunks {
            all_chunks.push(chunk.content);
        }
    }
    
    assert!(!all_chunks.is_empty(), "Should have chunks to test");
    
    // Test individual embeddings
    let start = std::time::Instant::now();
    let individual_embeddings: Vec<_> = all_chunks.iter()
        .map(|text| embedder.embed(text).unwrap())
        .collect();
    let individual_time = start.elapsed();
    
    // Test batch embeddings
    let text_refs: Vec<&str> = all_chunks.iter().map(|s| s.as_str()).collect();
    let start = std::time::Instant::now();
    let batch_embeddings = embedder.embed_batch(&text_refs).unwrap();
    let batch_time = start.elapsed();
    
    // Test chunked embeddings (large batch processing)
    let start = std::time::Instant::now();
    let chunked_embeddings = embedder.embed_chunked(&text_refs).unwrap();
    let chunked_time = start.elapsed();
    
    println!("\n=== PERFORMANCE RESULTS ===");
    println!("Chunks processed: {}", all_chunks.len());
    println!("Individual time: {:?}", individual_time);
    println!("Batch time: {:?}", batch_time);
    println!("Chunked time: {:?}", chunked_time);
    
    // All should produce same results
    assert_eq!(individual_embeddings.len(), batch_embeddings.len());
    assert_eq!(individual_embeddings.len(), chunked_embeddings.len());
    
    // Results should be identical (deterministic)
    for i in 0..individual_embeddings.len() {
        assert_eq!(individual_embeddings[i], batch_embeddings[i]);
        assert_eq!(individual_embeddings[i], chunked_embeddings[i]);
    }
}

#[test]
fn test_mixed_language_embedding_consistency() {
    let embedder = MiniLMEmbedder::mock();
    
    // Test that different programming languages produce different but consistent embeddings
    let test_samples = vec![
        ("rust", "fn main() { println!(\"Hello, world!\"); }"),
        ("python", "def main():\n    print(\"Hello, world!\")"),
        ("javascript", "function main() { console.log(\"Hello, world!\"); }"),
        ("java", "public static void main(String[] args) { System.out.println(\"Hello, world!\"); }"),
        ("cpp", "#include <iostream>\nint main() { std::cout << \"Hello, world!\" << std::endl; }"),
    ];
    
    let mut embeddings = HashMap::new();
    
    for (lang, code) in &test_samples {
        let embedding = embedder.embed(code).expect("Should create embedding");
        assert_eq!(embedding.len(), 384);
        
        // Verify normalization
        let magnitude: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((magnitude - 1.0).abs() < 0.1, "Embedding should be normalized for {}", lang);
        
        embeddings.insert(*lang, embedding);
    }
    
    // Verify that same input produces same output (deterministic)
    for (lang, code) in &test_samples {
        let embedding2 = embedder.embed(code).expect("Should create embedding");
        assert_eq!(embeddings[lang], embedding2, "Embeddings should be deterministic for {}", lang);
    }
    
    // Verify that different languages produce different embeddings
    let rust_embed = &embeddings["rust"];
    let python_embed = &embeddings["python"];
    assert_ne!(rust_embed, python_embed, "Different languages should produce different embeddings");
}

#[test]
fn test_large_file_handling() {
    let embedder = MiniLMEmbedder::mock();
    let chunker = SimpleRegexChunker::new();
    
    // Find the largest file in vectortest
    let vectortest_dir = "vectortest";
    let files = get_all_code_files(vectortest_dir);
    
    let largest_file = files.iter()
        .max_by_key(|path| {
            fs::metadata(path)
                .map(|meta| meta.len())
                .unwrap_or(0)
        })
        .expect("Should find at least one file");
    
    println!("Testing largest file: {}", largest_file.display());
    
    let content = fs::read_to_string(largest_file).expect("Should read largest file");
    println!("File size: {} characters", content.len());
    
    // Test chunking large file
    let chunks = chunker.chunk_file(&content);
    assert!(!chunks.is_empty(), "Should create chunks from large file");
    
    // Test embedding all chunks
    let mut embeddings = Vec::new();
    for chunk in &chunks {
        let embedding = embedder.embed(&chunk.content).expect("Should embed large file chunks");
        assert_eq!(embedding.len(), 384);
        embeddings.push(embedding);
    }
    
    println!("Created {} chunks and embeddings from large file", embeddings.len());
    
    // Test batch processing of large file
    let chunk_texts: Vec<&str> = chunks.iter().map(|c| c.content.as_str()).collect();
    let batch_embeddings = embedder.embed_batch(&chunk_texts).expect("Should batch embed large file");
    
    assert_eq!(embeddings.len(), batch_embeddings.len());
    
    // Verify batch results match individual results
    for (i, batch_embed) in batch_embeddings.iter().enumerate() {
        assert_eq!(&embeddings[i], batch_embed, "Batch and individual embeddings should match");
    }
}

#[test]
fn test_integration_with_chunking_system() {
    let embedder = MiniLMEmbedder::mock();
    let chunker = SimpleRegexChunker::new();
    
    // Test the integration helper methods
    let test_file = "vectortest/memory_cache.rs"; // Rust file for testing
    if !Path::new(test_file).exists() {
        panic!("Test file {} not found", test_file);
    }
    
    let content = fs::read_to_string(test_file).expect("Should read test file");
    let chunks = chunker.chunk_file(&content);
    assert!(!chunks.is_empty(), "Should create chunks");
    
    // Test embed_chunk helper
    for chunk in &chunks {
        let embedding = embedder.embed_chunk(chunk).expect("Should embed chunk using helper");
        assert_eq!(embedding.len(), 384);
    }
    
    // Test with three-chunk context
    for (idx, _) in chunks.iter().enumerate() {
        let context = ThreeChunkExpander::expand(&chunks, idx).expect("Should expand context");
        let context_embedding = embedder.embed_context(&context).expect("Should embed context");
        assert_eq!(context_embedding.len(), 384);
    }
    
    println!("Successfully tested integration with chunking system using {} chunks", chunks.len());
}

fn get_all_code_files(dir: &str) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
                    // Only include code files (not markdown/documentation)
                    if matches!(extension, "rs" | "py" | "js" | "ts" | "java" | "cpp" | "c" | "go" | "rb" | "cs" | "sql") {
                        files.push(path);
                    }
                }
            }
        }
    }
    
    files
}