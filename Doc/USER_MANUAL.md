# 📘 Guide Utilisateur WhytChat V1

Bienvenue dans WhytChat, votre assistant IA local et sécurisé.

## 🚀 Démarrage Rapide

### Créer une Nouvelle Session

1.  Cliquez sur le bouton **"+ Nouvelle"** dans la barre latérale gauche.
2.  L'assistant de création s'ouvre.
3.  **Titre** : Donnez un nom à votre conversation.
4.  **Fichiers de contexte** (Optionnel) : Sélectionnez des documents déjà présents dans votre bibliothèque pour les associer à cette session.
5.  **Options Avancées** (Optionnel) :
    - **System Prompt** : Définissez la personnalité de l'IA (ex: "Tu es un expert en Python").
    - **Température** : Ajustez la créativité (0.0 = précis, 2.0 = très créatif).

## 📂 Base de Connaissances (Knowledge Base)

WhytChat vous permet de discuter avec vos propres documents grâce au système RAG (Retrieval-Augmented Generation).

### Importer des Fichiers

1.  Cliquez sur l'icône **Base de Données** dans la barre de navigation pour ouvrir la vue **Knowledge Base**.
2.  Cliquez sur le bouton **"Import Data"**.
3.  Sélectionnez un ou plusieurs fichiers (upload multiple supporté).

#### Formats Supportés

| Format     | Extension       | Description                                     |
| ---------- | --------------- | ----------------------------------------------- |
| Texte brut | `.txt`          | Fichiers texte simples                          |
| Markdown   | `.md`           | Documentation formatée                          |
| CSV        | `.csv`          | Données tabulaires                              |
| JSON       | `.json`         | Données structurées                             |
| PDF        | `.pdf`          | Documents PDF (extraction automatique du texte) |
| Word       | `.docx`, `.doc` | Documents Microsoft Word                        |

> **Taille maximale** : 10 MB par fichier.

### Organiser vos Documents

- **Créer un dossier** : Cliquez sur "New Folder" pour organiser vos documents par catégorie.
- **Déplacer un fichier** : Survolez un fichier et cliquez sur l'icône dossier pour le déplacer.
- **Supprimer un fichier** : Cliquez sur l'icône corbeille (supprime aussi les vecteurs associés).
- **Réindexer** : Cliquez sur "Re-index" pour recalculer tous les embeddings de la bibliothèque.

### Associer des Documents à une Session

Lors de la création d'une nouvelle session, vous pouvez sélectionner des documents existants de votre bibliothèque. L'IA n'aura accès qu'aux documents explicitement liés à la session active.

### Analyser un Document

Survolez un fichier dans la Knowledge Base et cliquez sur l'icône 🧠 (cerveau) pour lancer une analyse automatique. Vous serez redirigé vers le chat avec un prompt pré-rempli.

## 💬 Utilisation du Chat

Une fois vos fichiers importés et associés à une session :

- Posez simplement vos questions dans la zone de texte.
- L'IA recherchera automatiquement les informations pertinentes dans vos documents.
- Les sources utilisées sont indiquées dans le contexte de la réponse.

**Exemples de questions** :

- "Résume le document que j'ai importé"
- "Quelles sont les conclusions principales ?"
- "Trouve les informations sur [sujet] dans mes fichiers"

## ⚙️ Paramètres

- **Thème** : Cliquez sur l'icône Lune/Soleil en haut à droite pour basculer entre thème clair et sombre.
- **Dossiers de sessions** : Organisez vos conversations par dossiers via la barre latérale gauche.
- **Favoris** : Cliquez sur l'étoile à côté d'une session pour la marquer comme favorite.

---

_WhytChat V1 - Novembre 2025_
