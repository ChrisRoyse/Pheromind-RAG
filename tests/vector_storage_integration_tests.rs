use embed_search::storage::{VectorStorage, EmbeddingRecord};
use embed_search::chunking::{SimpleRegexChunker, Chunk};
use tempfile::TempDir;

#[tokio::test]
async fn test_full_workflow_integration() {
    // Test complete workflow: chunking -> embedding storage -> search
    let temp_dir = TempDir::new().expect("Should create temp dir");
    let db_path = temp_dir.path().join("integration_test.db");
    
    let mut storage = VectorStorage::new(db_path).await.expect("Should create storage");
    storage.init_schema().await.expect("Should init schema");
    
    let chunker = SimpleRegexChunker::new();
    
    // Sample Rust code to chunk and store
    let rust_code = r#"
// Simple calculator functions
fn add(a: i32, b: i32) -> i32 {
    a + b
}

fn subtract(x: i32, y: i32) -> i32 {
    x - y
}

fn multiply(left: f64, right: f64) -> f64 {
    left * right
}

struct Calculator {
    memory: f64,
}

impl Calculator {
    fn new() -> Self {
        Calculator { memory: 0.0 }
    }
    
    fn clear(&mut self) {
        self.memory = 0.0;
    }
}
"#;
    
    // Chunk the code
    let chunks = chunker.chunk_file(rust_code);
    assert!(!chunks.is_empty(), "Should create chunks from Rust code");
    
    // Create different embedding patterns for different types of code
    let mut function_embeddings = Vec::new();
    let mut struct_embeddings = Vec::new();
    
    for (idx, chunk) in chunks.iter().enumerate() {
        let mut embedding = vec![0.0f32; 384];
        
        if chunk.content.contains("fn ") {
            // Function pattern
            embedding[0] = 1.0;
            embedding[1] = 0.8;
            embedding[idx % 10] = 0.5; // Add some variation
            function_embeddings.push((idx, embedding.clone()));
        } else if chunk.content.contains("struct ") || chunk.content.contains("impl ") {
            // Struct/impl pattern
            embedding[10] = 1.0;
            embedding[11] = 0.9;
            embedding[idx % 10] = 0.3;
            struct_embeddings.push((idx, embedding.clone()));
        } else {
            // Other code
            embedding[20] = 0.5;
            embedding[21] = 0.4;
        }
        
        // Normalize embedding
        let magnitude: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if magnitude > 0.0 {
            for val in &mut embedding {
                *val /= magnitude;
            }
        }
        
        // Store in vector database  
        storage.insert_embedding("calculator.rs", idx, chunk, embedding).await
            .expect("Should insert embedding");
    }
    
    println!("Stored {} chunks total", chunks.len());
    println!("Functions: {}, Structs: {}", function_embeddings.len(), struct_embeddings.len());
    
    // Test search for functions
    let mut function_query = vec![0.0f32; 384];
    function_query[0] = 1.0;
    function_query[1] = 0.8;
    let magnitude: f32 = function_query.iter().map(|x| x * x).sum::<f32>().sqrt();
    for val in &mut function_query {
        *val /= magnitude;
    }
    
    let function_results = storage.search_similar(function_query, 5).await
        .expect("Should search for functions");
    
    assert!(!function_results.is_empty(), "Should find function results");
    // Verify that function results actually contain functions
    let function_count = function_results.iter()
        .filter(|r| r.content.contains("fn "))
        .count();
    println!("Found {} function results out of {}", function_count, function_results.len());
    
    // Test search for structs
    let mut struct_query = vec![0.0f32; 384];
    struct_query[10] = 1.0;
    struct_query[11] = 0.9;
    let magnitude: f32 = struct_query.iter().map(|x| x * x).sum::<f32>().sqrt();
    for val in &mut struct_query {
        *val /= magnitude;
    }
    
    let struct_results = storage.search_similar(struct_query, 3).await
        .expect("Should search for structs");
        
    println!("Found {} struct/impl results", struct_results.len());
    
    // Verify database state
    let total_count = storage.count().await.expect("Should get count");
    assert_eq!(total_count, chunks.len(), "Should have stored all chunks");
    
    // Test bulk operations
    let all_embeddings = storage.get_all_embeddings().await.expect("Should get all embeddings");
    assert_eq!(all_embeddings.len(), chunks.len(), "Should retrieve all embeddings");
}

