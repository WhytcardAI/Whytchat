import { useState } from 'react';
import { Shield, Download, CheckCircle, ArrowRight, Zap, Brain } from 'lucide-react';
import { useAppStore } from '../../store/appStore';
import { cn } from '../../lib/utils';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

export function OnboardingWizard() {
  const [step, setStep] = useState(1);
  const [selectedModel, setSelectedModel] = useState('qwen2.5-7b');
  const [downloadProgress, setDownloadProgress] = useState(0);
  const [downloadStatus, setDownloadStatus] = useState('waiting'); // waiting, downloading, complete, error
  const { completeOnboarding } = useAppStore();

  const handleNext = () => {
    if (step === 2) {
      setStep(3);
      startDownload();
    } else {
      setStep(step + 1);
    }
  };

  const startDownload = async () => {
    setDownloadStatus('downloading');

    // Listen for progress events
    const unlisten = await listen('download-progress', (event) => {
      setDownloadProgress(event.payload);
      if (event.payload >= 100) {
        setDownloadStatus('complete');
        setTimeout(() => {
          completeOnboarding();
          unlisten();
        }, 1500);
      }
    });

    try {
      await invoke('download_model');
    } catch (error) {
      console.error("Download failed:", error);
      setDownloadStatus('error');
      unlisten();
    }
  };

  return (
    <div className="fixed inset-0 bg-background flex items-center justify-center z-50">
      <div className="w-full max-w-4xl h-[600px] bg-surface rounded-3xl shadow-2xl border border-border flex overflow-hidden">

        {/* Left Side: Visual / Context */}
        <div className="w-1/3 bg-primary/5 p-8 flex flex-col justify-between border-r border-border">
          <div>
            <div className="w-10 h-10 bg-primary rounded-xl flex items-center justify-center text-white font-bold mb-6">
              W
            </div>
            <h2 className="text-2xl font-bold text-text mb-2">
              {step === 1 && "Bienvenue"}
              {step === 2 && "Le Cerveau"}
              {step === 3 && "Installation"}
            </h2>
            <p className="text-muted text-sm leading-relaxed">
              {step === 1 && "WhytChat est une IA locale et privée. Vos données ne quittent jamais votre ordinateur."}
              {step === 2 && "Choisissez le modèle d'IA qui propulsera votre assistant. Ce choix détermine l'intelligence et la rapidité."}
              {step === 3 && "Nous téléchargeons et configurons votre modèle localement. Cela peut prendre quelques minutes."}
            </p>
          </div>

          {/* Steps Indicator */}
          <div className="flex gap-2">
            {[1, 2, 3].map(i => (
              <div key={i} className={cn("h-1.5 rounded-full transition-all duration-500", step >= i ? "w-8 bg-primary" : "w-2 bg-border")} />
            ))}
          </div>
        </div>

        {/* Right Side: Actions */}
        <div className="flex-1 p-12 flex flex-col">

          {step === 1 && (
            <div className="flex-1 flex flex-col items-center justify-center text-center space-y-8 animate-in fade-in slide-in-from-right-4 duration-500">
              <div className="w-24 h-24 bg-green-100 text-green-600 rounded-full flex items-center justify-center mb-4">
                <Shield size={48} />
              </div>
              <h3 className="text-xl font-semibold text-text">Confidentialité Totale</h3>
              <p className="text-muted max-w-md">
                Contrairement aux IA en ligne, WhytChat tourne 100% sur votre machine.
                Aucun log, aucun tracking, aucune fuite de données.
              </p>
              <button onClick={handleNext} className="btn-primary mt-8">
                Commencer la configuration <ArrowRight size={18} className="ml-2" />
              </button>
            </div>
          )}

          {step === 2 && (
            <div className="flex-1 flex flex-col animate-in fade-in slide-in-from-right-4 duration-500">
              <h3 className="text-lg font-semibold mb-6">Sélectionnez votre modèle</h3>

              <div className="grid grid-cols-1 gap-4 mb-8">
                {/* Option 1: Qwen */}
                <ModelCard
                  id="qwen2.5-7b"
                  selected={selectedModel === 'qwen2.5-7b'}
                  onSelect={setSelectedModel}
                  title="Qwen 2.5 (7B)"
                  badge="Recommandé"
                  desc="Le meilleur équilibre entre vitesse et intelligence. Idéal pour la plupart des PC."
                  specs={["Rapide", "4GB RAM", "Polyvalent"]}
                  icon={<Zap size={24} />}
                />

                {/* Option 2: Mistral */}
                <ModelCard
                  id="mistral-nemo-12b"
                  selected={selectedModel === 'mistral-nemo-12b'}
                  onSelect={setSelectedModel}
                  title="Mistral Nemo (12B)"
                  badge="Expert"
                  desc="Modèle français plus puissant. Meilleur raisonnement, mais demande plus de ressources."
                  specs={["Très Intelligent", "8GB RAM", "Expert FR"]}
                  icon={<Brain size={24} />}
                />
              </div>

              <div className="mt-auto flex justify-end">
                <button onClick={handleNext} className="btn-primary">
                  Installer {selectedModel === 'qwen2.5-7b' ? 'Qwen' : 'Mistral'} <Download size={18} className="ml-2" />
                </button>
              </div>
            </div>
          )}

          {step === 3 && (
            <div className="flex-1 flex flex-col items-center justify-center animate-in fade-in slide-in-from-right-4 duration-500">
              <div className="w-full max-w-md space-y-6">
                <div className="flex justify-between text-sm font-medium text-text mb-2">
                  <span>Téléchargement de {selectedModel}...</span>
                  <span>{Math.round(downloadProgress)}%</span>
                </div>

                {/* Progress Bar */}
                <div className="h-4 bg-border rounded-full overflow-hidden">
                  <div
                    className="h-full bg-primary transition-all duration-200 ease-out"
                    style={{ width: `${downloadProgress}%` }}
                  />
                </div>

                <p className="text-center text-xs text-muted">
                  {downloadStatus === 'downloading' && "Téléchargement en cours... Ne fermez pas l'application."}
                  {downloadStatus === 'complete' && "Installation terminée !"}
                  {downloadStatus === 'error' && "Erreur lors du téléchargement. Vérifiez votre connexion."}
                </p>

                <div className="bg-surface border border-border p-4 rounded-lg mt-8 space-y-3">
                  <StepItem label="Initialisation du moteur" done={downloadProgress > 0} />
                  <StepItem label="Téléchargement des poids (GGUF)" done={downloadProgress > 10} />
                  <StepItem label="Vérification de l'intégrité" done={downloadProgress > 90} />
                  <StepItem label="Chargement en mémoire" done={downloadProgress === 100} />
                </div>
              </div>
            </div>
          )}

        </div>
      </div>
    </div>
  );
}

