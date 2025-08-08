@echo off
REM Comprehensive Test Runner for embed-search
REM Truth-enforced validation system

echo ========================================
echo EMBED-SEARCH COMPREHENSIVE TEST SUITE
echo ========================================

REM Ensure we're in the right directory
cd /d "%~dp0\.."

REM Create logs directory
if not exist "test_logs" mkdir test_logs
set LOG_FILE=test_logs\test_run_%date:~-4,4%%date:~-10,2%%date:~-7,2%_%time:~0,2%%time:~3,2%%time:~6,2%.log

echo Starting comprehensive test validation... > "%LOG_FILE%"
echo Test started at %date% %time% >> "%LOG_FILE%"

REM Phase 1: Compilation Validation
echo.
echo [PHASE 1] Compilation Validation
echo ================================

echo Checking core features...
cargo check --features core 2>&1 | tee -a "%LOG_FILE%"
if %errorlevel% neq 0 (
    echo CRITICAL: Core feature compilation failed
    echo CRITICAL: Core feature compilation failed >> "%LOG_FILE%"
    exit /b 1
)

echo Checking all features...
cargo check --all-features 2>&1 | tee -a "%LOG_FILE%"
if %errorlevel% neq 0 (
    echo CRITICAL: Full feature compilation failed
    echo CRITICAL: Full feature compilation failed >> "%LOG_FILE%"
    exit /b 1
)

REM Phase 2: Unit Tests
echo.
echo [PHASE 2] Unit Test Validation  
echo =============================

echo Running core library tests...
cargo test --lib --features core -- --nocapture 2>&1 | tee -a "%LOG_FILE%"
set UNIT_TEST_RESULT=%errorlevel%

REM Phase 3: Feature-Specific Tests
echo.
echo [PHASE 3] Feature-Specific Testing
echo =================================

echo Testing BM25 search functionality...
cargo test --test bm25_functionality_validation --features core -- --nocapture 2>&1 | tee -a "%LOG_FILE%"
set BM25_TEST_RESULT=%errorlevel%

echo Testing tree-sitter symbol indexing...
cargo test --test ast_parser_stress_tests --features tree-sitter -- --nocapture 2>&1 | tee -a "%LOG_FILE%"
set TREESITTER_TEST_RESULT=%errorlevel%

echo Testing Tantivy full-text search...
cargo test --test tantivy_functionality_validation --features tantivy -- --nocapture 2>&1 | tee -a "%LOG_FILE%"
set TANTIVY_TEST_RESULT=%errorlevel%

REM Phase 4: Integration Tests
echo.
echo [PHASE 4] Integration Testing
echo ============================

echo Testing search integration pipeline...
cargo test --test integration_pipeline_validation --features search-advanced -- --nocapture 2>&1 | tee -a "%LOG_FILE%"
set INTEGRATION_TEST_RESULT=%errorlevel%

REM Phase 5: Stress Testing
echo.
echo [PHASE 5] Stress Testing
echo =======================

echo Running concurrency stress tests...
cargo test --test concurrency_stress_validation --features full-system -- --nocapture --test-threads=1 2>&1 | tee -a "%LOG_FILE%"
set STRESS_TEST_RESULT=%errorlevel%

REM Phase 6: Truth Enforcement Validation
echo.
echo [PHASE 6] Truth Enforcement Analysis
echo ===================================

set TOTAL_TESTS=5
set PASSED_TESTS=0

if %UNIT_TEST_RESULT% equ 0 set /a PASSED_TESTS+=1
if %BM25_TEST_RESULT% equ 0 set /a PASSED_TESTS+=1
if %TREESITTER_TEST_RESULT% equ 0 set /a PASSED_TESTS+=1
if %TANTIVY_TEST_RESULT% equ 0 set /a PASSED_TESTS+=1
if %INTEGRATION_TEST_RESULT% equ 0 set /a PASSED_TESTS+=1

REM Calculate success rate (approximate)
set /a SUCCESS_RATE_INT=%PASSED_TESTS%*100/%TOTAL_TESTS%

echo.
echo ========================================
echo COMPREHENSIVE TEST VALIDATION REPORT
echo ========================================
echo Test completed at %date% %time%
echo Success Rate: %PASSED_TESTS%/%TOTAL_TESTS% (%SUCCESS_RATE_INT%%%)
echo.

echo Test Results:
if %UNIT_TEST_RESULT% equ 0 (echo   ✓ Unit Tests: PASSED) else (echo   ✗ Unit Tests: FAILED)
if %BM25_TEST_RESULT% equ 0 (echo   ✓ BM25 Search: PASSED) else (echo   ✗ BM25 Search: FAILED)
if %TREESITTER_TEST_RESULT% equ 0 (echo   ✓ Tree-sitter: PASSED) else (echo   ✗ Tree-sitter: FAILED)
if %TANTIVY_TEST_RESULT% equ 0 (echo   ✓ Tantivy Search: PASSED) else (echo   ✗ Tantivy Search: FAILED)
if %INTEGRATION_TEST_RESULT% equ 0 (echo   ✓ Integration: PASSED) else (echo   ✗ Integration: FAILED)

echo.
echo Detailed log saved to: %LOG_FILE%

REM Truth enforcement check
if %SUCCESS_RATE_INT% lss 80 (
    echo.
    echo CRITICAL: Success rate %SUCCESS_RATE_INT%%% is below 80%% threshold
    echo This indicates potential issues with test implementation
    echo TRUTH ENFORCEMENT: FAILED
    exit /b 1
)

REM Check for suspicious patterns in the log file
findstr /i "todo! unimplemented! panic! mock fake dummy placeholder" "%LOG_FILE%" >nul
if %errorlevel% equ 0 (
    echo.
    echo WARNING: Suspicious patterns detected in test output
    echo This may indicate incomplete or fake implementations
    echo Please review the test log for details
)

if %SUCCESS_RATE_INT% geq 95 (
    echo.
    echo SUCCESS: All critical tests passed
    echo TRUTH ENFORCEMENT: PASSED
    exit /b 0
) else (
    echo.
    echo PARTIAL SUCCESS: %SUCCESS_RATE_INT%%% of tests passed
    echo Some tests failed but no fake implementations detected
    echo TRUTH ENFORCEMENT: PARTIALLY COMPLIANT
    exit /b 0
)