#[tokio::test]
async fn test_multi_file_storage() {
    // Test storing embeddings from multiple files
    let temp_dir = TempDir::new().expect("Should create temp dir");
    let db_path = temp_dir.path().join("multi_file_test.db");
    
    let mut storage = VectorStorage::new(db_path).await.expect("Should create storage");
    storage.init_schema().await.expect("Should init schema");
    
    // Simulate different files with different patterns
    let files = vec![
        ("math.rs", "fn add(a: i32, b: i32) -> i32 { a + b }"),
        ("utils.rs", "fn format_string(s: &str) -> String { s.to_uppercase() }"),
        ("models.rs", "struct User { id: u64, name: String }"),
        ("handlers.rs", "fn handle_request() -> Result<(), Error> { Ok(()) }"),
    ];
    
    for (file_path, content) in &files {
        let chunk = Chunk {
            content: content.to_string(),
            start_line: 1,
            end_line: 1,
        };
        
        // Create file-specific embedding patterns
        let mut embedding = vec![0.0f32; 384];
        match *file_path {
            "math.rs" => {
                embedding[0] = 1.0;
                embedding[1] = 0.9;
            },
            "utils.rs" => {
                embedding[5] = 1.0;
                embedding[6] = 0.8;
            },
            "models.rs" => {
                embedding[10] = 1.0;
                embedding[11] = 0.7;
            },
            "handlers.rs" => {
                embedding[15] = 1.0;
                embedding[16] = 0.6;
            },
            _ => {}
        }
        
        // Normalize
        let magnitude: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if magnitude > 0.0 {
            for val in &mut embedding {
                *val /= magnitude;
            }
        }
        
        storage.insert_embedding(file_path, 0, &chunk, embedding).await
            .expect("Should insert embedding");
    }
    
    // Test searching within specific patterns
    let mut math_query = vec![0.0f32; 384];
    math_query[0] = 1.0;
    math_query[1] = 0.9;
    let magnitude: f32 = math_query.iter().map(|x| x * x).sum::<f32>().sqrt();
    for val in &mut math_query {
        *val /= magnitude;
    }
    
    let results = storage.search_similar(math_query, 4).await.expect("Should search");
    
    // Should find the math.rs file first (most similar)
    assert!(!results.is_empty(), "Should find results");
    assert_eq!(results[0].file_path, "math.rs", "Math query should find math.rs first");
    
    // Test file deletion
    storage.delete_by_file("math.rs").await.expect("Should delete math.rs");
    let count_after_delete = storage.count().await.expect("Should get count");
    assert_eq!(count_after_delete, 3, "Should have 3 files after deleting math.rs");
    
    // Verify math.rs is gone
    let all_embeddings = storage.get_all_embeddings().await.expect("Should get all");
    let math_files: Vec<_> = all_embeddings.iter()
        .filter(|e| e.file_path == "math.rs")
        .collect();
    assert_eq!(math_files.len(), 0, "Math.rs should be completely removed");
}

