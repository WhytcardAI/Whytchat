# Plan de Refactoring "Enterprise-Grade" - WhytChat Frontend

Ce document définit les standards de qualité, la stratégie de gestion des erreurs et l'audit détaillé des composants clés de l'application. Il sert de référence pour élever la base de code à un niveau professionnel et maintenable.

## 1. Standards "Enterprise-Grade"

### A. Documentation & Typage
*   **JSDoc Obligatoire :** Tous les composants, hooks et fonctions utilitaires doivent avoir un bloc JSDoc décrivant :
    *   Le but du composant/fonction.
    *   Les paramètres (`@param`) avec leurs types et descriptions.
    *   La valeur de retour (`@returns`).
*   **PropTypes Exhaustifs :** Tous les composants React acceptant des props doivent définir `propTypes`.
    *   Utiliser `PropTypes.shape` pour les objets complexes.
    *   Définir `defaultProps` pour les props optionnelles.

### B. Gestion des Erreurs & Robustesse
*   **Error Boundaries :** Utilisation de `react-error-boundary` pour isoler les plantages de composants (ex: un widget qui plante ne doit pas casser toute l'app).
*   **Try/Catch Typés :** Les appels asynchrones et logiques critiques doivent être enveloppés dans des blocs `try/catch` avec une gestion explicite de l'erreur (logging, notification utilisateur).
*   **Validation des Données :** Vérification défensive de l'existence des objets imbriqués (ex: `conversation?.config?.debateConfig`) avant accès.

### C. Performance & Bonnes Pratiques
*   **Mémoïsation :** Utilisation judicieuse de `useMemo` et `useCallback` pour les fonctions passées en props et les calculs coûteux.
*   **Séparation des Responsabilités :** Extraction de la logique métier complexe des composants d'affichage vers des custom hooks.
*   **Nommage Cohérent :**
    *   Composants : PascalCase (ex: `DebateStage`).
    *   Hooks : camelCase commençant par `use` (ex: `useHotkeys`).
    *   Fonctions handlers : `handle[Event]` (ex: `handleSelect`).

---

## 2. Stratégie de Gestion d'Erreurs

### Installation
Ajout de la dépendance : `npm install react-error-boundary prop-types`

### Architecture
1.  **Global Error Boundary :** Envelopper l'application (`App.jsx` ou `main.jsx`) pour attraper les erreurs non gérées et afficher une page d'erreur générique ("Oups, quelque chose s'est mal passé").
2.  **Local Error Boundaries :** Envelopper les composants complexes et isolés :
    *   `DebateStage` : Si le débat plante, le chat doit rester fonctionnel.
    *   `ChatArea` : Si la zone de chat plante, la sidebar doit rester accessible.
    *   `PromptTemplates` : Si le menu plante, il doit simplement se fermer sans casser l'input.

---

## 3. Audit Détaillé & Actions

### 📁 `src/components/DebateStage/index.jsx`

**État Actuel :**
*   ❌ Pas de PropTypes.
*   ❌ Pas de JSDoc.
*   ❌ Accès profond aux objets sans validation défensive robuste (ex: `conversation.config.debateConfig`).
*   ❌ Composant `AgentAvatar` défini dans le même fichier (devrait être extrait ou au moins documenté).

**Actions de Refactoring :**
1.  [ ] Ajouter JSDoc pour `DebateStage` et `AgentAvatar`.
2.  [ ] Implémenter `PropTypes` pour `AgentAvatar` (props: `agent`, `isActive`, `side`, `color`, `roleLabel`).
3.  [ ] Sécuriser l'accès aux données du store (vérifier `conversation` et `debateConfig` avant déstructuration).
4.  [ ] Envelopper dans un `ErrorBoundary` local (avec composant de fallback "Erreur d'affichage du débat").

### 📁 `src/components/PromptTemplates/index.jsx`

**État Actuel :**
*   ❌ Pas de PropTypes (props: `isVisible`, `onSelect`, `onClose`, `filterText`, `position`).
*   ❌ Pas de JSDoc.
*   ⚠️ Logique de filtrage dans le rendu (pourrait être mémoïsée si la liste grandit).
*   ⚠️ Gestion des événements clavier (`useEffect`) un peu complexe, pourrait être extraite.

**Actions de Refactoring :**
1.  [ ] Ajouter JSDoc complet.
2.  [ ] Implémenter `PropTypes` exhaustifs.
3.  [ ] Mémoïser `filteredTemplates` avec `useMemo`.
4.  [ ] Extraire la logique de navigation clavier dans un hook `useKeyboardNavigation`.

### 📁 `src/hooks/useHotkeys.js`

**État Actuel :**
*   ✅ JSDoc présente (Bon point !).
*   ⚠️ Gestion des modificateurs (Ctrl/Cmd) un peu rigide.
*   ⚠️ Pas de vérification si `callback` est une fonction.

**Actions de Refactoring :**
1.  [ ] Ajouter une validation : vérifier que `callback` est bien une fonction.
2.  [ ] Améliorer la JSDoc pour préciser le comportement cross-platform (Ctrl vs Meta).
3.  [ ] Envisager d'utiliser une librairie robuste comme `react-hotkeys-hook` à terme, ou renforcer ce hook maison.

### 📁 `src/components/layout/ChatArea.jsx`

**État Actuel :**
*   ❌ Composant très monolithique (~380 lignes).
*   ❌ Pas de JSDoc.
*   ❌ Mélange de logique d'UI, de gestion d'état, de hotkeys et de logique métier.
*   ⚠️ `handleTemplateSelect` manipule le DOM directement (`setSelectionRange`) via `setTimeout`, ce qui est fragile.

**Actions de Refactoring :**
1.  [ ] Ajouter JSDoc pour le composant principal.
2.  [ ] Extraire la logique de gestion de l'input (templates, slash commands) dans un hook `useChatInput`.
3.  [ ] Extraire la logique de configuration (modales) dans un hook `useChatConfig`.
4.  [ ] Remplacer le `setTimeout` par une gestion d'effet plus propre ou `useLayoutEffect` pour le focus.
5.  [ ] Envelopper les zones critiques (`Virtuoso`, `DebateStage`) dans des `ErrorBoundary`.

### 📁 `src/components/MessageBubble/index.jsx`

**État Actuel :**
*   ❌ Pas de PropTypes.
*   ❌ Pas de JSDoc.
*   ⚠️ Regex `AGENT_REGEX` définie au top-level (ok, mais pourrait être dans une constante partagée si réutilisée).

**Actions de Refactoring :**
1.  [ ] Ajouter JSDoc.
2.  [ ] Implémenter `PropTypes` (props: `role`, `content`).
3.  [ ] Déplacer `AGENT_REGEX` dans `src/lib/constants.js` ou `utils.js` pour centralisation.

---

## 4. Plan d'Exécution

1.  **Installation :** `npm install prop-types react-error-boundary`
2.  **Refactoring Hooks :** Commencer par `useHotkeys` (base saine).
3.  **Refactoring Composants Simples :** `MessageBubble`, `PromptTemplates`.
4.  **Refactoring Composants Complexes :** `DebateStage`, puis `ChatArea` (le plus gros morceau).
5.  **Intégration Error Boundaries :** Ajouter les barrières de sécurité.
6.  **Vérification :** Linter (`npm run lint`) et test manuel.