param(
  [string]$FrontendPath = "$PSScriptRoot/../../Frontend"
)
Write-Host "[i18n] Using Frontend path: $FrontendPath"
if (-not (Test-Path $FrontendPath)) { Write-Error "[i18n] Frontend path not found"; exit 1 }
$env:TAILWIND_DISABLE_OXIDE = '1'
Push-Location $FrontendPath
try {
  npm install react-i18next i18next i18next-browser-languagedetector --save --no-fund --no-audit
} finally {
  Pop-Location
}
