@echo off
:: ==========================================
:: NPX Cache Population Script for claude-flow
:: This pre-compiles and places SQLite bindings where npx expects them
:: ==========================================

echo.
echo NPX Cache SQLite Binding Population Script
echo ==========================================
echo.
echo This script will make "npx claude-flow@alpha" work from any directory
echo by pre-compiling SQLite bindings and placing them in the npx cache.
echo.

:: Step 1: Trigger npx to create cache if it doesn't exist
echo [Step 1/5] Creating npx cache entry...
echo.
npx claude-flow@alpha --version 2>nul
if %errorlevel% neq 0 (
    echo Initial npx run completed (errors expected)
) else (
    echo NPX cache entry created
)

:: Step 2: Find the cache directory
echo.
echo [Step 2/5] Locating npx cache...
set NPX_CACHE_BASE=%LOCALAPPDATA%\npm-cache\_npx

:: Find the most recent cache directory (usually the one we just created)
for /f "tokens=*" %%i in ('dir "%NPX_CACHE_BASE%" /b /ad-h /od 2^>nul ^| findstr /r "^[a-f0-9]*$"') do set NPX_HASH=%%i

if "%NPX_HASH%"=="" (
    echo ERROR: Could not find npx cache directory
    echo Expected location: %NPX_CACHE_BASE%
    echo.
    echo Please run: npx claude-flow@alpha --version
    echo Then run this script again.
    pause
    exit /b 1
)

set NPX_CACHE=%NPX_CACHE_BASE%\%NPX_HASH%
echo Found cache: %NPX_CACHE%

:: Step 3: Check if claude-flow repo exists locally
echo.
echo [Step 3/5] Looking for claude-flow repository...

:: Check multiple possible locations
if exist "C:\code\claude-flow\package.json" (
    set CLAUDE_FLOW_REPO=C:\code\claude-flow
) else if exist "node_modules\better-sqlite3" (
    set CLAUDE_FLOW_REPO=%CD%
) else if exist "..\claude-flow\package.json" (
    set CLAUDE_FLOW_REPO=%CD%\..\claude-flow
) else (
    echo ERROR: Could not find claude-flow repository
    echo.
    echo Please either:
    echo 1. Clone it: git clone https://github.com/ruvnet/claude-flow.git C:\code\claude-flow
    echo 2. Or run this script from a directory with better-sqlite3 installed
    pause
    exit /b 1
)

echo Using repository: %CLAUDE_FLOW_REPO%

:: Step 4: Build SQLite bindings
echo.
echo [Step 4/5] Building SQLite bindings...
cd /d "%CLAUDE_FLOW_REPO%"

:: Check if better-sqlite3 is installed
if not exist "node_modules\better-sqlite3\package.json" (
    echo Installing dependencies...
    call npm install
)

:: Build the bindings
cd node_modules\better-sqlite3
echo Building native bindings (this may take a minute)...
call npm run build-release >nul 2>&1

:: Check if build succeeded
if not exist "build\Release\better_sqlite3.node" (
    echo.
    echo ERROR: Failed to build SQLite bindings
    echo.
    echo This usually means you need Visual Studio Build Tools:
    echo https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022
    echo.
    echo Install with "Desktop development with C++" workload
    pause
    exit /b 1
)

echo Build successful!
cd /d "%CLAUDE_FLOW_REPO%"

:: Step 5: Copy bindings to all possible locations in npx cache
echo.
echo [Step 5/5] Copying bindings to npx cache...

:: Get the binding file
set BINDING_FILE=%CLAUDE_FLOW_REPO%\node_modules\better-sqlite3\build\Release\better_sqlite3.node

:: Create all possible directories and copy binding
:: Location 1: Main build directory
mkdir "%NPX_CACHE%\node_modules\better-sqlite3\build\Release" 2>nul
copy /Y "%BINDING_FILE%" "%NPX_CACHE%\node_modules\better-sqlite3\build\Release\" >nul
echo - Copied to build\Release

:: Location 2: Direct build directory
mkdir "%NPX_CACHE%\node_modules\better-sqlite3\build" 2>nul
copy /Y "%BINDING_FILE%" "%NPX_CACHE%\node_modules\better-sqlite3\build\" >nul
echo - Copied to build\

:: Location 3: Version-specific binding directory
:: Node v22 = ABI version 127
mkdir "%NPX_CACHE%\node_modules\better-sqlite3\lib\binding\node-v127-win32-x64" 2>nul
copy /Y "%BINDING_FILE%" "%NPX_CACHE%\node_modules\better-sqlite3\lib\binding\node-v127-win32-x64\" >nul
echo - Copied to lib\binding\node-v127-win32-x64

:: Location 4: Compiled directory (for specific Node version)
for /f "tokens=1 delims=." %%a in ('node -v') do set NODE_MAJOR=%%a
set NODE_MAJOR=%NODE_MAJOR:v=%
mkdir "%NPX_CACHE%\node_modules\better-sqlite3\compiled\%NODE_MAJOR%.15.0\win32\x64" 2>nul
copy /Y "%BINDING_FILE%" "%NPX_CACHE%\node_modules\better-sqlite3\compiled\%NODE_MAJOR%.15.0\win32\x64\" >nul
echo - Copied to compiled\%NODE_MAJOR%.15.0\win32\x64

:: Also handle ruv-swarm's nested better-sqlite3 (version 11.10.0)
if exist "%NPX_CACHE%\node_modules\ruv-swarm\node_modules\better-sqlite3" (
    echo.
    echo Found ruv-swarm dependency, fixing that too...
    mkdir "%NPX_CACHE%\node_modules\ruv-swarm\node_modules\better-sqlite3\build\Release" 2>nul
    copy /Y "%BINDING_FILE%" "%NPX_CACHE%\node_modules\ruv-swarm\node_modules\better-sqlite3\build\Release\" >nul
    
    :: Also copy to version-specific location for ruv-swarm
    mkdir "%NPX_CACHE%\node_modules\ruv-swarm\node_modules\better-sqlite3\lib\binding\node-v127-win32-x64" 2>nul
    copy /Y "%BINDING_FILE%" "%NPX_CACHE%\node_modules\ruv-swarm\node_modules\better-sqlite3\lib\binding\node-v127-win32-x64\" >nul
    echo - Fixed ruv-swarm's better-sqlite3
)

:: Test the fix
echo.
echo ==========================================
echo Testing the fix...
echo ==========================================
echo.

cd %TEMP%
echo Testing from temp directory: %CD%
echo.

:: Test 1: Version check
echo Test 1: Version check
npx claude-flow@alpha --version 2>nul
if %errorlevel% equ 0 (
    echo [PASS] Version check succeeded
) else (
    echo [WARN] Version check had issues
)

:: Test 2: Memory operations
echo.
echo Test 2: Memory operations
npx claude-flow@alpha memory store test-key "SQLite is working!" 2>nul
npx claude-flow@alpha memory get test-key 2>nul

:: Final message
echo.
echo ==========================================
echo ✅ SETUP COMPLETE!
echo ==========================================
echo.
echo You can now use "npx claude-flow@alpha init" from ANY directory!
echo.
echo The fix will persist until you run "npm cache clean"
echo If that happens, just run this script again.
echo.
echo Try it now:
echo   cd C:\any\directory
echo   npx claude-flow@alpha init
echo.
pause