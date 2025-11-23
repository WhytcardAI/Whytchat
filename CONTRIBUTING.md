# 🤝 Guide de Contribution - WhytChat

Bienvenue dans le guide de contribution de WhytChat ! Nous sommes ravis que vous souhaitiez contribuer à ce projet open source.

## 📋 Table des Matières

- [Code de Conduite](#code-de-conduite)
- [Comment Contribuer](#comment-contribuer)
- [Configuration de l'Environnement](#configuration-de-lenvironnement)
- [Standards de Code](#standards-de-code)
- [Processus de Pull Request](#processus-de-pull-request)
- [Types de Contributions](#types-de-contributions)
- [Signaler un Bug](#signaler-un-bug)
- [Demander une Feature](#demander-une-feature)

## 🤝 Code de Conduite

Ce projet suit un code de conduite pour assurer un environnement accueillant pour tous. En participant, vous acceptez de :

- **Respecter** tous les participants
- **Être constructif** dans les retours
- **Accepter** les critiques constructives
- **Se concentrer** sur ce qui est meilleur pour la communauté
- **Montrer empathie** envers les autres participants

## 🚀 Comment Contribuer

### 1. Préparation

1. **🍴 Fork** le repository
2. **📋 Créez** une issue pour discuter de votre idée
3. **🌿 Créez** une branche : `git checkout -b feature/amazing-feature`

### 2. Développement

```bash
# Installez les dépendances
npm install

# Lancez le développement
npm run dev

# Vérifiez régulièrement le code
npm run lint
npm run format
```

### 3. Tests

```bash
# Tests frontend
cd apps/desktop-ui && npm test

# Tests backend (à implémenter)
cd apps/core && cargo test

# Linting complet
npm run lint
```

### 4. Commit

```bash
# Commitez avec un message descriptif
git commit -m "feat: add amazing feature

- Add new component X
- Update Y functionality
- Fix Z bug

Closes #123"
```

### 5. Pull Request

1. **🚀 Pushez** vers votre fork : `git push origin feature/amazing-feature`
2. **🔄 Ouvrez** une Pull Request sur GitHub
3. **📝 Remplissez** le template de PR
4. **👀 Attendez** la review

## 🛠️ Configuration de l'Environnement

### Prérequis

| Outil       | Version | Installation                        |
| ----------- | ------- | ----------------------------------- |
| **Node.js** | 20+     | [nodejs.org](https://nodejs.org/)   |
| **Rust**    | 1.75+   | [rustup.rs](https://rustup.rs/)     |
| **Git**     | 2.30+   | [git-scm.com](https://git-scm.com/) |

### Installation Détaillée

```bash
# 1. Clonez votre fork
git clone https://github.com/YOUR_USERNAME/WhytChat.git
cd WhytChat

# 2. Installez les dépendances
npm install

# 3. Installez Tauri CLI
cargo install tauri-cli --version "^2.0.0"

# 4. Vérifiez l'installation
npm run lint
npm run dev
```

### IDE Recommandé

- **VS Code** avec extensions :
  - Rust Analyzer
  - Tauri
  - Prettier
  - ESLint

## 📏 Standards de Code

### Rust

```rust
// ✅ Bon
pub async fn process_message(
    &self,
    message: String,
) -> Result<String, ActorError> {
    // Implementation
    Ok(result)
}

// ❌ Mauvais
pub async fn process_message(&self, message: String) -> Result<String, ActorError> {
    // Implementation
    result // Pas de gestion d'erreur
}
```

### JavaScript/React

```javascript
// ✅ Bon
function ChatInput({ onSend, disabled }) {
  const handleSubmit = useCallback(
    (message) => {
      if (message.trim()) {
        onSend(message.trim());
      }
    },
    [onSend]
  );

  return (
    <form onSubmit={handleSubmit}>
      <input
        type="text"
        disabled={disabled}
        placeholder={t("chat.placeholder")}
      />
    </form>
  );
}

// ❌ Mauvais
function ChatInput({ onSend }) {
  return (
    <input
      onKeyPress={(e) => {
        if (e.key === "Enter") {
          onSend(e.target.value); // Pas de validation
        }
      }}
    />
  );
}
```

### Git Commit Messages

Format : `type(scope): description`

Types :

- `feat` : Nouvelle fonctionnalité
- `fix` : Correction de bug
- `docs` : Documentation
- `style` : Formatage
- `refactor` : Refactorisation
- `test` : Tests
- `chore` : Maintenance

## 🔄 Processus de Pull Request

### Template de PR

```markdown
## Description

[Description claire de ce que fait la PR]

## Type de Changement

- [ ] 🐛 Bug fix
- [ ] ✨ New feature
- [ ] 💥 Breaking change
- [ ] 📚 Documentation
- [ ] 🎨 UI/UX
- [ ] 🧪 Tests

## Tests Réalisés

- [ ] Tests unitaires
- [ ] Tests d'intégration
- [ ] Tests manuels
- [ ] Linting passe

## Checklist

- [ ] Code review effectué
- [ ] Tests ajoutés/mis à jour
- [ ] Documentation mise à jour
- [ ] Changements backward compatible

## Issues Liées

Closes #123
```

### Review Process

1. **🤖 CI/CD** : Tests automatiques passent
2. **👀 Review** : Au moins 1 reviewer approuve
3. **✅ Merge** : Squash and merge avec message clean

## 🎯 Types de Contributions

### Code

- **🐛 Bug Fixes** : Corrections de bugs identifiés
- **✨ Features** : Nouvelles fonctionnalités
- **🔄 Refactoring** : Amélioration du code existant
- **🧪 Tests** : Ajout de tests

### Non-Code

- **📚 Documentation** : Guides, README, commentaires
- **🎨 Design** : UI/UX improvements
- **🌐 Translation** : Support de nouvelles langues
- **📊 Analytics** : Métriques et monitoring

## 🐛 Signaler un Bug

### Template de Bug Report

```markdown
## Description du Bug

[Description claire et concise]

## Étapes de Reproduction

1. Aller sur '...'
2. Cliquer sur '....'
3. Voir l'erreur

## Comportement Attendu

[Ce qui devrait se passer]

## Screenshots

[Si applicable]

## Environnement

- OS: [e.g. Ubuntu 22.04]
- Browser: [e.g. Chrome 120]
- Version: [e.g. 1.0.0]

## Logs

[Logs pertinents]
```

## 💡 Demander une Feature

### Template de Feature Request

```markdown
## Résumé

[Brève description de la feature]

## Problème

[Quel problème cela résout]

## Solution Proposée

[Description de la solution]

## Alternatives Considérées

[Autres solutions envisagées]

## Impact

[Impact sur les utilisateurs/développeurs]
```

## 🙏 Remerciements

Merci de contribuer à WhytChat ! Votre temps et votre expertise sont précieux pour la communauté.

---

_WhytCard Engineering - 2025_
