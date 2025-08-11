// build.rs - Build configuration for llama-cpp-2 integration
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=BUILD_LLAMA_FROM_SOURCE");
    println!("cargo:rerun-if-env-changed=LLAMA_CPP_PATH");
    
    // FORCE CPU-ONLY CONFIGURATION
    println!("cargo:warning=Building with CPU-ONLY configuration - GPU support disabled");
    
    // Configure CPU-optimized system libraries
    configure_cpu_only_libraries();
    
    // Build or link llama.cpp with CPU-only flags
    if env::var("BUILD_LLAMA_FROM_SOURCE").is_ok() {
        build_llama_cpp_cpu_only();
    } else {
        link_prebuilt_llama_cpp();
    }
    
    // Set CPU optimization flags
    set_cpu_optimization_flags();
}

#[derive(Debug)]
struct CpuConfig {
    thread_count: usize,
    use_avx2: bool,
    use_fma: bool,
}

// CPU-ONLY: GPU detection functions removed - force CPU configuration
fn get_cpu_config() -> CpuConfig {
    let cpu_count = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);
        
    println!("cargo:warning=CPU-only build configured with {} threads", cpu_count);
    println!("cargo:warning=GPU acceleration DISABLED - pure CPU inference");
    
    CpuConfig {
        thread_count: cpu_count,
        use_avx2: cfg!(target_feature = "avx2"),
        use_fma: cfg!(target_feature = "fma"),
    }
}

fn detect_cuda_version(cuda_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let nvcc_path = Path::new(cuda_path).join("bin").join("nvcc");
    let output = Command::new(nvcc_path)
        .arg("--version")
        .output()?;
    
    let version_output = String::from_utf8_lossy(&output.stdout);
    // Parse version from nvcc output
    if let Some(line) = version_output.lines().find(|l| l.contains("release")) {
        if let Some(version) = line.split("release").nth(1) {
            return Ok(version.trim().split(',').next().unwrap_or("").to_string());
        }
    }
    
    Ok("unknown".to_string())
}

fn detect_cuda_compute_capability() -> Option<String> {
    // Try to detect compute capability using nvidia-smi
    if let Ok(output) = Command::new("nvidia-smi")
        .arg("--query-gpu=compute_cap")
        .arg("--format=csv,noheader")
        .output()
    {
        let cap = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !cap.is_empty() {
            // Convert to CUDA arch format (e.g., "8.6" -> "sm_86")
            let arch = cap.replace(".", "");
            return Some(format!("sm_{}", arch));
        }
    }
    
    // Default to common architectures
    Some("sm_70;sm_75;sm_80;sm_86;sm_89;sm_90".to_string())
}

fn detect_rocm_version(rocm_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let version_file = Path::new(rocm_path).join(".info").join("version");
    if version_file.exists() {
        let version = std::fs::read_to_string(version_file)?;
        return Ok(version.trim().to_string());
    }
    
    Ok("unknown".to_string())
}

fn configure_cpu_only(config: &CpuConfig) {
    // Force CPU-only configuration
    println!("cargo:rustc-cfg=feature=\"cpu-only\"");
    println!("cargo:rustc-env=LLAMA_CPU_ONLY=1");
    println!("cargo:rustc-env=LLAMA_CUDA=0");
    println!("cargo:rustc-env=LLAMA_METAL=0");
    println!("cargo:rustc-env=LLAMA_HIPBLAS=0");
    
    // Set CPU thread configuration
    println!("cargo:rustc-env=LLAMA_THREADS={}", config.thread_count);
    
    // Enable CPU optimizations
    if config.use_avx2 {
        println!("cargo:rustc-cfg=feature=\"avx2\"");
        println!("cargo:rustc-env=LLAMA_AVX2=1");
    }
    
    if config.use_fma {
        println!("cargo:rustc-cfg=feature=\"fma\"");
        println!("cargo:rustc-env=LLAMA_FMA=1");
    }
    
    println!("cargo:warning=CPU-only configuration active with {} threads", config.thread_count);
}

fn configure_cpu_only_libraries() {
    // Platform-specific libraries
    #[cfg(target_os = "linux")]
    {
        println!("cargo:rustc-link-lib=dylib=stdc++");
        println!("cargo:rustc-link-lib=dylib=pthread");
        println!("cargo:rustc-link-lib=dylib=m");
        println!("cargo:rustc-link-lib=dylib=dl");
    }
    
    #[cfg(target_os = "macos")]
    {
        println!("cargo:rustc-link-lib=dylib=c++");
        println!("cargo:rustc-link-lib=dylib=pthread");
    }
    
    #[cfg(target_os = "windows")]
    {
        println!("cargo:rustc-link-lib=dylib=msvcrt");
    }
}

