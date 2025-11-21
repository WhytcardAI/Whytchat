# Force Push Local State Script
# WARNING: This script will overwrite the remote history with your local state.

Write-Host "=====================================================================" -ForegroundColor Red
Write-Host "ATTENTION : Ceci va écraser l'historique distant GitHub avec l'état local actuel." -ForegroundColor Red
Write-Host "=====================================================================" -ForegroundColor Red
Write-Host ""
Write-Host "Ce script va :"
Write-Host "1. Ajouter tous les fichiers (git add .)"
Write-Host "2. Créer un commit de release (git commit)"
Write-Host "3. Vous donner la commande pour forcer le push."
Write-Host ""

$confirmation = Read-Host "Êtes-vous sûr de vouloir continuer ? (Tapez 'OUI' pour confirmer)"

if ($confirmation -ne "OUI") {
    Write-Host "Opération annulée." -ForegroundColor Yellow
    exit
}

Write-Host "Exécution de git add . ..." -ForegroundColor Cyan
git add .

Write-Host "Exécution de git commit ..." -ForegroundColor Cyan
git commit -m "Release v0.5.0: Major Update (Local State Override)"

Write-Host ""
Write-Host "=====================================================================" -ForegroundColor Green
Write-Host "Commit créé avec succès." -ForegroundColor Green
Write-Host "Pour écraser l'historique distant et pousser cette version, exécutez manuellement :" -ForegroundColor Yellow
Write-Host ""
Write-Host "git push origin main --force" -ForegroundColor White -BackgroundColor Black
Write-Host ""
Write-Host "(Remplacez 'main' par votre branche principale si différent, ex: 'master')"
Write-Host "=====================================================================" -ForegroundColor Green