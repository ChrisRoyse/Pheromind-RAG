#!/bin/bash

# Nomic3 Brutal Stress Test Runner
# This script executes all 10 brutal stress tests designed to expose critical vulnerabilities

set -e

echo "🚨 NOMIC3 EMBEDDING SYSTEM BRUTAL STRESS TESTS"
echo "=============================================="
echo ""
echo "⚠️  WARNING: These tests are designed to BREAK the system"
echo "   They will expose critical vulnerabilities and failure modes"
echo "   DO NOT run in production environments"
echo ""
echo "📋 Test Coverage:"
echo "   1. Network Dependency Failure"
echo "   2. Memory Leak Validation" 
echo "   3. Quantization Format Breaking"
echo "   4. Index Threshold Violation"
echo "   5. Unicode Tokenization Chaos"
echo "   6. Dimension Mismatch Corruption"
echo "   7. NaN Injection Attack"
echo "   8. Concurrent Deadlock Induction"
echo "   9. Model Corruption Detection"
echo "   10. Embedding Cache Invalidation"
echo ""

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "❌ Error: Please run this script from the project root directory"
    exit 1
fi

# Check if test file exists
if [ ! -f "tests/nomic3_brutal_stress_tests.rs" ]; then
    echo "❌ Error: Stress test file not found at tests/nomic3_brutal_stress_tests.rs"
    exit 1
fi

echo "🔧 Building test suite..."
cargo build --tests --features ml || {
    echo "❌ Build failed. Trying without ml feature..."
    cargo build --tests || {
        echo "❌ Build failed completely. Check your Rust setup."
        exit 1
    }
}

echo ""
echo "🔥 STARTING BRUTAL STRESS TESTS"
echo "================================"

# Function to run individual test and capture results
run_test() {
    local test_name=$1
    local test_function=$2
    
    echo ""
    echo "🎯 Running: $test_name"
    echo "----------------------------------------"
    
    # Run with timeout to prevent hanging
    timeout 300s cargo test --features ml "$test_function" -- --nocapture --test-threads=1 2>&1 | tee "test_results_${test_function}.log" || {
        local exit_code=$?
        if [ $exit_code -eq 124 ]; then
            echo "⏰ TIMEOUT: Test $test_name exceeded 5 minutes (possible deadlock)"
        else
            echo "🚨 FAILURE DETECTED: Test $test_name failed with exit code $exit_code"
        fi
        echo "   This may indicate a critical vulnerability was exposed"
    }
    
    echo "📝 Results saved to: test_results_${test_function}.log"
}

# Run all individual tests
run_test "Network Dependency Failure" "test_1_network_dependency_failure"
run_test "Memory Leak Validation" "test_2_memory_leak_validation" 
run_test "Quantization Format Breaking" "test_3_quantization_format_breaking"
run_test "Index Threshold Violation" "test_4_index_threshold_violation"
run_test "Unicode Tokenization Chaos" "test_5_unicode_tokenization_chaos"
run_test "Dimension Mismatch Corruption" "test_6_dimension_mismatch_corruption"
run_test "NaN Injection Attack" "test_7_nan_injection_attack"
run_test "Concurrent Deadlock Induction" "test_8_concurrent_deadlock_induction"
run_test "Model Corruption Detection" "test_9_model_corruption_detection"
run_test "Embedding Cache Invalidation" "test_10_embedding_cache_invalidation"

echo ""
echo "🎯 Running Complete Test Suite"
echo "=============================="

# Run the comprehensive test suite
timeout 600s cargo test --features ml run_all_brutal_stress_tests -- --nocapture --test-threads=1 2>&1 | tee "test_results_complete_suite.log" || {
    local exit_code=$?
    if [ $exit_code -eq 124 ]; then
        echo "⏰ TIMEOUT: Complete test suite exceeded 10 minutes"
    else
        echo "🚨 COMPREHENSIVE TEST FAILURE: Exit code $exit_code"
    fi
}

echo ""
echo "📊 STRESS TEST EXECUTION COMPLETE"
echo "=================================="
echo ""
echo "📁 Test logs created:"
ls -la test_results_*.log 2>/dev/null || echo "   No test logs found"

echo ""
echo "🔍 ANALYZING RESULTS..."
echo ""

# Analyze test results
failure_count=0
vulnerability_count=0

for log_file in test_results_*.log; do
    if [ -f "$log_file" ]; then
        echo "📄 Analyzing: $log_file"
        
        # Count failures and vulnerabilities
        if grep -q "FAILURE\|ERROR\|PANIC\|CRASH" "$log_file"; then
            ((failure_count++))
            echo "   🚨 Contains failure indicators"
        fi
        
        if grep -q "VULNERABILITY CONFIRMED\|CRITICAL VULNERABILITY" "$log_file"; then
            ((vulnerability_count++))
            echo "   🎯 Contains vulnerability confirmations"
        fi
        
        # Extract key findings
        if grep -q "DEADLOCK DETECTED" "$log_file"; then
            echo "   🔒 DEADLOCK: Concurrent access issues detected"
        fi
        
        if grep -q "MEMORY LEAK" "$log_file"; then
            echo "   💾 MEMORY LEAK: Memory management issues detected"
        fi
        
        if grep -q "CORRUPTION" "$log_file"; then
            echo "   💀 CORRUPTION: Data integrity issues detected"
        fi
        
        if grep -q "INJECTION SUCCESS" "$log_file"; then
            echo "   💉 INJECTION: Security validation bypassed"
        fi
    fi
done

echo ""
echo "📈 FINAL SUMMARY"
echo "================"
echo "Tests executed: 10 + 1 comprehensive suite"
echo "Test failures: $failure_count"
echo "Vulnerabilities confirmed: $vulnerability_count"
echo ""

if [ $vulnerability_count -gt 0 ]; then
    echo "🚨 CRITICAL: $vulnerability_count vulnerabilities confirmed by stress testing"
    echo "   System is NOT ready for production deployment"
    echo "   Review test logs and implement fixes before proceeding"
    echo ""
    echo "📋 Next Steps:"
    echo "   1. Review all test_results_*.log files for detailed failure analysis"
    echo "   2. Read docs/nomic3_brutal_stress_test_analysis.md for remediation guidance"
    echo "   3. Implement fixes for each confirmed vulnerability"
    echo "   4. Re-run stress tests to verify fixes"
    echo "   5. Only deploy after ALL tests pass without vulnerabilities"
else
    echo "✅ No critical vulnerabilities detected in stress testing"
    echo "   System appears robust under stress conditions"
    echo "   Continue with normal testing and deployment procedures"
fi

echo ""
echo "🔗 Additional Resources:"
echo "   📖 Analysis Report: docs/nomic3_brutal_stress_test_analysis.md"
echo "   🧪 Test Source: tests/nomic3_brutal_stress_tests.rs"
echo "   📊 Test Logs: test_results_*.log"

echo ""
echo "⚠️  REMEMBER: These tests are designed to expose vulnerabilities"
echo "   Test failures often indicate successful vulnerability detection"
echo "   Use results to harden the system, not to blame the tests"

exit 0