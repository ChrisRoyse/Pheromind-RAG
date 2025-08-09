@echo off
:: MCP Server Wrapper for Claude-Flow
:: This wrapper ensures the SQLite binding is available and starts the MCP server

setlocal enabledelayedexpansion

:: Deploy SQLite binding to all NPX cache directories
set "BINDING_SOURCE=C:\code\embed\better-sqlite3\build\Release\better_sqlite3.node"

if exist "%BINDING_SOURCE%" (
    for /f "delims=" %%i in ('npm config get cache 2^>nul') do set "NPM_CACHE=%%i"
    if defined NPM_CACHE (
        for /d %%d in ("!NPM_CACHE!\_npx\*") do (
            if exist "%%d\node_modules\better-sqlite3\" (
                if not exist "%%d\node_modules\better-sqlite3\build\Release\" (
                    mkdir "%%d\node_modules\better-sqlite3\build\Release\" 2>nul
                )
                copy /Y "%BINDING_SOURCE%" "%%d\node_modules\better-sqlite3\build\Release\better_sqlite3.node" >nul 2>&1
            )
        )
    )
)

:: Start the MCP server
npx claude-flow@alpha mcp start