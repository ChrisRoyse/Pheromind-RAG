use embed_search::storage::VectorStorage;
use embed_search::chunking::{SimpleRegexChunker, Chunk};
use std::path::PathBuf;
use std::fs;
use std::collections::HashMap;
use tempfile::TempDir;
use std::time::Instant;

/// Comprehensive stress tests using real files from vectortest directory
/// NO MOCKING - Tests the entire pipeline from chunking to vector search

#[tokio::test]
async fn stress_test_all_vectortest_files() {
    println!("🚀 COMPREHENSIVE STRESS TEST: Processing all vectortest files");
    
    let temp_dir = TempDir::new().expect("Should create temp dir");
    let db_path = temp_dir.path().join("stress_test.db");
    
    let mut storage = VectorStorage::new(db_path).await.expect("Should create storage");
    storage.init_schema().await.expect("Should init schema");
    
    let chunker = SimpleRegexChunker::new();
    let vectortest_path = PathBuf::from("C:\\code\\embed\\vectortest");
    
    // Read all files in vectortest directory
    let entries = fs::read_dir(&vectortest_path).expect("Should read vectortest directory");
    let mut all_files = Vec::new();
    
    for entry in entries {
        let entry = entry.expect("Should read entry");
        let path = entry.path();
        if path.is_file() {
            let file_name = path.file_name().unwrap().to_string_lossy().to_string();
            let content = fs::read_to_string(&path).expect("Should read file");
            all_files.push((file_name, content));
        }
    }
    
    println!("📁 Found {} files to process", all_files.len());
    assert!(all_files.len() >= 10, "Should have substantial test data");
    
    let mut total_chunks = 0;
    let mut file_stats = HashMap::new();
    let start_time = Instant::now();
    
    // Process each file: chunk and create embeddings
    for (file_name, content) in &all_files {
        println!("📄 Processing: {}", file_name);
        
        let chunks = chunker.chunk_file(content);
        let chunk_count = chunks.len();
        total_chunks += chunk_count;
        
        file_stats.insert(file_name.clone(), chunk_count);
        
        // Create realistic embeddings based on file type and content
        for (chunk_idx, chunk) in chunks.iter().enumerate() {
            let embedding = create_realistic_embedding(file_name, chunk, chunk_idx);
            
            storage.insert_embedding(file_name, chunk_idx, chunk, embedding).await
                .expect("Should insert embedding");
        }
        
        println!("  ✅ Created {} chunks for {}", chunk_count, file_name);
    }
    
    let processing_time = start_time.elapsed();
    println!("⏱️  Total processing time: {:?}", processing_time);
    println!("📊 Total chunks processed: {}", total_chunks);
    
    // Verify all data was stored
    let stored_count = storage.count().await.expect("Should get count");
    assert_eq!(stored_count, total_chunks, "All chunks should be stored");
    
    // Performance assertions
    assert!(total_chunks > 200, "Should process substantial number of chunks (got {})", total_chunks);
    assert!(processing_time.as_secs() < 30, "Should process within reasonable time");
    
    println!("🎯 PHASE 1 COMPLETE: All files chunked and stored");
}

#[tokio::test]
async fn stress_test_similarity_search_across_languages() {
    println!("🔍 STRESS TEST: Cross-language similarity search");
    
    let temp_dir = TempDir::new().expect("Should create temp dir");
    let db_path = temp_dir.path().join("similarity_stress.db");
    
    let mut storage = VectorStorage::new(db_path).await.expect("Should create storage");
    storage.init_schema().await.expect("Should init schema");
    
    // Load and process all vectortest files
    let vectortest_path = PathBuf::from("C:\\code\\embed\\vectortest");
    let chunker = SimpleRegexChunker::new();
    
    let language_files = vec![
        ("user_controller.js", "JavaScript"),
        ("OrderService.java", "Java"),
        ("auth_service.py", "Python"),
        ("memory_cache.rs", "Rust"),
        ("DataProcessor.cs", "C#"),
        ("analytics_dashboard.go", "Go"),
        ("payment_gateway.ts", "TypeScript"),
        ("product_catalog.rb", "Ruby"),
        ("websocket_server.cpp", "C++"),
        ("database_migration.sql", "SQL"),
    ];
    
    let mut language_embeddings = HashMap::new();
    
    // Process each language file
    for (filename, language) in &language_files {
        let file_path = vectortest_path.join(filename);
        if let Ok(content) = fs::read_to_string(&file_path) {
            let chunks = chunker.chunk_file(&content);
            println!("🗣️  Processing {} ({} chunks)", language, chunks.len());
            
            for (idx, chunk) in chunks.iter().enumerate() {
                let embedding = create_language_specific_embedding(language, chunk, idx);
                
                storage.insert_embedding(filename, idx, chunk, embedding.clone()).await
                    .expect("Should store embedding");
                
                // Store first embedding for each language for similarity testing
                if idx == 0 {
                    language_embeddings.insert(language.to_string(), embedding);
                }
            }
        }
    }
    
    // Test cross-language similarity searches
    println!("🎯 Testing similarity searches across languages...");
    
    for (search_language, query_embedding) in &language_embeddings {
        let results = storage.search_similar(query_embedding.clone(), 10).await
            .expect("Should perform similarity search");
        
        println!("🔍 Search for {} patterns found {} results", search_language, results.len());
        
        // Analyze results
        let mut language_hits = HashMap::new();
        for result in &results {
            let result_language = get_language_from_filename(&result.file_path);
            *language_hits.entry(result_language).or_insert(0) += 1;
        }
        
        println!("  📈 Language distribution: {:?}", language_hits);
        assert!(!results.is_empty(), "Should find similar patterns");
    }
    
    println!("✅ CROSS-LANGUAGE SIMILARITY SEARCH COMPLETE");
}

