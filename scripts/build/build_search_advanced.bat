@echo off
echo ==========================================================
echo SEARCH ADVANCED BUILD - Text + Symbol Search
echo ==========================================================
echo Features: core + tree-sitter + tantivy (text + symbol search)
echo Estimated time: 60-120 seconds
echo Resource usage: Medium-High memory, moderate CPU
echo.

cargo build --features "search-advanced" --quiet
if %ERRORLEVEL% equ 0 (
    echo ✅ Search Advanced build completed successfully
) else (
    echo ❌ Search Advanced build failed
    exit /b 1
)