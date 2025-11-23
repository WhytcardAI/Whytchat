# Guide Utilisateur WhytChat

Bienvenue dans WhytChat, votre assistant IA personnel, sécurisé et totalement portable.

## 1. Qu'est-ce que la "Portabilité" ?

WhytChat est conçu pour être une **application portable**. Cela signifie qu'elle ne nécessite aucune installation complexe et qu'elle ne disperse pas vos fichiers un peu partout dans votre ordinateur.

**Tout ce dont WhytChat a besoin pour fonctionner se trouve dans son propre dossier.**

Vous pouvez :
*   Mettre le dossier `WhytChat` sur une **clé USB** et l'utiliser sur n'importe quel ordinateur compatible.
*   Déplacer le dossier d'un disque à un autre sans rien perdre.
*   Avoir plusieurs dossiers WhytChat différents pour des usages séparés (Travail, Personnel, Projet Secret), chacun étant totalement isolé des autres.

---

## 2. Où sont mes données ?

Toutes vos données sont stockées dans un dossier nommé `data` qui se trouve **juste à côté de l'exécutable** (le fichier `.exe` que vous lancez).

Si vous ouvrez le dossier de l'application, vous verrez quelque chose comme ceci :

```text
WhytChat/
├── WhytChat.exe        (L'application)
└── data/               (VOS DONNÉES PRÉCIEUSES)
    ├── db/             (Vos conversations et réglages)
    ├── models/         (Le "Cerveau" IA téléchargé)
    └── vectors/        (La mémoire de vos documents)
```

**Rien ne sort jamais de ce dossier.** WhytChat n'envoie rien sur le Cloud, n'écrit rien dans vos fichiers système cachés (`AppData`, etc.) et respecte totalement votre vie privée.

---

## 3. Comment sauvegarder mes données ?

Puisque tout est au même endroit, la sauvegarde est extrêmement simple.

### Pour faire une sauvegarde complète (Backup) :
1.  Assurez-vous que WhytChat est **fermé**.
2.  Copiez simplement le dossier `data`.
3.  Collez-le dans un endroit sûr (disque externe, cloud sécurisé, autre dossier).

### Pour restaurer une sauvegarde :
1.  Fermez WhytChat.
2.  Supprimez le dossier `data` actuel (ou renommez-le si vous voulez le garder par précaution).
3.  Placez votre dossier `data` de sauvegarde à côté de l'exécutable.
4.  Relancez WhytChat. Vous retrouverez vos conversations et vos modèles exactement comme ils étaient.

### Pour migrer vers un nouvel ordinateur :
1.  Copiez **tout le dossier** `WhytChat` (l'exécutable + le dossier `data`) sur une clé USB.
2.  Branchez la clé sur le nouvel ordinateur.
3.  Lancez `WhytChat.exe` directement depuis la clé ou après l'avoir copié sur le nouveau PC. C'est tout !
