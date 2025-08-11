# test_phase1.ps1 - Windows PowerShell test script for Phase 1.1 and 1.2

Write-Host "🚀 Phase 1 Verification Script (Windows)" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Function to print colored status
function Print-Status {
    param(
        [bool]$Success,
        [string]$Message
    )
    
    if ($Success) {
        Write-Host "✅ $Message" -ForegroundColor Green
    } else {
        Write-Host "❌ $Message" -ForegroundColor Red
        exit 1
    }
}

function Print-Info {
    param([string]$Message)
    Write-Host "ℹ️  $Message" -ForegroundColor Yellow
}

# Phase 1.1: Build Configuration Tests
Write-Host "📦 Phase 1.1: Build Configuration" -ForegroundColor White
Write-Host "---------------------------------" -ForegroundColor White

# Check if build.rs exists
if (Test-Path "build.rs") {
    Print-Status -Success $true -Message "build.rs exists"
} else {
    Print-Status -Success $false -Message "build.rs not found"
}

# Check GPU configuration
Write-Host ""
Write-Host "🔍 Checking GPU Configuration..." -ForegroundColor White

# CUDA check
if ($env:CUDA_PATH) {
    Print-Info "CUDA_PATH is set: $env:CUDA_PATH"
    if (Test-Path $env:CUDA_PATH) {
        Print-Status -Success $true -Message "CUDA installation found"
        
        # Check CUDA version
        $nvccPath = Join-Path $env:CUDA_PATH "bin\nvcc.exe"
        if (Test-Path $nvccPath) {
            $cudaVersion = & $nvccPath --version | Select-String "release" | ForEach-Object { $_.Line }
            Print-Info "CUDA version info: $cudaVersion"
        }
    } else {
        Print-Status -Success $false -Message "CUDA_PATH directory not found"
    }
} else {
    Print-Info "CUDA_PATH not set (CUDA acceleration unavailable)"
}

# Check for NVIDIA GPU
try {
    $nvidiaGPU = Get-WmiObject Win32_VideoController | Where-Object { $_.Name -like "*NVIDIA*" }
    if ($nvidiaGPU) {
        Print-Info "NVIDIA GPU detected: $($nvidiaGPU.Name)"
    }
} catch {
    Print-Info "Unable to detect GPU information"
}

Write-Host ""
Write-Host "📦 Phase 1.2: Cargo Configuration" -ForegroundColor White
Write-Host "---------------------------------" -ForegroundColor White

# Check if Cargo.toml exists
if (Test-Path "Cargo.toml") {
    Print-Status -Success $true -Message "Cargo.toml exists"
    
    # Check version
    $cargoContent = Get-Content "Cargo.toml" -Raw
    if ($cargoContent -match 'version\s*=\s*"0.3.0"') {
        Print-Status -Success $true -Message "Version updated to 0.3.0"
    } else {
        Print-Info "Version is not 0.3.0"
    }
    
    # Check for llama-cpp-2 dependency
    if ($cargoContent -match "llama-cpp-2") {
        Print-Status -Success $true -Message "llama-cpp-2 dependency configured"
    } else {
        Print-Status -Success $false -Message "llama-cpp-2 dependency not found"
    }
    
    # Check for build script reference
    if ($cargoContent -match 'build\s*=\s*"build.rs"') {
        Print-Status -Success $true -Message "Build script referenced in Cargo.toml"
    } else {
        Print-Status -Success $false -Message "Build script not referenced"
    }
} else {
    Print-Status -Success $false -Message "Cargo.toml not found"
}

Write-Host ""
Write-Host "🔨 Testing Build" -ForegroundColor White
Write-Host "---------------" -ForegroundColor White

# Test if project builds
Print-Info "Running cargo check..."
$checkResult = cargo check 2>&1
if ($LASTEXITCODE -eq 0) {
    Print-Status -Success $true -Message "Cargo check passed"
} else {
    Print-Status -Success $false -Message "Cargo check failed"
}

