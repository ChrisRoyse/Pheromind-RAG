@echo off
echo ==========================================================
echo OPTIMIZED BUILD - Resource-Conscious Compilation
echo ==========================================================
echo Features: Incremental compilation with memory limits
echo Strategy: Single-threaded compilation to avoid resource exhaustion
echo.

set CARGO_BUILD_JOBS=1
set CARGO_TARGET_DIR=%cd%\target_optimized

echo Building with single thread and separate target directory...
cargo build --features "tree-sitter,tantivy" --quiet
if %ERRORLEVEL% equ 0 (
    echo ✅ Optimized build completed successfully
) else (
    echo ❌ Optimized build failed
    exit /b 1
)