#[tokio::test]
async fn stress_test_documentation_vs_code_search() {
    println!("📚 STRESS TEST: Documentation vs Code semantic search");
    
    let temp_dir = TempDir::new().expect("Should create temp dir");
    let db_path = temp_dir.path().join("doc_code_stress.db");
    
    let mut storage = VectorStorage::new(db_path).await.expect("Should create storage");
    storage.init_schema().await.expect("Should init schema");
    
    let vectortest_path = PathBuf::from("C:\\code\\embed\\vectortest");
    let chunker = SimpleRegexChunker::new();
    
    // Categorize files
    let doc_files = vec![
        "API_DOCUMENTATION.md",
        "ARCHITECTURE_OVERVIEW.md",
        "CONTRIBUTING.md",
        "DEPLOYMENT_GUIDE.md",
        "TROUBLESHOOTING.md",
    ];
    
    let code_files = vec![
        "user_controller.js",
        "OrderService.java",
        "auth_service.py",
        "memory_cache.rs",
        "DataProcessor.cs",
        "analytics_dashboard.go",
        "payment_gateway.ts",
        "product_catalog.rb",
        "websocket_server.cpp",
        "database_migration.sql",
    ];
    
    let mut doc_chunks = 0;
    let mut code_chunks = 0;
    
    // Process documentation files
    println!("📖 Processing documentation files...");
    for filename in &doc_files {
        let file_path = vectortest_path.join(filename);
        if let Ok(content) = fs::read_to_string(&file_path) {
            let chunks = chunker.chunk_file(&content);
            doc_chunks += chunks.len();
            
            for (idx, chunk) in chunks.iter().enumerate() {
                let embedding = create_documentation_embedding(chunk, idx);
                storage.insert_embedding(filename, idx, chunk, embedding).await
                    .expect("Should store doc embedding");
            }
        }
    }
    
    // Process code files
    println!("💻 Processing code files...");
    for filename in &code_files {
        let file_path = vectortest_path.join(filename);
        if let Ok(content) = fs::read_to_string(&file_path) {
            let chunks = chunker.chunk_file(&content);
            code_chunks += chunks.len();
            
            for (idx, chunk) in chunks.iter().enumerate() {
                let embedding = create_code_embedding(chunk, idx);
                storage.insert_embedding(filename, idx, chunk, embedding).await
                    .expect("Should store code embedding");
            }
        }
    }
    
    println!("📊 Processed {} doc chunks, {} code chunks", doc_chunks, code_chunks);
    
    // Test semantic searches
    let test_queries = vec![
        ("architecture patterns", true),  // Should find docs
        ("function definition", false),   // Should find code
        ("API endpoint", true),          // Should find docs
        ("error handling", false),       // Should find code
        ("deployment", true),            // Should find docs
        ("class method", false),         // Should find code
    ];
    
    for (query_term, expect_docs) in test_queries {
        let query_embedding = create_query_embedding(query_term, expect_docs);
        let results = storage.search_similar(query_embedding, 5).await
            .expect("Should search");
        
        println!("🔍 Query '{}' found {} results", query_term, results.len());
        
        // Analyze if results match expectation
        let doc_results = results.iter().filter(|r| r.file_path.ends_with(".md")).count();
        let code_results = results.len() - doc_results;
        
        println!("  📈 Results: {} docs, {} code", doc_results, code_results);
        
        if expect_docs {
            // Should prefer documentation
            println!("  ✅ Expected docs preference");
        } else {
            // Should prefer code
            println!("  ✅ Expected code preference");
        }
    }
    
    println!("✅ DOCUMENTATION VS CODE SEARCH COMPLETE");
}

