# scripts/install.ps1
# PowerShell installation script for Windows

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Git Multi-Account Manager Installation" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Check Python version
Write-Host "Checking Python version..." -ForegroundColor Yellow
try {
    $pythonVersion = python --version 2>&1 | Select-String -Pattern "(\d+\.\d+\.\d+)"
    $version = $pythonVersion.Matches.Groups[1].Value
    Write-Host "✓ Python $version found" -ForegroundColor Green
} catch {
    Write-Host "✗ Python is not installed or not in PATH" -ForegroundColor Red
    exit 1
}

# Check Git
Write-Host "Checking Git..." -ForegroundColor Yellow
try {
    git --version | Out-Null
    Write-Host "✓ Git found" -ForegroundColor Green
} catch {
    Write-Host "✗ Git is not installed or not in PATH" -ForegroundColor Red
    exit 1
}

# Create virtual environment
Write-Host ""
Write-Host "Creating virtual environment..." -ForegroundColor Yellow
python -m venv venv
.\venv\Scripts\Activate.ps1
Write-Host "✓ Virtual environment created" -ForegroundColor Green

# Upgrade pip
Write-Host ""
Write-Host "Upgrading pip..." -ForegroundColor Yellow
python -m pip install --upgrade pip
Write-Host "✓ pip upgraded" -ForegroundColor Green

# Install package
Write-Host ""
Write-Host "Installing Git Manager..." -ForegroundColor Yellow
pip install -e .
Write-Host "✓ Git Manager installed" -ForegroundColor Green

# Create configuration directory
Write-Host ""
Write-Host "Setting up configuration..." -ForegroundColor Yellow
$configDir = Join-Path $env:USERPROFILE ".git-manager"
New-Item -ItemType Directory -Force -Path $configDir | Out-Null
Write-Host "✓ Configuration directory created" -ForegroundColor Green

# Copy example configurations
$exampleConfig = "config\config.example.json"
if (Test-Path $exampleConfig) {
    Copy-Item $exampleConfig (Join-Path $configDir "config.json")
    Write-Host "✓ Config file created" -ForegroundColor Green
}

$exampleAccounts = "config\accounts.example.json"
if (Test-Path $exampleAccounts) {
    Copy-Item $exampleAccounts (Join-Path $configDir "accounts.example.json")
    Write-Host "! Example accounts file created" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "========================================" -ForegroundColor Green
Write-Host "Installation complete!" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Green
Write-Host ""
Write-Host "To get started:"
Write-Host "  1. Activate virtual environment: .\venv\Scripts\Activate.ps1"
Write-Host "  2. Run CLI: git-manager"
Write-Host "  3. Run Web: git-manager-web"
Write-Host "  4. Run Desktop: git-manager-desktop"
Write-Host ""
Write-Host "Configuration files are in: $configDir"
Write-Host ""