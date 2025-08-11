// examples/check_gpu.rs - Verify GPU acceleration configuration

fn main() {
    println!("🔍 Checking GPU Acceleration Configuration\n");
    
    // Check CUDA
    #[cfg(feature = "cuda")]
    {
        println!("✅ CUDA: Available");
        if let Ok(arch) = std::env::var("CUDA_ARCHITECTURES") {
            println!("   Architecture: {}", arch);
        }
        if let Ok(path) = std::env::var("CUDA_PATH") {
            println!("   CUDA Path: {}", path);
        }
    }
    #[cfg(not(feature = "cuda"))]
    {
        println!("❌ CUDA: Not available");
    }
    
    // Check Metal
    #[cfg(feature = "metal")]
    {
        println!("✅ Metal: Available (macOS)");
    }
    #[cfg(not(feature = "metal"))]
    {
        println!("❌ Metal: Not available");
    }
    
    // Check ROCm/HIPBlas
    #[cfg(feature = "hipblas")]
    {
        println!("✅ ROCm: Available");
        if let Ok(path) = std::env::var("ROCM_PATH") {
            println!("   ROCm Path: {}", path);
        }
    }
    #[cfg(not(feature = "hipblas"))]
    {
        println!("❌ ROCm: Not available");
    }
    
    // Show thread configuration
    if let Ok(threads) = std::env::var("LLAMA_THREADS") {
        println!("\n🔧 Thread Configuration: {}", threads);
    } else {
        let num_cpus = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4);
        println!("\n🔧 Thread Configuration: {} (auto-detected)", num_cpus);
    }
    
    // Determine selected acceleration
    println!("\n🚀 Selected Acceleration:");
    #[cfg(feature = "cuda")]
    {
        println!("   Using CUDA for GPU acceleration");
    }
    #[cfg(all(feature = "metal", not(feature = "cuda")))]
    {
        println!("   Using Metal for GPU acceleration");
    }
    #[cfg(all(feature = "hipblas", not(feature = "cuda"), not(feature = "metal")))]
    {
        println!("   Using ROCm for GPU acceleration");
    }
    #[cfg(all(not(feature = "cuda"), not(feature = "metal"), not(feature = "hipblas")))]
    {
        println!("   Using CPU only (no GPU acceleration)");
    }
    
    // Check if llama.cpp was built from source
    if std::env::var("BUILD_LLAMA_FROM_SOURCE").is_ok() {
        println!("\n📦 llama.cpp: Built from source");
    } else if let Ok(path) = std::env::var("LLAMA_CPP_PATH") {
        println!("\n📦 llama.cpp: Using custom installation at {}", path);
    } else {
        println!("\n📦 llama.cpp: Using system/bundled version");
    }
    
    println!("\n✨ Build configuration check complete!");
}