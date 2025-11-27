# 🔍 Registre des Irrégularités - WhytChat V1

> Inventaire complet des problèmes identifiés lors de l'analyse du codebase

---

## 📊 Tableau Récapitulatif

| # | Sévérité | Fichier | Description | Impact |
|---|----------|---------|-------------|--------|
| 1 | 🔴 HIGH | `tests/supervisor_tests.rs` | Ne compile pas - module inexistant | Tests inutilisables |
| 2 | 🔴 HIGH | `tests/brain_tests.rs` | `ContextPacket.suggestions` inexistant | Tests cassés |
| 3 | 🔴 HIGH | `tests/flow_tests.rs` | `fs_manager::delete_file` inexistant | Tests cassés |
| 4 | 🔴 HIGH | `tests/supervisor_tests.rs` | Params `process_message` incorrects | Tests cassés |
| 5 | ⚠️ MEDIUM | `text_extract.rs` | `.doc` non supporté | Documents ignorés |
| 6 | ⚠️ MEDIUM | `main.rs` | Double `params` | Confusion API |
| 7 | ⚠️ MEDIUM | `MessageBubble.jsx` | ThinkingBubble désactivé | Feature UX manquante |
| 8 | ⚠️ MEDIUM | `encryption.rs` | Nonce fixe | Sécurité potentielle |
| 9 | ⚠️ MEDIUM | `llm.rs` | Unsafe sans doc | Risque mémoire |
| 10 | ⚠️ MEDIUM | `rag.rs` | Filtres ignorés | Feature incomplète |
| 11 | ⚠️ MEDIUM | `appStore.js` | Pas de error handler persist | Crash possible |
| 12 | ℹ️ LOW | `useChatStream.js` | Handler global leak | Memory leak |
| 13 | ℹ️ LOW | `main.rs` | Variables `_unused` | Code smell |
| 14 | ℹ️ LOW | `preflight.rs` | TODO comments | Dette technique |
| 15 | ℹ️ LOW | `ChatInterface.jsx` | useEffect sans cleanup | Best practice |
| 16 | ℹ️ LOW | `Rail.jsx` | Icons hardcodées | Maintenabilité |
| 17 | ℹ️ LOW | `fs_manager.rs` | Logging verbeux | Debug noise |
| 18 | ℹ️ LOW | Multiple | Messages fr/en | UX confuse |

---

## 🔴 Haute Sévérité (4)

### IRR-001 : Tests Supervisor Ne Compilent Pas

**Fichier** : `apps/core/src/tests/supervisor_tests.rs`

**Problème** :
```rust
use crate::actors::supervisor::{SupervisorHandle, SupervisorMessage};
// ERROR: module `supervisor` not found in `actors`
```

**Cause** : Le module `supervisor` n'est pas exposé dans `actors/mod.rs`.

**Impact** : Tous les tests Supervisor sont inutilisables.

**Solution** :
```rust
// Dans apps/core/src/actors/mod.rs
pub mod supervisor;  // Ajouter cette ligne
```

---

### IRR-002 : Brain Tests - Champ Inexistant

**Fichier** : `apps/core/src/tests/brain_tests.rs`

**Problème** :
```rust
let packet = brain.analyze("test query");
assert!(packet.suggestions.is_empty());  
// ERROR: no field `suggestions`
```

**Cause** : `ContextPacket` n'a pas de champ `suggestions`.

**Impact** : Tests Brain cassés.

**Solution** : Mettre à jour les tests pour correspondre à la vraie structure.

---

### IRR-003 : Flow Tests - Fonction Inexistante

**Fichier** : `apps/core/src/tests/flow_tests.rs`

**Problème** :
```rust
fs_manager::delete_file(&path);  
// ERROR: function `delete_file` not found
```

**Cause** : `delete_file` n'existe pas dans `fs_manager`.

**Impact** : Tests de flux cassés.

**Solution** : Implémenter `delete_file` ou utiliser `std::fs::remove_file`.

---

### IRR-004 : Signature Process Message Incorrecte

**Fichier** : `apps/core/src/tests/supervisor_tests.rs`

**Problème** :
```rust
supervisor.process_message(session_id, message).await;
// Real signature: process_message(session_id, message, Option<Window>)
```

**Impact** : Tests ne compilent pas.

**Solution** : Ajouter `None` comme 3ème paramètre.

---

## ⚠️ Moyenne Sévérité (7)

### IRR-005 : Format .doc Non Supporté

**Fichier** : `apps/core/src/text_extract.rs`

**Problème** : Le code liste `.doc` dans les extensions mais ne le supporte pas.

**Impact** : Documents Word pré-2007 ignorés silencieusement.

**Solution** :
```rust
match extension.as_str() {
    "doc" => {
        warn!("Old .doc format not supported");
        return Err(AppError::UnsupportedFormat(".doc"));
    }
    // ...
}
```

---

### IRR-006 : Double Paramètre `params`

**Fichier** : `apps/core/src/main.rs`

