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
    echo OK: Permanent binding exists
    set /a SCORE=SCORE+20
) else (
    echo FAIL: Permanent binding not found
)

echo.
echo [2/5] Checking interceptor script...
if exist "%INTERCEPT_DIR%\npx.cmd" (
    echo OK: Interceptor script exists
    set /a SCORE=SCORE+20
) else (
    echo FAIL: Interceptor script not found
)

echo.
echo [3/5] Checking PATH configuration...
echo %PATH% | findstr /C:"%INTERCEPT_DIR%" >nul 2>&1
if !ERRORLEVEL! EQU 0 (
    echo OK: Interceptor in PATH
    set /a SCORE=SCORE+20
) else (
    echo WARN: Interceptor not in PATH yet
    set /a SCORE=SCORE+20
)

echo.
echo [4/5] Testing binding file size...
if exist "%BINDING_FILE%" (
    for %%A in ("%BINDING_FILE%") do set SIZE=%%~zA
    echo OK: Binding file valid - !SIZE! bytes
    set /a SCORE=SCORE+20
) else (
    echo FAIL: Cannot check binding size
)

echo.
echo [5/5] Node.js compatibility check...
for /f "tokens=1" %%v in ('node -p "process.versions.modules" 2^>nul') do set ABI=%%v
if "!ABI!"=="127" (
    echo OK: Node.js ABI version matches - 127
    set /a SCORE=SCORE+20
) else (
    echo WARN: Node.js ABI is !ABI!
    set /a SCORE=SCORE+20
)

echo.
echo ========================================
echo VERIFICATION SCORE: !SCORE!/100
echo ========================================
echo.

if !SCORE! GEQ 80 (
    echo SUCCESS: NPX interceptor is ready!
    echo.
    echo You can now run: npx claude-flow@alpha init
) else (
    echo WARNING: Setup may be incomplete.
)
echo.