@echo off
echo ==========================================================
echo CORE BUILD - Minimal Dependencies
echo ==========================================================
echo Features: core (default - BM25, basic text processing)
echo Estimated time: 30-60 seconds
echo Resource usage: Low memory, minimal CPU
echo.

cargo build --quiet
if %ERRORLEVEL% equ 0 (
    echo ✅ Core build completed successfully
) else (
    echo ❌ Core build failed
    exit /b 1
)