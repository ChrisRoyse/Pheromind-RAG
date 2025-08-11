#!/bin/bash

# EDGE CASE TESTING SPECIALIST - COMPREHENSIVE SYSTEM BREAKER SCRIPT
# PRINCIPLE 0: NO FALLBACKS - All failures must be explicit and debuggable
# Mission: Execute all edge case tests systematically and verify error clarity

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test configuration
CARGO_TEST_FLAGS="--test edge_case_failure_tests --release"
LOG_LEVEL=${LOG_LEVEL:-"debug"}
TIMEOUT=${TIMEOUT:-300} # 5 minutes per test
MAX_MEMORY=${MAX_MEMORY:-"2G"}

echo -e "${BLUE}===========================================${NC}"
echo -e "${BLUE}EDGE CASE TESTING SPECIALIST - SYSTEM BREAKER${NC}"
echo -e "${BLUE}===========================================${NC}"
echo ""
echo -e "${YELLOW}Testing Configuration:${NC}"
echo "  Log Level: $LOG_LEVEL"
echo "  Timeout: ${TIMEOUT}s per test"
echo "  Memory Limit: $MAX_MEMORY"
echo "  Cargo Flags: $CARGO_TEST_FLAGS"
echo ""

# Function to run individual test category with resource monitoring
run_test_category() {
    local category="$1"
    local description="$2"
    
    echo -e "${BLUE}Testing: $description${NC}"
    echo -e "${YELLOW}Category: $category${NC}"
    
    # Record start time and memory
    local start_time=$(date +%s)
    local start_memory=$(ps -o pid,vsz,rss,comm -p $$ | tail -1 | awk '{print $2}')
    
    # Run the test with timeout and memory monitoring
    if timeout ${TIMEOUT} cargo test ${CARGO_TEST_FLAGS} $category -- --nocapture; then
        echo -e "${GREEN}✓ PASSED: $description${NC}"
        local result="PASSED"
    else
        local exit_code=$?
        if [ $exit_code -eq 124 ]; then
            echo -e "${RED}✗ TIMEOUT: $description (exceeded ${TIMEOUT}s)${NC}"
            local result="TIMEOUT"
        else
            echo -e "${RED}✗ FAILED: $description (exit code: $exit_code)${NC}"
            local result="FAILED"
        fi
    fi
    
    # Calculate test duration and memory usage
    local end_time=$(date +%s)
    local duration=$((end_time - start_time))
    local end_memory=$(ps -o pid,vsz,rss,comm -p $$ | tail -1 | awk '{print $2}')
    local memory_diff=$((end_memory - start_memory))
    
    echo -e "${YELLOW}  Duration: ${duration}s${NC}"
    echo -e "${YELLOW}  Memory Change: ${memory_diff}KB${NC}"
    echo ""
    
    # Log result for summary
    echo "$category,$description,$result,$duration,${memory_diff}" >> test_results.csv
}

# Function to check prerequisites
check_prerequisites() {
    echo -e "${BLUE}Checking Prerequisites...${NC}"
    
    # Check if Rust/Cargo is available
    if ! command -v cargo &> /dev/null; then
        echo -e "${RED}ERROR: Cargo not found. Please install Rust.${NC}"
        exit 1
    fi
    
    # Check if test file exists
    if [ ! -f "tests/edge_case_failure_tests.rs" ]; then
        echo -e "${RED}ERROR: Edge case test file not found.${NC}"
        exit 1
    fi
    
    # Check available memory
    local available_memory=$(free -m | awk 'NR==2{printf "%.0f", $7/1024}')
    echo -e "${YELLOW}Available Memory: ${available_memory}GB${NC}"
    
    if [ "$available_memory" -lt 1 ]; then
        echo -e "${YELLOW}WARNING: Low memory detected. Some tests may fail.${NC}"
    fi
    
    echo -e "${GREEN}✓ Prerequisites OK${NC}"
    echo ""
}

