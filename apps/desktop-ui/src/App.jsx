import React from 'react';
import { MainLayout } from './components/layout/MainLayout';
import { ChatInterface } from './components/chat/ChatInterface';
import { OnboardingWizard } from './components/onboarding/OnboardingWizard';
import { useAppStore } from './store/appStore';

function App() {
  const { isConfigured } = useAppStore();

  if (!isConfigured) {
    return <OnboardingWizard />;
  }

  return (
    <MainLayout>
      <ChatInterface />
    </MainLayout>
  );
}

export default App;
