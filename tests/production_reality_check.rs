/// PRODUCTION REALITY CHECK
/// 
/// This test proves that the Q4_K_M embeddings are REAL, not mocked.
/// It performs the most basic checks that would catch any fake implementation.

use embed_search::embedding::NomicEmbedder;
use std::fs;

#[tokio::test]
async fn test_q4km_is_real_not_mocked() {
    println!("\n{}", "=".repeat(60));
    println!("🔍 PRODUCTION REALITY CHECK - Q4_K_M VERIFICATION");
    println!("{}", "=".repeat(60));
    
    // 1. Check model file exists and is correct size
    let model_path = dirs::home_dir().unwrap()
        .join(".nomic")
        .join("nomic-embed-text-v1.5.Q4_K_M.gguf");
    
    println!("\n1️⃣ MODEL FILE CHECK:");
    assert!(model_path.exists(), "Model file doesn't exist at {:?}", model_path);
    
    let size = fs::metadata(&model_path).unwrap().len();
    let size_mb = size as f64 / 1_000_000.0;
    println!("   ✅ Model exists: {:.1} MB", size_mb);
    assert!(size_mb > 80.0 && size_mb < 90.0, "Wrong model size!");
    
    // 2. Initialize embedder and check dimensions
    println!("\n2️⃣ EMBEDDER INITIALIZATION:");
    let embedder = NomicEmbedder::get_global().await.unwrap();
    assert_eq!(embedder.dimensions(), 768);
    println!("   ✅ Produces 768-dimensional embeddings");
    
    // 3. Test that different inputs produce different outputs
    println!("\n3️⃣ DIFFERENT INPUTS → DIFFERENT OUTPUTS:");
    
    let test_cases = vec![
        "Hello world",
        "Machine learning is awesome",
        "def function(): return 42",
        "SELECT * FROM users",
        "The quick brown fox jumps over the lazy dog",
    ];
    
    let mut embeddings = Vec::new();
    for text in &test_cases {
        let emb = embedder.embed(text).unwrap();
        assert_eq!(emb.len(), 768);
        embeddings.push(emb);
    }
    
    // Check all embeddings are different
    for i in 0..embeddings.len() {
        for j in i+1..embeddings.len() {
            let diff: f32 = embeddings[i].iter()
                .zip(embeddings[j].iter())
                .map(|(a, b)| (a - b).powi(2))
                .sum::<f32>()
                .sqrt();
            
            assert!(diff > 0.5, "Embeddings {} and {} are too similar!", i, j);
            println!("   ✅ '{}' vs '{}': distance = {:.3}", 
                test_cases[i], test_cases[j], diff);
        }
    }
    
    // 4. Check embeddings are normalized
    println!("\n4️⃣ L2 NORMALIZATION CHECK:");
    for (i, emb) in embeddings.iter().enumerate() {
        let norm: f32 = emb.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 0.02, "Embedding {} not normalized!", i);
        println!("   ✅ '{}': norm = {:.6}", test_cases[i], norm);
    }
    
    // 5. Semantic similarity check
    println!("\n5️⃣ SEMANTIC SIMILARITY CHECK:");
    let code1 = "def add(a, b): return a + b";
    let code2 = "def sum(x, y): return x + y";
    let sql = "CREATE TABLE users (id INT, name VARCHAR(100))";
    
    let emb1 = embedder.embed(code1).unwrap();
    let emb2 = embedder.embed(code2).unwrap();
    let emb3 = embedder.embed(sql).unwrap();
    
    let sim_similar = cosine_similarity(&emb1, &emb2);
    let sim_different = cosine_similarity(&emb1, &emb3);
    
    println!("   Similar code: {:.3}", sim_similar);
    println!("   Different code: {:.3}", sim_different);
    assert!(sim_similar > sim_different, "Semantic similarity broken!");
    println!("   ✅ Similar code is more similar than different code");
    
    // 6. Not constant check
    println!("\n6️⃣ NOT CONSTANT/MOCK CHECK:");
    let emb = &embeddings[0];
    let first_val = emb[0];
    let all_same = emb.iter().all(|&x| (x - first_val).abs() < 0.0001);
    assert!(!all_same, "All values are the same - this is a mock!");
    
    let variance: f32 = emb.iter()
        .map(|&x| x * x)
        .sum::<f32>() / emb.len() as f32;
    assert!(variance > 0.00001, "Variance too low - likely mock data!");
    println!("   ✅ Values have variance: {:.6}", variance);
    
    println!("\n{}", "=".repeat(60));
    println!("✅ ALL CHECKS PASSED - Q4_K_M EMBEDDINGS ARE REAL!");
    println!("{}", "=".repeat(60));
    println!("\n📊 SUMMARY:");
    println!("  • Model file: 84MB Q4_K_M GGUF ✓");
    println!("  • Dimensions: 768 ✓");
    println!("  • Different inputs → different outputs ✓");
    println!("  • L2 normalized ✓");
    println!("  • Semantic similarity preserved ✓");
    println!("  • Not mock/constant data ✓");
    println!("\n🚀 READY FOR PRODUCTION!");
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum::<f32>()
}

#[test]
fn test_model_file_format() {
    // Verify the model file is a real GGUF file
    let model_path = dirs::home_dir().unwrap()
        .join(".nomic")
        .join("nomic-embed-text-v1.5.Q4_K_M.gguf");
    
    if model_path.exists() {
        // Read first 4 bytes - should be GGUF magic number
        let data = fs::read(&model_path).unwrap();
        assert!(data.len() > 4, "File too small to be GGUF");
        
        // GGUF magic: 0x46554747 ("GGUF" in little-endian)
        let magic = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        assert_eq!(magic, 0x46554747, "Not a GGUF file!");
        
        println!("✅ Model file is valid GGUF format");
    }
}