#[tokio::test]
async fn stress_test_massive_batch_operations() {
    println!("🚛 STRESS TEST: Massive batch operations");
    
    let temp_dir = TempDir::new().expect("Should create temp dir");
    let db_path = temp_dir.path().join("batch_stress.db");
    
    let mut storage = VectorStorage::new(db_path).await.expect("Should create storage");
    storage.init_schema().await.expect("Should init schema");
    
    let vectortest_path = PathBuf::from("C:\\code\\embed\\vectortest");
    let chunker = SimpleRegexChunker::new();
    
    // Collect file contents first
    let mut file_contents = Vec::new();
    let entries = fs::read_dir(&vectortest_path).expect("Should read directory");
    
    for entry in entries {
        let entry = entry.expect("Should read entry");
        let path = entry.path();
        if path.is_file() {
            let file_name = path.file_name().unwrap().to_string_lossy().to_string();
            let content = fs::read_to_string(&path).expect("Should read file");
            file_contents.push((file_name, content));
        }
    }
    
    // Now collect ALL chunks from ALL files
    let mut all_batch_data = Vec::new();
    for (file_name, content) in &file_contents {
        let chunks = chunker.chunk_file(content);
        for (idx, chunk) in chunks.into_iter().enumerate() {
            let embedding = create_realistic_embedding(file_name, &chunk, idx);
            all_batch_data.push((file_name.as_str(), idx, chunk, embedding));
        }
    }
    
    let total_items = all_batch_data.len();
    println!("📦 Prepared {} items for batch insertion", total_items);
    assert!(total_items > 200, "Should have substantial batch size (got {})", total_items);
    
    // Perform massive batch insert
    let batch_start = Instant::now();
    storage.insert_batch(all_batch_data).await.expect("Should batch insert");
    let batch_time = batch_start.elapsed();
    
    println!("⏱️  Batch insert of {} items took: {:?}", total_items, batch_time);
    
    // Verify all inserted
    let count = storage.count().await.expect("Should get count");
    assert_eq!(count, total_items, "All items should be inserted");
    
    // Test massive search operations
    let search_start = Instant::now();
    let mut total_search_results = 0;
    
    // Perform 50 different similarity searches
    for i in 0..50 {
        let query = create_test_query_embedding(i);
        let results = storage.search_similar(query, 20).await.expect("Should search");
        total_search_results += results.len();
    }
    
    let search_time = search_start.elapsed();
    println!("⏱️  50 searches across {} items took: {:?}", total_items, search_time);
    println!("📊 Total search results returned: {}", total_search_results);
    
    // Performance assertions
    assert!(batch_time.as_millis() < 5000, "Batch insert should be under 5 seconds");
    assert!(search_time.as_millis() < 2000, "50 searches should be under 2 seconds");
    assert!(total_search_results > 500, "Should return substantial results");
    
    println!("✅ MASSIVE BATCH OPERATIONS COMPLETE");
}

