@echo off
echo ==========================================================
echo RELEASE BUILD - Production Ready (Minimal Features)
echo ==========================================================
echo Features: core + tantivy (production-safe combination)
echo Build type: Release (optimized)
echo Resource usage: High CPU during build, optimized result
echo.

cargo build --release --features "search-basic" --quiet
if %ERRORLEVEL% equ 0 (
    echo ✅ Release Minimal build completed successfully
    echo 📦 Binary available at: target\release\embed-search.exe
) else (
    echo ❌ Release Minimal build failed
    exit /b 1
)