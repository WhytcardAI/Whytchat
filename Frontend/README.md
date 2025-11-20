# WhytChat Frontend Minimal

Version épurée concentrée sur l'essentiel : une seule interface conversationnelle.

## Structure

- `src/App.jsx` : UI principale (header, messages, formulaire envoi)
- `src/main.jsx` : bootstrap React
- `src/lib/store.js` : état conversations & chat (Zustand)
- `src/stores/settingsStore.js` : préférences (thème, taille police)
- `src/i18n.js` + `src/locales/` : internationalisation FR / EN
- `src/index.css` : thème, variables et utilitaires Tailwind v4

Tous les anciens composants ont été supprimés (dossier `src/components`).

## Scripts

```bash
npm install
npm run dev
```

## Principes

- Pas de duplication de styles : utilitaires Tailwind.
- i18n pour tout texte visible (placeholder, labels).
- Garde côté Tauri : le store simule si l'API native n'est pas présente.

## Pistes d'amélioration (optionnel)

- Simplifier encore `store.js` (retirer multi-agents, groupes) si non utilisés.
- Ajouter tests unitaires (Zustand + logique d'envoi) via Vitest.
- Intégrer ESLint + Prettier pour cohérence continue.

## Suppression résiduelle

Si des artefacts subsistent dans le cache, relancer une installation :

```bash
npm prune
npm install
```

## Licence

Interne / Non spécifiée.
