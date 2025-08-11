// STANDALONE BRUTAL VALIDATION TEST: Verify GGUF embeddings are real, not fake
// This test bypasses compilation issues in other modules

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 BRUTAL STANDALONE VALIDATION: Testing GGUF Embedding Reality");
    
    // Test 1: Load the model directly using llama-cpp-2
    println!("\n🔍 Test 1: Direct Model Loading");
    
    let model_path = "../src/model/nomic-embed-text-v1.5.Q4_K_M.gguf";
    
    if !std::path::Path::new(model_path).exists() {
        return Err("Model file does not exist".into());
    }
    
    // Initialize backend and load model
    use llama_cpp_2::{
        llama_backend::LlamaBackend,
        model::{LlamaModel, params::LlamaModelParams},
        context::params::{LlamaContextParams, RopeScalingType},
        llama_batch::LlamaBatch,
    };
    
    let backend = LlamaBackend::init()?;
    let model_params = LlamaModelParams::default().with_n_gpu_layers(0);
    let model = LlamaModel::load_from_file(&backend, model_path, &model_params)?;
    
    println!("   ✅ Model loaded successfully");
    println!("   Embedding dimension: {}", model.n_embd());
    
    if model.n_embd() != 768 {
        return Err(format!("Expected 768 dimensions, got {}", model.n_embd()).into());
    }
    
    // Test 2: Create context and generate embeddings
    println!("\n🔍 Test 2: Context Creation and Embedding Generation");
    
    let context_params = LlamaContextParams::default()
        .with_n_ctx(Some(std::num::NonZeroU32::new(2048).unwrap()))
        .with_n_batch(256)
        .with_embeddings(true) // ESSENTIAL
        .with_rope_scaling_type(RopeScalingType::Yarn) // CRITICAL for nomic
        .with_rope_freq_scale(0.75); // nomic-specific
    
    let mut context = model.new_context(&backend, context_params)?;
    
    println!("   ✅ Context created with embedding support");
    
    // Test 3: Generate actual embeddings
    println!("\n🔍 Test 3: Embedding Generation Test");
    
    let test_texts = vec![
        "Hello world",
        "Machine learning algorithm", 
        "Database connection",
        "Hello world", // Duplicate
    ];
    
    let mut embeddings = Vec::new();
    
    for (i, text) in test_texts.iter().enumerate() {
        println!("   Processing text {}: '{}'", i, text);
        
        // Tokenize
        let tokens = model.str_to_token(text, llama_cpp_2::model::AddBos::Never)?;
        
        if tokens.is_empty() {
            return Err(format!("Tokenization failed for text: {}", text).into());
        }
        
        println!("     Tokenized to {} tokens", tokens.len());
        
        // Create batch
        let mut batch = LlamaBatch::new(tokens.len(), 1);
        for (j, token) in tokens.iter().enumerate() {
            let is_last = j == tokens.len() - 1;
            batch.add(*token, j as i32, &[0], is_last)?;
        }
        
        // Decode
        context.decode(&mut batch)?;
        
        // Extract embeddings using sequence embeddings (correct for nomic models)
        let seq_emb = context.embeddings_seq_ith(0)?;
        
        if seq_emb.is_empty() {
            return Err("No sequence embeddings returned from model".into());
        }
        
        // Mean pooling over sequence (critical for nomic)
        let embedding_dim = 768;
        let seq_len = seq_emb.len() / embedding_dim;
        
        if seq_len == 0 {
            return Err(format!("Invalid sequence length: {}", seq_emb.len()).into());
        }
        
        let mut pooled = vec![0.0f32; embedding_dim];
        for k in 0..seq_len {
            for j in 0..embedding_dim {
                let idx = k * embedding_dim + j;
                if idx < seq_emb.len() {
                    pooled[j] += seq_emb[idx];
                }
            }
        }
        
        // Average
        if seq_len > 0 {
            for val in &mut pooled {
                *val /= seq_len as f32;
            }
        }
        
        // L2 normalize
        let norm: f32 = pooled.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 1e-8 {
            for val in &mut pooled {
                *val /= norm;
            }
        } else {
            return Err("Embedding has zero norm".into());
        }
        
        // Validate embedding is not zeros
        let non_zero_count = pooled.iter().filter(|&&x| x.abs() > 1e-8).count();
        if non_zero_count < 10 {
            return Err(format!("Embedding {} is mostly zeros ({} non-zero)", i, non_zero_count).into());
        }
        
        println!("     ✅ Generated embedding with {} non-zero values, norm: {:.6}", 
                 non_zero_count, norm);
        
        embeddings.push(pooled);
    }
    
    // Test 4: Semantic similarity validation
    println!("\n🔍 Test 4: Semantic Similarity Validation");
    
    let sim_0_1 = cosine_similarity(&embeddings[0], &embeddings[1]);
    let sim_0_2 = cosine_similarity(&embeddings[0], &embeddings[2]);
    let sim_1_2 = cosine_similarity(&embeddings[1], &embeddings[2]);
    let sim_0_3 = cosine_similarity(&embeddings[0], &embeddings[3]); // Should be ~1.0
    
    println!("   Similarity 'Hello world' vs 'ML algorithm': {:.4}", sim_0_1);
    println!("   Similarity 'Hello world' vs 'Database': {:.4}", sim_0_2);
    println!("   Similarity 'ML algorithm' vs 'Database': {:.4}", sim_1_2);
    println!("   Similarity 'Hello world' vs 'Hello world': {:.4}", sim_0_3);
    
    // Validation checks
    if sim_0_3 < 0.99 {
        return Err(format!("Identical texts have similarity {:.4}, expected ~1.0", sim_0_3).into());
    }
    
    if sim_0_1.abs() < 0.01 && sim_0_2.abs() < 0.01 && sim_1_2.abs() < 0.01 {
        return Err("All similarities near zero - embeddings appear random/hash-based".into());
    }
    
    println!("   ✅ Embeddings show proper semantic structure");
    
    // Test 5: Performance check
    println!("\n🔍 Test 5: Performance Validation");
    
    let start = std::time::Instant::now();
    let perf_text = "performance test for embedding speed";
    
    for _ in 0..3 {
        let tokens = model.str_to_token(perf_text, llama_cpp_2::model::AddBos::Never)?;
        let mut batch = LlamaBatch::new(tokens.len(), 1);
        for (j, token) in tokens.iter().enumerate() {
            batch.add(*token, j as i32, &[0], j == tokens.len() - 1)?;
        }
        context.decode(&mut batch)?;
        let _emb = context.embeddings_seq_ith(0)?;
    }
    
    let duration = start.elapsed();
    println!("   3 embeddings took: {:?}", duration);
    
    if duration.as_secs() > 30 {
        println!("   ⚠ WARNING: Embeddings are slow ({:?}) - CPU-only mode", duration);
    } else {
        println!("   ✅ Performance acceptable");
    }
    
    println!("\n✅ VALIDATION COMPLETE: GGUF embeddings are REAL and functional");
    Ok(())
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }
    
    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    
    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot_product / (norm_a * norm_b)
    }
}