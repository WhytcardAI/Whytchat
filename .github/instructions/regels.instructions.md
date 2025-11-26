---
applyTo: "**"
---

# Règles de Développement

## Internationalisation (i18n)

- **RÈGLE STRICTE** : Aucun texte en dur n'est autorisé dans les composants React ou le code UI.
- Tout le texte doit être extrait dans `apps/desktop-ui/src/locales/{lang}/common.json`.
- Utilisez le hook `useTranslation` de `react-i18next`.
- Les clés doivent être structurées logiquement (ex: `component.section.label`).

## Architecture Fichiers

- **ENTRÉE UNIQUE** : Les fichiers externes ne peuvent entrer QUE par `KnowledgeView`.
- **SessionWizard** : Sélectionne des fichiers EXISTANTS de la bibliothèque (pas d'upload).
- **ChatInput** : Texte uniquement, PAS d'upload de fichiers.

## Documentation

- Documentation technique : `Doc/` (ARCHITECTURE.md, RAG_SYSTEM.md, etc.)
- Guide agents : `AGENTS.md` à la racine

## MCP

- Toujours utiliser les MCP disponibles pour la recherche et la documentation.
