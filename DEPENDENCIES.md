# System Dependencies

## Required System Packages

### Ubuntu/Debian
```bash
sudo apt-get update
sudo apt-get install -y \
    build-essential \
    cmake \
    pkg-config \
    libssl-dev \
    clang \
    libclang-dev \
    protobuf-compiler  # Required for lance-encoding
```

### macOS
```bash
brew install cmake llvm protobuf
```

### Windows
- Install Visual Studio Build Tools
- Install CMake
- Install LLVM from https://releases.llvm.org/
- Install protoc from https://github.com/protocolbuffers/protobuf/releases

## Optional GPU Support

### CUDA (NVIDIA)
```bash
# Download and install CUDA Toolkit
export CUDA_PATH=/usr/local/cuda
```

### Metal (macOS)
Automatically available on macOS with Xcode

### ROCm (AMD)
```bash
# Install ROCm
export ROCM_PATH=/opt/rocm
```

## Quick Install Script

### Linux
```bash
#!/bin/bash
# Install all dependencies for Ubuntu/Debian
sudo apt-get update
sudo apt-get install -y \
    build-essential \
    cmake \
    pkg-config \
    libssl-dev \
    clang \
    libclang-dev \
    protobuf-compiler \
    git \
    curl

# Install Rust if not present
if ! command -v cargo &> /dev/null; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
fi
```