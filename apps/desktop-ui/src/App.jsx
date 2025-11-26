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

function App() {
  const {
    isConfigured,
    initializeApp,
    currentSessionId,
    setIsConfigured,
    isDiagnosticsOpen,
    setDiagnosticsOpen
  } = useAppStore();
  const [isCreatingSession, setIsCreatingSession] = useState(false);
  const [preflightState, setPreflightState] = useState('checking'); // 'checking' | 'passed' | 'failed' | 'onboarding'
  const [preflightReport, setPreflightReport] = useState(null);

  useEffect(() => {
    const runPreflight = async () => {
      try {
        console.log('Running preflight check...');
        // Use quick check first for fast feedback
        const report = await invoke('run_quick_preflight_check');
        console.log('Preflight report:', report);
        setPreflightReport(report);

        if (report.needs_onboarding) {
          // Need to download model/server
          console.log('Needs onboarding');
          setIsConfigured(false);
          setPreflightState('onboarding');
        } else if (report.ready_to_start) {
          // All good, initialize app
          console.log('Ready to start, initializing...');
          await initializeApp();
          setPreflightState('passed');
        } else {
          // Some checks failed
          console.log('Preflight failed');
          setPreflightState('failed');
        }
      } catch (error) {
        console.error('Preflight error:', error);
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
    setIsCreatingSession(true);
  };

  return (
    <>
      <MainLayout isCreatingSession={isCreatingSession} setIsCreatingSession={setIsCreatingSession}>
        {currentSessionId ? <ChatInterface /> : <Dashboard onNewChat={handleNewChat} />}
      </MainLayout>

      {/* Diagnostics Overlay */}
      {isDiagnosticsOpen && (
        <div className="fixed inset-0 bg-background/80 backdrop-blur-sm z-50 flex items-center justify-center p-8 animate-in fade-in duration-200">
          <div className="w-full max-w-4xl h-[600px] bg-surface rounded-xl shadow-2xl border border-border flex flex-col relative animate-in zoom-in-95 duration-200">
            <button
              onClick={() => setDiagnosticsOpen(false)}
              className="absolute top-4 right-4 p-2 hover:bg-white/10 rounded-lg transition-colors z-10"
            >
              <X className="text-gray-400 hover:text-white" size={20} />
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
