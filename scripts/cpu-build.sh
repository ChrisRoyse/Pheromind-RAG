#!/bin/bash
# CPU-Optimized Build Script for Pheromind-RAG

set -e

echo "===================================="
echo "CPU-ONLY GGUF Embedding Build Script"
echo "===================================="

# Clean previous builds
echo "Cleaning previous builds..."
cargo clean

# Set CPU optimization flags
export RUSTFLAGS="-C target-cpu=native -C opt-level=3"
export LLAMA_THREADS=$(nproc --all)
echo "Using $LLAMA_THREADS CPU threads for compilation"

# Disable GPU features entirely
export LLAMA_CUDA=0
export LLAMA_METAL=0
export LLAMA_HIPBLAS=0

# Build with maximum CPU optimizations
echo "Building with CPU optimizations..."
cargo build --release --features cpu-optimized

echo "===================================="
echo "Build completed successfully!"
echo "GPU layers set to: 0 (CPU-only)"
echo "Thread optimization: $LLAMA_THREADS threads"
echo "Target CPU: native"
echo "===================================="

# Test the build
if [ -f "target/release/test_phase2" ]; then
    echo "Testing CPU performance..."
    ./target/release/test_phase2
else
    echo "Warning: test_phase2 binary not found"
fi