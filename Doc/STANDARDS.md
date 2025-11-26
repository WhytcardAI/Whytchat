# 📏 Standards de Développement & Best Practices

Ce guide définit les règles d'or pour contribuer au projet WhytChat. L'objectif est de maintenir une base de code saine, lisible et robuste.

## 🛡️ Règles Absolues (Extraites de AGENTS.md)

1.  **Chemins Relatifs Interdits** : Ne jamais utiliser `../data`. Utilisez TOUJOURS `PortablePathManager` (`apps/core/src/fs_manager.rs`) pour résoudre les chemins.
    *   *Raison* : L'application doit tourner de manière portable (ex: clé USB) et le répertoire de travail change entre Debug et Release.
2.  **Architecture Acteur** : Le backend n'est pas MVC. C'est un système d'Acteurs asynchrones (Tokio). Ne mettez pas de logique métier dans `main.rs`, déléguez-la au `Supervisor`.
3.  **Port Hardcoding** : Le port Frontend est fixé à 1420 par Tauri. Ne le changez pas sans mettre à jour `tauri.conf.json`.

## 🦀 Rust Best Practices

### Gestion d'Erreur (Panic is NOT an Option)
*   🚫 **Interdit** : `unwrap()`, `expect()` sur du code de production runtime.
*   ✅ **Recommandé** : `match`, `if let`, `?` operator, ou mapper vers une erreur custom (`map_err`).
    *   *Exception* : Les tests (`#[test]`) et l'initialisation au démarrage (`lazy_static`) peuvent paniquer si l'état est irrécupérable.

### Concurrence
*   Utilisez `tokio::sync::Mutex` plutôt que `std::sync::Mutex` dans les contextes `async` pour ne pas bloquer le thread runtime.
*   Attention aux Deadlocks : Ne jamais acquérir un lock dans une boucle ou en attendant un `await`.

## ⚛️ React Best Practices (Frontend)

### Structure des Composants
*   **Composition** : Préférez des petits composants fonctionnels.
*   **Hooks** : Extrayez la logique complexe dans des hooks personnalisés (ex: `useChatStream.js`).

### State Management
*   **Zustand** : Utilisé pour l'état global (User session, settings).
*   **React Query** (Optionnel futur) : Pour les données serveur asynchrones.
*   **Context API** : À éviter pour les données à haute fréquence de mise à jour (problèmes de re-render).

## 📝 Conventions de Nommage

| Type | Convention | Exemple |
| :--- | :--- | :--- |
| **Rust Structs/Enums** | PascalCase | `MessageStruct`, `IntentType` |
| **Rust Variables/Fonctions** | snake_case | `process_message`, `user_id` |
| **Rust Constantes** | SCREAMING_SNAKE | `MAX_RETRIES`, `DEFAULT_TIMEOUT` |
| **JS/React Components** | PascalCase | `ChatInterface.jsx` |
| **JS Variables/Functions** | camelCase | `handleClick`, `userData` |
| **Fichiers React** | PascalCase | `UserProfile.jsx` |
| **Fichiers Utilitaires** | camelCase | `dateFormatter.js` |

## 📚 Références Externes Officielles

*   [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
*   [Tauri 2.0 Documentation](https://v2.tauri.app/)
*   [React Documentation](https://react.dev/)
*   [Tokio Tutorial](https://tokio.rs/tokio/tutorial)

---
*Dernière mise à jour : Novembre 2025*