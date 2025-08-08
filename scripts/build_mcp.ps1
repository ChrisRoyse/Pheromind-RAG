# PowerShell build script for Windows
$ErrorActionPreference = "Stop"

Write-Host "🚀 Building Embed-Search MCP Server" -ForegroundColor Green

# Step 1: Build Rust core
Write-Host "Building Rust core..." -ForegroundColor Yellow
cargo build --release
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

# Step 2: Build native bridge
Write-Host "Building native bridge..." -ForegroundColor Yellow
if (Test-Path "mcp-bridge") {
    Push-Location mcp-bridge
    cargo build --release
    if ($LASTEXITCODE -ne 0) { 
        Pop-Location
        exit $LASTEXITCODE 
    }
    Pop-Location
} else {
    Write-Host "⚠️ mcp-bridge directory not found, skipping..." -ForegroundColor Yellow
}

# Step 3: Setup Node project
Write-Host "Setting up Node.js project..." -ForegroundColor Yellow
if (Test-Path "mcp-server") {
    Push-Location mcp-server
    npm install
    if ($LASTEXITCODE -ne 0) { 
        Pop-Location
        exit $LASTEXITCODE 
    }
    
    # Step 4: Build TypeScript
    Write-Host "Building TypeScript..." -ForegroundColor Yellow
    npm run build
    if ($LASTEXITCODE -ne 0) { 
        Pop-Location
        exit $LASTEXITCODE 
    }
    
    Pop-Location
} else {
    Write-Host "⚠️ mcp-server directory not found, skipping..." -ForegroundColor Yellow
}

# Step 5: Run tests
Write-Host "Running tests..." -ForegroundColor Yellow
cargo test watcher
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "✅ Build complete!" -ForegroundColor Green
Write-Host "To register with Claude: claude mcp add embed-search ./mcp-server/dist/index.js" -ForegroundColor Cyan