# Memory Safety Validation Test Suite

This comprehensive test suite validates that the streaming GGUF reader prevents V8 crashes by maintaining strict memory safety guarantees.

## 🎯 Test Objectives

1. **Zero Large Allocations**: Ensure no single allocation exceeds 1MB
2. **Memory Pressure Handling**: Validate graceful degradation under memory constraints
3. **V8 Crash Prevention**: Test scenarios that historically crashed V8 
4. **Performance Validation**: Confirm acceptable performance within memory limits
5. **Integration Testing**: Verify MCP server integration works safely

## 📁 Test Structure

```
tests/
├── memory_safety/           # Core memory safety tests
│   ├── gguf_memory_validation.rs
│   └── memory_monitor_extension.rs
├── performance/             # Performance benchmarks
│   └── gguf_benchmark.rs
├── integration/             # Integration tests
│   ├── mcp_embedding_integration.rs
│   └── v8_crash_prevention.rs
├── benches/                 # Criterion benchmarks
│   └── gguf_memory_benchmarks.rs
└── memory_safety_test_runner.rs  # Main test runner
```

## 🚀 Running Tests

### Quick Validation
```bash
# Run core memory safety tests
cd tests
cargo test --features ml test_memory_allocation_limits

# Run stress test
cargo test --features ml test_embedding_stress_test -- --nocapture

# Run performance benchmark
cargo test --features ml test_performance_benchmark
```

### Comprehensive Test Suite
```bash
# Run all tests with detailed reporting
cd tests
cargo run --bin memory_safety_test_runner --features ml
```

### Individual Test Categories
```bash
# Memory safety only
cargo test --features ml memory_safety

# Performance benchmarks
cargo bench --features ml

# Integration tests
cargo test --features ml integration

# V8 crash prevention
cargo test --features ml v8_crash_prevention
```

## 📊 Test Categories

### 1. Memory Allocation Tests
- **Allocation Limit Enforcement**: Validates 1MB allocation limit
- **Memory Tracking**: Tests allocation/deallocation tracking
- **Peak Memory Monitoring**: Ensures memory usage stays bounded

### 2. Stress Tests  
- **1000+ Parallel Embeddings**: High-load concurrent processing
- **Memory Pressure**: System behavior under memory constraints
- **Resource Exhaustion**: Graceful handling of resource limits

### 3. Performance Benchmarks
- **File I/O Performance**: Seek and read latencies
- **Embedding Latency**: Time to generate embeddings (<50ms target)
- **Throughput**: Data processing rates (>10MB/s target)
- **Memory Efficiency**: Bytes processed per MB used

### 4. V8 Crash Prevention
- **Large Memory Allocations**: Prevent >1MB heap allocations
- **Rapid Memory Growth**: Detect and limit memory growth spikes
- **Array Buffer Overflow**: Prevent buffer overflows
- **Event Loop Blocking**: Ensure non-blocking operations

### 5. MCP Integration
- **Basic Functionality**: Single embedding requests
- **Batch Processing**: Multiple embedding requests  
- **Error Handling**: Graceful failure modes
- **Concurrent Requests**: Multiple simultaneous requests

## 🎛️ Configuration

Tests can be configured via environment variables:

```bash
# Memory limits
export MEMORY_LIMIT_MB=200
export MAX_SINGLE_ALLOCATION_MB=1

# Test parameters  
export STRESS_ITERATIONS=1000
export CONCURRENT_WORKERS=16
export TIMEOUT_SECONDS=300

# Output options
export GENERATE_REPORTS=true
export OUTPUT_DIRECTORY=test_reports
```

## 📈 Success Criteria

### Memory Safety Requirements
- ✅ **Zero allocations >1MB**: No single allocation exceeds limit
- ✅ **Peak memory <200MB**: Total memory usage stays bounded  
- ✅ **Zero memory leaks**: Memory properly deallocated
- ✅ **Violation detection**: Memory violations caught and reported

### Performance Requirements  
- ✅ **Seek latency <1ms**: File seek operations fast
- ✅ **Read latency <10ms**: File read operations fast
- ✅ **Throughput >10MB/s**: Acceptable data processing rate
- ✅ **Embedding latency <50ms**: Quick embedding generation