#[tokio::test]
async fn test_batch_operations_performance() {
    // Test performance of batch operations
    let temp_dir = TempDir::new().expect("Should create temp dir");
    let db_path = temp_dir.path().join("batch_perf_test.db");
    
    let mut storage = VectorStorage::new(db_path).await.expect("Should create storage");
    storage.init_schema().await.expect("Should init schema");
    
    // Prepare batch data
    let mut batch_data = Vec::new();
    for i in 0..100 {
        let chunk = Chunk {
            content: format!("fn test_{}() {{ return {}; }}", i, i),
            start_line: i + 1,
            end_line: i + 1,
        };
        
        let mut embedding = vec![0.0f32; 384];
        embedding[0] = (i as f32) / 100.0;
        embedding[1] = ((i * 2) as f32) / 100.0;
        
        // Normalize
        let magnitude: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if magnitude > 0.0 {
            for val in &mut embedding {
                *val /= magnitude;
            }
        }
        
        batch_data.push(("batch_test.rs", i, chunk, embedding));
    }
    
    // Time the batch insert
    let start = std::time::Instant::now();
    storage.insert_batch(batch_data).await.expect("Should batch insert");
    let batch_time = start.elapsed();
    
    println!("Batch insert of 100 embeddings took: {:?}", batch_time);
    
    // Verify all were inserted
    let count = storage.count().await.expect("Should get count");
    assert_eq!(count, 100, "Should have inserted all 100 embeddings");
    
    // Test batch search performance
    let query = vec![0.5f32; 384];
    let start = std::time::Instant::now();
    let results = storage.search_similar(query, 10).await.expect("Should search");
    let search_time = start.elapsed();
    
    println!("Search across 100 embeddings took: {:?}", search_time);
    assert_eq!(results.len(), 10, "Should return requested number of results");
    
    // Performance assertions (these are loose bounds)
    assert!(batch_time.as_millis() < 1000, "Batch insert should be under 1 second");
    assert!(search_time.as_millis() < 100, "Search should be under 100ms");
}

#[tokio::test]
async fn test_vector_storage_robustness() {
    // Test robustness under various conditions
    let temp_dir = TempDir::new().expect("Should create temp dir");
    let db_path = temp_dir.path().join("robustness_test.db");
    
    let mut storage = VectorStorage::new(db_path).await.expect("Should create storage");
    storage.init_schema().await.expect("Should init schema");
    
    // Test with empty content
    let empty_chunk = Chunk {
        content: "".to_string(),
        start_line: 1,
        end_line: 1,
    };
    let embedding = vec![0.1f32; 384];
    storage.insert_embedding("empty.rs", 0, &empty_chunk, embedding).await
        .expect("Should handle empty content");
    
    // Test with very long content
    let long_content = "fn very_long_function() {\n".to_string() + &"    println!(\"line\");\n".repeat(1000) + "}";
    let long_chunk = Chunk {
        content: long_content,
        start_line: 1,
        end_line: 1001,
    };
    let embedding2 = vec![0.2f32; 384];
    storage.insert_embedding("long.rs", 0, &long_chunk, embedding2).await
        .expect("Should handle long content");
    
    // Test with special characters
    let special_chunk = Chunk {
        content: "fn test() { println!(\"Hello, 世界! 🦀\"); }".to_string(),
        start_line: 1,
        end_line: 1,
    };
    let embedding3 = vec![0.3f32; 384];
    storage.insert_embedding("special.rs", 0, &special_chunk, embedding3).await
        .expect("Should handle special characters");
    
    // Test search and retrieval
    let query = vec![0.15f32; 384];
    let results = storage.search_similar(query, 3).await.expect("Should search all");
    assert_eq!(results.len(), 3, "Should find all three varied embeddings");
    
    // Test clearing and reinitializing
    storage.clear_all().await.expect("Should clear all");
    let count_after_clear = storage.count().await.expect("Should get count");
    assert_eq!(count_after_clear, 0, "Should be empty after clear");
    
    // Should be able to insert again after clearing
    let new_chunk = Chunk {
        content: "fn after_clear() {}".to_string(),
        start_line: 1,
        end_line: 1,
    };
    let embedding4 = vec![0.4f32; 384];
    storage.insert_embedding("after_clear.rs", 0, &new_chunk, embedding4).await
        .expect("Should work after clearing");
    
    let final_count = storage.count().await.expect("Should get final count");
    assert_eq!(final_count, 1, "Should have one embedding after reinsertion");
}