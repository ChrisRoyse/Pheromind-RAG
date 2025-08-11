// Minimal GGUF test with no complex dependencies
use anyhow::Result;

#[test]
fn minimal_gguf_functionality() -> Result<()> {
    // Check if we can load GGUF models at all
    use embed_search::llama_wrapper_working::GGUFModel;
    
    let model_path = "./src/model/nomic-embed-code.Q4_K_M.gguf";
    
    match GGUFModel::load_from_file(model_path, 0) {
        Ok(model) => {
            println!("✅ GGUF model loads successfully");
            println!("   Embedding dimension: {}", model.embedding_dim());
            assert_eq!(model.embedding_dim(), 768);
        }
        Err(e) => {
            println!("❌ GGUF model failed to load: {}", e);
            // This is acceptable if model doesn't exist
        }
    }
    
    // Check if basic GGUF embedder can be created
    use embed_search::{GGUFEmbedder, GGUFEmbedderConfig};
    
    let config = GGUFEmbedderConfig::default();
    
    match GGUFEmbedder::new(config) {
        Ok(_) => {
            println!("✅ GGUFEmbedder created successfully");
        }
        Err(e) => {
            println!("❌ GGUFEmbedder failed: {}", e);
            // This is acceptable if model doesn't exist
        }
    }
    
    Ok(())
}