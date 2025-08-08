@echo off
echo MCP Server Verification Script
echo ==============================

echo Compiling verification script...
cd ..
cargo run --bin=verification --manifest-path=scripts/Cargo.toml
if errorlevel 1 (
    echo Failed to run verification script
    pause
    exit /b 1
)

echo.
echo Verification complete!
pause