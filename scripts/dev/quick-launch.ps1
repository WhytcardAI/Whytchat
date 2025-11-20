# Quick launch script for WhytChat (skips compilation)
# Usage: powershell -ExecutionPolicy Bypass -File scripts/quick-launch.ps1

$ErrorActionPreference = 'Stop'
$Root = Split-Path -Parent $MyInvocation.MyCommand.Path | Split-Path -Parent | Split-Path -Parent
$BackendDir = Join-Path $Root 'Backend/src-tauri'
$ExePath = Join-Path $BackendDir 'target/debug/whytchat-backend.exe'

if (-not (Test-Path $ExePath)) {
    Write-Warning "Debug binary not found at: $ExePath"
    Write-Warning "Please run 'scripts/dev.ps1' or 'cargo build' at least once."
    exit 1
}

Write-Host "[quick-launch] Launching existing binary..."
Write-Host "  -> $ExePath"

# Set working directory to src-tauri so relative paths (if any) might work better
Push-Location $BackendDir
try {
    & $ExePath
} finally {
    Pop-Location
}
