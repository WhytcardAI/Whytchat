# Requires: PowerShell 5+, Node.js (for npx/node). Safe to run multiple times.
$ErrorActionPreference = 'Stop'

function New-EnsureDir($p) {
  if (-not (Test-Path -LiteralPath $p)) { New-Item -ItemType Directory -Path $p | Out-Null }
}

$Root = Split-Path -Parent $MyInvocation.MyCommand.Path | Split-Path -Parent | Split-Path -Parent
$Reports = Join-Path $Root 'reports'
New-EnsureDir $Reports

Write-Host "[analyze] Root: $Root"

# 1) Code clone detection via jscpd (multi-language)
$JscpdReportDir = Join-Path $Reports 'jscpd'
New-EnsureDir $JscpdReportDir

$hasNpx = $null -ne (Get-Command npx -ErrorAction SilentlyContinue)
if ($hasNpx) {
  Write-Host "[analyze] Running jscpd..."
  Push-Location $Root
  try {
    $args = @('--yes','jscpd','--silent','--output',"$JscpdReportDir",'--reporters','json,markdown','--pattern','**/*.{js,ts,tsx,jsx,html,css,rs}','--min-lines','5','--threshold','0')
    npx @args | Out-Null
  } catch {
    Write-Warning "jscpd failed: $($_.Exception.Message)"
  } finally { Pop-Location }
} else {
  Write-Warning "npx not found. Skipping jscpd clone detection. Install Node.js to enable."
}

# 2) Duplicate imports/functions (JS/TS/Rust) via Node script
$hasNode = $null -ne (Get-Command node -ErrorAction SilentlyContinue)
if ($hasNode) {
  Write-Host "[analyze] Scanning imports/functions..."
  node (Join-Path $Root 'scripts/analysis/scan-imports.mjs') $Root | Write-Host
} else {
  Write-Warning "node not found. Skipping import/function scan. Install Node.js to enable."
}

Write-Host "[analyze] Done. Reports in: $Reports"
