@echo off
echo ================================
echo AST Parser Stress Test Suite
echo ================================
echo.
echo This script runs 10 devastating stress tests targeting critical AST parsing vulnerabilities:
echo 1. Silent Parser Failure Detection
echo 2. Persistence Absence Validation 
echo 3. Query Pattern Rigidity Testing
echo 4. Concurrency Symbol Corruption
echo 5. Memory Leak Validation
echo 6. Malformed Code Recovery
echo 7. Stack Overflow Induction
echo 8. Language Detection Chaos
echo 9. Circular Dependency Loops
echo 10. Unicode Symbol Extraction
echo.

set START_TIME=%time%
echo Test suite started at: %START_TIME%
echo.

REM Check if tree-sitter feature is available
echo Checking tree-sitter feature availability...
cargo test --features tree-sitter --dry-run ast_stress_validation_disabled 2>nul
if %errorlevel% neq 0 (
    echo ERROR: tree-sitter feature not available or compilation failed
    echo Run: cargo build --features tree-sitter
    goto :error
)

REM Run validation framework first
echo.
echo ================================
echo Running AST Stress Test Validation Framework
echo ================================
cargo test --features tree-sitter validate_ast_stress_test_framework
if %errorlevel% neq 0 (
    echo ERROR: Stress test validation framework failed!
    goto :error
)

cargo test --features tree-sitter integration_run_stress_test_subset  
if %errorlevel% neq 0 (
    echo ERROR: Integration validation failed!
    goto :error
)

cargo test --features tree-sitter establish_performance_baseline
if %errorlevel% neq 0 (
    echo ERROR: Performance baseline establishment failed!
    goto :error
)

echo.
echo ================================
echo Running Individual AST Stress Tests
echo ================================

REM Test 1: Silent Parser Failure Detection
echo.
echo [1/10] Running Silent Parser Failure Detection Test...
cargo test --features tree-sitter test_silent_parser_failure_detection -- --nocapture
if %errorlevel% neq 0 (
    echo WARNING: Silent parser failure test failed - this may indicate initialization issues
)

REM Test 2: Persistence Absence Validation  
echo.
echo [2/10] Running Persistence Absence Validation Test...
cargo test --features tree-sitter test_persistence_absence_catastrophic_performance -- --nocapture
if %errorlevel% neq 0 (
    echo WARNING: Persistence absence test failed - this may indicate performance issues
)

REM Test 3: Query Pattern Rigidity
echo.
echo [3/10] Running Query Pattern Rigidity Test...
cargo test --features tree-sitter test_query_pattern_rigidity_failure -- --nocapture
if %errorlevel% neq 0 (
    echo WARNING: Query pattern rigidity test failed - this may indicate inflexibility
)

REM Test 4: Concurrency Symbol Corruption
echo.
echo [4/10] Running Concurrency Symbol Corruption Test...
cargo test --features tree-sitter test_concurrency_symbol_corruption -- --nocapture
if %errorlevel% neq 0 (
    echo CRITICAL: Thread safety test failed - concurrency issues detected!
)

REM Test 5: Memory Leak Validation
echo.
echo [5/10] Running Memory Leak Validation Test...
cargo test --features tree-sitter test_memory_leak_validation -- --nocapture
if %errorlevel% neq 0 (
    echo CRITICAL: Memory leak test failed - memory management issues detected!
)

REM Test 6: Malformed Code Recovery  
echo.
echo [6/10] Running Malformed Code Recovery Test...
cargo test --features tree-sitter test_malformed_code_recovery -- --nocapture
if %errorlevel% neq 0 (
    echo CRITICAL: Malformed code recovery test failed - parser crashes detected!
)

REM Test 7: Stack Overflow Induction
echo.
echo [7/10] Running Stack Overflow Induction Test...
cargo test --features tree-sitter test_stack_overflow_induction -- --nocapture
if %errorlevel% neq 0 (
    echo CRITICAL: Stack overflow test failed - stack safety issues detected!
)

REM Test 8: Language Detection Chaos
echo.
echo [8/10] Running Language Detection Chaos Test...
cargo test --features tree-sitter test_language_detection_chaos -- --nocapture
if %errorlevel% neq 0 (
    echo WARNING: Language detection chaos test failed - may indicate detection issues
)

REM Test 9: Circular Dependency Loops
echo.
echo [9/10] Running Circular Dependency Loop Test...
cargo test --features tree-sitter test_circular_dependency_loops -- --nocapture  
if %errorlevel% neq 0 (
    echo CRITICAL: Circular dependency test failed - infinite loops detected!
)

REM Test 10: Unicode Symbol Extraction
echo.
echo [10/10] Running Unicode Symbol Extraction Test...
cargo test --features tree-sitter test_unicode_symbol_extraction -- --nocapture
if %errorlevel% neq 0 (
    echo WARNING: Unicode extraction test failed - internationalization issues detected!
)

echo.
echo ================================
echo AST Stress Test Suite Complete
echo ================================

set END_TIME=%time%
echo Suite completed at: %END_TIME%
echo.
echo All 10 devastating AST parser stress tests have been executed.
echo Review the output above for any CRITICAL failures that require immediate attention.
echo.
echo To run individual tests:
echo   cargo test --features tree-sitter test_[test_name] -- --nocapture
echo.
echo To run with different thread counts:
echo   cargo test --features tree-sitter -- --test-threads=1
echo.
goto :end

:error
echo.
echo ================================
echo AST Stress Test Suite FAILED
echo ================================
echo Check the error messages above and ensure:
echo 1. tree-sitter feature is enabled: cargo build --features tree-sitter
echo 2. All dependencies are installed: cargo build
echo 3. System has sufficient memory for stress testing
echo.
exit /b 1

:end
echo Test runner completed successfully.
pause