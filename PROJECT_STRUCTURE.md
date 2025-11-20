# Project Structure — WhytChat

Cette arborescence sépare clairement UI (Frontend), logique/OS (Backend), et artefacts (modèles, binaires). Elle est contractuelle: toute génération/modification par l’agent doit s’y conformer.

## Vue d’ensemble

```
WhytChat/
  Frontend/
    components/
    pages/
    styles/
    translations/
    public/
    package.json
    tsconfig.json
    vite.config.ts

  Backend/
    src-tauri/
      Cargo.toml
      tauri.conf.json
      capabilities/
        default.json
      icons/
      src/
        main.rs
        llama.rs
        llama_install.rs
        db.rs
      build.rs
    llama/
      server/       # binaires llama.cpp/llama-server par OS/arch
      dll/          # bibliothèques natives requises
    models/
      presets.json
      pack-sources.json
      packs/        # archives modèles téléchargées
      cache/        # extractions temporaires

  Shared/           # optionnel (types, schémas partagés)
    types/
    schemas/
    messages/

  scripts/
    dev.ps1
    build.ps1
    release.ps1

  docs/
    I18N_GUIDE.md
    PROJECT_STRUCTURE.md
    ARCHITECTURE.md
    SECURITY.md

  .github/
    workflows/
```

## Rôles & Règles

- Frontend: UI uniquement; appels `tauri.invoke` pour la logique.
- Backend/src-tauri: commandes Tauri, processus llama, DB, I/O.
- Backend/models: sources et index des modèles avec `sha256`; pas de code.
- Backend/llama: binaires/runtime tiers isolés.
- Capabilities: principe du moindre privilège (Tauri v2).

## Événements & Progression (Tauri v2)

- `download/progress` (global): `{ downloaded: u64, total?: u64, percent?: number }`.
- `download/finished` (global): `{ path: string, sha256: string, verified: boolean, expected?: string }`.
- `server_ready` (commande): renvoie `bool`; la UI active l’envoi si `true`.

Notes:

- Le hashing SHA-256 est calculé côté backend durant le téléchargement; la vérification n’est faite que si un `expected_sha256` est fourni à `download_model`.
- La UI écoute les événements via `window.__TAURI__.event.listen`.

Installation serveur LLaMA:

- Utiliser la commande `install_server(backend?: "cpu" | "cuda-<ver>")` qui télécharge depuis les Releases officielles `ggml-org/llama.cpp` et extrait `llama-server.exe` dans `Backend/llama/server/`.
- Événements émis: `server/install/progress`, `server/install/finished` (inclut `zip_sha256`). Pour une validation stricte, comparez la somme avec la valeur publiée sur la page de Release.

## Build & Liaisons

- `Backend/src-tauri/tauri.conf.json` → `build.distDir` pointe vers le build Frontend (ex: `../Frontend/dist`).
- Sorties Tauri: `Backend/src-tauri/target/**` (non versionnées).
- Icônes/ressources: `Backend/src-tauri/icons/`.

## Contraintes de Conformité (Agent)

- Respect strict de cette arborescence pour toute création/modification.
- Toute déviation doit être explicitement validée puis répercutée ici.
- Mettre à jour ce document avant/pendant toute refonte structurelle.
