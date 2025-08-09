# Fix MCP Server Configuration for Windows
# Updates .claude.json to use cmd /c wrapper for npx commands

$claudeJsonPath = "$env:USERPROFILE\.claude.json"

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Fixing MCP Server Configuration" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Check if file exists
if (-not (Test-Path $claudeJsonPath)) {
    Write-Host "ERROR: .claude.json not found at $claudeJsonPath" -ForegroundColor Red
    exit 1
}

Write-Host "[1/4] Reading .claude.json..." -ForegroundColor Yellow
$config = Get-Content $claudeJsonPath -Raw | ConvertFrom-Json

Write-Host "[2/4] Fixing claude-flow MCP server..." -ForegroundColor Yellow
if ($config.mcpServers."claude-flow") {
    $config.mcpServers."claude-flow".command = "cmd"
    $config.mcpServers."claude-flow".args = @("/c", "npx", "claude-flow@alpha", "mcp", "start")
    Write-Host "  ✓ Updated claude-flow to use cmd /c wrapper" -ForegroundColor Green
} else {
    Write-Host "  ⚠ claude-flow not found in configuration" -ForegroundColor Yellow
}

Write-Host "[3/4] Fixing ruv-swarm MCP server..." -ForegroundColor Yellow
if ($config.mcpServers."ruv-swarm") {
    $config.mcpServers."ruv-swarm".command = "cmd"
    $config.mcpServers."ruv-swarm".args = @("/c", "npx", "ruv-swarm@latest", "mcp", "start")
    Write-Host "  ✓ Updated ruv-swarm to use cmd /c wrapper" -ForegroundColor Green
} else {
    Write-Host "  ⚠ ruv-swarm not found in configuration" -ForegroundColor Yellow
}

Write-Host "[4/4] Saving updated configuration..." -ForegroundColor Yellow
$config | ConvertTo-Json -Depth 100 | Set-Content $claudeJsonPath -Encoding UTF8
Write-Host "  ✓ Configuration saved" -ForegroundColor Green

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "✅ MCP Configuration Fixed!" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "Next steps:" -ForegroundColor Yellow
Write-Host "1. Restart Claude Desktop" -ForegroundColor White
Write-Host "2. MCP servers should now work correctly" -ForegroundColor White
Write-Host ""