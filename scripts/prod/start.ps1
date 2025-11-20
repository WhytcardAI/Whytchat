# Script de lancement Frontend + Backend pour WhytChat
# Auteur: WhytCard AI
# Date: 19/11/2025

Write-Host "🚀 Démarrage WhytChat..." -ForegroundColor Cyan
Write-Host ""

# Chemins
$frontendPath = Join-Path $PSScriptRoot "..\..\Frontend"
$backendPath = Join-Path $PSScriptRoot "..\..\Backend\src-tauri"

# Vérifier que les dossiers existent
if (-not (Test-Path $frontendPath)) {
    Write-Host "❌ Dossier Frontend introuvable: $frontendPath" -ForegroundColor Red
    exit 1
}

if (-not (Test-Path $backendPath)) {
    Write-Host "❌ Dossier Backend introuvable: $backendPath" -ForegroundColor Red
    exit 1
}

# Fonction pour tuer les processus existants
function Stop-ExistingProcesses {
    Write-Host "🧹 Nettoyage des processus existants..." -ForegroundColor Yellow

    # Arrêter Vite sur port 1420
    $viteProcess = Get-NetTCPConnection -LocalPort 1420 -ErrorAction SilentlyContinue
    if ($viteProcess) {
        $vitePid = $viteProcess.OwningProcess
        Write-Host "  ⏹️  Arrêt Vite (PID: $vitePid)" -ForegroundColor Yellow
        Stop-Process -Id $vitePid -Force -ErrorAction SilentlyContinue
        Start-Sleep -Milliseconds 500
    }

    # Arrêter cargo/tauri
    Get-Process | Where-Object {
        $_.ProcessName -like '*cargo*' -or
        $_.ProcessName -like '*tauri*' -or
        $_.ProcessName -like '*whytchat*'
    } | ForEach-Object {
        Write-Host "  ⏹️  Arrêt $($_.ProcessName) (PID: $($_.Id))" -ForegroundColor Yellow
        Stop-Process -Id $_.Id -Force -ErrorAction SilentlyContinue
    }

    Start-Sleep -Seconds 1
}

# Nettoyage initial
Stop-ExistingProcesses

Write-Host ""
Write-Host "📦 Étape 1/2: Démarrage Frontend (Vite)..." -ForegroundColor Green

# Démarrer le frontend en arrière-plan
$frontendJob = Start-Job -ScriptBlock {
    param($path)
    Set-Location $path
    npm run dev
} -ArgumentList $frontendPath

Write-Host "  ✅ Frontend démarré (Job ID: $($frontendJob.Id))" -ForegroundColor Green

# Attendre que Vite soit prêt
Write-Host "  ⏳ Attente du serveur Vite..." -ForegroundColor Cyan
$timeout = 30
$elapsed = 0
$viteReady = $false

while ($elapsed -lt $timeout -and -not $viteReady) {
    try {
        $response = Invoke-WebRequest -Uri "http://localhost:1420" -TimeoutSec 1 -ErrorAction SilentlyContinue
        $viteReady = $true
        Write-Host "  ✅ Vite prêt sur http://localhost:1420" -ForegroundColor Green
    } catch {
        Start-Sleep -Seconds 1
        $elapsed++
        Write-Host "." -NoNewline
    }
}

if (-not $viteReady) {
    Write-Host ""
    Write-Host "  ⚠️  Timeout Vite (continuer quand même)" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "🦀 Étape 2/2: Démarrage Backend (Tauri)..." -ForegroundColor Green

# Démarrer le backend Tauri
Set-Location $backendPath

Write-Host "  📦 Compilation Rust (peut prendre 2-5 min)..." -ForegroundColor Cyan
Write-Host ""

try {
    cargo tauri dev
} catch {
    Write-Host ""
    Write-Host "❌ Erreur lors du démarrage Tauri" -ForegroundColor Red
    Write-Host $_.Exception.Message -ForegroundColor Red
}

# Nettoyage à la fermeture
Write-Host ""
Write-Host "🛑 Arrêt de WhytChat..." -ForegroundColor Yellow

Stop-Job $frontendJob -ErrorAction SilentlyContinue
Remove-Job $frontendJob -ErrorAction SilentlyContinue

Stop-ExistingProcesses

Write-Host "✅ WhytChat arrêté proprement" -ForegroundColor Green
