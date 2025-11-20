# WhytChat — Plan d’Architecture Multi‑Agents (MoA)

## Objectif

Concevoir un système de conversation orchestrant un **LLM principal (Synthèse)** et plusieurs **agents experts (SLM)** hétérogènes (constructeurs différents) pour améliorer qualité, robustesse et diversité cognitive.

## Architecture Fonctionnelle

- **Utilisateur → Orchestrateur** (LLM principal) reçoit la requête.
- **Phase Experts**: 2–4 agents spécialisés produisent des analyses ciblées (logique, créativité, critique/sécurité, factuel).
- **Phase Synthèse**: LLM principal agrège les rapports et génère la réponse finale.

## Modèles (candidats GGUF)

- **Expert Logique**: Microsoft Phi‑3.5 mini (≈3.8B, Q4_K_M), ou fallback Qwen2‑1.5B.
- **Expert Créatif**: Google Gemma 2 2B (licence à accepter) ou SmolLM3‑3B (Apache‑2.0).
- **Expert Critique/Sécurité**: Llama 3.2 3B ou SmolLM3‑3B.
- **Synthétiseur (Main)**: Qwen2.5‑3B ou SmolLM3‑3B selon ressources.

Références (extraits):

- llama.cpp server flags: `--port`, `-c/--ctx-size`, `-ngl`, `--parallel`, `--metrics`, `--slots`.
- Multi‑instances: lancer plusieurs `llama-server` sur ports distincts; alternatives: RPC / proxy (llama-swap) ou mode séquentiel.

## Contraintes & Stratégies

- **Ressources**: multi‑serveurs consomme RAM/VRAM; prévoir mode **séquentiel** (mêmes binaire & port, prompts différents) comme fallback.
- **Licences**: Gemma 2 nécessite acceptation; fournir fallback libre (SmolLM3 / Qwen2).
- **Perfs**: Activer `--metrics` pour observabilité; paralléliser via `reqwest` + `tokio` si multi‑ports.

## Design Backend (Tauri/Rust)

- `AppState`: passer de `server: Option<Child>` à `servers: HashMap<u16, Child>`.
- `start_server(model_path, port, ctx, ngl, parallel, metrics)` → lance un `llama-server` paramétré.
- `stop_server(port)` → stoppe un serveur.
- `server_ready(port)` → check santé.
- `chat_single(port, messages)` → POST /v1/chat/completions.
- `chat_multi(agents: [{port, system, user}], synth_port)`:
  - Exécute en parallèle `chat_single` pour chaque agent (messages injectés: system+user+historique filtré),
  - Concatène les rapports,
  - Appelle `chat_single` sur `synth_port` pour produire la réponse finale.

## Design Frontend

- Par conversation, **Config Agents**: liste d’experts {alias, port, modèle, prompts spécialisés} + **Synthétiseur**.
- Bouton/flux "Débat" → 1 tour Experts → 1 tour Synthèse → réponse UI.
- Historiser: stocker rapports agents pour audit.

## Roadmap (V1 → V2)

- **V1**: Orchestration séquentielle (mono‑serveur), agents simulés par prompts spécialisés; robuste aux faibles ressources.
- **V2**: Multi‑serveurs parallèles (ports 8080/8081/8082), `tokio::join!`, `--metrics`.
- **V3**: Découverte LAN (option), partage d’analyses entre pairs.

## Tests & Observabilité

- Smoke tests: start/stop servers, healthcheck.
- Bench: latence/parallélisme, impact `-ngl` et `--ctx-size`.
- Logs: Brain.log + métriques `/metrics`.
