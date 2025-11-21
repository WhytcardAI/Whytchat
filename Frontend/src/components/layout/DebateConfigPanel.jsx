import { useState } from "react";
import { X, Play, Swords } from "lucide-react";
import useAppStore from "../../lib/store";

export default function DebateConfigPanel({ onClose }) {
  const { startDebate } = useAppStore();

  const [topic, setTopic] = useState("");
  const [rounds, setRounds] = useState(5);

  const [agentA, setAgentA] = useState({
    name: "IA1 (Déforestation)",
    prompt:
      "Tu es un expert environnementaliste radical. Tu défends la préservation absolue des forêts primaires. Tu utilises des données scientifiques alarmantes et un ton passionné mais factuel.",
  });

  const [agentB, setAgentB] = useState({
    name: "IA2 (Agro-industrie)",
    prompt:
      "Tu es un représentant pragmatique de l'industrie agro-alimentaire. Tu défends la nécessité de nourrir une population croissante et le développement économique. Tu proposes des solutions technologiques et une intensification durable.",
  });

  const handleStart = () => {
    if (!topic.trim() || !agentA.name.trim() || !agentB.name.trim()) return;
    startDebate(topic, agentA, agentB, rounds);
    onClose();
  };

  return (
    <div className="absolute inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm p-4 animate-in fade-in duration-200">
      <div className="w-full max-w-4xl bg-[#0f1115] border border-white/10 rounded-2xl shadow-2xl flex flex-col max-h-[90vh] overflow-hidden">
        {/* Header */}
        <div className="flex items-center justify-between p-6 border-b border-white/5 bg-white/5">
          <div className="flex items-center gap-3">
            <div className="p-2 bg-orange-500/10 rounded-lg">
              <Swords className="w-6 h-6 text-orange-500" />
            </div>
            <div>
              <h2 className="text-lg font-semibold text-white">Mode Débat Autonome</h2>
              <p className="text-sm text-muted-foreground">
                Configurez deux entités IA pour débattre d&apos;un sujet complexe.
              </p>
            </div>
          </div>
          <button
            onClick={onClose}
            className="p-2 hover:bg-white/10 rounded-full transition-colors"
          >
            <X className="w-5 h-5 text-muted-foreground" />
          </button>
        </div>

        {/* Content */}
        <div className="flex-1 overflow-y-auto p-6 space-y-8">
          {/* Sujet */}
          <div className="space-y-3">
            <label className="text-xs font-bold uppercase tracking-wider text-muted-foreground">
              Sujet du Débat
            </label>
            <input
              type="text"
              value={topic}
              onChange={(e) => setTopic(e.target.value)}
              placeholder="Ex: L'impact de la culture du soja sur la forêt amazonienne..."
              className="w-full bg-black/20 border border-white/10 rounded-xl p-4 text-white placeholder:text-white/20 focus:border-orange-500/50 focus:ring-1 focus:ring-orange-500/50 outline-none transition-all"
            />
          </div>

          {/* Agents Grid */}
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            {/* Agent A */}
            <div className="space-y-4 p-5 rounded-2xl bg-blue-500/5 border border-blue-500/10">
              <div className="flex items-center gap-2 mb-2">
                <div className="w-2 h-2 rounded-full bg-blue-500"></div>
                <h3 className="font-medium text-blue-400">Entité A (Thèse)</h3>
              </div>

              <div className="space-y-2">
                <label className="text-[10px] uppercase font-bold text-muted-foreground">Nom</label>
                <input
                  type="text"
                  value={agentA.name}
                  onChange={(e) => setAgentA({ ...agentA, name: e.target.value })}
                  className="w-full bg-black/20 border border-white/10 rounded-lg p-2.5 text-sm text-white focus:border-blue-500/50 outline-none"
                />
              </div>

              <div className="space-y-2">
                <label className="text-[10px] uppercase font-bold text-muted-foreground">
                  Prompt Système (Personnalité & Objectifs)
                </label>
                <textarea
                  value={agentA.prompt}
                  onChange={(e) => setAgentA({ ...agentA, prompt: e.target.value })}
                  className="w-full h-32 bg-black/20 border border-white/10 rounded-lg p-3 text-sm text-white/90 focus:border-blue-500/50 outline-none resize-none leading-relaxed"
                />
              </div>
            </div>

            {/* Agent B */}
            <div className="space-y-4 p-5 rounded-2xl bg-red-500/5 border border-red-500/10">
              <div className="flex items-center gap-2 mb-2">
                <div className="w-2 h-2 rounded-full bg-red-500"></div>
                <h3 className="font-medium text-red-400">Entité B (Antithèse)</h3>
              </div>

              <div className="space-y-2">
                <label className="text-[10px] uppercase font-bold text-muted-foreground">Nom</label>
                <input
                  type="text"
                  value={agentB.name}
                  onChange={(e) => setAgentB({ ...agentB, name: e.target.value })}
                  className="w-full bg-black/20 border border-white/10 rounded-lg p-2.5 text-sm text-white focus:border-red-500/50 outline-none"
                />
              </div>

              <div className="space-y-2">
                <label className="text-[10px] uppercase font-bold text-muted-foreground">
                  Prompt Système (Personnalité & Objectifs)
                </label>
                <textarea
                  value={agentB.prompt}
                  onChange={(e) => setAgentB({ ...agentB, prompt: e.target.value })}
                  className="w-full h-32 bg-black/20 border border-white/10 rounded-lg p-3 text-sm text-white/90 focus:border-red-500/50 outline-none resize-none leading-relaxed"
                />
              </div>
            </div>
          </div>

          {/* Settings */}
          <div className="flex items-center gap-4 p-4 rounded-xl bg-white/5 border border-white/5">
            <div className="flex-1">
              <label className="text-sm font-medium text-white">
                Nombre de tours d&apos;échange
              </label>
              <p className="text-xs text-muted-foreground">
                Combien de fois chaque entité prendra la parole.
              </p>
            </div>
            <input
              type="number"
              min="1"
              max="20"
              value={rounds}
              onChange={(e) => setRounds(parseInt(e.target.value))}
              className="w-20 bg-black/20 border border-white/10 rounded-lg p-2 text-center text-white outline-none focus:border-orange-500/50"
            />
          </div>
        </div>

        {/* Footer */}
        <div className="p-6 border-t border-white/5 bg-white/5 flex justify-end gap-3">
          <button
            onClick={onClose}
            className="px-6 py-3 rounded-xl text-sm font-medium text-muted-foreground hover:text-white hover:bg-white/5 transition-colors"
          >
            Annuler
          </button>
          <button
            onClick={handleStart}
            disabled={!topic.trim()}
            className="px-8 py-3 rounded-xl bg-gradient-to-r from-orange-500 to-red-600 text-white font-medium shadow-lg shadow-orange-500/20 hover:shadow-orange-500/40 hover:scale-[1.02] active:scale-[0.98] transition-all flex items-center gap-2 disabled:opacity-50 disabled:cursor-not-allowed"
          >
            <Play className="w-4 h-4 fill-current" />
            Lancer le Débat
          </button>
        </div>
      </div>
    </div>
  );
}
