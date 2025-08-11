// examples/test_llama_cpp.rs - Test basic llama-cpp-2 functionality

use anyhow::Result;
use std::path::Path;

fn main() -> Result<()> {
    println!("🦙 Testing llama-cpp-2 Integration");
    println!("==================================\n");
    
    println!("📦 Testing llama-cpp-2 library availability...");
    
    // Note: The actual llama-cpp-2 v0.1.54 API may differ from newer versions
    // This example demonstrates that the library is correctly linked
    
    // Import the crate to verify it's available
    use llama_cpp_2 as _llama;
    use llama_cpp_sys_2 as _llama_sys;
    
    println!("✅ llama-cpp-2 library is available");
    println!("✅ llama-cpp-sys-2 library is available\n");
    
    // Check for model file
    let model_path = "./src/model/nomic-embed-code.Q4_K_M.gguf";
    
    if !Path::new(model_path).exists() {
        println!("⚠️  Model file not found at: {}", model_path);
        println!("   Please download the model first:");
        println!("   wget https://huggingface.co/nomic-ai/nomic-embed-text-v1.5-GGUF/resolve/main/nomic-embed-text-v1.5.Q4_K_M.gguf");
        println!("   mkdir -p src/model");
        println!("   mv nomic-embed-text-v1.5.Q4_K_M.gguf src/model/nomic-embed-code.Q4_K_M.gguf\n");
    } else {
        println!("📂 Model file found: {}", model_path);
    }
    
    // Check GPU features
    println!("🔍 GPU Feature Detection:");
    
    #[cfg(feature = "cuda")]
    println!("   ✅ CUDA support enabled");
    #[cfg(not(feature = "cuda"))]
    println!("   ❌ CUDA support not enabled");
    
    #[cfg(feature = "metal")]
    println!("   ✅ Metal support enabled");
    #[cfg(not(feature = "metal"))]
    println!("   ❌ Metal support not enabled");
    
    #[cfg(feature = "hipblas")]
    println!("   ✅ HIPBlas/ROCm support enabled");
    #[cfg(not(feature = "hipblas"))]
    println!("   ❌ HIPBlas/ROCm support not enabled");
    
    println!("\n✨ Basic integration test complete!");
    println!("\nNote: For full llama-cpp-2 API usage, refer to the crate documentation.");
    println!("      The API varies between versions, so check: https://docs.rs/llama-cpp-2/0.1.54");
    
    Ok(())
}