### V8 Crash Prevention
- ✅ **80%+ success rate**: Most crash scenarios prevented
- ✅ **Critical scenarios pass**: All high-risk scenarios prevented
- ✅ **Graceful degradation**: Errors handled without crashes

### Integration Requirements
- ✅ **MCP functionality**: Basic embedding requests work
- ✅ **Batch processing**: Multiple requests handled
- ✅ **Error recovery**: System recovers from errors
- ✅ **Concurrent safety**: Multiple requests safe

## 📊 Reports and Output

### Test Reports
Generated in `test_reports/` directory:
- `memory_safety_report.json`: Detailed JSON results
- `test_summary.md`: Human-readable summary
- `benchmark_results/`: Performance benchmark data

### Console Output
Tests provide real-time feedback:
```
🚀 STARTING COMPREHENSIVE MEMORY SAFETY VALIDATION
=================================================
🧠 PHASE 1: Memory Safety Tests
  ✅ Allocation limits enforced
  ✅ Stress test: 1000 iterations completed
  ✅ No memory leaks detected

⚡ PHASE 2: Performance Tests  
  ✅ File I/O: <1ms seeks, >10MB/s throughput
  ✅ Embedding: <50ms latency

🔗 PHASE 3: Integration Tests
  ✅ MCP functionality working
  ✅ Batch processing stable

🛡️ PHASE 4: V8 Crash Prevention
  ✅ 8/8 scenarios prevented (100%)

🎉 ALL TESTS PASSED - SYSTEM IS MEMORY SAFE!
```

## 🔧 CI/CD Integration

### GitHub Actions
The test suite integrates with GitHub Actions:
- **Pull Request Checks**: Run on every PR
- **Nightly Testing**: Extensive tests overnight  
- **Performance Regression**: Compare against baseline
- **Cross-Platform**: Test on Linux, Windows, macOS

### Test Matrix
Tests run across:
- Rust versions: stable, beta
- Memory limits: 100MB, 200MB  
- Platforms: Ubuntu, Windows, macOS

## 🐛 Troubleshooting

### Common Issues

**Test Failures**
```bash
# Run with detailed output
cargo test -- --nocapture

# Run specific test
cargo test test_memory_allocation_limits -- --nocapture
```

**Memory Limit Exceeded**
- Increase `MEMORY_LIMIT_MB` environment variable
- Check for actual memory leaks vs. legitimate usage
- Review allocation patterns in failing tests

**Performance Regression**  
- Compare benchmark results with baseline
- Profile slow operations
- Check system resources during testing

**V8 Crash Prevention Failures**
- Review crash scenarios that failed
- Update prevention mechanisms
- Test in Node.js environment if needed

### Debug Mode
```bash
# Enable debug logging
export RUST_LOG=debug
cargo test -- --nocapture

# Run with memory profiling
valgrind --tool=memcheck cargo test test_memory_leaks
```

## 🤝 Contributing

When adding new tests:

1. **Follow naming convention**: `test_<category>_<specific_test>`
2. **Include documentation**: Explain what the test validates
3. **Set appropriate timeouts**: Prevent hanging tests
4. **Clean up resources**: Ensure proper cleanup in all paths
5. **Add to CI**: Include in GitHub Actions workflow

### Test Development Guidelines

- **Fail fast**: Return early on critical failures
- **Meaningful assertions**: Clear error messages  
- **Resource cleanup**: Use RAII patterns
- **Deterministic**: Tests should have consistent results
- **Isolated**: Tests should not depend on each other

## 📚 Additional Resources

- [Memory Safety Documentation](../docs/memory_safety.md)
- [GGUF Streaming Implementation](../src/embedding/streaming_core.rs)
- [Performance Benchmarking Guide](../docs/performance.md)
- [CI/CD Configuration](../.github/workflows/memory_safety_validation.yml)

---

This test suite provides comprehensive validation that the streaming GGUF reader maintains memory safety while delivering acceptable performance. All tests must pass before deployment to production environments.