---
applyTo: '**'
---

# Règles de Développement

## Internationalisation (i18n)
- **RÈGLE STRICTE** : Aucun texte en dur n'est autorisé dans les composants React ou le code UI.
- Tout le texte doit être extrait dans `src/locales/{lang}/common.json`.
- Utilisez le hook `useTranslation` de `react-i18next`.
- Les clés doivent être structurées logiquement (ex: `component.section.label`).

Toujours utiliser les mcp disponible

C:\Users\whyytya\Desktop\workspace\WhytChat\docs path de la documentation