fn build_llama_cpp_cpu_only() {
    println!("cargo:warning=Building llama.cpp from source with CPU-ONLY optimizations...");
    
    let mut cmake_config = cmake::Config::new("llama.cpp");
    
    // Basic configuration
    cmake_config
        .define("BUILD_SHARED_LIBS", "ON")
        .define("LLAMA_BUILD_TESTS", "OFF")
        .define("LLAMA_BUILD_EXAMPLES", "OFF")
        .define("LLAMA_BUILD_SERVER", "OFF");
    
    // FORCE CPU-ONLY: Explicitly disable all GPU acceleration
    cmake_config
        .define("LLAMA_CUDA", "OFF")
        .define("LLAMA_METAL", "OFF")
        .define("LLAMA_HIPBLAS", "OFF")
        .define("LLAMA_OPENCL", "OFF")
        .define("LLAMA_VULKAN", "OFF")
        .define("LLAMA_KOMPUTE", "OFF");
    
    // CPU optimization flags
    #[cfg(not(debug_assertions))]
    {
        cmake_config
            .define("CMAKE_BUILD_TYPE", "Release")
            .define("LLAMA_FAST", "ON")
            .define("LLAMA_LTO", "ON")
            .define("CMAKE_CXX_FLAGS_RELEASE", "-O3 -DNDEBUG -march=native -mtune=native");
    }
    
    #[cfg(debug_assertions)]
    {
        cmake_config.define("CMAKE_BUILD_TYPE", "Debug");
    }
    
    // CPU-specific optimizations
    cmake_config
        .define("LLAMA_NATIVE", "ON")
        .define("LLAMA_F16C", "ON")
        .define("LLAMA_FMA", "ON")
        .define("LLAMA_AVX", "ON")
        .define("LLAMA_AVX2", "ON")
        .define("LLAMA_AVX512", "OFF"); // Often slower than AVX2
    
    // Set thread count for compilation
    let thread_count = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);
    cmake_config.define("CMAKE_BUILD_PARALLEL_LEVEL", thread_count.to_string());
    
    // Build llama.cpp
    let dst = cmake_config.build();
    
    // Set library search path
    println!("cargo:rustc-link-search=native={}/lib", dst.display());
    println!("cargo:rustc-link-search=native={}/lib64", dst.display());
    
    // Link the library
    println!("cargo:rustc-link-lib=llama");
    
    println!("cargo:warning=CPU-optimized llama.cpp built successfully at: {}", dst.display());
}

fn link_prebuilt_llama_cpp() {
    // Check for custom llama.cpp path
    if let Ok(llama_path) = env::var("LLAMA_CPP_PATH") {
        println!("cargo:warning=Using prebuilt llama.cpp from: {}", llama_path);
        let lib_path = Path::new(&llama_path).join("lib");
        if lib_path.exists() {
            println!("cargo:rustc-link-search=native={}", lib_path.display());
        } else {
            println!("cargo:rustc-link-search=native={}", llama_path);
        }
    } else {
        // Use system-wide installation or llama-cpp-2's bundled version
        println!("cargo:warning=Using system llama.cpp or llama-cpp-2 bundled version");
    }
    
    // Link llama library
    println!("cargo:rustc-link-lib=llama");
}

fn set_cpu_optimization_flags() {
    // Aggressive CPU optimization flags
    if cfg!(target_arch = "x86_64") {
        println!("cargo:rustc-env=CXXFLAGS=-O3 -march=native -mtune=native -ffast-math -DNDEBUG");
        println!("cargo:rustc-env=CFLAGS=-O3 -march=native -mtune=native -ffast-math -DNDEBUG");
    } else if cfg!(target_arch = "aarch64") {
        println!("cargo:rustc-env=CXXFLAGS=-O3 -march=native -mtune=native -ffast-math -DNDEBUG");
        println!("cargo:rustc-env=CFLAGS=-O3 -march=native -mtune=native -ffast-math -DNDEBUG");
    }
    
    // Maximum CPU optimization in release builds
    #[cfg(not(debug_assertions))]
    {
        println!("cargo:rustc-env=CARGO_PROFILE_RELEASE_LTO=fat");
        println!("cargo:rustc-env=CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1");
        println!("cargo:rustc-env=CARGO_PROFILE_RELEASE_PANIC=abort");
        println!("cargo:rustc-link-arg=-Wl,--strip-all");
    }
    
    // CPU thread configuration optimized for inference
    let cpu_count = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);
    
    // Use 75% of CPU cores for inference, reserve 25% for system
    let optimal_threads = std::cmp::max(1, (cpu_count * 3) / 4);
    
    if let Ok(threads) = env::var("LLAMA_THREADS") {
        println!("cargo:rustc-env=LLAMA_THREADS={}", threads);
    } else {
        println!("cargo:rustc-env=LLAMA_THREADS={}", optimal_threads);
    }
    
    // Set CPU affinity and NUMA optimization flags
    println!("cargo:rustc-env=OMP_NUM_THREADS={}", optimal_threads);
    println!("cargo:rustc-env=MKL_NUM_THREADS={}", optimal_threads);
    println!("cargo:rustc-env=OPENBLAS_NUM_THREADS={}", optimal_threads);
    
    println!("cargo:warning=CPU-optimized build configuration complete ({} threads)", optimal_threads);
}