/// BRUTAL HONESTY TEST: Verify actual embedding functionality
#[cfg(test)]
mod embedding_reality_tests {
    use std::time::Instant;

    #[cfg(feature = "ml")]
    #[tokio::test]
    async fn test_nomic_embedder_reality() {
        use crate::embedding::nomic::NomicEmbedder;
        
        println!("🔍 TESTING NOMIC EMBEDDER REALITY");
        
        // Get the embedder
        let start = Instant::now();
        let embedder = match NomicEmbedder::get_global().await {
            Ok(e) => {
                println!("✅ Model loading time: {:?}", start.elapsed());
                e
            },
            Err(e) => {
                panic!("❌ FATAL: Cannot load NomicEmbedder: {}", e);
            }
        };
        
        // Test basic embedding
        let test_text = "fn hello_world() { println!(\"Hello, world!\"); }";
        let start = Instant::now();
        let embedding = match embedder.embed(test_text) {
            Ok(e) => {
                println!("✅ Embedding generation time: {:?}", start.elapsed());
                e
            },
            Err(e) => {
                panic!("❌ FATAL: Cannot generate embedding: {}", e);
            }
        };
        
        println!("📊 Embedding Analysis:");
        println!("   - Dimensions: {}", embedding.len());
        println!("   - First 5 values: {:?}", &embedding[..5]);
        
        let norm = embedding.iter().map(|x| x*x).sum::<f32>().sqrt();
        println!("   - L2 norm: {:.6}", norm);
        
        // BRUTAL VALIDATION
        assert_eq!(embedding.len(), 768, "WRONG DIMENSIONS: Expected 768, got {}", embedding.len());
        assert!((norm - 1.0).abs() < 0.01, "NOT NORMALIZED: norm = {:.6}", norm);
        
        // Test consistency
        let embedding2 = embedder.embed(test_text).expect("Second embedding failed");
        let consistency_diff: f32 = embedding.iter().zip(embedding2.iter()).map(|(a, b)| (a - b).abs()).sum();
        assert!(consistency_diff < 1e-5, "INCONSISTENT: Same input produces different outputs, diff = {:.6}", consistency_diff);
        
        // Test different inputs produce different outputs
        let different_text = "class MyClass: pass";
        let embedding3 = embedder.embed(different_text).expect("Different embedding failed");
        let semantic_diff: f32 = embedding.iter().zip(embedding3.iter()).map(|(a, b)| (a - b).abs()).sum();
        assert!(semantic_diff > 0.1, "SUSPICIOUS: Different inputs produce too similar outputs, diff = {:.6}", semantic_diff);
        
        println!("✅ ALL REALITY CHECKS PASSED - EMBEDDINGS ARE REAL");
    }
    
    #[cfg(feature = "ml")]
    #[tokio::test]
    async fn test_lazy_embedder_reality() {
        use crate::embedding::lazy_embedder::LazyEmbedder;
        
        println!("🔍 TESTING LAZY EMBEDDER REALITY");
        
        let lazy = LazyEmbedder::new();
        assert!(!lazy.is_initialized(), "LazyEmbedder should not be initialized initially");
        
        let test_text = "async fn test() { println!(\"test\"); }";
        let embedding = lazy.embed(test_text).await.expect("LazyEmbedder embed failed");
        
        assert!(lazy.is_initialized(), "LazyEmbedder should be initialized after first use");
        assert_eq!(embedding.len(), 768, "LazyEmbedder wrong dimensions");
        
        println!("✅ LAZY EMBEDDER REALITY CHECK PASSED");
    }
    
    #[cfg(feature = "ml")]
    #[tokio::test]
    async fn test_semantic_quality_reality() {
        use crate::embedding::nomic::NomicEmbedder;
        
        println!("🔍 TESTING SEMANTIC QUALITY REALITY");
        
        let embedder = NomicEmbedder::get_global().await.expect("Failed to get embedder");
        
        // Test semantic understanding
        let code1 = "def add(a, b): return a + b";
        let code2 = "function add(a, b) { return a + b; }";
        let unrelated = "import matplotlib.pyplot as plt";
        
        let emb1 = embedder.embed(code1).expect("Embedding 1 failed");
        let emb2 = embedder.embed(code2).expect("Embedding 2 failed");
        let emb3 = embedder.embed(unrelated).expect("Embedding 3 failed");
        
        // Calculate cosine similarity
        let similarity_12 = cosine_similarity(&emb1, &emb2);
        let similarity_13 = cosine_similarity(&emb1, &emb3);
        
        println!("   - Similar code similarity: {:.4}", similarity_12);
        println!("   - Different code similarity: {:.4}", similarity_13);
        
        assert!(similarity_12 > similarity_13, 
               "SEMANTIC UNDERSTANDING BROKEN: similar code ({:.4}) <= different code ({:.4})", 
               similarity_12, similarity_13);
        
        if similarity_12 < 0.3 {
            println!("⚠️  WARNING: Low semantic similarity for similar code: {:.4}", similarity_12);
        }
        
        println!("✅ SEMANTIC QUALITY REALITY CHECK PASSED");
    }
    
    #[cfg(not(feature = "ml"))]
    #[test]
    fn test_ml_features_disabled() {
        println!("❌ ML FEATURES ARE DISABLED");
        println!("The embedding system is completely non-functional without ML features");
        println!("LazyEmbedder will return 'ML features are not enabled' errors");
        
        use crate::embedding::lazy_embedder::LazyEmbedder;
        
        let lazy = LazyEmbedder::new();
        assert!(!lazy.is_initialized());
        
        // This should fail with ML features disabled
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(lazy.embed("test"));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("ML features are not enabled"));
        
        println!("✅ CONFIRMED: ML features are properly disabled");
    }

    fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
        dot / (norm_a * norm_b)
    }
}