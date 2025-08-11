# Build Configuration for llama-cpp-2 Integration

## Overview

The build.rs script provides comprehensive build configuration for integrating llama-cpp-2 with automatic GPU acceleration detection and optimization.

## Features

### 🚀 GPU Acceleration Detection
- **CUDA**: Automatic detection of NVIDIA GPUs with compute capability discovery
- **Metal**: Native macOS GPU acceleration
- **ROCm**: AMD GPU support with HIP/ROCm detection

### 🔧 Build Modes

#### 1. Using Prebuilt llama.cpp (Default)
```bash
cargo build --release
```

#### 2. Building llama.cpp from Source
```bash
BUILD_LLAMA_FROM_SOURCE=1 cargo build --release
```

#### 3. Using Custom llama.cpp Installation
```bash
LLAMA_CPP_PATH=/path/to/llama.cpp cargo build --release
```

## Environment Variables

### GPU Configuration
- `CUDA_PATH`: Path to CUDA installation (auto-detected on most systems)
- `CUDA_ARCH`: Override CUDA compute capability (e.g., "sm_86")
- `ROCM_PATH`: Path to ROCm installation
- `LLAMA_THREADS`: Number of CPU threads (defaults to CPU core count)

### Build Options
- `BUILD_LLAMA_FROM_SOURCE`: Build llama.cpp from source
- `LLAMA_CPP_PATH`: Path to prebuilt llama.cpp installation

## Platform-Specific Setup

### Linux (Ubuntu/Debian)
```bash
# Install dependencies
sudo apt-get update
sudo apt-get install -y build-essential cmake pkg-config

# For CUDA support
# Install CUDA Toolkit from NVIDIA

# Build with CUDA
CUDA_PATH=/usr/local/cuda cargo build --release --features cuda
```

### macOS
```bash
# Install dependencies
brew install cmake llvm

# Build with Metal acceleration (automatic)
cargo build --release --features metal
```

### Windows
```powershell
# Install Visual Studio Build Tools
# Install CMake

# Build with CUDA (if available)
$env:CUDA_PATH = "C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v11.8"
cargo build --release --features cuda
```

## Build Examples

### Maximum Performance Build
```bash
# With all optimizations and GPU acceleration
CARGO_PROFILE_RELEASE_LTO=true \
CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 \
cargo build --release --features cuda
```

### Debug Build
```bash
cargo build
```

### Cross-Platform Build
```bash
# For specific target
cargo build --release --target aarch64-unknown-linux-gnu
```

## Troubleshooting

### CUDA Not Detected
1. Ensure CUDA Toolkit is installed
2. Set CUDA_PATH manually:
   ```bash
   export CUDA_PATH=/usr/local/cuda-11.8
   ```

### Build Fails with "llama not found"
1. Build from source:
   ```bash
   BUILD_LLAMA_FROM_SOURCE=1 cargo build --release
   ```
2. Or specify custom path:
   ```bash
   LLAMA_CPP_PATH=/path/to/llama.cpp cargo build --release
   ```

### Metal Not Working on macOS
- Ensure Xcode Command Line Tools are installed:
  ```bash
  xcode-select --install
  ```

## Performance Optimization

The build script automatically enables:
- Native CPU optimizations (`-march=native`)
- Link Time Optimization (LTO) in release builds
- AVX/AVX2 instructions (x86_64)
- F16C and FMA instructions
- Parallel compilation

## Verification

After building, verify GPU support:
```bash
cargo run --example check_gpu
```

Expected output:
```
CUDA: Available (sm_86)
Metal: Not available
ROCm: Not available
Selected acceleration: CUDA
```