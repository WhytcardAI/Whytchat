# WhytChat Codebase Analyzer
# Analyse complète de tous les fichiers, imports et dépendances

$ErrorActionPreference = "Continue"
$ProjectRoot = Split-Path -Parent $PSScriptRoot

Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host "  WhytChat Codebase Analyzer v1.0" -ForegroundColor Cyan
Write-Host "========================================`n" -ForegroundColor Cyan

# ============================================================================
# CONFIGURATION
# ============================================================================
$RustSrcDir = Join-Path $ProjectRoot "apps\core\src"
$FrontendSrcDir = Join-Path $ProjectRoot "apps\desktop-ui\src"
$OutputDir = Join-Path $ProjectRoot "documentation\analysis"
$Timestamp = Get-Date -Format "yyyy-MM-dd_HH-mm-ss"

# Créer le dossier de sortie
if (-not (Test-Path $OutputDir)) {
    New-Item -ItemType Directory -Path $OutputDir -Force | Out-Null
}

# ============================================================================
# ANALYSE RUST BACKEND
# ============================================================================
Write-Host "🦀 ANALYSE BACKEND RUST" -ForegroundColor Yellow
Write-Host "========================`n" -ForegroundColor Yellow

$RustReport = @{
    timestamp = $Timestamp
    backend = @{
        files = @()
        modules = @{}
        dependencies = @{
            internal = @()
            external = @()
        }
        errors = @()
        warnings = @()
        stats = @{
            total_files = 0
            total_lines = 0
            total_functions = 0
            total_structs = 0
            total_enums = 0
            total_impls = 0
            total_tests = 0
        }
    }
}

# Analyser chaque fichier Rust
$RustFiles = Get-ChildItem -Path $RustSrcDir -Filter "*.rs" -Recurse -ErrorAction SilentlyContinue

