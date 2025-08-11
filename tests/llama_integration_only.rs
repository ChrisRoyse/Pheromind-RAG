// Test just the llama-cpp-2 integration without dependencies that cause issues
use anyhow::Result;

#[test]
fn test_llama_types_exist() -> Result<()> {
    // Test that we can import the llama wrapper types
    use embed_search::llama_wrapper::{GGUFModel, GGUFContext};
    use embed_search::llama_bindings;
    
    println!("llama-cpp-2 integration types are accessible");
    Ok(())
}

#[test]
fn test_llama_dependencies() -> Result<()> {
    // Test that llama-cpp-2 dependencies are available
    // This will fail at compile time if llama-cpp-2 isn't properly configured
    use llama_cpp_2::{
        context::{LlamaContext, params::LlamaContextParams},
        llama::LlamaModel,
        model::LlamaModelParams,
    };
    
    println!("llama-cpp-2 dependencies are available");
    Ok(())
}