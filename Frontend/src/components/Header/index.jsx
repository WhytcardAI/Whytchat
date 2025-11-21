import { appWindow } from "@tauri-apps/api/window";
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { useSettingsStore } from "../../stores/settingsStore";
import { Moon, Sun, Languages } from "lucide-react";

export default function Header() {
  const { t, i18n } = useTranslation();
  const [showLanguages, setShowLanguages] = useState(false);
  const { theme, setTheme } = useSettingsStore();

  const toggleTheme = () => {
    const newTheme = theme === "dark" ? "light" : "dark";
    setTheme(newTheme);
  };

  const changeLanguage = (lng) => {
    i18n.changeLanguage(lng);
    setShowLanguages(false);
  };

  const languages = [
    { code: "fr", label: "Français", flag: "🇫🇷" },
    { code: "en", label: "English", flag: "🇬🇧" },
    { code: "de", label: "Deutsch", flag: "🇩🇪" },
    { code: "es", label: "Español", flag: "🇪🇸" },
    { code: "it", label: "Italiano", flag: "🇮🇹" },
    { code: "pt", label: "Português", flag: "🇵🇹" },
    { code: "nl", label: "Nederlands", flag: "🇳🇱" },
  ];

  const handleMinimize = async () => {
    if (window.__TAURI__) {
      await appWindow.minimize();
    }
  };

  const handleMaximize = async () => {
    if (window.__TAURI__) {
      await appWindow.toggleMaximize();
    }
  };

  const handleClose = async () => {
    if (window.__TAURI__) {
      await appWindow.close();
    }
  };

  return (
    <header
      className="flex items-center justify-between px-4 py-2 border-b border-border bg-background h-10"
      data-tauri-drag-region
    >
      <div className="flex items-center gap-3">
        <h1 className="text-sm font-semibold tracking-tight select-none text-muted-foreground/80">
          WhytChat
        </h1>
      </div>

      <div className="flex items-center gap-1">
        <div className="relative">
          <button
            onClick={() => setShowLanguages(!showLanguages)}
            className="w-8 h-8 flex items-center justify-center text-muted-foreground hover:text-foreground hover:bg-secondary/50 rounded transition-colors"
            type="button"
            title={t("settings.general.language")}
          >
            <Languages className="w-4 h-4" />
          </button>

          {showLanguages && (
            <div className="absolute top-10 right-0 w-40 bg-card border border-border rounded-lg shadow-strong py-1 z-50">
              {languages.map((lang) => (
                <button
                  key={lang.code}
                  onClick={() => changeLanguage(lang.code)}
                  className={`w-full text-left px-3 py-2 text-xs hover:bg-secondary flex items-center gap-2 ${
                    i18n.language === lang.code ? "bg-secondary/50 font-medium" : ""
                  }`}
                >
                  <span className="text-base">{lang.flag}</span>
                  <span>{lang.label}</span>
                </button>
              ))}
            </div>
          )}
        </div>

        <button
          onClick={toggleTheme}
          className="w-8 h-8 flex items-center justify-center text-muted-foreground hover:text-foreground hover:bg-secondary/50 rounded transition-colors"
          type="button"
          title={theme === "dark" ? "Mode Clair" : "Mode Sombre"}
        >
          {theme === "dark" ? <Sun className="w-4 h-4" /> : <Moon className="w-4 h-4" />}
        </button>
        <button
          onClick={handleMinimize}
          className="w-8 h-8 flex items-center justify-center text-muted-foreground hover:text-foreground hover:bg-secondary/50 rounded transition-colors"
          type="button"
        >
          <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M20 12H4" />
          </svg>
        </button>
        <button
          onClick={handleMaximize}
          className="w-8 h-8 flex items-center justify-center text-muted-foreground hover:text-foreground hover:bg-secondary/50 rounded transition-colors"
          type="button"
        >
          <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <rect x="3" y="3" width="18" height="18" rx="2" />
          </svg>
        </button>
        <button
          onClick={handleClose}
          className="w-8 h-8 flex items-center justify-center text-muted-foreground hover:text-red-500 hover:bg-red-500/10 rounded transition-colors"
          type="button"
        >
          <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M6 18L18 6M6 6l12 12"
            />
          </svg>
        </button>
      </div>
    </header>
  );
}
