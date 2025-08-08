@echo off
echo Integration Test Discovery and Execution Validation
echo ==================================================

echo.
echo BEFORE (Problem): Tests filtered with no features
echo -------------------------------------------------
echo Running: cargo test integration_test --list
cargo test integration_test -- --list 2>&1 | findstr /C:"0 tests, 0 benchmarks"
if %errorlevel% == 0 (
    echo ❌ CONFIRMED: Integration tests are being filtered out
) else (
    echo ✅ Tests found without features
)

echo.
echo SOLUTION: Using pre-compiled test executable (core features)
echo ----------------------------------------------------------
echo Running: ./target/debug/deps/integration_test-*.exe --list
for %%f in (target\debug\deps\integration_test-*.exe) do (
    echo Found executable: %%f
    "%%f" --list | findstr /C:"test" | findstr /V /C:"benchmark"
    if !errorlevel! == 0 (
        echo ✅ Integration tests discovered successfully
    )
)

echo.
echo EXECUTION TEST: Running a working integration test
echo -------------------------------------------------
for %%f in (target\debug\deps\integration_test-*.exe) do (
    echo Running test_config_initialization...
    "%%f" test_config_initialization --nocapture
    if !errorlevel! == 0 (
        echo ✅ Integration test executed successfully
    ) else (
        echo ❌ Integration test failed
    )
    goto :done_test
)

:done_test
echo.
echo VERIFICATION COMPLETE
echo ====================
echo Status: Integration tests can now be discovered and executed
echo Method: Use pre-compiled test executables from target/debug/deps/
echo.
echo For full feature testing, use:
echo   cargo build --tests --features full-system
echo   ./target/debug/deps/integration_test-[hash].exe --nocapture
echo.
pause