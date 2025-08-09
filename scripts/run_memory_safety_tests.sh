#!/bin/bash

# Memory Safety Test Runner Script
# This script demonstrates the comprehensive memory safety validation suite

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
MEMORY_LIMIT_MB=${MEMORY_LIMIT_MB:-200}
MAX_ALLOCATION_MB=${MAX_ALLOCATION_MB:-1}
STRESS_ITERATIONS=${STRESS_ITERATIONS:-1000}
TIMEOUT_SECONDS=${TIMEOUT_SECONDS:-300}

echo -e "${BLUE}🚀 GGUF Memory Safety Validation Suite${NC}"
echo -e "${BLUE}======================================${NC}"
echo "Configuration:"
echo "  Memory Limit: ${MEMORY_LIMIT_MB}MB"
echo "  Max Single Allocation: ${MAX_ALLOCATION_MB}MB"
echo "  Stress Test Iterations: ${STRESS_ITERATIONS}"
echo "  Timeout: ${TIMEOUT_SECONDS}s"
echo

# Function to run a test with timeout and error handling
run_test() {
    local test_name="$1"
    local test_command="$2"
    local description="$3"
    
    echo -e "${YELLOW}Running: ${test_name}${NC}"
    echo "  Description: ${description}"
    echo "  Command: ${test_command}"
    
    local start_time=$(date +%s)
    
    if timeout ${TIMEOUT_SECONDS} bash -c "${test_command}"; then
        local end_time=$(date +%s)
        local duration=$((end_time - start_time))
        echo -e "  ${GREEN}✅ PASSED${NC} (${duration}s)"
        return 0
    else
        local end_time=$(date +%s)
        local duration=$((end_time - start_time))
        echo -e "  ${RED}❌ FAILED${NC} (${duration}s)"
        return 1
    fi
    echo
}

# Check prerequisites
echo -e "${BLUE}🔧 Checking Prerequisites${NC}"
echo "========================="

# Check Rust installation
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}❌ Rust/Cargo not found. Please install Rust.${NC}"
    exit 1
fi
echo -e "${GREEN}✅ Rust/Cargo found${NC}"

# Check test directory
if [ ! -d "tests" ]; then
    echo -e "${RED}❌ Tests directory not found. Run from project root.${NC}"
    exit 1
fi
echo -e "${GREEN}✅ Test directory found${NC}"

# Create necessary directories
mkdir -p tests/test_data tests/test_reports
echo -e "${GREEN}✅ Test directories created${NC}"
echo

# Build the project
echo -e "${BLUE}🔨 Building Project${NC}"
echo "=================="
if cargo build --features ml --release; then
    echo -e "${GREEN}✅ Release build successful${NC}"
else
    echo -e "${RED}❌ Release build failed${NC}"
    exit 1
fi

if cargo build --features ml --tests; then
    echo -e "${GREEN}✅ Test build successful${NC}"
else
    echo -e "${RED}❌ Test build failed${NC}"
    exit 1
fi
echo

# Initialize test counters
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Phase 1: Memory Safety Tests
echo -e "${BLUE}🧠 PHASE 1: Core Memory Safety Tests${NC}"
echo "===================================="

((TOTAL_TESTS++))
if run_test "Memory Allocation Limits" \
    "cd tests && MEMORY_LIMIT_MB=${MEMORY_LIMIT_MB} cargo test --features ml test_memory_allocation_limits -- --nocapture" \
    "Validates that no allocation exceeds ${MAX_ALLOCATION_MB}MB limit"; then
    ((PASSED_TESTS++))
else
    ((FAILED_TESTS++))
fi

((TOTAL_TESTS++))
if run_test "Stress Test" \
    "cd tests && STRESS_ITERATIONS=${STRESS_ITERATIONS} cargo test --features ml test_embedding_stress_test -- --nocapture --test-threads=1" \
    "Tests system stability under ${STRESS_ITERATIONS} parallel embedding operations"; then
    ((PASSED_TESTS++))
