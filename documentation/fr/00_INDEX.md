# 📚 Documentation WhytChat V1

> Index de la documentation technique française

---

## 🗺️ Guide de Lecture

### Ordre Recommandé

| # | Document | Description |
|---|----------|-------------|
| 1 | [01_VUE_ENSEMBLE.md](01_VUE_ENSEMBLE.md) | Introduction, stack, métriques globales |
| 2 | [02_ARCHITECTURE.md](02_ARCHITECTURE.md) | Structure monorepo, actor system, patterns |
| 3 | [03_BACKEND_RUST.md](03_BACKEND_RUST.md) | Tous les modules Rust détaillés |
| 4 | [04_FRONTEND_REACT.md](04_FRONTEND_REACT.md) | Composants React, store, hooks |
| 5 | [05_FLUX_DONNEES.md](05_FLUX_DONNEES.md) | Flux complet d'un message chat |
| 6 | [06_SECURITE.md](06_SECURITE.md) | Chiffrement, authentification, vulnérabilités |
| 7 | [07_IRREGULARITES.md](07_IRREGULARITES.md) | 18 problèmes identifiés avec solutions |
| 8 | [08_RECOMMANDATIONS.md](08_RECOMMANDATIONS.md) | 12 actions suggérées avec roadmap |
| 9 | [09_METRIQUES.md](09_METRIQUES.md) | Statistiques, dépendances, complexité |

---

## 📊 Résumé du Projet

```
╔════════════════════════════════════════════════════════════╗
║                    WHYTCHAT V1                              ║
╠════════════════════════════════════════════════════════════╣
║  Stack           │ Tauri 2.0 + Rust + React                ║
║  Lignes de code  │ ~8,100                                  ║
║  Fichiers        │ 30+                                     ║
║  Commandes IPC   │ 22                                      ║
║  Irrégularités   │ 18 (4 critiques)                        ║
║  Couverture test │ ~15%                                    ║
╚════════════════════════════════════════════════════════════╝
```

---

## 🚨 Points d'Attention

### Critiques (à corriger immédiatement)
- 🔴 **Tests cassés** : 4 fichiers ne compilent pas → [07_IRREGULARITES.md](07_IRREGULARITES.md#-haute-sévérité-4)
- 🔴 **Nonce fixe** : Risque crypto → [06_SECURITE.md](06_SECURITE.md#62-moyenne-sévérité)

### Importants (court terme)
- ⚠️ Double params snake_case/camelCase
- ⚠️ ThinkingBubble désactivé
- ⚠️ Filtres RAG non implémentés

---

## 🔗 Liens Rapides

| Besoin | Document |
|--------|----------|
| Comprendre l'architecture | [02_ARCHITECTURE.md](02_ARCHITECTURE.md) |
| Débugger le chat | [05_FLUX_DONNEES.md](05_FLUX_DONNEES.md) |
| Audit sécurité | [06_SECURITE.md](06_SECURITE.md) |
| Corriger des bugs | [07_IRREGULARITES.md](07_IRREGULARITES.md) |
| Planifier le dev | [08_RECOMMANDATIONS.md](08_RECOMMANDATIONS.md) |

---

## 📁 Structure du Projet

```
WhytChat_V1/
├── apps/
│   ├── core/           # Backend Rust (Tauri)
│   └── desktop-ui/     # Frontend React
├── data/               # Données locales
├── documentation/      # ← Vous êtes ici
│   └── fr/            # Documentation française
└── Doc/                # [LEGACY - À supprimer]
```

---

## Légende des Symboles

| Symbole | Signification |
|---------|---------------|
| ✅ | Fonctionnel |
| ⚠️ | Attention requise |
| 🔴 | Problème critique |
| ℹ️ | Information |

---

_Documentation générée le 27 novembre 2025_
_Analyse complète du codebase par stratégie "Follow the Data Flow" (niveau PROFOND)_