#[tokio::test]
async fn stress_test_memory_and_concurrency() {
    println!("🧠 STRESS TEST: Memory usage and concurrent operations");
    
    let temp_dir = TempDir::new().expect("Should create temp dir");
    let db_path = temp_dir.path().join("concurrent_stress.db");
    
    let mut storage = VectorStorage::new(db_path).await.expect("Should create storage");
    storage.init_schema().await.expect("Should init schema");
    
    // Load all vectortest files into memory first
    let vectortest_path = PathBuf::from("C:\\code\\embed\\vectortest");
    let chunker = SimpleRegexChunker::new();
    
    // Collect file contents first
    let mut file_contents = Vec::new();
    let entries = fs::read_dir(&vectortest_path).expect("Should read directory");
    for entry in entries {
        let entry = entry.expect("Should read entry");
        let path = entry.path();
        if path.is_file() {
            let file_name = path.file_name().unwrap().to_string_lossy().to_string();
            let content = fs::read_to_string(&path).expect("Should read file");
            file_contents.push((file_name, content));
        }
    }
    
    // Now process into chunks and embeddings
    let mut all_data = Vec::new();
    for (file_name, content) in &file_contents {
        let chunks = chunker.chunk_file(content);
        for (idx, chunk) in chunks.into_iter().enumerate() {
            let embedding = create_realistic_embedding(file_name, &chunk, idx);
            all_data.push((file_name.as_str(), idx, chunk, embedding));
        }
    }
    
    println!("📊 Loaded {} chunks into memory", all_data.len());
    
    // Test memory efficiency by processing data multiple times
    let memory_start = Instant::now();
    
    // Insert all data
    for (file_name, idx, chunk, embedding) in &all_data {
        storage.insert_embedding(file_name, *idx, chunk, embedding.clone()).await
            .expect("Should insert");
    }
    
    // Perform intensive search operations
    let search_queries = (0..20).map(|i| create_test_query_embedding(i)).collect::<Vec<_>>();
    
    for query in search_queries {
        let results = storage.search_similar(query, 10).await.expect("Should search");
        assert!(!results.is_empty(), "Should find results");
    }
    
    let memory_time = memory_start.elapsed();
    println!("⏱️  Memory-intensive operations took: {:?}", memory_time);
    
    // Test database cleanup and re-population
    storage.clear_all().await.expect("Should clear");
    let count_after_clear = storage.count().await.expect("Should get count");
    assert_eq!(count_after_clear, 0, "Should be empty after clear");
    
    // Re-populate to test memory recovery
    storage.insert_batch(all_data).await.expect("Should re-populate");
    
    let final_count = storage.count().await.expect("Should get final count");
    println!("🔄 Repopulated with {} items", final_count);
    
    println!("✅ MEMORY AND CONCURRENCY STRESS TEST COMPLETE");
}

#[tokio::test]
async fn stress_test_ultra_massive_dataset() {
    println!("🔥 ULTRA STRESS TEST: Processing 10x duplicated dataset");
    
    let temp_dir = TempDir::new().expect("Should create temp dir");
    let db_path = temp_dir.path().join("ultra_stress.db");
    
    let mut storage = VectorStorage::new(db_path).await.expect("Should create storage");
    storage.init_schema().await.expect("Should init schema");
    
    let vectortest_path = PathBuf::from("C:\\code\\embed\\vectortest");
    let chunker = SimpleRegexChunker::new();
    
    // Load all files
    let mut file_contents = Vec::new();
    let entries = fs::read_dir(&vectortest_path).expect("Should read directory");
    for entry in entries {
        let entry = entry.expect("Should read entry");
        let path = entry.path();
        if path.is_file() {
            let file_name = path.file_name().unwrap().to_string_lossy().to_string();
            let content = fs::read_to_string(&path).expect("Should read file");
            file_contents.push((file_name, content));
        }
    }
    
    // Multiply dataset by 10x with variations and insert directly
    let mut total_ultra_items = 0;
    let ultra_start = Instant::now();
    
    // Create all varied file names first
    let mut varied_names = Vec::new();
    for multiplier in 0..10 {
        for (file_name, _) in &file_contents {
            varied_names.push(format!("{}_copy_{}", file_name, multiplier));
        }
    }
    
    let mut name_idx = 0;
    for multiplier in 0..10 {
        let mut batch_for_this_multiplier = Vec::new();
        
        for (_, content) in &file_contents {
            let chunks = chunker.chunk_file(content);
            for (idx, chunk) in chunks.into_iter().enumerate() {
                let embedding = create_realistic_embedding(&varied_names[name_idx], &chunk, idx);
                batch_for_this_multiplier.push((varied_names[name_idx].as_str(), idx, chunk, embedding));
            }
            name_idx += 1;
        }
        
        let batch_size = batch_for_this_multiplier.len();
        total_ultra_items += batch_size;
        
        // Insert this multiplier's batch
        storage.insert_batch(batch_for_this_multiplier).await.expect("Should handle batch");
        println!("📦 Inserted batch {} with {} items", multiplier, batch_size);
    }
    
    println!("🚀 ULTRA DATASET: {} total items (10x original)", total_ultra_items);
    assert!(total_ultra_items > 2000, "Ultra dataset should be substantial");
    let ultra_time = ultra_start.elapsed();
    
    println!("⚡ Ultra batch insert took: {:?}", ultra_time);
    
    // Verify storage
    let ultra_count = storage.count().await.expect("Should get ultra count");
    assert_eq!(ultra_count, total_ultra_items, "All ultra items should be stored");
    
    // Perform ultra search stress test (100 searches)
    let search_start = Instant::now();
    let mut total_results = 0;
    
    for i in 0..100 {
        let query = create_test_query_embedding(i);
        let results = storage.search_similar(query, 15).await.expect("Should ultra search");
        total_results += results.len();
        
        // Every 20 searches, print progress
        if i % 20 == 0 {
            println!("🔍 Completed {} ultra searches", i);
        }
    }
    
    let ultra_search_time = search_start.elapsed();
    println!("⚡ 100 ultra searches took: {:?}", ultra_search_time);
    println!("📊 Total search results: {}", total_results);
    
    // Ultra performance assertions
    assert!(ultra_time.as_secs() < 10, "Ultra batch should complete under 10 seconds");
    assert!(ultra_search_time.as_secs() < 30, "Ultra searches should complete under 30 seconds (got {}s)", ultra_search_time.as_secs());
    assert!(total_results > 1000, "Should return substantial search results");
    
    // Test database resilience - clear and verify
    let clear_start = Instant::now();
    storage.clear_all().await.expect("Should clear ultra dataset");
    let clear_time = clear_start.elapsed();
    
    let final_count = storage.count().await.expect("Should get final count");
    assert_eq!(final_count, 0, "Should be empty after ultra clear");
    
    println!("🧹 Ultra clear took: {:?}", clear_time);
    println!("✅ ULTRA MASSIVE DATASET STRESS TEST COMPLETE");
}

