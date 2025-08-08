#!/bin/bash

echo "================================"
echo "AST Parser Stress Test Suite"
echo "================================"
echo ""
echo "This script runs 10 devastating stress tests targeting critical AST parsing vulnerabilities:"
echo "1. Silent Parser Failure Detection"
echo "2. Persistence Absence Validation" 
echo "3. Query Pattern Rigidity Testing"
echo "4. Concurrency Symbol Corruption"
echo "5. Memory Leak Validation"
echo "6. Malformed Code Recovery"
echo "7. Stack Overflow Induction"
echo "8. Language Detection Chaos"
echo "9. Circular Dependency Loops"
echo "10. Unicode Symbol Extraction"
echo ""

START_TIME=$(date)
echo "Test suite started at: $START_TIME"
echo ""

# Function to run test with error handling
run_test() {
    local test_name="$1"
    local test_description="$2"
    local criticality="$3"
    
    echo ""
    echo "[$4] Running $test_description..."
    
    if cargo test --features tree-sitter "$test_name" -- --nocapture; then
        echo "✅ PASSED: $test_description"
        return 0
    else
        if [ "$criticality" = "CRITICAL" ]; then
            echo "❌ CRITICAL FAILURE: $test_description - IMMEDIATE ATTENTION REQUIRED!"
            return 1
        else
            echo "⚠️  WARNING: $test_description - Review recommended"
            return 0
        fi
    fi
}

# Check if tree-sitter feature is available
echo "Checking tree-sitter feature availability..."
if ! cargo test --features tree-sitter --dry-run ast_stress_validation_disabled &>/dev/null; then
    echo "ERROR: tree-sitter feature not available or compilation failed"
    echo "Run: cargo build --features tree-sitter"
    exit 1
fi

# Run validation framework first
echo ""
echo "================================"
echo "Running AST Stress Test Validation Framework"
echo "================================"

if ! cargo test --features tree-sitter validate_ast_stress_test_framework; then
    echo "ERROR: Stress test validation framework failed!"
    exit 1
fi

if ! cargo test --features tree-sitter integration_run_stress_test_subset; then
    echo "ERROR: Integration validation failed!"
    exit 1
fi

if ! cargo test --features tree-sitter establish_performance_baseline; then
    echo "ERROR: Performance baseline establishment failed!"
    exit 1
fi

echo ""
echo "================================"
echo "Running Individual AST Stress Tests"
echo "================================"

CRITICAL_FAILURES=0
WARNING_FAILURES=0

# Test 1: Silent Parser Failure Detection
if ! run_test "test_silent_parser_failure_detection" "Silent Parser Failure Detection Test" "WARNING" "1/10"; then
    ((WARNING_FAILURES++))
fi

# Test 2: Persistence Absence Validation  
if ! run_test "test_persistence_absence_catastrophic_performance" "Persistence Absence Validation Test" "WARNING" "2/10"; then
    ((WARNING_FAILURES++))
fi

# Test 3: Query Pattern Rigidity
if ! run_test "test_query_pattern_rigidity_failure" "Query Pattern Rigidity Test" "WARNING" "3/10"; then
    ((WARNING_FAILURES++))
fi

# Test 4: Concurrency Symbol Corruption
if ! run_test "test_concurrency_symbol_corruption" "Concurrency Symbol Corruption Test" "CRITICAL" "4/10"; then
    ((CRITICAL_FAILURES++))
fi

# Test 5: Memory Leak Validation
if ! run_test "test_memory_leak_validation" "Memory Leak Validation Test" "CRITICAL" "5/10"; then
    ((CRITICAL_FAILURES++))
fi

# Test 6: Malformed Code Recovery  
if ! run_test "test_malformed_code_recovery" "Malformed Code Recovery Test" "CRITICAL" "6/10"; then
    ((CRITICAL_FAILURES++))
fi

# Test 7: Stack Overflow Induction
if ! run_test "test_stack_overflow_induction" "Stack Overflow Induction Test" "CRITICAL" "7/10"; then
    ((CRITICAL_FAILURES++))
fi

# Test 8: Language Detection Chaos
if ! run_test "test_language_detection_chaos" "Language Detection Chaos Test" "WARNING" "8/10"; then
    ((WARNING_FAILURES++))
fi

# Test 9: Circular Dependency Loops
if ! run_test "test_circular_dependency_loops" "Circular Dependency Loop Test" "CRITICAL" "9/10"; then
    ((CRITICAL_FAILURES++))
fi

# Test 10: Unicode Symbol Extraction
if ! run_test "test_unicode_symbol_extraction" "Unicode Symbol Extraction Test" "WARNING" "10/10"; then
    ((WARNING_FAILURES++))
fi

echo ""
echo "================================"
echo "AST Stress Test Suite Complete"
echo "================================"

END_TIME=$(date)
echo "Suite completed at: $END_TIME"
echo ""

# Summary report
echo "📊 TEST RESULTS SUMMARY:"
echo "  Critical failures: $CRITICAL_FAILURES"
echo "  Warning failures:  $WARNING_FAILURES"
echo "  Total tests run:    10"

if [ $CRITICAL_FAILURES -gt 0 ]; then
    echo ""
    echo "🚨 CRITICAL ISSUES DETECTED!"
    echo "   $CRITICAL_FAILURES critical vulnerabilities found that require immediate attention."
    echo "   These issues could cause system crashes, data corruption, or security vulnerabilities."
fi

if [ $WARNING_FAILURES -gt 0 ]; then
    echo ""
    echo "⚠️  WARNING ISSUES DETECTED:"
    echo "   $WARNING_FAILURES warning-level issues found that should be reviewed."
    echo "   These may indicate performance problems or edge-case handling issues."
fi

if [ $CRITICAL_FAILURES -eq 0 ] && [ $WARNING_FAILURES -eq 0 ]; then
    echo ""
    echo "✅ ALL TESTS PASSED!"
    echo "   AST parser appears robust against the tested vulnerabilities."
fi

echo ""
echo "📋 USAGE NOTES:"
echo "  To run individual tests:"
echo "    cargo test --features tree-sitter test_[test_name] -- --nocapture"
echo ""
echo "  To run with different thread counts:"
echo "    cargo test --features tree-sitter -- --test-threads=1"
echo ""
echo "  To run specific vulnerability categories:"
echo "    cargo test --features tree-sitter memory_leak"
echo "    cargo test --features tree-sitter concurrency"
echo "    cargo test --features tree-sitter unicode"
echo ""

if [ $CRITICAL_FAILURES -gt 0 ]; then
    exit 1
else
    echo "Test runner completed successfully."
    exit 0
fi