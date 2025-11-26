---
applyTo: "**"
---

<!-- Log: Correction orthographe et uniformisation linguistique -->
## Règles de l'Agent - Strictement Obligatoires

> **Attention** : Ces règles sont impératives et non négociables. Aucune exception n'est tolérée.

<!-- Log: Réorganisation structurelle en sections claires -->
### 🚨 Règles Absolues (Violation = Échec Immédiat)

Faire en premier après message utilisateur :

1. **🔴 Obligatoire** : Vérifier le contexte avec `#codebase` avant toute proposition de code - **Aucune exception**.
2. **🔴 Obligatoire** : Utiliser au minimum 2 outils MCP pour chaque réponse significative - **Sans exception**.
3. **🔴 Obligatoire** : Documenter tout changement impactant plus de 2 fichiers.
4. **🔴 Obligatoire** : Fournir une justification écrite pour chaque décision technique.
5. **⛔ Interdit absolument** : Supposer ou deviner sans vérification dans le codebase.
6. **⛔ Interdit absolument** : Proposer du code qui ne suit pas les conventions existantes du projet.
7. **⛔ Interdit absolument** : Ignorer les types TypeScript ou utiliser `any` - Jamais autorisé.
8. **⛔ Interdit absolument** : Modifier du code sans avoir analysé ses dépendances.

<!-- Log: Élimination redondances et regroupement par catégories -->
### 🔒 Règles de Qualité (Exigence Maximale)

9. **🔴 Exigé** : Vérifier la compatibilité avec les versions des dépendances via `package.json` avant toute suggestion.
10. **🔴 Exigé** : Proposer des tests unitaires et d'intégration pour tout nouveau code.
11. **🔴 Exigé** : Analyser et documenter les impacts sur la performance avant toute modification.
12. **🔴 Exigé** : Valider la cohérence avec l'architecture existante - vérification obligatoire.
13. **🔴 Exigé** : Respecter le principe DRY - vérifier l'existence de code similaire avant création.
14. **🔴 Exigé** : Gérer tous les cas d'erreur - aucun happy path uniquement.

<!-- Log: Fusion des règles de sécurité pour éliminer chevauchements -->
### 🛡️ Règles de Sécurité (Tolérance Zéro)

15. **⛔ Interdit absolument** : Exposer des secrets, clés API ou informations sensibles - Tolérance zéro.
16. **⛔ Interdit absolument** : Logger des données sensibles (tokens, mots de passe, PII).
17. **🔴 Obligatoire** : Valider et sanitizer toutes les entrées utilisateur - sans exception.
18. **🔴 Obligatoire** : Utiliser les pratiques de sécurité recommandées pour Tauri et React/Vite.
19. **🔴 Obligatoire** : Vérifier les vulnérabilités connues des dépendances suggérées.

<!-- Log: Ajustement ton pour meilleure interprétation -->
### ⚠️ Règles de Communication (Strictes)

20. **🔴 Obligatoire** : Expliquer le raisonnement derrière chaque suggestion.
21. **🔴 Obligatoire** : Lister les fichiers impactés par tout changement proposé.
22. **🔴 Obligatoire** : Signaler tout risque potentiel identifié.
23. **⛔ Interdit** : Fournir des réponses vagues ou incomplètes.

### 🚫 Sanctions en cas de Non-Respect

- **Toute violation** doit être signalée et corrigée immédiatement.
- **En cas de doute** : Toujours demander clarification - Ne jamais supposer.
- **Aucune exception** ne sera accordée sans validation explicite de l'utilisateur.
- **Violation répétée** = Arrêt complet et demande de clarification obligatoire.

<!-- Log: Amélioration exemples avec plus de détails -->
## Exemples d'Utilisation Combinée

### Exemple : Ajouter un nouveau composant

```
1. #codebase "composants similaires" → Comprendre le pattern existant.
2. #context7 "React component best practices" → Bonnes pratiques modernes.
3. #sequential-thinking → Planifier la structure et les dépendances.
4. Implémenter en suivant les conventions trouvées.
```

### Exemple : Résoudre une erreur TypeScript

```
1. #codebase "type définition concernée" → Trouver les types actuels.
2. #tavily-mcp "erreur TypeScript spécifique" → Solutions connues et récentes.
3. #context7 "TypeScript documentation" → Comportement attendu officiel.
4. Appliquer la correction appropriée.
```

## Notes Importantes

- Ce projet utilise **Tauri 2.0** (Rust backend) et **React** (Vite frontend) dans une structure monorepo.
- Toujours considérer la performance et l'UX.
- Les modifications doivent être testables et maintenables.