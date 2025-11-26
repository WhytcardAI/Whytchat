import { useState, useRef, useEffect } from 'react';
import { Database, Settings, Moon, Sun } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { useAppStore } from '../../store/appStore';
import { cn } from '../../lib/utils';
import { SettingsDropdown } from './SettingsDropdown';

export function HeaderActions() {
  const { t } = useTranslation('common');
  const { theme, toggleTheme, isRightSidebarOpen, toggleRightSidebar } = useAppStore();
  const [activeDropdown, setActiveDropdown] = useState(null);
  const containerRef = useRef(null);

  // Close dropdown when clicking outside
  useEffect(() => {
    function handleClickOutside(event) {
      if (containerRef.current && !containerRef.current.contains(event.target)) {
        setActiveDropdown(null);
      }
    }
    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  const toggleDropdown = (name) => {
    setActiveDropdown(activeDropdown === name ? null : name);
  };

  return (
    <div ref={containerRef} className="flex items-center gap-1 relative z-50">
      {/* Data Sidebar Toggle */}
      <button
        onClick={toggleRightSidebar}
        className={cn(
          'p-2 rounded-xl transition-all',
          isRightSidebarOpen
            ? 'text-primary bg-primary/10'
            : 'text-muted hover:text-text hover:bg-surface/80'
        )}
        title={t('header.data', 'Data & Knowledge')}
      >
        <Database size={18} />
      </button>

      {/* Settings Button */}
      <div className="relative">
        <button
          onClick={() => toggleDropdown('settings')}
          className={cn(
            'p-2 rounded-xl transition-all',
            activeDropdown === 'settings'
              ? 'text-primary bg-primary/10'
              : 'text-muted hover:text-text hover:bg-surface/80'
          )}
          title={t('header.settings', 'Settings')}
        >
          <Settings size={18} />
        </button>
        {activeDropdown === 'settings' && (
          <SettingsDropdown onClose={() => setActiveDropdown(null)} />
        )}
      </div>

      {/* Theme Toggle */}
      <button
        onClick={toggleTheme}
        className="p-2 rounded-xl text-muted hover:text-text hover:bg-surface/80 transition-all"
        title={theme === 'light' ? t('theme.dark') : t('theme.light')}
      >
        {theme === 'light' ? <Moon size={18} /> : <Sun size={18} />}
      </button>
    </div>
  );
}