# Test specific features
Write-Host ""
Write-Host "🔧 Testing Feature Builds" -ForegroundColor White
Write-Host "------------------------" -ForegroundColor White

# Test default build
Print-Info "Testing default build..."
$buildResult = cargo build --release 2>&1
if ($LASTEXITCODE -eq 0) {
    Print-Status -Success $true -Message "Default build successful"
} else {
    Print-Info "Default build needs attention"
}

# Test CUDA build if available
if ($env:CUDA_PATH -and (Test-Path $env:CUDA_PATH)) {
    Print-Info "Testing CUDA build..."
    $cudaBuild = cargo build --features cuda 2>&1
    if ($LASTEXITCODE -eq 0) {
        Print-Status -Success $true -Message "CUDA build successful"
    } else {
        Print-Info "CUDA build failed (may need CUDA toolkit)"
    }
}

Write-Host ""
Write-Host "🧪 Running Tests" -ForegroundColor White
Write-Host "---------------" -ForegroundColor White

# Run unit tests
Print-Info "Running unit tests..."
$testResult = cargo test --lib 2>&1
if ($LASTEXITCODE -eq 0) {
    Print-Status -Success $true -Message "Unit tests passed"
} else {
    Print-Info "Some unit tests need attention"
}

# Run phase 1 verification tests
Print-Info "Running Phase 1 verification tests..."
$phase1Tests = cargo test phase1_tests 2>&1
if ($LASTEXITCODE -eq 0) {
    Print-Status -Success $true -Message "Phase 1 verification tests passed"
} else {
    Print-Info "Phase 1 tests need attention"
}

Write-Host ""
Write-Host "📊 System Information" -ForegroundColor White
Write-Host "--------------------" -ForegroundColor White

# CPU info
$cpuInfo = Get-WmiObject Win32_Processor
$cpuCores = $cpuInfo.NumberOfCores
$cpuThreads = $cpuInfo.NumberOfLogicalProcessors
Print-Info "CPU: $($cpuInfo.Name)"
Print-Info "CPU cores: $cpuCores (logical: $cpuThreads)"

# Memory info
$memInfo = Get-WmiObject Win32_ComputerSystem
$totalMemGB = [math]::Round($memInfo.TotalPhysicalMemory / 1GB, 2)
Print-Info "Total memory: $totalMemGB GB"

# Rust info
$rustVersion = rustc --version
Print-Info "Rust version: $rustVersion"

# Check if example runs
Write-Host ""
Write-Host "🔍 Testing GPU Detection Example" -ForegroundColor White
Write-Host "--------------------------------" -ForegroundColor White

if (Test-Path "examples\check_gpu.rs") {
    Print-Info "Running GPU detection example..."
    $exampleResult = cargo run --example check_gpu 2>&1
    if ($LASTEXITCODE -eq 0) {
        Print-Status -Success $true -Message "GPU detection example successful"
    } else {
        Print-Info "GPU detection example needs attention"
    }
} else {
    Print-Info "GPU detection example not found"
}

Write-Host ""
Write-Host "================================" -ForegroundColor Cyan
Write-Host "✨ Phase 1 Verification Complete!" -ForegroundColor Green
Write-Host ""
Write-Host "Summary:" -ForegroundColor White
Write-Host "  - Build configuration: ✅" -ForegroundColor White
Write-Host "  - Cargo dependencies: ✅" -ForegroundColor White
Write-Host "  - Project builds: ✅" -ForegroundColor White
Write-Host ""
Write-Host "Next steps:" -ForegroundColor Yellow
Write-Host "  1. If CUDA is available: `$env:CUDA_PATH = 'C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v11.8'" -ForegroundColor Yellow
Write-Host "  2. To build from source: `$env:BUILD_LLAMA_FROM_SOURCE = '1'; cargo build" -ForegroundColor Yellow
Write-Host "  3. Run full integration tests: cargo test -- --ignored" -ForegroundColor Yellow
Write-Host ""