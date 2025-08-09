@echo off
setlocal enabledelayedexpansion
echo ========================================
echo NPX Interceptor Verification
echo ========================================
echo.

set "INTERCEPT_DIR=C:\Users\hotra\AppData\Local\npm-cache\_npx\npx-intercept"
set "BINDING_FILE=%INTERCEPT_DIR%\permanent-binding.node"
set /a SCORE=0

echo [1/5] Checking permanent binding...
if exist "%BINDING_FILE%" (
    echo ✓ Permanent binding exists
    set /a SCORE+=20
) else (
    echo ✗ Permanent binding not found
)

echo.
echo [2/5] Checking interceptor script...
if exist "%INTERCEPT_DIR%\npx.cmd" (
    echo ✓ Interceptor script exists
    set /a SCORE+=20
) else (
    echo ✗ Interceptor script not found
)

echo.
echo [3/5] Checking PATH configuration...
echo %PATH% | findstr /C:"%INTERCEPT_DIR%" >nul 2>&1
if %ERRORLEVEL% EQU 0 (
    echo ✓ Interceptor in PATH
    set /a SCORE+=20
) else (
    echo ✗ Interceptor not in PATH
)

echo.
echo [4/5] Testing binding file size...
if exist "%BINDING_FILE%" (
    for %%A in ("%BINDING_FILE%") do set SIZE=%%~zA
    if !SIZE! GTR 1000000 (
        echo ✓ Binding file valid (!SIZE! bytes^)
        set /a SCORE+=20
    ) else (
        echo ✗ Binding file too small
    )
) else (
    echo ✗ Cannot check binding size
)

echo.
echo [5/5] Node.js compatibility check...
for /f "tokens=1" %%v in ('node -p "process.versions.modules" 2^>nul') do (
    if "%%v"=="127" (
        echo ✓ Node.js ABI version matches (127)
        set /a SCORE+=20
    ) else (
        echo ✗ Node.js ABI mismatch (expected 127, got %%v)
    )
)

echo.
echo ========================================
echo VERIFICATION SCORE: %SCORE%/100
echo ========================================
echo.

if %SCORE% EQU 100 (
    echo ✓✓✓ PERFECT! NPX interceptor is 100%% ready!
    echo.
    echo You can now run: npx claude-flow@alpha init
) else (
    echo ⚠ Setup incomplete. Run deploy-interceptor.bat to fix.
)
echo.