foreach ($file in $RustFiles) {
    $relativePath = $file.FullName.Replace($ProjectRoot + "\", "")
    Write-Host "  📄 $relativePath" -ForegroundColor Gray
    
    $content = Get-Content $file.FullName -Raw -ErrorAction SilentlyContinue
    if (-not $content) { continue }
    
    $lines = ($content -split "`n").Count
    $RustReport.backend.stats.total_lines += $lines
    
    # Extraire les use statements (imports)
    $useStatements = [regex]::Matches($content, 'use\s+([^;]+);')
    $imports = @()
    foreach ($match in $useStatements) {
        $import = $match.Groups[1].Value.Trim()
        $imports += $import
        
        # Classifier: interne vs externe
        if ($import -match "^crate::" -or $import -match "^super::" -or $import -match "^self::") {
            if ($RustReport.backend.dependencies.internal -notcontains $import) {
                $RustReport.backend.dependencies.internal += $import
            }
        } else {
            $crateName = ($import -split "::")[0]
            if ($RustReport.backend.dependencies.external -notcontains $crateName) {
                $RustReport.backend.dependencies.external += $crateName
            }
        }
    }
    
    # Extraire les mod declarations
    $modDeclarations = [regex]::Matches($content, 'mod\s+(\w+)\s*[;{]')
    $mods = @()
    foreach ($match in $modDeclarations) {
        $mods += $match.Groups[1].Value
    }
    
    # Compter les éléments
    $functions = ([regex]::Matches($content, '(?:pub\s+)?(?:async\s+)?fn\s+\w+')).Count
    $structs = ([regex]::Matches($content, '(?:pub\s+)?struct\s+\w+')).Count
    $enums = ([regex]::Matches($content, '(?:pub\s+)?enum\s+\w+')).Count
    $impls = ([regex]::Matches($content, 'impl(?:<[^>]+>)?\s+(?:\w+::)*\w+')).Count
    $tests = ([regex]::Matches($content, '#\[(?:tokio::)?test\]')).Count
    
    $RustReport.backend.stats.total_functions += $functions
    $RustReport.backend.stats.total_structs += $structs
    $RustReport.backend.stats.total_enums += $enums
    $RustReport.backend.stats.total_impls += $impls
    $RustReport.backend.stats.total_tests += $tests
    
    # Détecter les problèmes potentiels
    $unwraps = ([regex]::Matches($content, '\.unwrap\(\)')).Count
    $expects = ([regex]::Matches($content, '\.expect\(')).Count
    $panics = ([regex]::Matches($content, 'panic!\(')).Count
    $unsafeBlocks = ([regex]::Matches($content, 'unsafe\s*\{')).Count
    $todos = ([regex]::Matches($content, '(?i)//\s*TODO')).Count
    $fixmes = ([regex]::Matches($content, '(?i)//\s*FIXME')).Count
    
    # Ajouter au rapport
    $fileInfo = @{
        path = $relativePath
        lines = $lines
        imports = $imports
        modules = $mods
        functions = $functions
        structs = $structs
        enums = $enums
        impls = $impls
        tests = $tests
        potential_issues = @{
            unwraps = $unwraps
            expects = $expects
            panics = $panics
            unsafe_blocks = $unsafeBlocks
            todos = $todos
            fixmes = $fixmes
        }
    }
    
    $RustReport.backend.files += $fileInfo
    
    # Ajouter warnings si problèmes détectés
    if ($unwraps -gt 0 -and $file.Name -notmatch "test") {
        $RustReport.backend.warnings += "[$relativePath] $unwraps unwrap() calls (consider using ? operator)"
    }
    if ($unsafeBlocks -gt 0) {
        $RustReport.backend.warnings += "[$relativePath] $unsafeBlocks unsafe block(s) - ensure SAFETY comments exist"
    }
}

$RustReport.backend.stats.total_files = $RustFiles.Count

Write-Host "`n  ✅ Analysé $($RustFiles.Count) fichiers Rust" -ForegroundColor Green
Write-Host "     📊 $($RustReport.backend.stats.total_lines) lignes de code" -ForegroundColor Cyan
Write-Host "     🔧 $($RustReport.backend.stats.total_functions) fonctions" -ForegroundColor Cyan
Write-Host "     📦 $($RustReport.backend.stats.total_structs) structs, $($RustReport.backend.stats.total_enums) enums" -ForegroundColor Cyan
Write-Host "     🧪 $($RustReport.backend.stats.total_tests) tests" -ForegroundColor Cyan

# ============================================================================
# ANALYSE FRONTEND REACT
# ============================================================================
Write-Host "`n⚛️  ANALYSE FRONTEND REACT" -ForegroundColor Yellow
Write-Host "==========================`n" -ForegroundColor Yellow

$FrontendReport = @{
    files = @()
    dependencies = @{
        internal = @()
        external = @()
        tauri = @()
    }
    components = @()
    hooks = @()
    stores = @()
    stats = @{
        total_files = 0
        total_lines = 0
        total_components = 0
        total_hooks = 0
    }
    warnings = @()
}

$FrontendFiles = Get-ChildItem -Path $FrontendSrcDir -Include "*.jsx","*.js","*.ts","*.tsx" -Recurse -ErrorAction SilentlyContinue

foreach ($file in $FrontendFiles) {
    $relativePath = $file.FullName.Replace($ProjectRoot + "\", "")
    Write-Host "  📄 $relativePath" -ForegroundColor Gray
    
    $content = Get-Content $file.FullName -Raw -ErrorAction SilentlyContinue
    if (-not $content) { continue }
    
    $lines = ($content -split "`n").Count
    $FrontendReport.stats.total_lines += $lines
    
    # Extraire les imports
    $importMatches = [regex]::Matches($content, "import\s+(?:\{[^}]+\}|\*\s+as\s+\w+|\w+)\s+from\s+['""]([^'""]+)['""]")
    $imports = @()
    
    foreach ($match in $importMatches) {
        $source = $match.Groups[1].Value
        $imports += $source
        
        if ($source -match "^@tauri-apps") {
            if ($FrontendReport.dependencies.tauri -notcontains $source) {
                $FrontendReport.dependencies.tauri += $source
            }
        } elseif ($source -match "^[./]") {
            # Import local
            if ($FrontendReport.dependencies.internal -notcontains $source) {
                $FrontendReport.dependencies.internal += $source
            }
        } else {
            # Package npm
            $pkgName = ($source -split "/")[0]
            if ($FrontendReport.dependencies.external -notcontains $pkgName) {
                $FrontendReport.dependencies.external += $pkgName
            }
        }
    }
    
    # Détecter les composants React
    $componentMatches = [regex]::Matches($content, '(?:export\s+)?(?:function|const)\s+([A-Z]\w+)')
    $components = @()
    foreach ($match in $componentMatches) {
        $compName = $match.Groups[1].Value
        if ($compName -notmatch "^[A-Z][A-Z_]+$") { # Exclure les constantes
            $components += $compName
            if ($FrontendReport.components -notcontains $compName) {
                $FrontendReport.components += $compName
            }
        }
    }
    
    # Détecter les hooks personnalisés
    $hookMatches = [regex]::Matches($content, '(?:export\s+)?(?:function|const)\s+(use[A-Z]\w+)')
    $hooks = @()
    foreach ($match in $hookMatches) {
        $hookName = $match.Groups[1].Value
        $hooks += $hookName
        if ($FrontendReport.hooks -notcontains $hookName) {
            $FrontendReport.hooks += $hookName
        }
    }
    
    # Détecter les appels Tauri invoke
    $invokeMatches = [regex]::Matches($content, 'invoke\s*\(\s*[''"](\w+)[''"]')
    $tauriCommands = @()
    foreach ($match in $invokeMatches) {
        $tauriCommands += $match.Groups[1].Value
    }
    
    # Détecter les stores Zustand
    $storeMatches = [regex]::Matches($content, 'create\s*\(\s*\(')
    $isStore = $storeMatches.Count -gt 0
    if ($isStore -and $file.Name -match "Store") {
        $FrontendReport.stores += $file.Name
    }
    
    # Ajouter au rapport
    $fileInfo = @{
        path = $relativePath
        lines = $lines
        imports = $imports
        components = $components
        hooks = $hooks
        tauri_commands = $tauriCommands
        is_store = $isStore
    }
    
    $FrontendReport.files += $fileInfo
    $FrontendReport.stats.total_components += $components.Count
    $FrontendReport.stats.total_hooks += $hooks.Count
}

$FrontendReport.stats.total_files = $FrontendFiles.Count

Write-Host "`n  ✅ Analysé $($FrontendFiles.Count) fichiers Frontend" -ForegroundColor Green
Write-Host "     📊 $($FrontendReport.stats.total_lines) lignes de code" -ForegroundColor Cyan
Write-Host "     🧩 $($FrontendReport.components.Count) composants React" -ForegroundColor Cyan
Write-Host "     🪝 $($FrontendReport.hooks.Count) hooks personnalisés" -ForegroundColor Cyan
Write-Host "     🏪 $($FrontendReport.stores.Count) stores Zustand" -ForegroundColor Cyan

# ============================================================================
# VÉRIFICATION DES IMPORTS TAURI (FRONTEND <-> BACKEND)
# ============================================================================
Write-Host "`n🔗 VÉRIFICATION COHÉRENCE TAURI" -ForegroundColor Yellow
Write-Host "================================`n" -ForegroundColor Yellow

# Extraire toutes les commandes Tauri du backend
$BackendCommands = @()
foreach ($file in $RustReport.backend.files) {
    $filePath = Join-Path $ProjectRoot $file.path
    $content = Get-Content $filePath -Raw -ErrorAction SilentlyContinue
    if ($content) {
        $cmdMatches = [regex]::Matches($content, '#\[tauri::command\][^f]*fn\s+(\w+)')
        foreach ($match in $cmdMatches) {
            $BackendCommands += $match.Groups[1].Value
        }
    }
}
$BackendCommands = $BackendCommands | Sort-Object -Unique

# Extraire toutes les commandes appelées par le frontend
$FrontendCommands = @()
foreach ($file in $FrontendReport.files) {
    $FrontendCommands += $file.tauri_commands
}
$FrontendCommands = $FrontendCommands | Sort-Object -Unique

Write-Host "  Backend Tauri Commands: $($BackendCommands.Count)" -ForegroundColor Cyan
Write-Host "  Frontend invoke() calls: $($FrontendCommands.Count)" -ForegroundColor Cyan

# Vérifier la cohérence
$MissingInBackend = $FrontendCommands | Where-Object { $BackendCommands -notcontains $_ }
$UnusedInFrontend = $BackendCommands | Where-Object { $FrontendCommands -notcontains $_ }

$TauriCoherence = @{
    backend_commands = $BackendCommands
    frontend_commands = $FrontendCommands
    missing_in_backend = $MissingInBackend
    unused_by_frontend = $UnusedInFrontend
    is_coherent = ($MissingInBackend.Count -eq 0)
}

if ($MissingInBackend.Count -gt 0) {
    Write-Host "`n  ❌ Commands called by frontend but NOT in backend:" -ForegroundColor Red
    foreach ($cmd in $MissingInBackend) {
        Write-Host "     - $cmd" -ForegroundColor Red
    }
} else {
    Write-Host "`n  ✅ All frontend commands exist in backend" -ForegroundColor Green
}

if ($UnusedInFrontend.Count -gt 0) {
    Write-Host "`n  ⚠️  Backend commands NOT used by frontend:" -ForegroundColor Yellow
    foreach ($cmd in $UnusedInFrontend) {
        Write-Host "     - $cmd" -ForegroundColor Yellow
    }
}

# ============================================================================
# GÉNÉRATION DU RAPPORT JSON
# ============================================================================
$FullReport = @{
    metadata = @{
        timestamp = $Timestamp
        project = "WhytChat V1"
        analyzer_version = "1.0"
    }
    backend = $RustReport.backend
    frontend = $FrontendReport
    tauri_coherence = $TauriCoherence
}

$JsonPath = Join-Path $OutputDir "codebase-analysis.json"
$FullReport | ConvertTo-Json -Depth 10 | Out-File -FilePath $JsonPath -Encoding UTF8
Write-Host "`n📁 Rapport JSON: $JsonPath" -ForegroundColor Green

# ============================================================================
# GÉNÉRATION DE LA CARTE MERMAID
# ============================================================================
Write-Host "`n🗺️  GÉNÉRATION CARTE DE DÉPENDANCES" -ForegroundColor Yellow
Write-Host "====================================`n" -ForegroundColor Yellow

$MermaidContent = @"
# Carte des Dépendances - WhytChat V1

> Généré automatiquement le $(Get-Date -Format "yyyy-MM-dd HH:mm:ss")

## 🏗️ Architecture Globale

\`\`\`mermaid
graph TB
    subgraph Frontend["⚛️ Frontend React"]
        App[App.jsx]
        subgraph Components["📦 Components"]
            Chat[ChatInterface]
            Dashboard[Dashboard]
            Onboarding[OnboardingWizard]
            Preflight[PreflightCheck]
            Diagnostics[TestConsole]
        end
        subgraph Stores["🏪 Stores"]
            AppStore[appStore]
            ChatStore[chatStore]
        end
        subgraph Layout["🎨 Layout"]
            MainLayout[MainLayout]
            Sidebar[Sidebar]
            TitleBar[TitleBar]
        end
    end

    subgraph Backend["🦀 Backend Rust"]
        Main[main.rs]
        subgraph Actors["🎭 Actors"]
            Supervisor[SupervisorHandle]
            LlmActor[LlmActorHandle]
            RagActor[RagActorHandle]
        end
        subgraph Core["⚙️ Core Modules"]
            Database[database.rs]
            Encryption[encryption.rs]
            FsManager[fs_manager.rs]
            Preflight_rs[preflight.rs]
        end
        subgraph Brain["🧠 Brain"]
            ContextPacket[context_packet.rs]
        end
    end

    subgraph Data["💾 Data Layer"]
        SQLite[(SQLite DB)]
        LanceDB[(LanceDB Vectors)]
        Models[("GGUF Models")]
    end

    %% Frontend -> Backend via Tauri
    App --> |"invoke()"| Main
    Chat --> |"send_message"| Supervisor
    Dashboard --> |"get_sessions"| Database
    Onboarding --> |"download_model"| Main

    %% Backend internal
    Main --> Supervisor
    Supervisor --> LlmActor
    Supervisor --> RagActor
    LlmActor --> Models
    RagActor --> LanceDB
    Database --> SQLite
    Main --> Encryption
    Main --> FsManager
\`\`\`

## 🦀 Modules Backend

\`\`\`mermaid
graph LR
    subgraph MainModule["main.rs"]
        Commands[Tauri Commands]
        State[AppState]
        Init[initialize_app]
    end

    subgraph ActorSystem["Actor System"]
        supervisor[actors/supervisor.rs]
        llm[actors/llm.rs]
        rag[actors/rag.rs]
        messages[actors/messages.rs]
    end

    subgraph CoreModules["Core"]
        database[database.rs]
        error[error.rs]
        encryption[encryption.rs]
        fs_manager[fs_manager.rs]
        models[models.rs]
        preflight[preflight.rs]
        diagnostics[diagnostics.rs]
    end

    subgraph BrainModule["Brain"]
        context[brain/context_packet.rs]
    end

    Commands --> supervisor
    supervisor --> llm
    supervisor --> rag
    llm --> messages
    rag --> messages
    Commands --> database
    Commands --> preflight
    Init --> encryption
    Init --> fs_manager
\`\`\`

## ⚛️ Composants Frontend

\`\`\`mermaid
graph TD
    subgraph AppRoot["App.jsx"]
        App
    end

    subgraph Pages["Pages"]
        Chat[ChatInterface]
        Dash[Dashboard]
        Onboard[OnboardingWizard]
        Pre[PreflightCheck]
        Test[TestConsole]
    end

    subgraph ChatComponents["Chat Components"]
        MessageList[MessageList]
        MessageBubble[MessageBubble]
        ChatInput[ChatInput]
        PromptBar[PromptBar]
    end

    subgraph LayoutComponents["Layout"]
        MainLayout
        Sidebar
        TitleBar
        SessionList
        FilesDropdown
    end

    subgraph SharedComponents["Shared"]
        ErrorBoundary
        LoadingSpinner
        Modal
    end

    App --> MainLayout
    MainLayout --> Sidebar
    MainLayout --> TitleBar
    MainLayout --> Pages
    Chat --> ChatComponents
    Sidebar --> SessionList
    Sidebar --> FilesDropdown
    App --> ErrorBoundary
\`\`\`

## 📊 Statistiques

| Métrique | Backend | Frontend |
|----------|---------|----------|
| **Fichiers** | $($RustReport.backend.stats.total_files) | $($FrontendReport.stats.total_files) |
| **Lignes de code** | $($RustReport.backend.stats.total_lines) | $($FrontendReport.stats.total_lines) |
| **Fonctions/Composants** | $($RustReport.backend.stats.total_functions) | $($FrontendReport.stats.total_components) |
| **Tests** | $($RustReport.backend.stats.total_tests) | - |
| **Structs/Hooks** | $($RustReport.backend.stats.total_structs) | $($FrontendReport.stats.total_hooks) |

## 🔗 Commandes Tauri

### Backend → Frontend

| Commande | Utilisée Frontend |
|----------|-------------------|
"@

foreach ($cmd in $BackendCommands) {
    $used = if ($FrontendCommands -contains $cmd) { "✅" } else { "⚠️ Non utilisée" }
    $MermaidContent += "| ``$cmd`` | $used |`n"
}

$MermaidContent += @"

## 📦 Dépendances Externes

### Backend (Cargo.toml)
"@

foreach ($dep in ($RustReport.backend.dependencies.external | Sort-Object -Unique | Select-Object -First 20)) {
    $MermaidContent += "- ``$dep```n"
}

$MermaidContent += @"

### Frontend (package.json)
"@

foreach ($dep in ($FrontendReport.dependencies.external | Sort-Object -Unique | Select-Object -First 15)) {
    $MermaidContent += "- ``$dep```n"
}

$MermaidContent += @"

## ⚠️ Avertissements Détectés

### Backend
"@

foreach ($warning in $RustReport.backend.warnings) {
    $MermaidContent += "- $warning`n"
}

if ($RustReport.backend.warnings.Count -eq 0) {
    $MermaidContent += "- ✅ Aucun avertissement`n"
}

$MermaidContent += @"

### Cohérence Tauri
"@

if ($MissingInBackend.Count -gt 0) {
    $MermaidContent += "- ❌ Commandes manquantes dans le backend: $($MissingInBackend -join ', ')`n"
} else {
    $MermaidContent += "- ✅ Toutes les commandes frontend existent dans le backend`n"
}

# Écrire le fichier Mermaid
$MermaidPath = Join-Path $OutputDir "DEPENDENCY_MAP.md"
$MermaidContent | Out-File -FilePath $MermaidPath -Encoding UTF8
Write-Host "📁 Carte Mermaid: $MermaidPath" -ForegroundColor Green

# ============================================================================
# RÉSUMÉ FINAL
# ============================================================================
Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host "  📊 RÉSUMÉ DE L'ANALYSE" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

Write-Host "`n  🦀 Backend Rust:" -ForegroundColor Yellow
Write-Host "     - $($RustReport.backend.stats.total_files) fichiers" 
Write-Host "     - $($RustReport.backend.stats.total_lines) lignes"
Write-Host "     - $($RustReport.backend.stats.total_functions) fonctions"
Write-Host "     - $($RustReport.backend.stats.total_tests) tests"
Write-Host "     - $($RustReport.backend.dependencies.external.Count) dépendances externes"

Write-Host "`n  ⚛️  Frontend React:" -ForegroundColor Yellow
Write-Host "     - $($FrontendReport.stats.total_files) fichiers"
Write-Host "     - $($FrontendReport.stats.total_lines) lignes"
Write-Host "     - $($FrontendReport.components.Count) composants"
Write-Host "     - $($FrontendReport.hooks.Count) hooks"
Write-Host "     - $($FrontendReport.dependencies.external.Count) packages npm"

Write-Host "`n  🔗 Intégration Tauri:" -ForegroundColor Yellow
Write-Host "     - $($BackendCommands.Count) commandes backend"
Write-Host "     - $($FrontendCommands.Count) appels frontend"
if ($MissingInBackend.Count -eq 0) {
    Write-Host "     - ✅ Cohérence validée" -ForegroundColor Green
} else {
    Write-Host "     - ❌ $($MissingInBackend.Count) commandes incohérentes" -ForegroundColor Red
}

Write-Host "`n  📁 Fichiers générés:" -ForegroundColor Yellow
Write-Host "     - $JsonPath"
Write-Host "     - $MermaidPath"

Write-Host "`n✅ Analyse terminée!`n" -ForegroundColor Green
