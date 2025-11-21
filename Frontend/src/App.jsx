import { useEffect } from "react";
import { Toaster } from "sonner";
import { ErrorBoundary } from "react-error-boundary";
import useAppStore from "./lib/store";
import { useSettingsStore } from "./stores/settingsStore";
import { useUIStore } from "./stores/uiStore";
import { ChevronLeft, ChevronRight } from "lucide-react";

import Header from "./components/Header";
import Sidebar from "./components/layout/Sidebar";
import ChatArea from "./components/layout/ChatArea";
import ContextPanel from "./components/layout/ContextPanel";

function GlobalErrorFallback({ error }) {
  return (
    <div className="flex flex-col items-center justify-center h-screen w-screen bg-background text-foreground p-8 text-center">
      <h1 className="text-2xl font-bold text-red-500 mb-4">Something went wrong</h1>
      <pre className="text-xs text-muted-foreground bg-black/20 p-4 rounded mb-6 max-w-2xl overflow-auto">
        {error.message}
      </pre>
      <button
        onClick={() => window.location.reload()}
        className="px-6 py-2 bg-primary text-white rounded-lg hover:bg-primary/90 transition-colors"
      >
        Reload Application
      </button>
    </div>
  );
}

function AppContent() {
  const { theme } = useSettingsStore();
  const { isSidebarOpen, isContextPanelOpen, toggleSidebar, toggleContextPanel } = useUIStore();
  const { conversations, currentConversationId, createConversation } = useAppStore();

  useEffect(() => {
    if (!currentConversationId && conversations.length === 0) {
      createConversation();
    }
  }, [currentConversationId, conversations, createConversation]);

  useEffect(() => {
    document.documentElement.setAttribute("data-theme", theme || "dark");
  }, [theme]);

  return (
    <div className="flex flex-col h-screen w-screen bg-transparent text-foreground overflow-hidden font-sans selection:bg-primary/20 selection:text-primary">
      {/* En-tête global - Intégré visuellement mais structurellement séparé */}
      <div className="z-50 relative">
        <Header />
      </div>

      {/* Layout principal - Structure flottante avec espacement */}
      <div className="flex flex-1 overflow-hidden relative p-3 gap-3">
        {/* Colonne 1 : Navigation - Panneau Flottant */}
        <div
          className={`
            ${isSidebarOpen ? "w-[280px] translate-x-0 opacity-100" : "w-0 -translate-x-full opacity-0 p-0 overflow-hidden"}
            transition-all duration-500 cubic-bezier(0.4, 0, 0.2, 1)
            h-full flex flex-col shrink-0 relative z-20
            rounded-2xl glass border-white/5 shadow-strong
          `}
        >
          <Sidebar />
        </div>

        {/* Toggle Sidebar Button (Flottant & Discret) */}
        <div
          className={`absolute left-3 top-6 z-30 transition-transform duration-500 ${isSidebarOpen ? "translate-x-[265px]" : "translate-x-0"}`}
        >
          <button
            onClick={toggleSidebar}
            className="p-1.5 rounded-full glass-hover text-muted-foreground hover:text-foreground transition-colors"
            title={isSidebarOpen ? "Collapse sidebar" : "Expand sidebar"}
          >
            {isSidebarOpen ? (
              <ChevronLeft className="w-4 h-4" />
            ) : (
              <ChevronRight className="w-4 h-4" />
            )}
          </button>
        </div>

        {/* Colonne 2 : Chat Principal - Panneau Central */}
        <main className="flex-1 flex flex-col min-w-0 relative z-10 rounded-2xl glass shadow-strong overflow-hidden">
          <ChatArea />
        </main>

        {/* Toggle ContextPanel Button (Flottant & Discret) */}
        <div
          className={`hidden lg:block absolute right-3 top-6 z-30 transition-transform duration-500 ${isContextPanelOpen ? "-translate-x-[305px]" : "translate-x-0"}`}
        >
          <button
            onClick={toggleContextPanel}
            className="p-1.5 rounded-full glass-hover text-muted-foreground hover:text-foreground transition-colors"
            title={isContextPanelOpen ? "Collapse context" : "Expand context"}
          >
            {isContextPanelOpen ? (
              <ChevronRight className="w-4 h-4" />
            ) : (
              <ChevronLeft className="w-4 h-4" />
            )}
          </button>
        </div>

        {/* Colonne 3 : Contexte & Intelligence - Panneau Flottant */}
        <div
          className={`
            ${isContextPanelOpen ? "w-[320px] translate-x-0 opacity-100" : "w-0 translate-x-full opacity-0 p-0 overflow-hidden"}
            transition-all duration-500 cubic-bezier(0.4, 0, 0.2, 1)
            h-full flex flex-col shrink-0 relative z-20
            rounded-2xl glass border-white/5 shadow-strong
          `}
        >
          <ContextPanel />
        </div>
      </div>

      <Toaster
        theme={theme === "light" ? "light" : "dark"}
        position="top-center"
        toastOptions={{
          className: "glass border-border text-foreground",
          style: { background: "rgba(20, 20, 23, 0.8)", backdropFilter: "blur(12px)" },
        }}
      />
    </div>
  );
}

function App() {
  return (
    <ErrorBoundary FallbackComponent={GlobalErrorFallback}>
      <AppContent />
    </ErrorBoundary>
  );
}

export default App;
