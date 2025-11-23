import React, { useState } from 'react';
import { BrainCircuit, Database, Activity } from 'lucide-react';
import { cn } from '../../lib/utils';
import { useTranslation } from 'react-i18next';

export function RightPanel({ isThinking, thinkingSteps }) {
  const [activeTab, setActiveTab] = useState('orchestration');
  const { t } = useTranslation('common');

  return (
    <aside className="w-80 bg-surface border-l border-border flex flex-col h-full shadow-sm">
      {/* Tabs */}
      <div className="flex p-2 gap-2 border-b border-border bg-background/50">
        <TabButton
          active={activeTab === 'orchestration'}
          onClick={() => setActiveTab('orchestration')}
          icon={<Activity size={16} />}
          label={t('panel.orchestration', 'Orchestration')}
        />
        <TabButton
          active={activeTab === 'knowledge'}
          onClick={() => setActiveTab('knowledge')}
          icon={<Database size={16} />}
          label={t('panel.knowledge', 'Connaissances')}
        />
      </div>

      {/* Content */}
      <div className="flex-1 overflow-y-auto p-4">
        {activeTab === 'orchestration' && (
          <div className="flex flex-col items-center justify-center h-full text-center space-y-4">
            {!isThinking && thinkingSteps.length === 0 ? (
              <>
                <div className="w-24 h-24 rounded-full bg-background flex items-center justify-center text-muted/20">
                  <BrainCircuit size={48} />
                </div>
                <p className="text-sm text-muted">
                  {t('panel.waiting', 'En attente de raisonnement...')}
                </p>
              </>
            ) : (
              <div className="w-full space-y-4">
                <div className="flex items-center justify-center gap-2 text-primary animate-pulse">
                  <BrainCircuit size={20} />
                  <span className="font-medium text-sm">Thinking...</span>
                </div>
                <div className="space-y-2 text-left">
                  {thinkingSteps.map((step, idx) => (
                    <div key={idx} className="p-3 bg-background rounded-lg text-xs border border-border animate-in slide-in-from-bottom-2">
                      <span className="font-mono text-muted mr-2">{idx + 1}.</span>
                      {step}
                    </div>
                  ))}
                </div>
              </div>
            )}
          </div>
        )}

        {activeTab === 'knowledge' && (
          <div className="text-center text-muted text-sm mt-10">
            <Database size={32} className="mx-auto mb-2 opacity-20" />
            <p>Aucun document actif</p>
          </div>
        )}
      </div>
    </aside>
  );
}

function TabButton({ active, onClick, icon, label }) {
  return (
    <button
      onClick={onClick}
      className={cn(
        "flex-1 flex items-center justify-center gap-2 py-2 px-3 rounded-lg text-xs font-medium transition-all",
        active
          ? "bg-white shadow-sm text-primary ring-1 ring-border"
          : "text-muted hover:bg-white/50"
      )}
    >
      {icon}
      <span>{label}</span>
    </button>
  );
}