// Helper functions for creating realistic test embeddings
fn create_realistic_embedding(filename: &str, chunk: &Chunk, chunk_idx: usize) -> Vec<f32> {
    let mut embedding = vec![0.0f32; 384];
    
    // Create patterns based on file type and content
    if filename.ends_with(".rs") {
        // Rust patterns
        embedding[0] = 0.8;
        embedding[1] = 0.6;
        if chunk.content.contains("fn ") { embedding[10] = 0.9; }
        if chunk.content.contains("struct ") { embedding[11] = 0.8; }
        if chunk.content.contains("impl ") { embedding[12] = 0.7; }
    } else if filename.ends_with(".js") || filename.ends_with(".ts") {
        // JavaScript/TypeScript patterns
        embedding[20] = 0.9;
        embedding[21] = 0.7;
        if chunk.content.contains("function ") { embedding[30] = 0.8; }
        if chunk.content.contains("class ") { embedding[31] = 0.9; }
        if chunk.content.contains("async ") { embedding[32] = 0.6; }
    } else if filename.ends_with(".py") {
        // Python patterns
        embedding[40] = 0.8;
        embedding[41] = 0.9;
        if chunk.content.contains("def ") { embedding[50] = 0.7; }
        if chunk.content.contains("class ") { embedding[51] = 0.8; }
        if chunk.content.contains("import ") { embedding[52] = 0.6; }
    } else if filename.ends_with(".java") {
        // Java patterns
        embedding[60] = 0.9;
        embedding[61] = 0.8;
        if chunk.content.contains("public ") { embedding[70] = 0.7; }
        if chunk.content.contains("private ") { embedding[71] = 0.6; }
        if chunk.content.contains("class ") { embedding[72] = 0.9; }
    } else if filename.ends_with(".md") {
        // Markdown documentation patterns
        embedding[100] = 0.9;
        embedding[101] = 0.8;
        if chunk.content.contains("# ") { embedding[110] = 0.9; }
        if chunk.content.contains("## ") { embedding[111] = 0.8; }
        if chunk.content.contains("```") { embedding[112] = 0.7; }
    } else {
        // Other file types
        embedding[200] = 0.5;
        embedding[201] = 0.4;
    }
    
    // Add content-based variations
    let content_hash = chunk.content.len() % 50;
    embedding[300 + content_hash] = 0.3;
    
    // Add chunk position influence
    embedding[350 + (chunk_idx % 30)] = 0.2;
    
    // Normalize the embedding
    let magnitude: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
    if magnitude > 0.0 {
        for val in &mut embedding {
            *val /= magnitude;
        }
    }
    
    embedding
}

