use embed_search::embedding::LazyEmbedder;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔬 Testing LazyEmbedder functionality...");
    
    // Create lazy embedder
    let embedder = LazyEmbedder::new();
    println!("✅ LazyEmbedder created");
    
    // Test embedding generation
    let test_text = "function calculateSum(a, b) { return a + b; }";
    println!("🔄 Attempting to generate embedding for: {}", test_text);
    
    match embedder.embed(test_text).await {
        Ok(embedding) => {
            println!("✅ Embedding generated successfully!");
            println!("   Dimensions: {}", embedding.len());
            println!("   First 10 values: {:?}", &embedding[..10.min(embedding.len())]);
            
            // Validate embedding
            let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
            println!("   L2 norm: {:.6}", norm);
            
            let has_nan = embedding.iter().any(|x| x.is_nan());
            let has_inf = embedding.iter().any(|x| x.is_infinite());
            println!("   Has NaN: {}, Has Inf: {}", has_nan, has_inf);
            
            if embedding.len() == 768 && !has_nan && !has_inf && norm > 0.5 {
                println!("🎉 EMBEDDING SYSTEM WORKS CORRECTLY!");
            } else {
                println!("❌ Embedding has issues - dimensions: {}, norm: {}", embedding.len(), norm);
            }
        }
        Err(e) => {
            println!("❌ Failed to generate embedding: {}", e);
            println!("   Error details: {:?}", e);
        }
    }
    
    Ok(())
}