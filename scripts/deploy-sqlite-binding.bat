@echo off
echo Deploying SQLite binding to NPX cache...

set "BINDING_SOURCE=C:\code\embed\better-sqlite3\build\Release\better_sqlite3.node"
set /a COUNT=0

if not exist "%BINDING_SOURCE%" (
    echo ERROR: Binding not found at %BINDING_SOURCE%
    exit /b 1
)

for /f "delims=" %%i in ('npm config get cache 2^>nul') do set "NPM_CACHE=%%i"

if not defined NPM_CACHE (
    echo ERROR: Could not determine NPM cache location
    exit /b 1
)

echo NPM Cache: %NPM_CACHE%
echo.

for /d %%d in ("%NPM_CACHE%\_npx\*") do (
    if exist "%%d\node_modules\better-sqlite3\" (
        echo Found better-sqlite3 in: %%~nd
        
        if not exist "%%d\node_modules\better-sqlite3\build\Release\" (
            mkdir "%%d\node_modules\better-sqlite3\build\Release\" 2>nul
        )
        
        copy /Y "%BINDING_SOURCE%" "%%d\node_modules\better-sqlite3\build\Release\better_sqlite3.node" >nul 2>&1
        
        if !ERRORLEVEL! EQU 0 (
            echo   - Deployed binding successfully
            set /a COUNT+=1
        ) else (
            echo   - Failed to deploy binding
        )
    )
)

echo.
echo Deployed to %COUNT% NPX cache directories
echo.