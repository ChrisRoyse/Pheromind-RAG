# Phase 1 Testing Guide

## Quick Start

### Linux/macOS
```bash
# Run the test script
./scripts/test_phase1.sh

# Or run individual tests
cargo test phase1_tests
cargo run --example check_gpu
cargo run --example test_llama_cpp
```

### Windows
```powershell
# Run the test script
.\scripts\test_phase1.ps1

# Or run individual tests
cargo test phase1_tests
cargo run --example check_gpu
cargo run --example test_llama_cpp
```

## Test Components

### 1. Build Configuration (Phase 1.1)
- ✅ `build.rs` - Comprehensive build script with GPU detection
- ✅ GPU acceleration detection (CUDA, Metal, ROCm)
- ✅ System library configuration
- ✅ llama.cpp build options

### 2. Cargo Configuration (Phase 1.2)
- ✅ Updated to version 0.3.0
- ✅ llama-cpp-2 dependencies added
- ✅ Feature flags for GPU acceleration
- ✅ Build dependencies configured

## Available Tests

### Unit Tests
```bash
# Run all Phase 1 verification tests
cargo test phase1_tests

# Run with output
cargo test phase1_tests -- --nocapture
```

### Integration Tests
```bash
# Run integration tests (includes build tests)
cargo test -- --ignored
```

### Examples
```bash
# Check GPU configuration
cargo run --example check_gpu

# Test llama-cpp-2 integration
cargo run --example test_llama_cpp
```

## Build Variations

### CPU Only
```bash
cargo build --release --no-default-features --features "vectordb,tree-sitter"
```

### CUDA Acceleration
```bash
CUDA_PATH=/usr/local/cuda cargo build --release --features cuda
```

### Metal Acceleration (macOS)
```bash
cargo build --release --features metal
```

### ROCm Acceleration (AMD)
```bash
ROCM_PATH=/opt/rocm cargo build --release --features hipblas
```

### Build from Source
```bash
BUILD_LLAMA_FROM_SOURCE=1 cargo build --release
```

## Verification Checklist

- [ ] build.rs exists and is valid
- [ ] Cargo.toml has llama-cpp-2 dependencies
- [ ] Project builds successfully
- [ ] GPU detection works (if available)
- [ ] Tests pass
- [ ] Examples run

## Troubleshooting

### Build Fails
1. Check Rust version: `rustc --version` (need 1.70+)
2. Install build tools:
   - Linux: `sudo apt install build-essential cmake`
   - macOS: `xcode-select --install`
   - Windows: Install Visual Studio Build Tools

### GPU Not Detected
1. Set environment variables:
   - CUDA: `export CUDA_PATH=/usr/local/cuda`
   - ROCm: `export ROCM_PATH=/opt/rocm`
2. Check drivers are installed
3. Rebuild: `cargo clean && cargo build --release`

### Tests Fail
1. Run with verbose output: `RUST_LOG=debug cargo test`
2. Check individual components:
   - Build config: `cargo run --example check_gpu`
   - Dependencies: `cargo tree | grep llama`

## Success Indicators

When everything is working correctly, you should see:
- ✅ All tests passing
- ✅ GPU acceleration detected (if available)
- ✅ Examples running successfully
- ✅ Build completing without errors

## Next Steps

After Phase 1 is verified:
1. Download the GGUF model file
2. Implement Phase 2 (llama-cpp-2 integration)
3. Test embedding generation
4. Integrate with existing codebase