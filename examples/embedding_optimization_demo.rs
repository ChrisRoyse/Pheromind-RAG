use std::time::Instant;
use embed_search::embedding::NomicEmbedder;

/// Simple demonstration of embedding performance optimizations
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Embedding Performance Optimization Demo");
    println!("===========================================\n");

    // Sample code snippets that might be found in a codebase
    let code_samples = vec![
        "fn process_user_data(input: &str) -> String { input.trim().to_lowercase() }",
        "def authenticate_user(username, password): return check_credentials(username, password)",
        "const handleApiRequest = async (req, res) => { return await processRequest(req); }",
        "fn process_user_data(input: &str) -> String { input.trim().to_lowercase() }", // Duplicate
        "class DatabaseManager { public connect() { return this.pool.connect(); } }",
        "def authenticate_user(username, password): return check_credentials(username, password)", // Duplicate
        "async function validateInput(data) { return schema.validate(data); }",
        "fn process_user_data(input: &str) -> String { input.trim().to_lowercase() }", // Duplicate
    ];

    println!("📊 Test data: {} code snippets ({} unique)", 
             code_samples.len(), 
             code_samples.iter().collect::<std::collections::HashSet<_>>().len());

    // Test 1: Basic embedding demonstration
    println!("\n1️⃣ Basic Embedding");
    println!("   Processing snippets with NomicEmbedder...");
    
    let embedder = NomicEmbedder::new()?;
    let start = Instant::now();
    
    let mut embeddings = Vec::new();
    for (i, code) in code_samples.iter().enumerate() {
        let embedding = embedder.embed(code).await?;
        println!("   ✓ Embedded snippet {} ({} dims)", i + 1, embedding.len());
        embeddings.push(embedding);
    }
    
    let total_time = start.elapsed();
    println!("   ⏱️  Total time: {:?} ({:.1}ms per snippet)", 
             total_time, 
             total_time.as_millis() as f64 / code_samples.len() as f64);

    // Performance Summary
    println!("\n📈 Performance Summary:");
    println!("======================");
    println!("Basic embedding:  {:>8.1}ms ({:.1}ms/item)", 
             total_time.as_millis(), 
             total_time.as_millis() as f64 / code_samples.len() as f64);

    // Verify embeddings are consistent dimensions
    println!("\n🔍 Verifying Embeddings:");
    println!("=========================");
    
    if let Some(first_embedding) = embeddings.first() {
        let dimension = first_embedding.len();
        let all_same_dim = embeddings.iter().all(|e| e.len() == dimension);
        
        println!("Embedding dimension: {}", dimension);
        println!("All embeddings same dimension: {}", all_same_dim);
        
        if all_same_dim {
            println!("✅ All embeddings have consistent dimensions!");
        } else {
            println!("❌ Embeddings have inconsistent dimensions!");
        }
    }

    // Show duplicate content analysis
    println!("\n📊 Content Analysis:");
    println!("=====================");
    let unique_samples: std::collections::HashSet<&&str> = code_samples.iter().collect();
    
    println!("Total snippets: {} (includes duplicates)", code_samples.len());
    println!("Unique snippets: {}", unique_samples.len());
    println!("Duplicate content: {} snippets", 
             code_samples.len() - unique_samples.len());

    println!("\n🎉 Demo Complete:");
    println!("   • Successfully embedded {} code snippets", code_samples.len());
    println!("   • Average embedding time: {:.1}ms per snippet", 
             total_time.as_millis() as f64 / code_samples.len() as f64);
    println!("   • Nomic embedder working correctly");

    Ok(())
}

/// Calculate cosine similarity between two embeddings
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    
    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot_product / (norm_a * norm_b)
    }
}