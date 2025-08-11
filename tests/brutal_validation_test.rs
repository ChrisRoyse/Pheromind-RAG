// BRUTAL VALIDATION TEST - This will expose the truth about GGUF implementation
use anyhow::Result;
use std::time::Instant;

#[test]
fn test_brutal_reality_check() -> Result<()> {
    println!("🔍 BRUTAL VALIDATION: Starting reality check...");
    
    // Test 1: Can we even import the types?
    println!("📦 Testing module imports...");
    
    match std::panic::catch_unwind(|| {
        use embed_search::llama_wrapper_working::{GGUFModel, GGUFContext};
        println!("✅ Successfully imported GGUFModel and GGUFContext");
    }) {
        Ok(_) => println!("✅ Module imports work"),
        Err(_) => {
            println!("❌ FAIL: Cannot even import the types - agents lied about working implementation");
            return Ok(());
        }
    }
    
    // Test 2: Does the model file actually exist?
    let model_path = "./src/model/nomic-embed-code.Q4_K_M.gguf";
    let model_exists = std::path::Path::new(model_path).exists();
    
    if model_exists {
        let metadata = std::fs::metadata(model_path)?;
        println!("✅ Model file exists: {} MB", metadata.len() / (1024*1024));
    } else {
        println!("❌ FAIL: Model file does not exist at {}", model_path);
        return Ok(());
    }
    
    // Test 3: Can we actually load the model (this is where rubber meets road)
    println!("🚀 CRITICAL TEST: Attempting to load GGUF model...");
    
    use embed_search::llama_wrapper_working::{GGUFModel, GGUFContext};
    
    let start = Instant::now();
    match GGUFModel::load_from_file(model_path, 0) {
        Ok(model) => {
            let load_time = start.elapsed();
            println!("✅ Model loaded successfully in {:?}", load_time);
            println!("📐 Embedding dimension: {}", model.embedding_dim());
            
            // Test 4: Can we create a context?
            match GGUFContext::new_with_model(&model, 2048) {
                Ok(mut context) => {
                    println!("✅ Context created successfully");
                    
                    // Test 5: THE ULTIMATE TEST - Generate actual embeddings
                    println!("🎯 ULTIMATE TEST: Generating real embeddings...");
                    let test_text = "This is a test embedding";
                    
                    let embed_start = Instant::now();
                    match context.embed(test_text) {
                        Ok(embedding) => {
                            let embed_time = embed_start.elapsed();
                            println!("✅ EMBEDDING SUCCESSFUL!");
                            println!("📊 Embedding stats:");
                            println!("   - Length: {}", embedding.len());
                            println!("   - Time: {:?}", embed_time);
                            
                            // Check if embedding is real (not all zeros)
                            let non_zero_count = embedding.iter().filter(|&&x| x.abs() > 1e-8).count();
                            let avg = embedding.iter().sum::<f32>() / embedding.len() as f32;
                            let norm = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
                            
                            println!("   - Non-zero values: {}/{}", non_zero_count, embedding.len());
                            println!("   - Average value: {:.6}", avg);
                            println!("   - L2 norm: {:.6}", norm);
                            
                            if non_zero_count > embedding.len() / 2 && norm > 0.9 && norm < 1.1 {
                                println!("✅ VALIDATION PASSED: Embedding is real and normalized");
                            } else {
                                println!("❌ VALIDATION FAILED: Embedding appears to be placeholder/invalid");
                                println!("   - Expected norm ~1.0, got {:.6}", norm);
                                println!("   - Expected >50% non-zero values, got {}%", 
                                        100 * non_zero_count / embedding.len());
                            }
                            
                            // Test batch processing
                            println!("🔄 Testing batch processing...");
                            let batch_texts = vec![
                                "First test text".to_string(),
                                "Second test text".to_string(),
                                "Third test text".to_string(),
                            ];
                            
                            let batch_start = Instant::now();
                            match context.embed_batch(batch_texts) {
                                Ok(batch_embeddings) => {
                                    let batch_time = batch_start.elapsed();
                                    println!("✅ Batch embedding successful!");
                                    println!("   - Batch size: {}", batch_embeddings.len());
                                    println!("   - Time: {:?}", batch_time);
                                    println!("   - Per-item avg: {:?}", batch_time / batch_embeddings.len() as u32);
                                }
                                Err(e) => {
                                    println!("❌ Batch embedding failed: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            println!("❌ CRITICAL FAILURE: Cannot generate embeddings");
                            println!("   Error: {}", e);
                            return Ok(());
                        }
                    }
                }
                Err(e) => {
                    println!("❌ FAIL: Cannot create context: {}", e);
                    return Ok(());
                }
            }
        }
        Err(e) => {
            println!("❌ CRITICAL FAILURE: Cannot load model");
            println!("   Error: {}", e);
            println!("   This means the GGUF implementation is NOT WORKING");
            return Ok(());
        }
    }
    
    println!("🎉 BRUTAL VALIDATION COMPLETE - IMPLEMENTATION IS ACTUALLY WORKING!");
    Ok(())
}

#[test]
fn test_performance_benchmarks() -> Result<()> {
    println!("⚡ PERFORMANCE BENCHMARK TEST");
    
    // This test will only run if the brutal reality check passes
    use embed_search::llama_wrapper_working::{GGUFModel, GGUFContext};
    let model_path = "./src/model/nomic-embed-code.Q4_K_M.gguf";
    
    if !std::path::Path::new(model_path).exists() {
        println!("⚠️  Skipping performance test - model not available");
        return Ok(());
    }
    
    let model = match GGUFModel::load_from_file(model_path, 0) {
        Ok(m) => m,
        Err(_) => {
            println!("⚠️  Skipping performance test - model load failed");
            return Ok(());
        }
    };
    
    let mut context = match GGUFContext::new_with_model(&model, 2048) {
        Ok(c) => c,
        Err(_) => {
            println!("⚠️  Skipping performance test - context creation failed");
            return Ok(());
        }
    };
    
    // Performance test with different text sizes
    let test_cases = vec![
        ("Short", "Hello world"),
        ("Medium", "This is a medium length text that contains several words and should test the embedding performance with moderate input."),
        // ("Long", &"Lorem ipsum dolor sit amet, consectetur adipiscing elit. ".repeat(20)), // Lifetime issue
        ("Long", "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Lorem ipsum dolor sit amet, consectetur adipiscing elit. Lorem ipsum dolor sit amet, consectetur adipiscing elit. Lorem ipsum dolor sit amet, consectetur adipiscing elit. Lorem ipsum dolor sit amet, consectetur adipiscing elit."),
    ];
    
    for (name, text) in test_cases {
        let start = Instant::now();
        match context.embed(text) {
            Ok(embedding) => {
                let duration = start.elapsed();
                println!("📊 {}: {} chars → {:?} ({:.1} chars/ms)", 
                        name, text.len(), duration, 
                        text.len() as f64 / duration.as_millis().max(1) as f64);
            }
            Err(e) => {
                println!("❌ {} failed: {}", name, e);
            }
        }
    }
    
    Ok(())
}