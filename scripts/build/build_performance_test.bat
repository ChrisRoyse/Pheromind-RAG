@echo off
echo ==========================================================
echo PERFORMANCE BUILD TEST - Resource Monitoring
echo ==========================================================
echo Testing different feature combinations with timing
echo.

set CARGO_BUILD_JOBS=1
echo Single-threaded compilation enabled (CARGO_BUILD_JOBS=1)
echo.

echo === Test 1: Core Build ===
echo Building core features...
powershell -Command "Measure-Command { cargo build --quiet 2>&1 | Out-Host }"
echo.

echo === Test 2: Search Basic ===
echo Building search-basic features...
powershell -Command "Measure-Command { cargo build --features 'search-basic' --quiet 2>&1 | Out-Host }"
echo.

echo === Test 3: Search Advanced ===
echo Building search-advanced features...
powershell -Command "Measure-Command { cargo build --features 'search-advanced' --quiet 2>&1 | Out-Host }"
echo.

echo === Test 4: Individual Tree-sitter ===
echo Building tree-sitter only...
powershell -Command "Measure-Command { cargo build --features 'tree-sitter' --quiet 2>&1 | Out-Host }"
echo.

echo === Test 5: Individual Tantivy ===
echo Building tantivy only...
powershell -Command "Measure-Command { cargo build --features 'tantivy' --quiet 2>&1 | Out-Host }"
echo.

echo Performance test complete!