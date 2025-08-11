use embed_search::gguf_embedder::{GGUFEmbedder, GGUFEmbedderConfig};
use embed_search::embedding_prefixes::EmbeddingTask;
use anyhow::Result;

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    dot_product  // Already normalized vectors, so just dot product
}

fn main() -> Result<()> {
    println!("🧠 BRUTAL SEMANTIC VALIDATION TEST");
    println!("=====================================\n");

    let mut config = GGUFEmbedderConfig::default();
    config.model_path = "./src/model/nomic-embed-text-v1.5.Q4_K_M.gguf".to_string();
    config.batch_size = 1;
    config.context_size = 2048;

    let embedder = GGUFEmbedder::new(config)?;

    // Test semantic similarity with clear cases
    let texts = [
        "The cat sat on the mat",           // 0: cat sentence
        "A feline was resting on the rug", // 1: similar to cat (should be HIGH similarity)
        "Dog barked loudly in the yard",   // 2: different animal (should be MEDIUM)
        "Car engine roared on highway",    // 3: completely different (should be LOW)
        "Programming in Python language",  // 4: tech topic (should be LOW to animals)
    ];

    println!("📊 Generating embeddings for semantic test:");
    let mut embeddings = Vec::new();
    
    for (i, text) in texts.iter().enumerate() {
        let emb = embedder.embed(text, EmbeddingTask::SearchDocument)?;
        println!("   Text {}: '{}'", i + 1, text);
        println!("     Dimension: {}, First 3 values: [{:.4}, {:.4}, {:.4}]", 
                emb.len(), emb[0], emb[1], emb[2]);
        embeddings.push(emb);
    }

    println!("\n🔍 CRITICAL SEMANTIC ANALYSIS:");
    
    // Most important test: semantically similar vs different
    let cat_feline_sim = cosine_similarity(&embeddings[0], &embeddings[1]); 
    let cat_car_sim = cosine_similarity(&embeddings[0], &embeddings[3]);
    let cat_prog_sim = cosine_similarity(&embeddings[0], &embeddings[4]);

    println!("   Cat vs Feline similarity: {:.4} (MUST be >0.6 for real embeddings)", cat_feline_sim);
    println!("   Cat vs Car similarity: {:.4} (MUST be <0.5 for real embeddings)", cat_car_sim);  
    println!("   Cat vs Programming: {:.4} (MUST be <0.4 for real embeddings)", cat_prog_sim);

    // Hash-based detection: if all similarities are similar, it's likely hash-based
    let all_sims = vec![cat_feline_sim, cat_car_sim, cat_prog_sim];
    let sim_variance = {
        let mean = all_sims.iter().sum::<f32>() / all_sims.len() as f32;
        all_sims.iter().map(|x| (x - mean).powi(2)).sum::<f32>() / all_sims.len() as f32
    };
    
    println!("   Similarity variance: {:.6} (>0.01 indicates semantic)", sim_variance);

    // BRUTAL VERDICT
    println!("\n🎯 FINAL VERDICT:");
    
    let is_semantic = cat_feline_sim > 0.6 && cat_car_sim < 0.5 && sim_variance > 0.01;
    let is_hash_based = sim_variance < 0.005 || (cat_feline_sim - cat_car_sim).abs() < 0.1;

    if is_semantic && !is_hash_based {
        println!("✅ REAL SEMANTIC EMBEDDINGS CONFIRMED");
        println!("   ✓ Similar concepts show high similarity");
        println!("   ✓ Different concepts show low similarity");
        println!("   ✓ Sufficient variance indicates semantic understanding");
        println!("   ✓ This is NOT hash-based fallback");
        println!("\n🏆 SYSTEM PASSES: Using genuine GGUF embeddings");
    } else if is_hash_based {
        println!("❌ HASH-BASED FALLBACK DETECTED");
        println!("   ✗ All similarities are too uniform");
        println!("   ✗ No semantic distinction between concepts");
        println!("   ✗ System is NOT using real GGUF model");
        println!("\n💀 SYSTEM FAILS: Using fake embeddings");
    } else {
        println!("⚠️  UNCLEAR RESULTS");
        println!("   ? Embeddings generated but unclear if semantic");
        println!("   ? May be partial implementation");
        println!("\n🤔 SYSTEM STATUS: Inconclusive");
    }

    Ok(())
}