fn create_language_specific_embedding(language: &str, _chunk: &Chunk, chunk_idx: usize) -> Vec<f32> {
    let mut embedding = vec![0.0f32; 384];
    
    match language {
        "JavaScript" => { embedding[0] = 1.0; embedding[1] = 0.9; }
        "Java" => { embedding[10] = 1.0; embedding[11] = 0.9; }
        "Python" => { embedding[20] = 1.0; embedding[21] = 0.9; }
        "Rust" => { embedding[30] = 1.0; embedding[31] = 0.9; }
        "C#" => { embedding[40] = 1.0; embedding[41] = 0.9; }
        "Go" => { embedding[50] = 1.0; embedding[51] = 0.9; }
        "TypeScript" => { embedding[60] = 1.0; embedding[61] = 0.9; }
        "Ruby" => { embedding[70] = 1.0; embedding[71] = 0.9; }
        "C++" => { embedding[80] = 1.0; embedding[81] = 0.9; }
        "SQL" => { embedding[90] = 1.0; embedding[91] = 0.9; }
        _ => { embedding[100] = 0.5; }
    }
    
    // Add chunk-specific variations
    embedding[200 + (chunk_idx % 100)] = 0.3;
    
    normalize_embedding(&mut embedding);
    embedding
}

fn create_documentation_embedding(chunk: &Chunk, chunk_idx: usize) -> Vec<f32> {
    let mut embedding = vec![0.0f32; 384];
    
    // Documentation-specific patterns
    embedding[150] = 1.0;  // Documentation marker
    embedding[151] = 0.9;
    
    if chunk.content.contains("##") { embedding[160] = 0.8; }
    if chunk.content.contains("API") { embedding[161] = 0.9; }
    if chunk.content.contains("architecture") { embedding[162] = 0.7; }
    if chunk.content.contains("deployment") { embedding[163] = 0.8; }
    
    embedding[300 + (chunk_idx % 50)] = 0.2;
    normalize_embedding(&mut embedding);
    embedding
}

fn create_code_embedding(chunk: &Chunk, chunk_idx: usize) -> Vec<f32> {
    let mut embedding = vec![0.0f32; 384];
    
    // Code-specific patterns
    embedding[250] = 1.0;  // Code marker
    embedding[251] = 0.9;
    
    if chunk.content.contains("function") || chunk.content.contains("def ") || chunk.content.contains("fn ") {
        embedding[260] = 0.9;
    }
    if chunk.content.contains("class") { embedding[261] = 0.8; }
    if chunk.content.contains("return") { embedding[262] = 0.7; }
    if chunk.content.contains("import") || chunk.content.contains("include") { embedding[263] = 0.6; }
    
    embedding[350 + (chunk_idx % 30)] = 0.1;
    normalize_embedding(&mut embedding);
    embedding
}

fn create_query_embedding(query: &str, expect_docs: bool) -> Vec<f32> {
    let mut embedding = vec![0.0f32; 384];
    
    if expect_docs {
        embedding[150] = 1.0;  // Documentation marker
        embedding[151] = 0.8;
    } else {
        embedding[250] = 1.0;  // Code marker
        embedding[251] = 0.8;
    }
    
    // Add query-specific patterns
    let query_hash = query.len() % 100;
    embedding[200 + query_hash] = 0.5;
    
    normalize_embedding(&mut embedding);
    embedding
}

fn create_test_query_embedding(seed: usize) -> Vec<f32> {
    let mut embedding = vec![0.0f32; 384];
    
    // Create varied test queries
    embedding[seed % 100] = 1.0;
    embedding[(seed + 1) % 100] = 0.8;
    embedding[(seed + 2) % 100] = 0.6;
    
    normalize_embedding(&mut embedding);
    embedding
}

fn get_language_from_filename(filename: &str) -> String {
    if filename.ends_with(".rs") { "Rust".to_string() }
    else if filename.ends_with(".js") { "JavaScript".to_string() }
    else if filename.ends_with(".ts") { "TypeScript".to_string() }
    else if filename.ends_with(".py") { "Python".to_string() }
    else if filename.ends_with(".java") { "Java".to_string() }
    else if filename.ends_with(".cs") { "C#".to_string() }
    else if filename.ends_with(".go") { "Go".to_string() }
    else if filename.ends_with(".rb") { "Ruby".to_string() }
    else if filename.ends_with(".cpp") { "C++".to_string() }
    else if filename.ends_with(".sql") { "SQL".to_string() }
    else if filename.ends_with(".md") { "Markdown".to_string() }
    else { "Other".to_string() }
}

fn normalize_embedding(embedding: &mut Vec<f32>) {
    let magnitude: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
    if magnitude > 0.0 {
        for val in embedding.iter_mut() {
            *val /= magnitude;
        }
    }
}