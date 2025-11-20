# Launch Frontend (if Node project) and Backend (Tauri) together
# Usage: run from any dir
#   powershell -ExecutionPolicy Bypass -File scripts/dev.ps1

$ErrorActionPreference = 'Stop'

function Test-Command($name) {
  return $null -ne (Get-Command $name -ErrorAction SilentlyContinue)
}

function Wait-Port($hostname, $port, $timeoutSec) {
  $deadline = [DateTime]::UtcNow.AddSeconds($timeoutSec)
  while ([DateTime]::UtcNow -lt $deadline) {
    try {
      $client = New-Object System.Net.Sockets.TcpClient
      $iar = $client.BeginConnect($hostname, $port, $null, $null)
      if ($iar.AsyncWaitHandle.WaitOne(250)) {
        $client.EndConnect($iar)
        $client.Close(); return $true
      }
      $client.Close()
    } catch {}
  }
  return $false
}

$Root = Split-Path -Parent $MyInvocation.MyCommand.Path | Split-Path -Parent | Split-Path -Parent
$Frontend = Join-Path $Root 'Frontend'
$Backend = Join-Path $Root 'Backend/src-tauri'

$frontendProc = $null
$hasPkg = Test-Path (Join-Path $Frontend 'package.json')

if ($hasPkg -and (Test-Command npm)) {
  Write-Host "[dev] Starting Frontend (npm run dev) in: $Frontend"

  # Resolve npm executable explicitly (npm.cmd on Windows)
  $cmd = Get-Command 'npm.cmd' -ErrorAction SilentlyContinue
  $npmCmd = $null
  if ($cmd) { $npmCmd = $cmd.Source }
  if (-not $npmCmd) {
    $cmd = Get-Command 'npm' -ErrorAction SilentlyContinue
    if ($cmd) { $npmCmd = $cmd.Source }
  }
  if (-not $npmCmd) { throw "npm introuvable dans le PATH." }

  # Ensure vite is available (install if missing)
  $viteCmd = Join-Path $Frontend 'node_modules/.bin/vite.cmd'
  if (-not (Test-Path $viteCmd)) {
    Write-Host "[dev] Dépendances manquantes: installation en cours..." -ForegroundColor Yellow
    $env:TAILWIND_DISABLE_OXIDE = '1'
    Push-Location $Frontend
    try {
      if (Test-Path (Join-Path $Frontend 'package-lock.json')) {
        & $npmCmd ci --no-fund --no-audit
      } else {
        & $npmCmd install --no-fund --no-audit
      }
    } finally { Pop-Location }
  }

  $frontendProc = Start-Process -FilePath $npmCmd -ArgumentList @('run','dev') -WorkingDirectory $Frontend -PassThru
  # Wait vite on configured port (1420)
  if (Wait-Port '127.0.0.1' 1420 25) {
    Write-Host "[dev] Frontend dev server détecté sur http://127.0.0.1:1420"
  } else {
    Write-Warning "[dev] Frontend non détecté sur 1420 (poursuite quand même)."
  }
} else {
  Write-Host "[dev] No Frontend dev detected (package.json or npm missing). Static HTML will be served by Tauri."
}

if (-not (Test-Path $Backend)) { throw "Backend path not found: $Backend" }

Write-Host "[dev] Starting Backend in: $Backend"
if (-not (Test-Command cargo)) { throw 'Rust cargo not found in PATH.' }

Push-Location $Backend
try {
  try {
    Write-Host "[dev] Trying: cargo tauri dev"
    cargo tauri dev
  } catch {
    Write-Warning "[dev] 'cargo tauri' not available. Falling back to 'cargo run'."
    Write-Host "[dev] Tip: install CLI with 'cargo install tauri-cli --locked'"
    cargo run
  }
} finally {
  Pop-Location
  if ($frontendProc -ne $null -and -not $frontendProc.HasExited) {
    Write-Host "[dev] Stopping Frontend dev server (PID=$($frontendProc.Id))"
    try { $frontendProc.CloseMainWindow() | Out-Null } catch {}
    Start-Sleep -Seconds 1
    if (-not $frontendProc.HasExited) { try { $frontendProc.Kill() } catch {} }
  }
}
