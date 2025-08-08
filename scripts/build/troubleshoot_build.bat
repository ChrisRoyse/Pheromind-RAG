@echo off
echo ==========================================================
echo BUILD TROUBLESHOOTING SCRIPT
echo ==========================================================
echo Diagnosing build issues and resource constraints
echo.

echo === System Information ===
echo Available Memory:
wmic computersystem get TotalPhysicalMemory
echo.
echo Available Disk Space:
wmic logicaldisk where "DeviceID='C:'" get Size,FreeSpace
echo.
echo CPU Information:
wmic cpu get Name,NumberOfCores,NumberOfLogicalProcessors
echo.

echo === Cargo Environment ===
echo Cargo Version:
cargo --version
echo.
echo Rust Version:
rustc --version
echo.
echo Build Jobs Setting:
echo CARGO_BUILD_JOBS=%CARGO_BUILD_JOBS%
echo.

echo === Target Directory Size ===
if exist target\ (
    echo Target directory exists
    dir target /s /-c | findstr "bytes"
) else (
    echo Target directory does not exist
)
echo.

echo === Previous Build Artifacts ===
if exist target\debug\embed-search.exe (
    echo Debug binary exists: target\debug\embed-search.exe
    dir target\debug\embed-search.exe
) else (
    echo Debug binary not found
)

if exist target\release\embed-search.exe (
    echo Release binary exists: target\release\embed-search.exe
    dir target\release\embed-search.exe
) else (
    echo Release binary not found
)
echo.

echo === Process Check ===
echo Checking for running Rust/Cargo processes:
tasklist | findstr /I cargo
tasklist | findstr /I rustc
echo.

echo === Recommended Actions ===
echo 1. If builds are timing out:
echo    - Use single-threaded: set CARGO_BUILD_JOBS=1
echo    - Clean build cache: cargo clean
echo    - Close other applications
echo.
echo 2. If memory is low (^<8GB available):
echo    - Build minimal features: search-basic or core only
echo    - Use release builds (more memory efficient)
echo.
echo 3. If disk space is low (^<5GB free):
echo    - Clean target directory: cargo clean
echo    - Remove unused dependencies
echo.
echo Troubleshooting complete!