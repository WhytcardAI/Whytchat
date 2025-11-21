# Proposition d'Améliorations Techniques et Fonctionnelles - WhytChat

Basée sur l'analyse de la codebase actuelle (v0.0.0), voici une feuille de route détaillée pour faire passer WhytChat au niveau supérieur.

## 1. UX/UI Ludique et Fluide

L'objectif est de rendre l'interface plus "vivante", en particulier pour les modes interactifs comme le Débat.

### A. Animations Avancées (Ajout de `framer-motion`)
Actuellement, l'application utilise des animations CSS simples (`animate-in`). Pour des interactions complexes, `framer-motion` est recommandé.

*   **Transitions de Messages :** Animer l'arrivée des bulles de message avec un effet de ressort (spring) pour un ressenti plus naturel.
*   **Mode Débat Visuel :**
    *   Créer un composant `DebateStage` au-dessus du chat qui visualise les deux agents (avatars) face à face.
    *   Animer l'avatar "actif" (celui qui parle) : effet de "glow", légère augmentation de taille.
    *   Afficher une barre de progression pour les tours (ex: "Tour 3/5").
*   **Micro-interactions :**
    *   Feedback visuel lors du clic sur "Copier" ou "J'aime" (petite animation d'icône).
    *   Transition fluide lors de l'ouverture/fermeture de la Sidebar et du ConfigPanel.

### B. Indicateurs de Frappe Contextuels
Remplacer le loader générique "Réflexion..." par des indicateurs spécifiques au contexte.

*   **Standard :** "L'assistant rédige..."
*   **Mode Débat :** "L'Agent A [Nom] analyse l'argument de B..." / "L'Agent B prépare sa réfutation..."
*   **Mode MoA :** "Le Logicien déconstruit la demande..." -> "Le Chercheur vérifie les faits..." (Afficher l'étape en cours du processus MoA).

## 2. Fonctionnalités Puissantes mais Simples

### A. Raccourcis Clavier (Productivité)
Implémenter un hook global `useHotkeys` pour permettre une navigation sans souris.

*   `Ctrl + K` / `Cmd + K` : Ouvrir une palette de commandes (type Spotlight) pour :
    *   Changer de conversation.
    *   Changer de modèle/température rapidement.
    *   Activer/Désactiver le mode MoA.
*   `Ctrl + /` : Ouvrir les réglages.
*   `Esc` : Fermer les modales ou la sidebar.
*   `Ctrl + N` : Nouvelle conversation.

### B. Système de Templates (Prompts)
Actuellement, les personas sont gérés via le store mais restent basiques.

*   **Bibliothèque de Prompts :** Créer un fichier `src/lib/templates.js` ou un store dédié.
*   **Interface de Sélection :** Ajouter un bouton "/" dans la zone de saisie qui ouvre un menu pop-up pour insérer rapidement un template (ex: "/dev" -> "Tu es un expert React...", "/mail" -> "Rédige un email professionnel pour...").
*   **Variables dynamiques :** Permettre des placeholders dans les templates (ex: `{{langage}}`) que l'utilisateur remplit au moment de l'insertion.

## 3. Qualité du Code et Performance

### A. Virtualisation des Messages (Critique pour les longs chats)
Le composant `ChatArea.jsx` rend tous les messages du tableau. Cela va ralentir l'application si une conversation dépasse 50-100 messages.

*   **Solution :** Intégrer `react-virtuoso`.
*   **Implémentation :** Remplacer le conteneur de scroll manuel par `<Virtuoso />`. Cela ne rendra que les messages visibles à l'écran + un petit buffer, garantissant une fluidité constante (60fps) peu importe la longueur de l'historique.
*   **Auto-scroll :** `react-virtuoso` gère très bien le "stick to bottom" (rester en bas quand un nouveau message arrive).

### B. Optimisation des Rendus (React Compiler / Memo)
*   **Mémoïsation :** Envelopper `MessageBubble` dans `React.memo` pour éviter qu'il ne se re-rende si son contenu n'a pas changé (utile quand on tape dans l'input, ce qui peut provoquer des re-renders du parent).
*   **Code Splitting :** Utiliser `React.lazy` pour les composants lourds comme `DebateConfigPanel` ou les modales de réglages, afin d'alléger le bundle initial.

### C. Gestion d'Erreurs Globale
*   Ajouter un `ErrorBoundary` React autour de l'application pour capturer les crashs inattendus (ex: erreur de parsing JSON d'un message) et afficher une UI de secours ("Oups, quelque chose s'est mal passé") au lieu d'une page blanche.

## 4. Accessibilité et Internationalisation

### A. Audit et Amélioration A11y
*   **Navigation au Clavier :** S'assurer que tous les boutons interactifs (icônes) ont un `tabIndex` et un état `:focus-visible` clair.
*   **ARIA Labels :** Ajouter `aria-label` sur les boutons qui n'ont que des icônes (ex: bouton "Settings", "Send").
*   **Contraste :** Vérifier que les couleurs du thème sombre (textes gris sur fond noir) respectent le ratio WCAG AA.

### B. Extension de l'Internationalisation
L'infrastructure est déjà bonne (`i18next`), mais il manque :
*   **Traduction des Prompts Système :** Les prompts par défaut (ex: "You are a highly capable AI...") sont hardcodés en anglais dans `store.js`. Il faut les déplacer dans les fichiers de traduction (`locales/*.json`) ou les générer dynamiquement selon la langue choisie.
*   **Formatage des Dates :** Utiliser `Intl.DateTimeFormat` ou `date-fns` avec la locale active pour afficher les timestamps des messages ("Il y a 5 min" vs "5 min ago").

---

## Plan d'Action Technique (Priorisé)

1.  **Performance First :** Installer `react-virtuoso` et refactoriser `ChatArea` pour la virtualisation.
2.  **UX Core :** Installer `framer-motion` et animer les bulles de messages + ajouter les indicateurs de frappe.
3.  **Fonctionnalités :** Imémenter le hook de raccourcis clavier.
4.  **A11y/I18n :** Faire une passe sur les `aria-labels` et extraire les prompts système hardcodés.