# Function to setup test environment
setup_test_environment() {
    echo -e "${BLUE}Setting up Test Environment...${NC}"
    
    # Create test results log
    echo "Category,Description,Result,Duration(s),MemoryChange(KB)" > test_results.csv
    
    # Set environment variables for testing
    export RUST_LOG=${LOG_LEVEL}
    export RUST_BACKTRACE=1
    
    # Create temporary directory for test artifacts
    mkdir -p test_artifacts
    
    echo -e "${GREEN}✓ Environment Ready${NC}"
    echo ""
}

# Function to run all edge case categories
run_all_edge_cases() {
    echo -e "${BLUE}===========================================${NC}"
    echo -e "${BLUE}EXECUTING COMPREHENSIVE EDGE CASE TESTS${NC}"
    echo -e "${BLUE}===========================================${NC}"
    echo ""
    
    # EDGE CASE 1: Empty Input Tests
    run_test_category "empty_input_edge_cases" \
        "Empty Strings, Null Inputs, Zero-length Vectors"
    
    # EDGE CASE 2: Massive Input Tests  
    run_test_category "massive_input_edge_cases" \
        "100k+ Characters, Massive Batches, Memory Exhaustion"
    
    # EDGE CASE 3: Malformed Input Tests
    run_test_category "malformed_input_edge_cases" \
        "Non-UTF8, Control Characters, Invalid Unicode"
    
    # EDGE CASE 4: Resource Exhaustion Tests
    run_test_category "resource_exhaustion_edge_cases" \
        "Cache Overflow, Concurrent Access, Memory Pressure"
    
    # EDGE CASE 5: Model Corruption Tests
    run_test_category "model_corruption_edge_cases" \
        "Missing Models, Corrupted GGUF, Wrong Model Types"
    
    # EDGE CASE 6: Concurrent Access Tests
    run_test_category "concurrent_access_edge_cases" \
        "Thread Safety, Race Conditions, Deadlock Prevention"
    
    # EDGE CASE 7: Filesystem Edge Cases
    run_test_category "filesystem_edge_cases" \
        "Missing Directories, Permission Denials, Disk Issues"
    
    # EDGE CASE 8: Data Validation Tests
    run_test_category "validation_edge_cases" \
        "NaN/Infinity Values, Dimension Mismatches, Corruption"
    
    # EDGE CASE 9: Performance Regression Tests
    run_test_category "performance_regression_edge_cases" \
        "Latency Limits, Memory Usage, Cache Thrashing"
    
    # EDGE CASE 10: Error Message Quality Tests
    run_test_category "error_message_quality_tests" \
        "Actionable Errors, Context Preservation, Message Clarity"
}

