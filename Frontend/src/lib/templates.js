export const PROMPT_TEMPLATES = [
  {
    id: "explain-code",
    label: "Explique-moi ce code",
    description: "Analyse détaillée du code fourni",
    content:
      "Explique-moi ce code en détail, ligne par ligne si nécessaire, et identifie les potentiels problèmes ou améliorations :",
    category: "coding",
  },
  {
    id: "summarize",
    label: "Résume ce texte",
    description: "Synthèse concise du contenu",
    content:
      "Fais un résumé concis et structuré du texte suivant, en mettant en avant les points clés :",
    category: "writing",
  },
  {
    id: "debate",
    label: "Débat : [Sujet]",
    description: "Lancer un débat sur un sujet",
    content:
      'Je souhaite lancer un débat sur le sujet suivant : "[Sujet]". Présente les arguments pour et contre, puis propose une synthèse nuancée.',
    category: "discussion",
  },
  {
    id: "refactor",
    label: "Refactorisation",
    description: "Améliorer la qualité du code",
    content:
      "Propose une refactorisation de ce code pour le rendre plus propre, plus performant et plus maintenable, en suivant les bonnes pratiques (SOLID, DRY) :",
    category: "coding",
  },
  {
    id: "unit-tests",
    label: "Générer des tests unitaires",
    description: "Créer des tests pour le code",
    content:
      "Écris des tests unitaires complets pour le code suivant, en couvrant les cas nominaux et les cas limites. Utilise le framework de test approprié pour le langage :",
    category: "coding",
  },
  {
    id: "translate-en",
    label: "Traduire en Anglais",
    description: "Traduction professionnelle",
    content: "Traduis le texte suivant en anglais professionnel et naturel :",
    category: "writing",
  },
  {
    id: "translate-fr",
    label: "Traduire en Français",
    description: "Traduction professionnelle",
    content: "Traduis le texte suivant en français professionnel et naturel :",
    category: "writing",
  },
  {
    id: "email-pro",
    label: "Email Professionnel",
    description: "Rédiger un email formel",
    content:
      "Rédige un email professionnel pour [Destinataire] concernant [Sujet]. Le ton doit être courtois, clair et concis.",
    category: "writing",
  },
];
