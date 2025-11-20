# TODO — WhytChat Multi‑Agents

## Phase 0 — Initialisation

- [x] Analyse (Sequential Thinking 1)
- [x] Recherche (Tavily: search + extract)
- [x] Docs techniques (Context7)
- [x] Synthèse (Sequential Thinking 2)
- [x] Rédiger `brain.md`
- [ ] Créer `Brain.log`

## Phase 1 — Backend (V1 séquentiel)

- [x] API: `chat_multi` (appel séquentiel des experts, puis synthèse)
- [x] Injection prompts spécialisés par agent (system + persona + consignes)
- [ ] Paramétrage par conversation (store + schéma minimal)
- [ ] Journaux: tracer inputs/outputs agents (minimaux) dans conversation

## Phase 2 — Backend (V2 multi‑serveurs)

- [ ] `AppState.servers: HashMap<u16, Child>`
- [ ] `start_server(model, port, ctx, ngl, parallel, metrics)`
- [ ] `stop_server(port)`, `server_ready(port)`
- [ ] `chat_multi` parallèle (tokio::join!/try_join!)
- [ ] Flags perfs: `--metrics`, `--slots`, `--parallel`

## Phase 3 — Frontend

- [ ] UI Agents par conversation (config experts + synthétiseur)
- [ ] Orchestration UI (Débat: 1 tour Experts → Synthèse)
- [ ] Affichage rapports experts (repliables)

## Phase 4 — Modèles & Ressources

- [ ] Ajout téléchargeur GGUF (SmolLM3‑3B, Qwen2‑1.5B, Gemma 2 2B – sous conditions)
- [ ] Sélection de quantization (Q4_K_M recommandé)
- [ ] Documentation mode séquentiel vs multi‑serveurs

## Phase 5 — Tests & Observabilité

- [ ] Smoke tests (start/stop/health)
- [ ] Bench latence/parallélisme
- [ ] Export métriques `/metrics` (scrape manuel)