# Function to generate test report
generate_test_report() {
    echo -e "${BLUE}===========================================${NC}"
    echo -e "${BLUE}EDGE CASE TEST RESULTS SUMMARY${NC}"
    echo -e "${BLUE}===========================================${NC}"
    echo ""
    
    # Count results
    local total_tests=$(tail -n +2 test_results.csv | wc -l)
    local passed_tests=$(grep -c "PASSED" test_results.csv || echo 0)
    local failed_tests=$(grep -c "FAILED" test_results.csv || echo 0)
    local timeout_tests=$(grep -c "TIMEOUT" test_results.csv || echo 0)
    
    echo -e "${BLUE}Test Summary:${NC}"
    echo "  Total Categories: $total_tests"
    echo -e "  ${GREEN}Passed: $passed_tests${NC}"
    echo -e "  ${RED}Failed: $failed_tests${NC}"
    echo -e "  ${YELLOW}Timeouts: $timeout_tests${NC}"
    echo ""
    
    # Calculate pass rate
    if [ $total_tests -gt 0 ]; then
        local pass_rate=$((passed_tests * 100 / total_tests))
        echo -e "${BLUE}Success Rate: ${pass_rate}%${NC}"
    fi
    echo ""
    
    # Show detailed results
    echo -e "${BLUE}Detailed Results:${NC}"
    echo "----------------------------------------"
    while IFS=',' read -r category description result duration memory; do
        if [ "$result" = "PASSED" ]; then
            echo -e "${GREEN}✓${NC} $description (${duration}s, ${memory}KB)"
        elif [ "$result" = "FAILED" ]; then
            echo -e "${RED}✗${NC} $description (${duration}s, ${memory}KB)"
        elif [ "$result" = "TIMEOUT" ]; then
            echo -e "${YELLOW}⏱${NC} $description (timeout after ${TIMEOUT}s)"
        fi
    done < <(tail -n +2 test_results.csv)
    echo ""
    
    # Error Analysis
    if [ $failed_tests -gt 0 ] || [ $timeout_tests -gt 0 ]; then
        echo -e "${RED}EDGE CASE FAILURES DETECTED${NC}"
        echo "This indicates potential system vulnerabilities or error handling issues."
        echo "Review failed test output for specific error messages and debugging info."
        echo ""
    fi
    
    # Performance Analysis
    echo -e "${BLUE}Performance Analysis:${NC}"
    local avg_duration=$(awk -F',' 'NR>1 && $4!="" {sum+=$4; count++} END {if(count>0) print sum/count; else print 0}' test_results.csv)
    local max_duration=$(awk -F',' 'NR>1 && $4!="" {if($4>max) max=$4} END {print max+0}' test_results.csv)
    local total_memory=$(awk -F',' 'NR>1 && $5!="" {sum+=$5} END {print sum+0}' test_results.csv)
    
    echo "  Average Test Duration: ${avg_duration}s"
    echo "  Maximum Test Duration: ${max_duration}s"
    echo "  Total Memory Change: ${total_memory}KB"
    echo ""
    
    # Truth Assessment
    echo -e "${BLUE}===========================================${NC}"
    echo -e "${BLUE}TRUTH ASSESSMENT - NO FALLBACKS ANALYSIS${NC}"
    echo -e "${BLUE}===========================================${NC}"
    echo ""
    
    if [ $failed_tests -eq 0 ] && [ $timeout_tests -eq 0 ]; then
        echo -e "${GREEN}✓ TRUTH PRINCIPLE UPHELD${NC}"
        echo "All edge cases handled properly with explicit error messages."
        echo "System demonstrates robust error handling without silent failures."
    else
        echo -e "${YELLOW}⚠ TRUTH PRINCIPLE VIOLATIONS DETECTED${NC}"
        echo "Some edge cases failed or timed out, indicating potential issues:"
        echo "  - Hidden failures instead of explicit errors"
        echo "  - Performance regressions under stress"
        echo "  - Insufficient error message clarity"
        echo "  - Resource handling problems"
    fi
    echo ""
}

# Function to cleanup test environment
cleanup_test_environment() {
    echo -e "${BLUE}Cleaning up Test Environment...${NC}"
    
    # Archive test results
    local timestamp=$(date +%Y%m%d_%H%M%S)
    mkdir -p test_archive
    cp test_results.csv "test_archive/edge_case_results_${timestamp}.csv"
    
    # Clean up temporary files
    rm -rf test_artifacts
    
    echo -e "${GREEN}✓ Cleanup Complete${NC}"
    echo ""
}

# Main execution flow
main() {
    echo -e "${BLUE}EDGE CASE TESTING SPECIALIST - STARTING COMPREHENSIVE TESTS${NC}"
    echo ""
    
    check_prerequisites
    setup_test_environment
    
    # Execute all edge case tests
    run_all_edge_cases
    
    # Generate comprehensive report
    generate_test_report
    
    # Cleanup
    cleanup_test_environment
    
    echo -e "${BLUE}===========================================${NC}"
    echo -e "${BLUE}EDGE CASE TESTING COMPLETE${NC}"
    echo -e "${BLUE}===========================================${NC}"
    echo ""
    echo "Results archived in test_archive/"
    echo "Review detailed output above for specific failure analysis."
}

# Handle script interruption
trap 'echo -e "\n${RED}Test execution interrupted${NC}"; cleanup_test_environment; exit 1' INT TERM

# Execute main function
main "$@"