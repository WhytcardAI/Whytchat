param(
  [string]$FrontendPath = "$PSScriptRoot/../../Frontend"
)
Write-Host "[fix] Frontend path: $FrontendPath"
if (-not (Test-Path $FrontendPath)) { Write-Error "[fix] Frontend introuvable: $FrontendPath"; exit 1 }

# Stop ALL node processes to release locks
Get-Process node -ErrorAction SilentlyContinue | ForEach-Object { try { $_.Kill() } catch {} }
Start-Sleep -Milliseconds 500

# Remove node_modules entirely to avoid EPERM on oxide
$nm = Join-Path $FrontendPath "node_modules"
if (Test-Path $nm) {
  try { Remove-Item -Recurse -Force $nm -ErrorAction SilentlyContinue } catch {}
}

# Install deps
Push-Location $FrontendPath
try {
  if (Test-Path (Join-Path $FrontendPath 'package-lock.json')) { npm ci --no-fund --no-audit } else { npm install --no-fund --no-audit }
} finally {
  Pop-Location
}
