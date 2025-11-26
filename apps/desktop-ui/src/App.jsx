import { MainLayout } from './components/layout/MainLayout';
import { ChatInterface } from './components/chat/ChatInterface';
import { Dashboard } from './components/dashboard/Dashboard';
import { OnboardingWizard } from './components/onboarding/OnboardingWizard';
import { PreflightCheck } from './components/preflight/PreflightCheck';
import { TestConsole } from './components/diagnostics/TestConsole';
import { useAppStore } from './store/appStore';
import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { X } from 'lucide-react';
import { logger } from './lib/logger';

function App() {
  const {
    isConfigured,
    initializeApp,
    currentSessionId,
    setIsConfigured,
    isDiagnosticsOpen,
    setDiagnosticsOpen,
    setIsCreatingSession
  } = useAppStore();
  const [preflightState, setPreflightState] = useState('checking'); // 'checking' | 'passed' | 'failed' | 'onboarding'
  const [preflightReport, setPreflightReport] = useState(null);

  useEffect(() => {
    const runPreflight = async () => {
      try {
        logger.system.preflight('checking');
        // Use quick check first for fast feedback
        const report = await invoke('run_quick_preflight_check');
        logger.system.preflight('complete', {
          ready: report.ready_to_start,
          needsOnboarding: report.needs_onboarding
        });
        setPreflightReport(report);

        if (report.needs_onboarding) {
          // Need to download model/server
          logger.system.preflight('needs_onboarding');
          setIsConfigured(false);
          setPreflightState('onboarding');
        } else if (report.ready_to_start) {
          // All good, initialize app
          logger.system.init('initializing');
          await initializeApp();
          logger.system.ready();
          setPreflightState('passed');
        } else {
          // Some checks failed
          logger.system.preflight('failed', report.summary);
          setPreflightState('failed');
        }
      } catch (error) {
        logger.system.error('preflight', error);
        setPreflightReport({
          all_passed: false,
          checks: [{ name: 'preflight', passed: false, message: error.toString() }],
          ready_to_start: false,
          needs_onboarding: true,
          summary: 'Preflight check failed'
        });
        setPreflightState('failed');
      }
    };

    runPreflight();
  }, [initializeApp, setIsConfigured]);

  // Show preflight checking state
  if (preflightState === 'checking') {
    return (
      <div className="h-screen w-screen flex flex-col items-center justify-center bg-background gap-4">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
        <div className="text-muted">Running system checks...</div>
      </div>
    );
  }

  // Show preflight failure
  if (preflightState === 'failed' && preflightReport) {
    return <PreflightCheck report={preflightReport} onRetry={() => window.location.reload()} />;
  }

  // Show onboarding
  if (preflightState === 'onboarding' || !isConfigured) {
    return <OnboardingWizard />;
  }

  const handleNewChat = () => {
    logger.ui.click('Dashboard:NewChat');
    setIsCreatingSession(true);
  };

  return (
    <>
      <MainLayout>
        {currentSessionId ? <ChatInterface /> : <Dashboard onNewChat={handleNewChat} />}
      </MainLayout>

      {/* Diagnostics Overlay */}
      {isDiagnosticsOpen && (
        <div className="fixed inset-0 bg-background/80 backdrop-blur-sm z-50 flex items-center justify-center p-8 animate-in fade-in duration-200">
          <div className="w-full max-w-4xl h-[600px] bg-surface rounded-xl shadow-2xl border border-border flex flex-col relative animate-in zoom-in-95 duration-200">
            <button
              onClick={() => {
                logger.navigation.closeModal('Diagnostics');
                setDiagnosticsOpen(false);
              }}
              className="absolute top-4 right-4 p-2 hover:bg-muted rounded-lg transition-colors z-10"
            >
              <X className="text-muted-foreground hover:text-foreground" size={20} />
            </button>
            <TestConsole
              className="h-full rounded-xl"
              onComplete={() => {}} // Optional: auto-close on success?
            />
          </div>
        </div>
      )}
    </>
  );
}

export default App;
