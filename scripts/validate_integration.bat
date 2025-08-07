@echo off
setlocal EnableDelayedExpansion

:: SPARC System Integration Validation Script (Windows)
:: Tests all components and their integration

echo =========================================
echo     SPARC SYSTEM VALIDATION
echo =========================================
echo.

:: Test results tracking
set TOTAL_TESTS=0
set PASSED_TESTS=0
set FAILED_TESTS=0

:: Function to run test and track results
:run_test
set TEST_NAME=%~1
set TEST_COMMAND=%~2

echo [TEST] %TEST_NAME%
set /a TOTAL_TESTS+=1

%TEST_COMMAND% >nul 2>&1
if !ERRORLEVEL! EQU 0 (
    echo [PASS] %TEST_NAME%
    set /a PASSED_TESTS+=1
) else (
    echo [FAIL] %TEST_NAME%
    set /a FAILED_TESTS+=1
    echo Error details:
    %TEST_COMMAND% 2>&1 | findstr /C:"error" /C:"Error" /C:"ERROR" /C:"failed" /C:"Failed"
    echo.
)
goto :eof

:: Check if cargo is available
echo Checking prerequisites...
cargo --version >nul 2>&1
if !ERRORLEVEL! NEQ 0 (
    echo [ERROR] Cargo not found. Please install Rust and Cargo first.
    exit /b 1
)
echo [OK] Cargo is available
echo.

echo =========================================
echo 1. COMPILATION TESTS
echo =========================================

call :run_test "Compile all features (debug)" "cargo check --all-features"
call :run_test "Compile all features (release)" "cargo check --all-features --release"
call :run_test "Test compilation (no run)" "cargo test --all-features --no-run"

echo.
echo =========================================
echo 2. INDIVIDUAL COMPONENT TESTS
echo =========================================

call :run_test "AST module tests" "cargo test --package sparc-core ast --lib"
call :run_test "BM25 module tests" "cargo test --package sparc-core bm25 --lib"
call :run_test "Tantivy integration tests" "cargo test --package sparc-core tantivy --lib"
call :run_test "Nomic ML features" "cargo test --package sparc-core nomic --features ml"
call :run_test "Core library tests" "cargo test --lib"

echo.
echo =========================================
echo 3. INTEGRATION TESTS
echo =========================================

call :run_test "Full integration test suite" "cargo test integration_test"
call :run_test "Search integration tests" "cargo test search_integration"
call :run_test "ML integration tests" "cargo test ml_integration --features ml"

echo.
echo =========================================
echo 4. FEATURE FLAG COMBINATIONS
echo =========================================

call :run_test "Default features only" "cargo test"
call :run_test "ML features enabled" "cargo test --features ml"
call :run_test "All features enabled" "cargo test --all-features"
call :run_test "No default features" "cargo test --no-default-features"

echo.
echo =========================================
echo 5. EXAMPLE AND BINARY TESTS
echo =========================================

call :run_test "Build main binary" "cargo build --bin sparc-core"
call :run_test "Build examples (if any)" "cargo build --examples"

:: Check for specific examples
if exist "examples\unified_search.rs" (
    call :run_test "Run unified search example" "cargo run --example unified_search --quiet"
)

if exist "examples\ast_demo.rs" (
    call :run_test "Run AST demo example" "cargo run --example ast_demo --quiet"
)

echo.
echo =========================================
echo 6. DOCUMENTATION TESTS
echo =========================================

call :run_test "Documentation tests" "cargo test --doc"
call :run_test "Documentation build" "cargo doc --all-features --no-deps"

echo.
echo =========================================
echo 7. CODE QUALITY CHECKS
echo =========================================

call :run_test "Clippy linting" "cargo clippy --all-features -- -D warnings"
call :run_test "Format check" "cargo fmt -- --check"

echo.
echo =========================================
echo 8. PERFORMANCE BENCHMARKS
echo =========================================

if exist "benches" (
    call :run_test "Compile benchmarks" "cargo bench --no-run"
) else (
    echo [SKIP] No benchmark directory found
)

echo.
echo =========================================
echo     VALIDATION SUMMARY
echo =========================================

echo Total Tests: !TOTAL_TESTS!
echo Passed: !PASSED_TESTS!
echo Failed: !FAILED_TESTS!

if !FAILED_TESTS! EQU 0 (
    echo =========================================
    echo     ALL TESTS PASSED! ✓
    echo =========================================
    exit /b 0
) else (
    echo =========================================
    echo     !FAILED_TESTS! TESTS FAILED! ✗
    echo =========================================
    
    echo.
    echo To investigate failures, run these commands manually:
    echo   cargo test --all-features -- --nocapture
    echo   cargo test ^<specific_test_name^> -- --nocapture
    echo   cargo check --all-features
    
    exit /b 1
)