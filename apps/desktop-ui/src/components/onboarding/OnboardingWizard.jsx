import { useState, useEffect } from 'react';
import { Shield, CheckCircle, ArrowRight } from 'lucide-react';
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

    // Listen for progress events
    const unlisten = await listen('download-progress', (event) => {
      setDownloadProgress(event.payload);
      if (event.payload >= 100) {
        setDownloadStatus('complete');
      }
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
    } finally {
      unlisten();
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
            <div className="flex-1 flex flex-col items-center justify-center animate-in fade-in slide-in-from-right-4 duration-500">
              <div className="w-full max-w-md space-y-6">
                <div className="flex justify-between text-sm font-medium text-text mb-2">
                  <span>{t('onboarding.model.downloading')}</span>
                  <span>{Math.round(downloadProgress)}%</span>
                </div>

                {/* Progress Bar */}
                <div className="h-4 bg-border rounded-full overflow-hidden">
                  <div
                    className="h-full bg-primary transition-all duration-200 ease-out"
                    style={{ width: `${downloadProgress}%` }}
                  />
                </div>

                <div className="mt-6 min-h-[80px] flex items-center justify-center p-4 bg-primary/5 rounded-lg border border-primary/10">
                  <p key={tipIndex} className="text-sm text-center text-muted animate-in fade-in slide-in-from-bottom-2 duration-500">
                    {t(TIPS[tipIndex])}
                  </p>
                </div>

                <p className="text-center text-xs text-muted">
                  {downloadStatus === 'downloading' && t('onboarding.model.downloading')}
                  {downloadStatus === 'complete' && t('onboarding.model.complete')}
                  {downloadStatus === 'error' && t('onboarding.model.error')}
                </p>

                <div className="bg-surface border border-border p-4 rounded-lg mt-8 space-y-3">
                  <StepItem label={t('onboarding.model.steps.init')} done={downloadProgress > 0} />
                  <StepItem label={t('onboarding.model.steps.download')} done={downloadProgress >= 20} />
                  <StepItem label={t('onboarding.model.steps.verify')} done={downloadProgress >= 80} />
                  <StepItem label={t('onboarding.model.steps.embeddings')} done={downloadProgress >= 95} />
                  <StepItem label={t('onboarding.model.steps.load')} done={downloadProgress === 100} />
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

function StepItem({ label, done }) {
  return (
    <div className="flex items-center gap-3 text-sm">
      <div className={cn("w-5 h-5 rounded-full flex items-center justify-center border", done ? "bg-green-500 border-green-500 text-primary-foreground" : "border-border text-transparent")}>
        <CheckCircle size={12} />
      </div>
      <span className={cn(done ? "text-text" : "text-muted")}>{label}</span>
    </div>
  );
}
