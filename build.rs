// build.rs - Build configuration for llama-cpp-2 integration
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=CUDA_PATH");
    println!("cargo:rerun-if-env-changed=ROCM_PATH");
    println!("cargo:rerun-if-env-changed=BUILD_LLAMA_FROM_SOURCE");
    println!("cargo:rerun-if-env-changed=LLAMA_CPP_PATH");
    
    // Detect and configure GPU acceleration
    let gpu_config = detect_gpu_configuration();
    configure_gpu_acceleration(&gpu_config);
    
    // Configure system libraries
    configure_system_libraries();
    
    // Build or link llama.cpp
    if env::var("BUILD_LLAMA_FROM_SOURCE").is_ok() {
        build_llama_cpp_from_source(&gpu_config);
    } else {
        link_prebuilt_llama_cpp();
    }
    
    // Set additional build configurations
    set_optimization_flags();
}

#[derive(Debug, Default)]
struct GpuConfig {
    cuda: bool,
    cuda_version: Option<String>,
    cuda_arch: Option<String>,
    metal: bool,
    rocm: bool,
    rocm_version: Option<String>,
}

fn detect_gpu_configuration() -> GpuConfig {
    let mut config = GpuConfig::default();
    
    // CUDA detection
    if let Ok(cuda_path) = env::var("CUDA_PATH") {
        config.cuda = true;
        println!("cargo:warning=CUDA detected at: {}", cuda_path);
        
        // Detect CUDA version
        if let Ok(version) = detect_cuda_version(&cuda_path) {
            config.cuda_version = Some(version.clone());
            println!("cargo:warning=CUDA version: {}", version);
        }
        
        // Detect CUDA compute capability
        if let Ok(arch) = env::var("CUDA_ARCH") {
            config.cuda_arch = Some(arch);
        } else {
            // Auto-detect compute capability
            config.cuda_arch = detect_cuda_compute_capability();
        }
    }
    
    // Metal detection (macOS)
    #[cfg(target_os = "macos")]
    {
        config.metal = true;
        println!("cargo:warning=Metal acceleration available on macOS");
    }
    
    // ROCm detection (AMD GPUs)
    if let Ok(rocm_path) = env::var("ROCM_PATH") {
        config.rocm = true;
        println!("cargo:warning=ROCm detected at: {}", rocm_path);
        
        // Detect ROCm version
        if let Ok(version) = detect_rocm_version(&rocm_path) {
            config.rocm_version = Some(version.clone());
            println!("cargo:warning=ROCm version: {}", version);
        }
    }
    
    config
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

fn configure_gpu_acceleration(config: &GpuConfig) {
    // Set CUDA configuration
    if config.cuda {
        println!("cargo:rustc-cfg=feature=\"cuda\"");
        println!("cargo:rustc-env=LLAMA_CUDA=1");
        
        if let Some(ref arch) = config.cuda_arch {
            println!("cargo:rustc-env=CUDA_ARCHITECTURES={}", arch);
        }
        
        // Set CUDA path for linking
        if let Ok(cuda_path) = env::var("CUDA_PATH") {
            let cuda_lib = Path::new(&cuda_path).join("lib64");
            if cuda_lib.exists() {
                println!("cargo:rustc-link-search=native={}", cuda_lib.display());
            } else {
                let cuda_lib = Path::new(&cuda_path).join("lib");
                if cuda_lib.exists() {
                    println!("cargo:rustc-link-search=native={}", cuda_lib.display());
                }
            }
            
            // Link CUDA libraries
            println!("cargo:rustc-link-lib=cuda");
            println!("cargo:rustc-link-lib=cudart");
            println!("cargo:rustc-link-lib=cublas");
            println!("cargo:rustc-link-lib=cublasLt");
        }
    }
    
    // Set Metal configuration
    if config.metal {
        println!("cargo:rustc-cfg=feature=\"metal\"");
        println!("cargo:rustc-env=LLAMA_METAL=1");
        
        // Link Metal frameworks
        println!("cargo:rustc-link-lib=framework=Metal");
        println!("cargo:rustc-link-lib=framework=MetalPerformanceShaders");
        println!("cargo:rustc-link-lib=framework=MetalPerformanceShadersGraph");
        println!("cargo:rustc-link-lib=framework=Foundation");
    }
    
    // Set ROCm configuration
    if config.rocm {
        println!("cargo:rustc-cfg=feature=\"hipblas\"");
        println!("cargo:rustc-env=LLAMA_HIPBLAS=1");
        
        if let Ok(rocm_path) = env::var("ROCM_PATH") {
            let rocm_lib = Path::new(&rocm_path).join("lib");
            if rocm_lib.exists() {
                println!("cargo:rustc-link-search=native={}", rocm_lib.display());
            }
            
            // Link ROCm libraries
            println!("cargo:rustc-link-lib=hipblas");
            println!("cargo:rustc-link-lib=rocblas");
            println!("cargo:rustc-link-lib=amdhip64");
        }
    }
}

fn configure_system_libraries() {
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

fn build_llama_cpp_from_source(gpu_config: &GpuConfig) {
    println!("cargo:warning=Building llama.cpp from source...");
    
    let mut cmake_config = cmake::Config::new("llama.cpp");
    
    // Basic configuration
    cmake_config
        .define("BUILD_SHARED_LIBS", "ON")
        .define("LLAMA_BUILD_TESTS", "OFF")
        .define("LLAMA_BUILD_EXAMPLES", "OFF")
        .define("LLAMA_BUILD_SERVER", "OFF");
    
    // GPU-specific configuration
    if gpu_config.cuda {
        cmake_config.define("LLAMA_CUDA", "ON");
        
        if let Some(ref arch) = gpu_config.cuda_arch {
            cmake_config.define("CMAKE_CUDA_ARCHITECTURES", arch);
        }
    }
    
    if gpu_config.metal {
        cmake_config.define("LLAMA_METAL", "ON");
        cmake_config.define("LLAMA_METAL_EMBED_LIBRARY", "ON");
    }
    
    if gpu_config.rocm {
        cmake_config.define("LLAMA_HIPBLAS", "ON");
    }
    
    // Set optimization flags
    #[cfg(not(debug_assertions))]
    {
        cmake_config
            .define("CMAKE_BUILD_TYPE", "Release")
            .define("LLAMA_FAST", "ON")
            .define("LLAMA_LTO", "ON");
    }
    
    #[cfg(debug_assertions)]
    {
        cmake_config.define("CMAKE_BUILD_TYPE", "Debug");
    }
    
    // Additional optimizations
    cmake_config
        .define("LLAMA_NATIVE", "ON")
        .define("LLAMA_F16C", "ON")
        .define("LLAMA_FMA", "ON")
        .define("LLAMA_AVX", "ON")
        .define("LLAMA_AVX2", "ON")
        .define("LLAMA_AVX512", "OFF"); // Disable by default, can cause issues
    
    // Build llama.cpp
    let dst = cmake_config.build();
    
    // Set library search path
    println!("cargo:rustc-link-search=native={}/lib", dst.display());
    println!("cargo:rustc-link-search=native={}/lib64", dst.display());
    
    // Link the library
    println!("cargo:rustc-link-lib=llama");
    
    println!("cargo:warning=llama.cpp built successfully at: {}", dst.display());
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

fn set_optimization_flags() {
    // Set CPU optimization flags for better performance
    if cfg!(target_arch = "x86_64") {
        println!("cargo:rustc-env=CXXFLAGS=-march=native -mtune=native");
    } else if cfg!(target_arch = "aarch64") {
        println!("cargo:rustc-env=CXXFLAGS=-march=native");
    }
    
    // Enable Link Time Optimization in release builds
    #[cfg(not(debug_assertions))]
    {
        println!("cargo:rustc-env=CARGO_PROFILE_RELEASE_LTO=true");
        println!("cargo:rustc-env=CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1");
    }
    
    // Set thread configuration
    if let Ok(threads) = env::var("LLAMA_THREADS") {
        println!("cargo:rustc-env=LLAMA_THREADS={}", threads);
    } else {
        // Default to number of CPU cores
        let num_cpus = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4);
        println!("cargo:rustc-env=LLAMA_THREADS={}", num_cpus);
    }
    
    println!("cargo:warning=Build configuration complete");
}