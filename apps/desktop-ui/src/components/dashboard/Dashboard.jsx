import { MessageSquare, Plus } from 'lucide-react';
import { useTranslation } from 'react-i18next';

export function Dashboard({ onNewChat }) {
  const { t } = useTranslation('common');

  const handleStartNewChat = () => {
    if (onNewChat) {
      onNewChat();
    }
  };

  return (
    <div className="flex flex-col items-center justify-center h-full bg-background text-text p-8 animate-in fade-in zoom-in duration-500">
      <div className="max-w-2xl w-full text-center space-y-8">
        {/* Icon / Brand */}
        <div className="flex justify-center">
          <div className="w-20 h-20 bg-primary/10 rounded-3xl flex items-center justify-center shadow-xl shadow-primary/5 ring-1 ring-primary/20">
            <MessageSquare className="w-10 h-10 text-primary" strokeWidth={1.5} />
          </div>
        </div>

        {/* Welcome Text */}
        <div className="space-y-4">
          <h1 className="text-4xl font-bold tracking-tight text-text">
            {t('dashboard.welcome')}
          </h1>
          <p className="text-lg text-muted max-w-md mx-auto leading-relaxed">
            {t('dashboard.subtitle')}
          </p>
        </div>

        {/* Actions */}
        <div className="flex flex-col sm:flex-row items-center justify-center gap-4 pt-4">
          <button
            onClick={handleStartNewChat}
            className="group relative inline-flex items-center justify-center gap-2 px-8 py-3 text-sm font-medium text-white transition-all duration-300 bg-primary rounded-xl hover:bg-primary/90 hover:shadow-lg hover:shadow-primary/20 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-primary"
          >
            <Plus className="w-5 h-5 transition-transform group-hover:rotate-90" />
            <span>{t('dashboard.new_chat')}</span>
          </button>
        </div>

        {/* Quick hints or recent sessions could go here later */}
      </div>
    </div>
  );
}
