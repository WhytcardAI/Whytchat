import { useState, useEffect } from 'react';
import { Shield, ArrowRight, Terminal, Server, Brain, Database, Cpu } from 'lucide-react';
import { useAppStore } from '../../store/appStore';
import { cn } from '../../lib/utils';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { useTranslation } from 'react-i18next';
import { TestConsole } from '../diagnostics/TestConsole';

const TIPS = [
  'onboarding.model.education.local_vs_cloud',
  'onboarding.model.education.privacy',
  'onboarding.model.education.performance'
];

export function OnboardingWizard() {
  const { t, i18n } = useTranslation('common');
  const [step, setStep] = useState(1);
  const [selectedLanguage, setSelectedLanguage] = useState(i18n.language || 'en');
  const [downloadProgress, setDownloadProgress] = useState(0);
  const [downloadStatus, setDownloadStatus] = useState('waiting'); // waiting, downloading, complete, error
  const [diagnosticsComplete, setDiagnosticsComplete] = useState(false);
  const [diagnosticsPassed, setDiagnosticsPassed] = useState(false);
  const [statusLogs, setStatusLogs] = useState([]);
  const [currentStep, setCurrentStep] = useState('init');
  const { completeOnboarding } = useAppStore();

  const [tipIndex, setTipIndex] = useState(0);

  useEffect(() => {
    if (step === 3 && downloadStatus === 'downloading') {
      const interval = setInterval(() => {
        setTipIndex((prev) => (prev + 1) % TIPS.length);
      }, 5000);
      return () => clearInterval(interval);
    }
  }, [step, downloadStatus]);

  const handleLanguageSelect = (lang) => {
    setSelectedLanguage(lang);
    i18n.changeLanguage(lang);
  };

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
    setStatusLogs([]);

    // Listen for progress events
    const unlistenProgress = await listen('download-progress', (event) => {
      setDownloadProgress(event.payload);
      if (event.payload >= 100) {
        setDownloadStatus('complete');
      }
    });

    // Listen for detailed status events
    const unlistenStatus = await listen('download-status', (event) => {
      const { step, detail } = event.payload;
      setCurrentStep(step);
      setStatusLogs(prev => {
        // Keep last 10 logs for performance
        const newLogs = [...prev, { step, detail, time: new Date().toLocaleTimeString() }];
        return newLogs.slice(-10);
      });
    });

    try {
      await invoke('download_model');
      // Only mark as complete AFTER download_model returns successfully
      // This ensures the file is fully written to disk
      const modelExists = await invoke('check_model_exists');
      if (modelExists) {
        // Move to diagnostics step instead of completing onboarding
        setStep(4);
      } else {
        console.error('Download completed but model check failed');
        setDownloadStatus('error');
      }
    } catch (error) {
      console.error("Download failed:", error);
      setDownloadStatus('error');
      setStatusLogs(prev => [...prev, { step: 'error', detail: error.toString(), time: new Date().toLocaleTimeString() }]);
    } finally {
      unlistenProgress();
      unlistenStatus();
    }
  };

  const handleDiagnosticsComplete = (results) => {
    setDiagnosticsComplete(true);
    setDiagnosticsPassed(results.failed === 0);
  };

  const handleFinishOnboarding = () => {
    completeOnboarding();
  };

  return (
    <div className="fixed inset-0 bg-background flex items-center justify-center z-50">
      <div className="w-full max-w-4xl h-[600px] bg-surface rounded-3xl shadow-2xl border border-border flex overflow-hidden">

        {/* Left Side: Visual / Context */}
        <div className="w-1/3 bg-primary/5 p-8 flex flex-col justify-between border-r border-border">
          <div>
            <img src="/logo.png" alt="WhytChat Logo" className="w-10 h-10 rounded-xl mb-6 object-contain" />
            <h2 className="text-2xl font-bold text-text mb-2">
              {step === 1 && t('onboarding.language.title')}
              {step === 2 && t('onboarding.welcome.title')}
              {step === 3 && t('onboarding.model.title')}
              {step === 4 && t('onboarding.diagnostics.sideTitle', 'Verification')}
            </h2>
            <p className="text-muted text-sm leading-relaxed">
              {step === 1 && t('onboarding.language.subtitle')}
              {step === 2 && t('onboarding.welcome.subtitle')}
              {step === 3 && t('onboarding.model.subtitle')}
              {step === 4 && t('onboarding.diagnostics.sideSubtitle', 'Running system diagnostics to ensure everything is ready.')}
            </p>
          </div>

          {/* Steps Indicator */}
          <div className="flex gap-2">
            {[1, 2, 3, 4].map(i => (
              <div key={i} className={cn("h-1.5 rounded-full transition-all duration-500", step >= i ? "w-8 bg-primary" : "w-2 bg-border")} />
            ))}
          </div>
        </div>

        {/* Right Side: Actions */}
        <div className="flex-1 p-12 flex flex-col">

          {step === 1 && (
            <div className="flex-1 flex flex-col items-center justify-center text-center space-y-8 animate-in fade-in slide-in-from-right-4 duration-500">
              <h3 className="text-xl font-semibold text-text">{t('onboarding.language.title')}</h3>
              <div className="space-y-4">
                <button
                  onClick={() => handleLanguageSelect('en')}
                  className={cn(
                    "w-full p-4 rounded-xl border-2 transition-all",
                    selectedLanguage === 'en' ? "border-primary bg-primary/5" : "border-border hover:border-primary/30"
                  )}
                >
                  <div className="font-bold text-text">{t('onboarding.language.english')}</div>
                </button>
                <button
                  onClick={() => handleLanguageSelect('fr')}
                  className={cn(
                    "w-full p-4 rounded-xl border-2 transition-all",
                    selectedLanguage === 'fr' ? "border-primary bg-primary/5" : "border-border hover:border-primary/30"
                  )}
                >
                  <div className="font-bold text-text">{t('onboarding.language.french')}</div>
                </button>
              </div>
              <button onClick={handleNext} className="btn-primary mt-8">
                {t('onboarding.welcome.start')} <ArrowRight size={18} className="ml-2" />
              </button>
            </div>
          )}

          {step === 2 && (
            <div className="flex-1 flex flex-col items-center justify-center text-center space-y-8 animate-in fade-in slide-in-from-right-4 duration-500">
              <div className="w-24 h-24 bg-green-100 text-green-600 rounded-full flex items-center justify-center mb-4">
                <Shield size={48} />
              </div>
              <h3 className="text-xl font-semibold text-text">{t('onboarding.welcome.privacy.title')}</h3>
              <p className="text-muted max-w-md">
                {t('onboarding.welcome.privacy.description')}
              </p>
              <button onClick={handleNext} className="btn-primary mt-8">
                {t('onboarding.welcome.start')} <ArrowRight size={18} className="ml-2" />
              </button>
            </div>
          )}

          {step === 3 && (
            <div className="flex-1 flex flex-col animate-in fade-in slide-in-from-right-4 duration-500 overflow-hidden">
              <div className="w-full max-w-lg mx-auto space-y-4">
                <div className="flex justify-between text-sm font-medium text-text mb-2">
                  <span>{t('onboarding.model.downloading')}</span>
                  <span>{Math.round(downloadProgress)}%</span>
                </div>

                {/* Progress Bar */}
                <div className="h-3 bg-border rounded-full overflow-hidden">
                  <div
                    className="h-full bg-gradient-to-r from-primary to-green-500 transition-all duration-200 ease-out"
                    style={{ width: `${downloadProgress}%` }}
                  />
                </div>

                {/* Backend Console - Shows what's happening */}
                <div className="mt-4 bg-gray-900 rounded-lg border border-gray-700 overflow-hidden">
                  <div className="flex items-center gap-2 px-3 py-2 bg-gray-800 border-b border-gray-700">
                    <Terminal size={14} className="text-green-400" />
                    <span className="text-xs font-mono text-gray-400">
                      {currentStep !== 'init' ? `Backend Console — ${currentStep}` : 'Backend Console'}
                    </span>
                    <div className={cn(
                      "ml-auto w-2 h-2 rounded-full",
                      downloadStatus === 'downloading' ? "bg-green-500 animate-pulse" :
                      downloadStatus === 'complete' ? "bg-green-500" : "bg-gray-500"
                    )} />
                  </div>
                  <div className="p-3 h-32 overflow-y-auto font-mono text-xs space-y-1">
                    {statusLogs.length === 0 ? (
                      <div className="text-gray-500">Initializing...</div>
                    ) : (
                      statusLogs.map((log, i) => (
                        <div key={i} className={cn(
                          "flex gap-2",
                          log.step === 'error' ? "text-red-400" :
                          log.detail.startsWith('✓') ? "text-green-400" :
                          log.detail.startsWith('⚠') ? "text-yellow-400" :
                          "text-gray-300"
                        )}>
                          <span className="text-gray-600">[{log.time}]</span>
                          <span>{log.detail}</span>
                        </div>
                      ))
                    )}
                  </div>
                </div>

                {/* Component Status Grid */}
                <div className="grid grid-cols-3 gap-2 mt-4">
                  <ComponentStatus
                    icon={<Server size={16} />}
                    label="Inference Server"
                    status={downloadProgress >= 20 ? 'done' : downloadProgress > 0 ? 'loading' : 'pending'}
                  />
                  <ComponentStatus
                    icon={<Brain size={16} />}
                    label="AI Model (GGUF)"
                    status={downloadProgress >= 80 ? 'done' : downloadProgress >= 20 ? 'loading' : 'pending'}
                  />
                  <ComponentStatus
                    icon={<Database size={16} />}
                    label="Embeddings (ONNX)"
                    status={downloadProgress >= 95 ? 'done' : downloadProgress >= 80 ? 'loading' : 'pending'}
                  />
                </div>

                {/* Educational Tips */}
                <div className="mt-4 min-h-[60px] flex items-center justify-center p-3 bg-primary/5 rounded-lg border border-primary/10">
                  <p key={tipIndex} className="text-xs text-center text-muted animate-in fade-in slide-in-from-bottom-2 duration-500">
                    {t(TIPS[tipIndex])}
                  </p>
                </div>

                {/* Architecture Info */}
                <div className="bg-surface border border-border p-3 rounded-lg space-y-2 text-xs">
                  <div className="flex items-center gap-2 text-muted">
                    <Cpu size={12} />
                    <span>Installing to: <code className="text-primary">{"<install_path>/data/"}</code></span>
                  </div>
                  <div className="text-muted opacity-75">
                    • llama-server.exe → Inference engine (llama.cpp)<br/>
                    • default-model.gguf → Qwen2.5-Coder 7B quantized<br/>
                    • embeddings/ → all-MiniLM-L6-v2 for RAG search
                  </div>
                </div>
              </div>
            </div>
          )}

          {step === 4 && (
            <div className="flex-1 flex flex-col animate-in fade-in slide-in-from-right-4 duration-500">
              <div className="mb-4">
                <h3 className="text-xl font-semibold text-text">{t('onboarding.diagnostics.title', 'System Diagnostics')}</h3>
                <p className="text-muted text-sm mt-1">
                  {t('onboarding.diagnostics.subtitle', 'Verifying all components are working correctly...')}
                </p>
              </div>

              <TestConsole
                autoStart={true}
                onComplete={handleDiagnosticsComplete}
                className="flex-1"
              />

              {diagnosticsComplete && (
                <div className="mt-4 flex justify-end">
                  <button
                    onClick={handleFinishOnboarding}
                    className={cn(
                      "btn-primary",
                      !diagnosticsPassed && "bg-yellow-600 hover:bg-yellow-500"
                    )}
                  >
                    {diagnosticsPassed
                      ? t('onboarding.diagnostics.finish', 'Start Using WhytChat')
                      : t('onboarding.diagnostics.continueAnyway', 'Continue Anyway')
                    }
                    <ArrowRight size={18} className="ml-2" />
                  </button>
                </div>
              )}
            </div>
          )}

        </div>
      </div>
    </div>
  );
}

function ComponentStatus({ icon, label, status }) {
  return (
    <div className={cn(
      "flex flex-col items-center p-2 rounded-lg border text-center",
      status === 'done' ? "bg-green-500/10 border-green-500/30 text-green-400" :
      status === 'loading' ? "bg-primary/10 border-primary/30 text-primary animate-pulse" :
      "bg-border/30 border-border text-muted"
    )}>
      <div className="mb-1">{icon}</div>
      <span className="text-xs font-medium">{label}</span>
      <span className="text-[10px] opacity-75">
        {status === 'done' ? '✓ Ready' : status === 'loading' ? 'Installing...' : 'Pending'}
      </span>
    </div>
  );
}
