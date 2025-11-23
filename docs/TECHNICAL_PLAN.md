# Plan Technique : Intégration du "Thinking Mode"

Ce document définit le contrat d'interface et les étapes d'implémentation pour connecter le Frontend React au Backend Rust afin de visualiser le processus de raisonnement de l'IA.

## 1. Contrat d'Interface (Tauri Bridge)

### Commandes (Frontend vers Backend)

- **Nom** : `debug_chat`
- **Arguments** : `{ message: string }`
- **Retour** : `Promise<string>` (La réponse complète, bien que l'UI se mette à jour via streaming)
- **Description** : Point d'entrée principal pour envoyer une requête utilisateur au Superviseur.

### Événements (Backend vers Frontend)

- **Nom** : `thinking-step`
  - **Payload** : `string` (ex: "Analyse de l'intention...", "Recherche dans les documents...")
  - **Usage** : Met à jour la liste des étapes de pensée dans l'UI.
- **Nom** : `chat-token`
  - **Payload** : `string` (Token de texte)
  - **Usage** : Affiche la réponse de l'assistant en temps réel (streaming).

## 2. État des Lieux

- ✅ **Backend** : La commande `debug_chat` et les émissions d'événements sont implémentées dans `apps/core/src/actors/supervisor.rs`.
- ✅ **Store** : `apps/desktop-ui/src/store/appStore.js` gère déjà `thinkingSteps` et `isThinking`.
- ✅ **Composant UI** : `apps/desktop-ui/src/components/chat/ThinkingBubble.jsx` est créé.
- ❌ **Intégration** : `ThinkingBubble` n'est **pas utilisé** dans `ChatInterface.jsx`.

## 3. Plan d'Implémentation (Mode Code)

### Étape 1 : Intégration dans `ChatInterface.jsx`

1.  Ouvrir `apps/desktop-ui/src/components/chat/ChatInterface.jsx`.
2.  Ajouter l'import : `import { ThinkingBubble } from './ThinkingBubble';`.
3.  Dans le hook `useAppStore`, récupérer la propriété manquante : `thinkingSteps`.
    ```javascript
    const {
      setThinking,
      addThinkingStep,
      clearThinkingSteps,
      isThinking,
      thinkingSteps,
    } = useAppStore();
    ```
4.  Ajouter le composant `<ThinkingBubble />` dans le rendu.
    - **Positionnement suggéré** : À la fin de la liste des messages, juste avant la div vide `messagesEndRef`. Cela permet de voir le raisonnement se dérouler en temps réel comme s'il s'agissait d'un message temporaire.
    - **Props** : Passer `steps={thinkingSteps}`.

### Étape 2 : Vérification du Flux

1.  S'assurer que `handleSend` appelle bien `clearThinkingSteps()` et `setThinking(true)` avant l'appel backend (déjà présent).
2.  S'assurer que l'écouteur d'événement `thinking-step` appelle bien `addThinkingStep` (déjà présent).

### Étape 3 : Test

1.  Lancer l'application (`npm run tauri dev`).
2.  Envoyer un message (ex: "Qui es-tu ?").
3.  Observer l'apparition de la bulle de pensée et l'ajout progressif des étapes.
