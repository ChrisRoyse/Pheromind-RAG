@echo off
echo ==========================================================
echo SEARCH BASIC BUILD - Text Search Only
echo ==========================================================
echo Features: core + tantivy (full-text search with fuzzy matching)
echo Estimated time: 45-90 seconds
echo Resource usage: Medium memory, moderate CPU
echo.

cargo build --features "search-basic" --quiet
if %ERRORLEVEL% equ 0 (
    echo ✅ Search Basic build completed successfully
) else (
    echo ❌ Search Basic build failed
    exit /b 1
)