import { useAppStore } from '../../store/appStore';
import { LayoutGrid, MessageSquare, Settings, Database } from 'lucide-react';
import { cn } from '../../lib/utils';
import { useTranslation } from 'react-i18next';

export function Rail() {
  const { currentView, setView } = useAppStore();
  const { t } = useTranslation('common');

  const navItems = [
    {
      id: 'knowledge',
      icon: LayoutGrid,
      label: t('nav.knowledge', 'Knowledge'),
    },
    {
      id: 'chat',
      icon: MessageSquare,
      label: t('nav.chat', 'Chat'),
    },
    // Placeholder for future features
    // {
    //   id: 'settings',
    //   icon: Settings,
    //   label: t('nav.settings', 'Settings'),
    // },
  ];

  return (
    <div className="w-16 bg-surface border-r border-border flex flex-col items-center pt-4 pb-6 gap-4 shrink-0 z-50">
      {/* Logo Placeholder */}
      <div className="w-10 h-10 flex items-center justify-center mb-2">
        <Database size={24} className="text-primary" />
      </div>

      {/* Nav Items */}
      <div className="flex flex-col gap-2 w-full px-2">
        {navItems.map((item) => (
          <button
            key={item.id}
            onClick={() => setView(item.id)}
            className={cn(
              "w-full aspect-square rounded-xl flex flex-col items-center justify-center gap-1 transition-all duration-200 group relative",
              currentView === item.id
                ? "bg-primary text-primary-foreground dark:text-zinc-900"
                : "text-muted hover:text-text hover:bg-surface/80"
            )}
            title={item.label}
          >
            <item.icon size={20} strokeWidth={currentView === item.id ? 2.5 : 2} />
            <span className="text-[10px] font-medium truncate max-w-full px-1">{item.label}</span>
          </button>
        ))}
      </div>

      <div className="mt-auto flex flex-col gap-2 w-full px-2">
        <button
          className="w-full aspect-square rounded-xl flex items-center justify-center text-muted hover:text-text hover:bg-surface/80 transition-colors"
          title={t('nav.settings', 'Settings')}
        >
          <Settings size={20} />
        </button>
      </div>
    </div>
  );
}
