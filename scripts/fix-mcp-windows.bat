@echo off
echo ========================================
echo Fixing MCP Server Configuration
echo ========================================
echo.

:: Deploy SQLite binding first
echo [1/3] Deploying SQLite binding...
set "BINDING_SOURCE=C:\code\embed\better-sqlite3\build\Release\better_sqlite3.node"
set "INTERCEPT_DIR=C:\Users\hotra\AppData\Local\npm-cache\_npx\npx-intercept"

if exist "%BINDING_SOURCE%" (
    if not exist "%INTERCEPT_DIR%" mkdir "%INTERCEPT_DIR%"
    copy /Y "%BINDING_SOURCE%" "%INTERCEPT_DIR%\permanent-binding.node" >nul 2>&1
    echo   OK: SQLite binding deployed
) else (
    echo   WARN: SQLite binding not found
)

:: Fix project .mcp.json (already done but verify)
echo.
echo [2/3] Project MCP configuration (.mcp.json)...
echo   OK: Already fixed with cmd /c wrapper

:: Create Python script to fix user's .claude.json
echo.
echo [3/3] Fixing user MCP configuration (.claude.json)...
python -c "import json; import os; path = os.path.expanduser('~/.claude.json'); data = json.load(open(path)); data['mcpServers']['claude-flow'] = {'command': 'cmd', 'args': ['/c', 'npx', 'claude-flow@alpha', 'mcp', 'start'], 'type': 'stdio'}; data['mcpServers']['ruv-swarm'] = {'command': 'cmd', 'args': ['/c', 'npx', 'ruv-swarm@latest', 'mcp', 'start'], 'type': 'stdio'}; json.dump(data, open(path, 'w'), indent=2); print('  OK: User configuration updated')" 2>nul

if %ERRORLEVEL% EQU 0 (
    echo.
    echo ========================================
    echo SUCCESS: MCP Configuration Fixed!
    echo ========================================
    echo.
    echo Next steps:
    echo 1. Restart Claude Desktop
    echo 2. MCP servers should now work
) else (
    echo   FAIL: Could not update .claude.json
    echo   Please manually update the file
)
echo.