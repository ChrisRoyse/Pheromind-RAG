# Nomic3 Brutal Stress Tests - Quick Reference

## 🚀 Quick Start

```bash
# Run all stress tests
./scripts/run_brutal_stress_tests.sh

# Run individual test
cargo test --features ml test_1_network_dependency_failure -- --nocapture

# Run without ML features (storage tests only)
cargo test test_4_index_threshold_violation -- --nocapture
```

## 🎯 Individual Test Commands

### Network & ML Tests (require `--features ml`)
```bash
# Test 1: Network Dependency Failure
cargo test --features ml test_1_network_dependency_failure -- --nocapture

# Test 2: Memory Leak Validation  
cargo test --features ml test_2_memory_leak_validation -- --nocapture

# Test 3: Quantization Format Breaking
cargo test --features ml test_3_quantization_format_breaking -- --nocapture

# Test 5: Unicode Tokenization Chaos
cargo test --features ml test_5_unicode_tokenization_chaos -- --nocapture

# Test 8: Concurrent Deadlock Induction
cargo test --features ml test_8_concurrent_deadlock_induction -- --nocapture

# Test 9: Model Corruption Detection
cargo test --features ml test_9_model_corruption_detection -- --nocapture
```

### Storage Tests (no ML features required)
```bash
# Test 4: Index Threshold Violation
cargo test test_4_index_threshold_violation -- --nocapture

# Test 6: Dimension Mismatch Corruption
cargo test test_6_dimension_mismatch_corruption -- --nocapture

# Test 7: NaN Injection Attack
cargo test test_7_nan_injection_attack -- --nocapture

# Test 10: Embedding Cache Invalidation
cargo test test_10_embedding_cache_invalidation -- --nocapture
```

### Complete Test Suite
```bash
# Run all tests together
cargo test --features ml run_all_brutal_stress_tests -- --nocapture --test-threads=1
```

## 🔍 Expected Outputs

### ✅ Successful Vulnerability Detection
```
🚨 CRITICAL VULNERABILITY: Network dependency test should have failed without internet access
✅ EXPECTED NETWORK FAILURE detected in 2.3s
🎯 VULNERABILITY CONFIRMED: System requires internet access with no fallback mechanism
```

### ❌ Test Execution Errors
```
❌ UNEXPECTED: System should have failed with unsupported quantization
⚠️ Skipping unicode test - embedder initialization failed
```

### 🔒 Deadlock Detection
```
🚨 DEADLOCK DETECTED: Task 42 timed out after 60 seconds
🎯 VULNERABILITY CONFIRMED: Singleton pattern deadlocks under concurrent requests
```

### 💾 Memory Issues
```
🚨 MEMORY LEAK DETECTED: Growth of 623.4 MB exceeds acceptable threshold
🎯 VULNERABILITY CONFIRMED: Token embeddings cause memory accumulation
```

## 🚨 Critical Failure Indicators

Watch for these patterns in test output:

- `VULNERABILITY CONFIRMED` - A security/stability issue was detected
- `CRITICAL VULNERABILITY` - System-breaking issue found
- `DEADLOCK DETECTED` - Concurrency problem identified
- `MEMORY LEAK DETECTED` - Resource management issue
- `CORRUPTION` - Data integrity problem
- `INJECTION SUCCESS` - Security validation bypassed
- `CRASH` - System stability failure

## 📊 Result Analysis

### Test Log Locations
```
test_results_test_1_network_dependency_failure.log
test_results_test_2_memory_leak_validation.log
test_results_test_3_quantization_format_breaking.log
... (one for each test)
test_results_complete_suite.log
```

### Key Metrics to Monitor
- **Memory Growth**: >500MB indicates memory leak
- **Timeout Frequency**: >10% suggests deadlock issues  
- **Injection Success Rate**: >0% indicates security vulnerability
- **Crash Count**: >0 indicates stability problems

## 🛠️ Troubleshooting

### Common Issues

**Build Failures:**
```bash
# Missing ML dependencies
cargo build --features ml
rustup component add rustfmt clippy

# Missing system dependencies (Windows)
# Install Visual Studio Build Tools
```

**Network Test Skipping:**
```bash
# Clear cache to force network dependency
rm -rf ~/.nomic/
# Or set offline environment
export RUST_TEST_OFFLINE=1
```

**Permission Errors:**
```bash
# On Windows, run as Administrator for file permission tests
# On Unix, ensure write permissions to temp directories
```

### Test Configuration

**Environment Variables:**
```bash
export RUST_LOG=debug          # Enable debug logging
export RUST_BACKTRACE=1        # Show stack traces on panic
export RUST_TEST_THREADS=1     # Prevent race conditions
export RUST_TEST_TIMEOUT=300   # 5 minute timeout per test
```

**Feature Flags:**
```bash
# Full feature set
cargo test --all-features

# Minimal feature set
cargo test --no-default-features

# Specific features only
cargo test --features "ml,storage"
```

## 🎯 Success Criteria

### Test Should PASS When:
- Vulnerabilities are properly detected and reported
- Error messages clearly identify the failure mode
- System fails gracefully without crashing
- Recovery mechanisms work as expected

### Test Should FAIL When:
- System accepts invalid input without validation
- Memory leaks or resource exhaustion occurs
- Deadlocks or hangs prevent normal operation
- Data corruption is not detected

## 📈 Performance Baselines

### Expected Execution Times
- Network tests: 5-30 seconds (depends on cache state)
- Memory tests: 30-120 seconds  
- Storage tests: 10-60 seconds
- Concurrency tests: 60-300 seconds
- Complete suite: 5-15 minutes

### Resource Usage Limits  
- Memory growth: <100MB per test
- Disk usage: <1GB for temp files
- CPU usage: <90% sustained load
- File handles: <1000 open handles

## 📚 Additional Resources

- **Detailed Analysis**: `docs/nomic3_brutal_stress_test_analysis.md`
- **Test Source Code**: `tests/nomic3_brutal_stress_tests.rs`
- **Execution Script**: `scripts/run_brutal_stress_tests.sh`

## ⚠️ Safety Notes

1. **DO NOT RUN IN PRODUCTION** - These tests can crash systems
2. **Backup Important Data** - Tests may create/delete files
3. **Monitor Resource Usage** - Tests stress memory and CPU
4. **Review Results Carefully** - Failures often indicate success
5. **Implement Fixes Before Deployment** - Address all confirmed vulnerabilities