**Problème** :
```rust
async fn update_model_config(
    params: Option<GenerationParams>,  // 1er params
    // ...
) {
    let params = GenerationParams { ... };  // Shadow
}
```

**Impact** : Le premier `params` n'est jamais utilisé.

**Solution** : Supprimer le paramètre inutilisé.

---

### IRR-007 : ThinkingBubble Désactivé

**Fichier** : `apps/desktop-ui/src/components/Chat/MessageBubble.jsx`

**Problème** :
```javascript
// Commenté:
// {isThinking && <ThinkingBubble steps={thinkingSteps} />}
```

**Impact** : Les utilisateurs ne voient pas les étapes de réflexion.

**Solution** : Réactiver ou créer un setting pour activer/désactiver.

---

### IRR-008 : Nonce Fixe pour Encryption

**Fichier** : `apps/core/src/encryption.rs`

**Problème** :
```rust
let nonce = Nonce::from_slice(&[0u8; 12]);  // Fixed nonce!
```

**Impact** : Risque cryptographique avec réutilisation du nonce.

**Solution** : Générer un nonce aléatoire et le stocker avec le ciphertext.

---

### IRR-009 : Unsafe FFI Sans Documentation

**Fichier** : `apps/core/src/actors/llm.rs`

**Problème** : Blocs `unsafe` sans commentaires `// SAFETY:`.

**Impact** : Risque de comportement indéfini.

**Solution** : Documenter les invariants pour chaque bloc unsafe.

---

### IRR-010 : Filtres RAG Non Implémentés

**Fichier** : `apps/core/src/actors/rag.rs`

**Problème** :
```rust
pub async fn search_with_filters(
    &self,
    query: String,
    _filters: Vec<String>,  // Ignoré!
) { ... }
```

**Impact** : L'API promet des filtres mais ne les utilise pas.

**Solution** : Implémenter ou renommer la fonction.

---

### IRR-011 : Persist Sans Error Handler

**Fichier** : `apps/desktop-ui/src/store/appStore.js`

**Problème** : Pas de `onRehydrateStorage` handler.

**Impact** : Crash silencieux si LocalStorage corrompu.

**Solution** :
```javascript
{
  name: 'whytchat-storage',
  onRehydrateStorage: () => (state, error) => {
    if (error) console.error('Hydration failed', error);
  },
}
```

---

## ℹ️ Basse Sévérité (7)

### IRR-012 : Memory Leak Potentiel Handler

**Fichier** : `apps/desktop-ui/src/hooks/useChatStream.js`

**Problème** : Variables globales `messageHandler`/`thinkingHandler` jamais nettoyées.

---

### IRR-013 : Variables Non Utilisées

**Fichier** : `apps/core/src/main.rs`

**Problème** : `let _pool = &state.pool;` préfixé mais conservé.

---

### IRR-014 : TODO Comments

**Fichier** : `apps/core/src/preflight.rs`

**Problème** : `// TODO: Add model validation`

---

### IRR-015 : useEffect Sans Cleanup

**Fichier** : `apps/desktop-ui/src/components/Chat/ChatInterface.jsx`

**Problème** : Setup listeners sans `return () => unlisten()`.

---

### IRR-016 : Icons Hardcodées

**Fichier** : `apps/desktop-ui/src/components/Layout/Rail.jsx`

**Problème** : Liste statique dans le composant.

---

### IRR-017 : Logging Verbeux

**Fichier** : `apps/core/src/fs_manager.rs`

**Problème** : Trop de `info!` en production.

---

### IRR-018 : Messages d'Erreur Incohérents

**Fichiers** : Multiples

**Problème** : Mix anglais/français dans les messages d'erreur.

---

## 📋 Plan de Correction

### Phase 1 : Critique (Immédiat)

- [ ] IRR-001 à IRR-004 : Corriger les tests
- [ ] IRR-008 : Auditer le nonce encryption

### Phase 2 : Important (Court terme)

- [ ] IRR-006 : Nettoyer double params
- [ ] IRR-010 : Implémenter filtres RAG
- [ ] IRR-011 : Ajouter error handler persist

### Phase 3 : Amélioration (Moyen terme)

- [ ] IRR-007 : Réactiver ThinkingBubble
- [ ] IRR-005 : Support .doc ou message clair
- [ ] IRR-009 : Documenter unsafe

### Phase 4 : Qualité (Long terme)

- [ ] IRR-014 : Résoudre TODOs
- [ ] IRR-018 : Standardiser messages
- [ ] IRR-012, IRR-015 : Cleanup React

---

## 📊 Métriques

| Catégorie | Nombre | Pourcentage |
|-----------|--------|-------------|
| 🔴 HIGH | 4 | 22% |
| ⚠️ MEDIUM | 7 | 39% |
| ℹ️ LOW | 7 | 39% |
| **Total** | **18** | 100% |

---

## 📚 Voir Aussi

- [06_SECURITE.md](06_SECURITE.md) - Détails sécurité (IRR-008)
- [08_RECOMMANDATIONS.md](08_RECOMMANDATIONS.md) - Actions complètes

---

_Document généré le 27 novembre 2025_
