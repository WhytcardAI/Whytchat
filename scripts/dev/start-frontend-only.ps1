# Lancement Frontend uniquement (développement web)
# Plus rapide, pas de compilation Rust

Write-Host "🌐 Démarrage Frontend WhytChat (Web)..." -ForegroundColor Cyan

$frontendPath = Join-Path $PSScriptRoot "..\..\Frontend"

if (-not (Test-Path $frontendPath)) {
    Write-Host "❌ Dossier Frontend introuvable" -ForegroundColor Red
    exit 1
}

# Arrêter processus existants
$viteProcess = Get-NetTCPConnection -LocalPort 1420 -ErrorAction SilentlyContinue
if ($viteProcess) {
    Stop-Process -Id $viteProcess.OwningProcess -Force -ErrorAction SilentlyContinue
    Start-Sleep -Milliseconds 500
}

Set-Location $frontendPath
Write-Host "✅ Lancement sur http://localhost:1420" -ForegroundColor Green
Write-Host ""

npm run dev