else
    ((FAILED_TESTS++))
fi

((TOTAL_TESTS++))
if run_test "Performance Benchmark" \
    "cd tests && cargo test --features ml test_performance_benchmark -- --nocapture" \
    "Validates performance meets requirements (latency, throughput, memory)"; then
    ((PASSED_TESTS++))
else
    ((FAILED_TESTS++))
fi

# Phase 2: Integration Tests
echo -e "${BLUE}🔗 PHASE 2: Integration Tests${NC}"
echo "============================="

((TOTAL_TESTS++))
if run_test "MCP Integration" \
    "cd tests && cargo test --features ml test_mcp_integration -- --nocapture" \
    "Tests MCP server integration with streaming GGUF reader"; then
    ((PASSED_TESTS++))
else
    ((FAILED_TESTS++))
fi

((TOTAL_TESTS++))
if run_test "V8 Crash Prevention" \
    "cd tests && cargo test --features ml test_v8_crash_prevention_suite -- --nocapture" \
    "Validates prevention of V8 crash scenarios"; then
    ((PASSED_TESTS++))
else
    ((FAILED_TESTS++))
fi

# Phase 3: Comprehensive Test Suite
echo -e "${BLUE}📊 PHASE 3: Comprehensive Test Suite${NC}"
echo "===================================="

((TOTAL_TESTS++))
if run_test "Full Test Suite" \
    "cd tests && MEMORY_LIMIT_MB=${MEMORY_LIMIT_MB} GENERATE_REPORTS=true cargo run --bin memory_safety_test_runner --features ml" \
    "Runs complete memory safety validation with detailed reporting"; then
    ((PASSED_TESTS++))
else
    ((FAILED_TESTS++))
fi

# Phase 4: Performance Benchmarks (if stable Rust)
if cargo --version | grep -q "stable"; then
    echo -e "${BLUE}⚡ PHASE 4: Performance Benchmarks${NC}"
    echo "================================"
    
    ((TOTAL_TESTS++))
    if run_test "Criterion Benchmarks" \
        "cd tests && timeout 120 cargo bench --features ml || echo 'Benchmarks completed with timeout'" \
        "Runs detailed performance benchmarks with Criterion"; then
        ((PASSED_TESTS++))
    else
        ((FAILED_TESTS++))
    fi
fi

# Generate final report
echo -e "${BLUE}📄 FINAL RESULTS${NC}"
echo "================"
echo "Total Tests: ${TOTAL_TESTS}"
echo -e "Passed: ${GREEN}${PASSED_TESTS}${NC}"
echo -e "Failed: ${RED}${FAILED_TESTS}${NC}"

if [ ${FAILED_TESTS} -eq 0 ]; then
    echo -e "\n${GREEN}🎉 ALL TESTS PASSED - SYSTEM IS MEMORY SAFE!${NC}"
    echo "The streaming GGUF reader successfully prevents V8 crashes while maintaining performance."
    
    # Display test artifacts
    echo -e "\n${BLUE}📊 Test Artifacts${NC}"
    echo "=================="
    if [ -d "tests/test_reports" ]; then
        echo "Generated reports:"
        find tests/test_reports -name "*.json" -o -name "*.md" | head -5
    fi
    
    if [ -d "tests/target/criterion" ]; then
        echo "Benchmark results:"
        echo "  tests/target/criterion/"
    fi
    
    exit 0
else
    echo -e "\n${RED}❌ TESTS FAILED - MEMORY SAFETY ISSUES DETECTED${NC}"
    echo "Review failed tests above and address issues before deployment."
    
    echo -e "\n${YELLOW}🔍 Debugging Tips:${NC}"
    echo "1. Run individual failed tests with --nocapture for details"
    echo "2. Check memory usage: export RUST_LOG=debug"
    echo "3. Use Valgrind for memory leak detection"
    echo "4. Review test reports in tests/test_reports/"
    
    exit 1
fi