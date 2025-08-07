#!/bin/bash

# SPARC System Integration Validation Script
# Tests all components and their integration

echo "========================================="
echo "    SPARC SYSTEM VALIDATION"
echo "========================================="
echo ""

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test results tracking
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Function to run test and track results
run_test() {
    local test_name="$1"
    local test_command="$2"
    
    echo -e "${BLUE}[TEST]${NC} $test_name"
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    if eval "$test_command" >/dev/null 2>&1; then
        echo -e "${GREEN}[PASS]${NC} $test_name"
        PASSED_TESTS=$((PASSED_TESTS + 1))
    else
        echo -e "${RED}[FAIL]${NC} $test_name"
        FAILED_TESTS=$((FAILED_TESTS + 1))
        # Show error details for failed tests
        echo -e "${YELLOW}Error details:${NC}"
        eval "$test_command" 2>&1 | head -10
        echo ""
    fi
}

# Function to check if command exists
check_command() {
    local cmd="$1"
    if ! command -v "$cmd" &> /dev/null; then
        echo -e "${RED}[ERROR]${NC} Command '$cmd' not found. Please install it first."
        exit 1
    fi
}

echo "Checking prerequisites..."
check_command "cargo"
echo -e "${GREEN}[OK]${NC} Cargo is available"
echo ""

echo "========================================="
echo "1. COMPILATION TESTS"
echo "========================================="

run_test "Compile all features (debug)" "cargo check --all-features"
run_test "Compile all features (release)" "cargo check --all-features --release"
run_test "Test compilation (no run)" "cargo test --all-features --no-run"

echo ""
echo "========================================="
echo "2. INDIVIDUAL COMPONENT TESTS"
echo "========================================="

run_test "AST module tests" "cargo test --package sparc-core ast --lib"
run_test "BM25 module tests" "cargo test --package sparc-core bm25 --lib"
run_test "Tantivy integration tests" "cargo test --package sparc-core tantivy --lib"
run_test "Nomic ML features" "cargo test --package sparc-core nomic --features ml"
run_test "Core library tests" "cargo test --lib"

echo ""
echo "========================================="
echo "3. INTEGRATION TESTS"
echo "========================================="

run_test "Full integration test suite" "cargo test integration_test"
run_test "Search integration tests" "cargo test search_integration"
run_test "ML integration tests" "cargo test ml_integration --features ml"

echo ""
echo "========================================="
echo "4. FEATURE FLAG COMBINATIONS"
echo "========================================="

run_test "Default features only" "cargo test"
run_test "ML features enabled" "cargo test --features ml"
run_test "All features enabled" "cargo test --all-features"
run_test "No default features" "cargo test --no-default-features"

echo ""
echo "========================================="
echo "5. EXAMPLE AND BINARY TESTS"
echo "========================================="

run_test "Build main binary" "cargo build --bin sparc-core"
run_test "Build examples (if any)" "cargo build --examples"

# Check for specific examples
if [ -f "examples/unified_search.rs" ]; then
    run_test "Run unified search example" "cargo run --example unified_search --quiet"
fi

if [ -f "examples/ast_demo.rs" ]; then
    run_test "Run AST demo example" "cargo run --example ast_demo --quiet"
fi

echo ""
echo "========================================="
echo "6. DOCUMENTATION TESTS"
echo "========================================="

run_test "Documentation tests" "cargo test --doc"
run_test "Documentation build" "cargo doc --all-features --no-deps"

echo ""
echo "========================================="
echo "7. CODE QUALITY CHECKS"
echo "========================================="

run_test "Clippy linting" "cargo clippy --all-features -- -D warnings"
run_test "Format check" "cargo fmt -- --check"

echo ""
echo "========================================="
echo "8. PERFORMANCE BENCHMARKS"
echo "========================================="

if [ -d "benches" ]; then
    run_test "Compile benchmarks" "cargo bench --no-run"
else
    echo -e "${YELLOW}[SKIP]${NC} No benchmark directory found"
fi

echo ""
echo "========================================="
echo "    VALIDATION SUMMARY"
echo "========================================="

echo "Total Tests: $TOTAL_TESTS"
echo -e "Passed: ${GREEN}$PASSED_TESTS${NC}"
echo -e "Failed: ${RED}$FAILED_TESTS${NC}"

if [ $FAILED_TESTS -eq 0 ]; then
    echo -e "${GREEN}=========================================${NC}"
    echo -e "${GREEN}    ALL TESTS PASSED! ✓${NC}"
    echo -e "${GREEN}=========================================${NC}"
    exit 0
else
    echo -e "${RED}=========================================${NC}"
    echo -e "${RED}    $FAILED_TESTS TESTS FAILED! ✗${NC}"
    echo -e "${RED}=========================================${NC}"
    
    echo ""
    echo "To investigate failures, run these commands manually:"
    echo "  cargo test --all-features -- --nocapture"
    echo "  cargo test <specific_test_name> -- --nocapture"
    echo "  cargo check --all-features"
    
    exit 1
fi