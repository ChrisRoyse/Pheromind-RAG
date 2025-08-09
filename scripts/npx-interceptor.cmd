@echo off
setlocal enabledelayedexpansion

:: NPX Interceptor for Your Machine
:: Ensures claude-flow@alpha always works

set "INTERCEPT_DIR=C:\Users\hotra\AppData\Local\npm-cache\_npx\npx-intercept\"
set "BINDING_FILE=%INTERCEPT_DIR%permanent-binding.node"

:: Check if this is a claude-flow command
echo %* | findstr /C:"claude-flow" >nul 2>&1
if %ERRORLEVEL% EQU 0 (
    :: Deploy binding silently
    if exist "%BINDING_FILE%" (
        for /f "delims=" %%i in ('npm config get cache 2^>nul') do set "NPM_CACHE=%%i"
        if defined NPM_CACHE (
            for /d %%d in ("!NPM_CACHE!\_npx\*") do (
                if exist "%%d\node_modules\better-sqlite3\" (
                    xcopy "%BINDING_FILE%" "%%d\node_modules\better-sqlite3\build\Release\better_sqlite3.node" /Y /Q >nul 2>&1
                )
            )
        )
    )
)

:: Find and execute real NPX
set "REAL_NPX="
where npx.cmd >nul 2>&1
if %ERRORLEVEL% EQU 0 (
    for /f "delims=" %%i in ('where npx.cmd 2^>nul') do (
        if not "%%~dpi"=="%~dp0" (
            set "REAL_NPX=%%i"
            goto :found
        )
    )
)
:found

if defined REAL_NPX (
    "%REAL_NPX%" %*
) else (
    echo Error: Could not find real NPX
    exit /b 1
)