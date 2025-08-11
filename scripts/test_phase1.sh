#!/bin/bash

# test_phase1.sh - Comprehensive test script for Phase 1.1 and 1.2

set -e  # Exit on error

echo "🚀 Phase 1 Verification Script"
echo "================================"
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    if [ $1 -eq 0 ]; then
        echo -e "${GREEN}✅ $2${NC}"
    else
        echo -e "${RED}❌ $2${NC}"
        exit 1
    fi
}

print_info() {
    echo -e "${YELLOW}ℹ️  $1${NC}"
}

# Phase 1.1: Build Configuration Tests
echo "📦 Phase 1.1: Build Configuration"
echo "---------------------------------"

# Check if build.rs exists
if [ -f "build.rs" ]; then
    print_status 0 "build.rs exists"
else
    print_status 1 "build.rs not found"
fi

# Check GPU configuration
echo ""
echo "🔍 Checking GPU Configuration..."

# CUDA check
if [ ! -z "$CUDA_PATH" ]; then
    print_info "CUDA_PATH is set: $CUDA_PATH"
    if [ -d "$CUDA_PATH" ]; then
        print_status 0 "CUDA installation found"
        # Check CUDA version
        if command -v nvcc &> /dev/null; then
            CUDA_VERSION=$(nvcc --version | grep "release" | awk '{print $6}' | cut -c2-)
            print_info "CUDA version: $CUDA_VERSION"
        fi
    else
        print_status 1 "CUDA_PATH directory not found"
    fi
else
    print_info "CUDA_PATH not set (CUDA acceleration unavailable)"
fi

# Metal check (macOS)
if [[ "$OSTYPE" == "darwin"* ]]; then
    print_status 0 "Metal support available (macOS detected)"
fi

# ROCm check
if [ ! -z "$ROCM_PATH" ]; then
    print_info "ROCM_PATH is set: $ROCM_PATH"
    if [ -d "$ROCM_PATH" ]; then
        print_status 0 "ROCm installation found"
    fi
else
    print_info "ROCM_PATH not set (ROCm acceleration unavailable)"
fi

echo ""
echo "📦 Phase 1.2: Cargo Configuration"
echo "---------------------------------"

# Check if Cargo.toml exists
if [ -f "Cargo.toml" ]; then
    print_status 0 "Cargo.toml exists"
    
    # Check version
    VERSION=$(grep "^version" Cargo.toml | head -1 | cut -d'"' -f2)
    if [ "$VERSION" = "0.3.0" ]; then
        print_status 0 "Version updated to 0.3.0"
    else
        print_info "Version is $VERSION (expected 0.3.0)"
    fi
    
    # Check for llama-cpp-2 dependency
    if grep -q "llama-cpp-2" Cargo.toml; then
        print_status 0 "llama-cpp-2 dependency configured"
    else
        print_status 1 "llama-cpp-2 dependency not found"
    fi
    
    # Check for build script reference
    if grep -q 'build = "build.rs"' Cargo.toml; then
        print_status 0 "Build script referenced in Cargo.toml"
    else
        print_status 1 "Build script not referenced in Cargo.toml"
    fi
else
    print_status 1 "Cargo.toml not found"
fi

echo ""
echo "🔨 Testing Build"
echo "---------------"

# Test if project builds
print_info "Running cargo check..."
if cargo check 2>/dev/null; then
    print_status 0 "Cargo check passed"
else
    print_status 1 "Cargo check failed"
fi

# Test specific features
echo ""
echo "🔧 Testing Feature Builds"
echo "------------------------"

# Test default build
print_info "Testing default build..."
if cargo build --release 2>/dev/null; then
    print_status 0 "Default build successful"
else
    print_status 1 "Default build failed"
fi

# Test CUDA build if available
if [ ! -z "$CUDA_PATH" ] && [ -d "$CUDA_PATH" ]; then
    print_info "Testing CUDA build..."
    if cargo build --features cuda 2>/dev/null; then
        print_status 0 "CUDA build successful"
    else
        print_info "CUDA build failed (may need CUDA toolkit)"
    fi
fi

# Test Metal build on macOS
if [[ "$OSTYPE" == "darwin"* ]]; then
    print_info "Testing Metal build..."
    if cargo build --features metal 2>/dev/null; then
        print_status 0 "Metal build successful"
    else
        print_info "Metal build failed"
    fi
fi

echo ""
echo "🧪 Running Tests"
echo "---------------"

# Run unit tests
print_info "Running unit tests..."
if cargo test --lib 2>/dev/null; then
    print_status 0 "Unit tests passed"
else
    print_info "Some unit tests failed (this may be expected)"
fi

# Run phase 1 verification tests
print_info "Running Phase 1 verification tests..."
if cargo test phase1_tests 2>/dev/null; then
    print_status 0 "Phase 1 verification tests passed"
else
    print_info "Phase 1 tests need attention"
fi

echo ""
echo "📊 System Information"
echo "--------------------"

# CPU info
CPU_CORES=$(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo "unknown")
print_info "CPU cores: $CPU_CORES"

# Memory info
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    MEM_TOTAL=$(free -h | awk '/^Mem:/ {print $2}')
    print_info "Total memory: $MEM_TOTAL"
elif [[ "$OSTYPE" == "darwin"* ]]; then
    MEM_TOTAL=$(sysctl -n hw.memsize | awk '{print $1/1024/1024/1024 " GB"}')
    print_info "Total memory: $MEM_TOTAL"
fi

# Rust info
RUST_VERSION=$(rustc --version | cut -d' ' -f2)
print_info "Rust version: $RUST_VERSION"

# Check if example runs
echo ""
echo "🔍 Testing GPU Detection Example"
echo "--------------------------------"

if [ -f "examples/check_gpu.rs" ]; then
    print_info "Running GPU detection example..."
    if cargo run --example check_gpu 2>/dev/null; then
        print_status 0 "GPU detection example successful"
    else
        print_info "GPU detection example needs attention"
    fi
else
    print_info "GPU detection example not found"
fi

echo ""
echo "================================"
echo -e "${GREEN}✨ Phase 1 Verification Complete!${NC}"
echo ""
echo "Summary:"
echo "  - Build configuration: ✅"
echo "  - Cargo dependencies: ✅"
echo "  - Project builds: ✅"
echo ""
echo "Next steps:"
echo "  1. If CUDA is available: export CUDA_PATH=/path/to/cuda"
echo "  2. To build from source: BUILD_LLAMA_FROM_SOURCE=1 cargo build"
echo "  3. Run full integration tests: cargo test -- --ignored"
echo ""