function ModelCard({ id, selected, onSelect, title, badge, desc, specs, icon }) {
  return (
    <div
      onClick={() => onSelect(id)}
      className={cn(
        "relative p-4 rounded-xl border-2 cursor-pointer transition-all hover:shadow-md flex gap-4",
        selected ? "border-primary bg-primary/5" : "border-border bg-surface hover:border-primary/30"
      )}
    >
      <div className={cn("w-12 h-12 rounded-lg flex items-center justify-center shrink-0", selected ? "bg-primary text-white" : "bg-border text-muted")}>
        {icon}
      </div>
      <div className="flex-1">
        <div className="flex justify-between items-start mb-1">
          <h4 className="font-bold text-text">{title}</h4>
          {badge && <span className="text-[10px] font-bold uppercase tracking-wider bg-background px-2 py-1 rounded text-primary border border-primary/20">{badge}</span>}
        </div>
        <p className="text-sm text-muted mb-3">{desc}</p>
        <div className="flex gap-2">
          {specs.map((spec, i) => (
            <span key={i} className="text-xs bg-background px-2 py-1 rounded text-text border border-border">{spec}</span>
          ))}
        </div>
      </div>
      {selected && (
        <div className="absolute top-4 right-4 text-primary">
          <CheckCircle size={20} fill="currentColor" className="text-white" />
        </div>
      )}
    </div>
  );
}

function StepItem({ label, done }) {
  return (
    <div className="flex items-center gap-3 text-sm">
      <div className={cn("w-5 h-5 rounded-full flex items-center justify-center border", done ? "bg-green-500 border-green-500 text-white" : "border-border text-transparent")}>
        <CheckCircle size={12} />
      </div>
      <span className={cn(done ? "text-text" : "text-muted")}>{label}</span>
    </div>
  );
}
