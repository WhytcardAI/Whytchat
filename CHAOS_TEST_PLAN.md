# Plan de Test Exploratoire Chaotique - WhytChat UI

**Objectif**: Identifier les failles de robustesse de l'UI/UX en simulant des actions utilisateur non prévues, illogiques ou malveillantes.

---

## Zone 1: Gestion des Sessions (Sidebar)

### Scénario 1.1: Création de sessions en rafale ("Race Condition")

*   **Action Utilisateur Chaotique**:
    1.  Cliquer sur le bouton "Nouveau Chat" le plus rapidement possible, 5 à 10 fois de suite, sans attendre que l'interface se mette à jour.
    2.  Utiliser un script d'automatisation pour simuler 20 clics en moins d'une seconde.

*   **Comportement Attendu**:
    *   L'interface doit rester réactive.
    *   Soit une seule nouvelle session est créée (si l'action est "debounced" ou verrouillée), soit le nombre exact de sessions demandées est créé.
    *   Aucun état intermédiaire invalide ne doit être visible (par exemple, des sessions sans nom ou dupliquées).
    *   La session active (`currentSessionId`) doit être clairement définie sur l'une des nouvelles sessions (probablement la dernière créée).

*   **Bug Potentiel à Détecter**:
    *   **Crash / Gel de l'UI**: Le frontend ne répond plus à cause des multiples re-renderings et appels asynchrones.
    *   **Corruption de l'état (State)**: Création de sessions fantômes, `null` ou `undefined` dans le tableau `sessions` du store Zustand.
    *   **Désynchronisation UI/Backend**: L'interface affiche un certain nombre de sessions, mais la base de données en contient un nombre différent.
    *   **Perte de focus**: La session active devient `null`, affichant une interface de chat vide ou invalide.

### Scénario 1.2: Changement de session frénétique

*   **Action Utilisateur Chaotique**:
    1.  Avoir au moins 5 sessions dans la liste.
    2.  Cliquer alternativement et très rapidement sur différentes sessions dans la barre latérale.
    3.  Pendant le chargement apparent d'une session, cliquer immédiatement sur une autre.

*   **Comportement Attendu**:
    *   L'UI ne doit pas geler. Un indicateur de chargement peut apparaître brièvement.
    *   Seule la dernière session cliquée doit être chargée et affichée dans la zone de chat principale.
    *   Les requêtes précédentes pour charger les messages des sessions intermédiaires doivent être annulées ou leurs résultats ignorés pour éviter d'afficher le contenu d'une mauvaise session.

*   **Bug Potentiel à Détecter**:
    *   **Mélange de contextes**: Les messages d'une session A s'affichent brièvement (ou de façon permanente) alors que la session B est sélectionnée.
    *   **Gel de l'UI**: L'application se bloque en essayant de gérer les multiples mises à jour d'état concurrentes.
    *   **Curseur de sélection incohérent**: La session mise en surbrillance dans la sidebar ne correspond pas à celle affichée dans la fenêtre de chat.

### Scénario 1.3: Actions concurrentes (Création + Changement)

*   **Action Utilisateur Chaotique**:
    1.  Cliquer sur "Nouveau Chat".
    2.  Immédiatement après (avant que la nouvelle session n'apparaisse et ne soit sélectionnée), cliquer sur une session existante.
    3.  Inverser l'action : cliquer sur une session existante, puis immédiatement sur "Nouveau Chat".

*   **Comportement Attendu**:
    *   L'état final doit être cohérent et refléter la *dernière* action initiée par l'utilisateur.
    *   Si le changement de session est la dernière action, cette session doit être chargée. La nouvelle session créée en arrière-plan doit simplement apparaître dans la liste.
    *   Si la création est la dernière action, la nouvelle session doit être créée et devenir la session active.

*   **Bug Potentiel à Détecter**:
    *   **État imprévisible**: L'application se retrouve dans un état non défini (ex: aucune session sélectionnée, ou la mauvaise session est active).
    *   **Perte d'une action**: Une des deux actions (la création ou le changement) est complètement ignorée.
    *   **Erreur dans la console**: Erreurs JavaScript liées à la tentative de mise à jour d'un état qui n'est plus pertinent.

### Scénario 1.4: Suppression de la session active pendant la saisie

*   **Action Utilisateur Chaotique**:
    1.  Sélectionner une session et commencer à taper un long message dans la zone de saisie, sans l'envoyer.
    2.  Pendant que le texte est dans l'input, trouver un moyen de supprimer la session active (via un menu contextuel, un bouton, etc.).

*   **Comportement Attendu**:
    *   L'application ne doit pas crasher.
    *   Après la suppression, l'interface doit basculer vers un état stable : soit en sélectionnant une autre session (la précédente ou la suivante dans la liste), soit en affichant l'interface de "nouvelle session".
    *   Le texte non envoyé dans la zone de saisie doit être effacé pour éviter de l'envoyer accidentellement dans une autre session. Un message d'avertissement demandant confirmation avant de supprimer une session contenant un brouillon serait un plus.

*   **Bug Potentiel à Détecter**:
    *   **Crash de l'application**: L'application tente d'accéder à des propriétés de la session active qui vient d'être supprimée (`currentSessionId` pointant vers un objet inexistant).
    *   **État "fantôme"**: L'interface de chat reste affichée pour la session supprimée, permettant potentiellement d'essayer d'envoyer des messages vers une session qui n'existe plus, ce qui pourrait causer des erreurs backend.
    *   **Fuite de brouillon**: Après la suppression et le basculement vers une nouvelle session, le texte qui était en cours de saisie est toujours présent dans la zone de saisie et pourrait être envoyé par erreur dans la mauvaise conversation.

---

## Zone 2: Interface de Chat (Zone de saisie)

### Scénario 2.1: Saisie de contenu volumineux ("Input Overload")

*   **Action Utilisateur Chaotique**:
    1.  Copier/coller un texte très long (ex: 10 000+ mots) dans le `textarea`.
    2.  Taper du texte en continu sans s'arrêter pendant une longue période.
    3.  Tenter d'envoyer ce message volumineux.

*   **Comportement Attendu**:
    *   Le `textarea` doit gérer le texte sans geler, en utilisant sa barre de défilement (`max-h-[150px]`).
    *   L'application ne doit pas ralentir de manière significative pendant la saisie.
    *   Lors de l'envoi, l'application doit soit tronquer le message avec un avertissement, soit le refuser poliment, soit le gérer sans planter. Le backend doit avoir une limite de taille de payload.

*   **Bug Potentiel à Détecter**:
    *   **Gel de l'UI**: Le thread principal est bloqué par le traitement du texte, rendant l'interface non réactive.
    *   **Crash du Frontend/Backend**: La taille du message dépasse les limites de mémoire allouées ou les limites de payload IPC de Tauri.
    *   **Latence extrême**: L'envoi prend un temps déraisonnable, sans feedback pour l'utilisateur.

### Scénario 2.2: Caractères spéciaux et injections

*   **Action Utilisateur Chaotique**:
    1.  Copier/coller des chaînes de caractères complexes : emojis en grand nombre, caractères Unicode rares (Zalgo text), scripts de droite à gauche (arabe, hébreu).
    2.  Tenter d'injecter des séquences de formatage : `\n`, `\t`, et des extraits de Markdown ou HTML (`<script>`, `![img]()`).
    3.  Envoyer des messages contenant uniquement des espaces ou des retours à la ligne ( contournant le `trim()` avec des caractères d'espacement non standards).

*   **Comportement Attendu**:
    *   L'affichage dans le `textarea` et dans la bulle de message doit être correct, sans casser la mise en page.
    *   Toute tentative d'injection de code (HTML/JS) doit être neutralisée (`sanitized`) avant l'affichage. Le texte doit apparaître tel quel, sans être interprété.
    *   La logique `!input.trim()` doit résister aux espaces non standards.

*   **Bug Potentiel à Détecter**:
    *   **UI/Layout Corruption**: Le texte injecté casse le style CSS, fait déborder les bulles de chat ou perturbe la mise en page globale.
    *   **Faille XSS (Cross-Site Scripting)**: Si le contenu est un jour rendu dans un contexte web, une injection réussie pourrait exécuter du code arbitraire (peu probable avec Tauri, mais bon à tester).
    *   **Contournement de la validation**: Un message vide est envoyé, créant une bulle de chat vide ou provoquant une erreur côté backend.

---

## Zone 3: Fenêtre Principale et Environnement

### Scénario 3.1: Redimensionnement frénétique et minimaliste

*   **Action Utilisateur Chaotique**:
    1.  Redimensionner la fenêtre de l'application très rapidement et de manière répétée.
    2.  Réduire la fenêtre à sa taille minimale absolue autorisée par le système d'exploitation.
    3.  Passer rapidement du mode fenêtré au mode plein écran et vice-versa.

*   **Comportement Attendu**:
    *   Le layout doit s'adapter de manière fluide sans clignotements excessifs ou artefacts graphiques.
    *   À la taille minimale, l'interface doit rester utilisable, quitte à masquer des éléments non essentiels ou à utiliser des barres de défilement. Le contenu ne doit pas déborder de manière illisible.
    *   Aucun crash dû à des erreurs de calcul de layout.

*   **Bug Potentiel à Détecter**:
    *   **Crash de l'application**: Le moteur de rendu (WebView) ne parvient pas à gérer les cycles de redessinage rapides.
    *   **État de layout corrompu**: Des éléments se superposent, disparaissent ou sont mal positionnés de façon permanente après le redimensionnement.
    *   **Problèmes de performance**: L'utilisation du CPU monte en flèche pendant le redimensionnement.

### Scénario 3.2: Perte et récupération du focus / de la connexion

*   **Action Utilisateur Chaotique**:
    1.  Pendant que l'application effectue une opération (ex: envoi d'un message, création de session), changer rapidement de bureau virtuel, minimiser la fenêtre ou donner le focus à une autre application.
    2.  Désactiver la connexion réseau (Wi-Fi/Ethernet) pendant que le LLM est en train de "penser" ou de streamer une réponse.
    3.  Réactiver la connexion réseau après quelques secondes.

*   **Comportement Attendu**:
    *   L'opération en arrière-plan doit se terminer correctement.
    *   L'UI doit gérer la perte de connexion de manière gracieuse, en affichant un message d'erreur clair (ex: "Connexion perdue") et en arrêtant l'indicateur "Thinking...".
    *   À la récupération de la connexion, l'application doit retrouver un état stable, permettant à l'utilisateur de retenter l'action.

*   **Bug Potentiel à Détecter**:
    *   **État de chargement infini**: L'indicateur "Thinking..." reste actif indéfiniment après la perte de connexion.
    *   **Crash ou état invalide**: L'application ne gère pas l'erreur réseau et se retrouve dans un état incohérent.
    *   **Comportement inattendu à la récupération**: L'application tente de poursuivre une opération interrompue, provoquant des erreurs ou des données corrompues.
