@echo off
echo ========================================
echo Deploying NPX Interceptor
echo ========================================
echo.

set "TARGET_DIR=C:\Users\hotra\AppData\Local\npm-cache\_npx\npx-intercept"

:: Copy interceptor script
echo Copying interceptor script...
copy /Y scripts\npx-interceptor.cmd "%TARGET_DIR%\npx.cmd" >nul 2>&1
if %ERRORLEVEL% NEQ 0 (
    echo Failed to copy interceptor script
    exit /b 1
)

:: Add to PATH
echo Adding to user PATH...
setx PATH "%TARGET_DIR%;%PATH%" >nul 2>&1

echo.
echo ✓ Interceptor deployed successfully!
echo.
echo Please open a NEW terminal for PATH changes to take effect.
echo Then run: npx claude-flow@alpha init
echo.