@echo off
echo ==========================================
echo Installing Claude Flow Globally with SQLite
echo ==========================================
echo.

echo Step 1: Installing claude-flow globally...
call npm install -g claude-flow@alpha --force

echo.
echo Step 2: Building SQLite bindings...
call node scripts\fix-global-sqlite.js

echo.
echo Step 3: Creating global command wrapper...
(
echo @echo off
echo node "%APPDATA%\npm\node_modules\claude-flow\cli.js" %%*
) > "%APPDATA%\npm\cf.cmd"

echo.
echo Step 4: Testing installation...
call cf --version

echo.
echo ==========================================
echo Installation Complete!
echo ==========================================
echo.
echo You can now use these commands from any directory:
echo   - cf init           (instead of npx claude-flow init)
echo   - cf memory store   (for memory operations)
echo   - cf mcp start      (for MCP server)
echo.
echo The SQLite bindings are permanently compiled and will work everywhere.
echo.