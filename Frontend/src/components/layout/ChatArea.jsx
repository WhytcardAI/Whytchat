import { useMemo, useRef } from "react";
import { useTranslation } from "react-i18next";
import { Virtuoso } from "react-virtuoso";
import { ErrorBoundary } from "react-error-boundary";
import useAppStore from "../../lib/store";
import { useUIStore } from "../../stores/uiStore";
import { useHotkeys } from "../../hooks/useHotkeys";
import { useChatInput } from "../../hooks/useChatInput";
import { useChatConfig } from "../../hooks/useChatConfig";
import MessageBubble from "../MessageBubble";
import ThinkingIndicator from "../ThinkingIndicator";
import DebateConfigPanel from "./DebateConfigPanel";
import DebateStage from "../DebateStage";
import PromptTemplates from "../PromptTemplates";
import { Send, Paperclip, Bot, Users, Settings, Swords } from "lucide-react";

/**
 * Fallback component for ChatArea errors.
 *
 * @param {Object} props
 * @param {Error} props.error - The error object.
 * @param {Function} props.resetErrorBoundary - Function to reset the error boundary.
 */
function ChatAreaErrorFallback({ error, resetErrorBoundary }) {
  const { t } = useTranslation();
  return (
    <div className="flex-1 flex flex-col items-center justify-center p-8 text-center">
      <h2 className="text-xl font-bold text-red-500 mb-4">
        {t("error.chat_crashed") || "Chat Error"}
      </h2>
      <pre className="text-xs text-muted-foreground bg-black/20 p-4 rounded mb-4 max-w-lg overflow-auto">
        {error.message}
      </pre>
      <button
        onClick={resetErrorBoundary}
        className="px-4 py-2 bg-primary text-white rounded-lg hover:bg-primary/90"
      >
        {t("error.retry") || "Retry"}
      </button>
    </div>
  );
}

/**
 * Main chat area component handling message display, input, and configuration.
 *
 * @component
 */
function ChatAreaContent() {
  const { t } = useTranslation();
  const { getCurrentConversation, startChat, isGenerating, updateConversationConfig } =
    useAppStore();

  const { toggleSidebar } = useUIStore();
  const virtuosoRef = useRef(null);
  const currentConversation = getCurrentConversation();

  // Custom Hooks
  const {
    inputValue,
    showTemplates,
    setShowTemplates,
    templateFilter,
    inputRef,
    handleInputChange,
    handleTemplateSelect,
    handleSend,
    handleAttachFile,
  } = useChatInput(startChat, isGenerating);

  const { showConfig, setShowConfig, showDebateConfig, setShowDebateConfig } = useChatConfig(
    currentConversation?.id
  );

  // --- Hotkeys ---

  // Ctrl+K / Cmd+K: Focus Input
  useHotkeys(
    "k",
    () => {
      inputRef.current?.focus();
    },
    { ctrlKey: true }
  );

  // Ctrl+B / Cmd+B: Toggle Sidebar
  useHotkeys(
    "b",
    () => {
      toggleSidebar();
    },
    { ctrlKey: true }
  );

  // Escape: Close all modals/menus
  useHotkeys("Escape", () => {
    if (showTemplates) setShowTemplates(false);
    else if (showConfig) setShowConfig(false);
    else if (showDebateConfig) setShowDebateConfig(false);
  });

  const isMoA = currentConversation?.config?.useMoA || false;

  const toggleMoA = () => {
    if (currentConversation) {
      updateConversationConfig(currentConversation.id, { useMoA: !isMoA });
    }
  };

  // Filter messages for display
  const displayMessages = useMemo(() => {
    if (!currentConversation || !currentConversation.messages) return [];
    return currentConversation.messages.filter((m) => m.role !== "system");
  }, [currentConversation]);

  if (!currentConversation) {
    return (
      <div className="flex-1 flex flex-col items-center justify-center bg-transparent text-muted-foreground relative overflow-hidden">
        {/* Background decorative elements */}
        <div className="absolute inset-0 bg-[radial-gradient(circle_at_center,_var(--tw-gradient-stops))] from-primary/5 via-transparent to-transparent opacity-50" />

        <div className="relative z-10 text-center p-8 animate-in fade-in zoom-in-95 duration-700">
          <div className="w-20 h-20 rounded-2xl bg-gradient-to-br from-white/10 to-white/5 border border-white/10 shadow-strong backdrop-blur-xl flex items-center justify-center mx-auto mb-6 rotate-3 hover:rotate-0 transition-transform duration-500">
            <Bot className="w-10 h-10 text-primary opacity-80" />
          </div>
          <h3 className="text-xl font-semibold text-foreground mb-2 tracking-tight">WhytChat</h3>
          <p className="text-sm text-muted-foreground max-w-md leading-relaxed">
            {t("chat.input.select_create")}
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="flex-1 flex flex-col h-full relative bg-transparent">
      {/* Chat Header (Floating & Glass) */}
      <div className="absolute top-0 left-0 right-0 z-20 px-6 py-4">
        <div className="flex items-center justify-between glass-strong rounded-full px-4 py-2 shadow-sm mx-auto max-w-2xl transition-all hover:shadow-md">
          <div className="flex items-center gap-3 min-w-0">
            <div className="w-2 h-2 rounded-full bg-green-500 animate-pulse shadow-[0_0_8px_rgba(34,197,94,0.5)]"></div>
            <h2 className="font-medium text-sm truncate text-foreground/90">
              {currentConversation.title}
            </h2>
          </div>
          <div className="flex items-center gap-1 border-l border-white/10 pl-3 ml-3">
            <button
              onClick={() => setShowConfig(!showConfig)}
              className={`p-1.5 rounded-full transition-all duration-300 ${showConfig ? "bg-primary text-primary-foreground shadow-lg shadow-primary/25" : "hover:bg-white/10 text-muted-foreground hover:text-foreground"}`}
              title={t("chat.settings.title")}
            >
              <Settings className="w-4 h-4" />
            </button>
          </div>
        </div>
      </div>

      {/* Config Modal (Glassmorphism Popover) */}
      {showConfig && (
        <div className="absolute top-20 right-1/2 translate-x-1/2 w-96 glass-strong rounded-2xl shadow-strong p-5 z-50 animate-in fade-in zoom-in-95 slide-in-from-top-4 duration-300 border border-white/10 backdrop-blur-2xl">
          <div className="flex justify-between items-center mb-4 border-b border-white/5 pb-3">
            <h3 className="text-sm font-semibold tracking-wide">{t("settings.config_title")}</h3>
            <button
              onClick={() => setShowConfig(false)}
              className="text-xs text-muted-foreground hover:text-primary transition-colors"
            >
              {t("settings.close")}
            </button>
          </div>

          <div className="space-y-4">
            <div className="space-y-1.5">
              <label className="text-[10px] uppercase tracking-wider font-bold text-muted-foreground/70">
                {t("settings.system_prompt")}
              </label>
              <textarea
                className="w-full h-24 bg-black/20 text-xs p-3 rounded-xl border border-white/5 resize-none outline-none focus:border-primary/50 focus:ring-1 focus:ring-primary/50 transition-all placeholder:text-white/20"
                value={currentConversation.config.systemPrompt}
                onChange={(e) =>
                  updateConversationConfig(currentConversation.id, { systemPrompt: e.target.value })
                }
                placeholder="You are a helpful assistant..."
              />
            </div>
            <div className="space-y-1.5">
              <label className="text-[10px] uppercase tracking-wider font-bold text-muted-foreground/70">
                {t("settings.user_persona")}
              </label>
              <textarea
                className="w-full h-16 bg-black/20 text-xs p-3 rounded-xl border border-white/5 resize-none outline-none focus:border-primary/50 focus:ring-1 focus:ring-primary/50 transition-all placeholder:text-white/20"
                value={currentConversation.config.userPersona || ""}
                onChange={(e) =>
                  updateConversationConfig(currentConversation.id, { userPersona: e.target.value })
                }
                placeholder="Ex: Senior Developer..."
              />
            </div>

            <div className="grid grid-cols-2 gap-4 pt-2">
              <div className="space-y-1.5">
                <label className="text-[10px] uppercase tracking-wider font-bold text-muted-foreground/70">
                  {t("settings.models.temperature")}
                </label>
                <input
                  type="number"
                  step="0.1"
                  min="0.1"
                  max="2.0"
                  className="w-full bg-black/20 text-xs p-2.5 rounded-xl border border-white/5 outline-none focus:border-primary/50 focus:ring-1 focus:ring-primary/50 text-center font-mono"
                  value={currentConversation.config.temperature || 0.7}
                  onChange={(e) =>
                    updateConversationConfig(currentConversation.id, {
                      temperature: parseFloat(e.target.value),
                    })
                  }
                />
              </div>
              <div className="space-y-1.5">
                <label className="text-[10px] uppercase tracking-wider font-bold text-muted-foreground/70">
                  {t("settings.models.max_tokens")}
                </label>
                <input
                  type="number"
                  step="128"
                  min="128"
                  className="w-full bg-black/20 text-xs p-2.5 rounded-xl border border-white/5 outline-none focus:border-primary/50 focus:ring-1 focus:ring-primary/50 text-center font-mono"
                  value={currentConversation.config.maxTokens || 4096}
                  onChange={(e) =>
                    updateConversationConfig(currentConversation.id, {
                      maxTokens: parseInt(e.target.value),
                    })
                  }
                />
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Debate Config Panel */}
      {showDebateConfig && <DebateConfigPanel onClose={() => setShowDebateConfig(false)} />}

      {/* Debate Stage Visualization - Fixed at top */}
      <div className="pt-20 px-4 sm:px-8 lg:px-12">
        <DebateStage />
      </div>

      {/* Messages Area - Virtualized */}
      <div className="flex-1 relative min-h-0">
        <Virtuoso
          ref={virtuosoRef}
          data={displayMessages}
          followOutput="smooth"
          initialTopMostItemIndex={displayMessages.length - 1}
          className="scrollbar-thin scrollbar-thumb-white/5 hover:scrollbar-thumb-white/10 scrollbar-track-transparent"
          itemContent={(index, m) => (
            <div className="px-4 sm:px-8 lg:px-12 py-4 animate-in fade-in slide-in-from-bottom-4 duration-500 fill-mode-backwards">
              <MessageBubble role={m.role} content={m.content} />
            </div>
          )}
          components={{
            Footer: () => (
              <div className="pb-32 px-4 sm:px-8 lg:px-12">
                {isGenerating && (
                  <div className="flex justify-start pl-4 py-4">
                    <div className="glass rounded-2xl shadow-sm overflow-hidden">
                      <ThinkingIndicator />
                    </div>
                  </div>
                )}
              </div>
            ),
            Header: () => <div className="h-4" />, // Spacer for top
          }}
        />
      </div>

      {/* Input Area (Floating Dock) */}
      <div className="p-6 pb-6 flex justify-center relative z-20">
        <form
          onSubmit={handleSend}
          className="relative w-full max-w-3xl glass-strong rounded-[2rem] shadow-2xl p-1.5 pl-4 flex items-end gap-2 transition-all duration-300 focus-within:shadow-[0_0_30px_rgba(var(--primary),0.15)] focus-within:border-primary/30 border border-white/10"
        >
          {/* Template Menu Popover */}
          <PromptTemplates
            isVisible={showTemplates}
            onSelect={handleTemplateSelect}
            onClose={() => setShowTemplates(false)}
            filterText={templateFilter}
            position={{ bottom: "100%", left: "16px", marginBottom: "8px" }}
          />

          {/* Actions Rapides */}
          <div className="flex items-center pb-2 gap-1">
            <button
              type="button"
              onClick={handleAttachFile}
              className="p-2 text-muted-foreground hover:text-foreground hover:bg-white/10 rounded-xl transition-all duration-200 active:scale-95"
              title={t("chat.actions.attach")}
            >
              <Paperclip className="w-5 h-5" />
            </button>

            <button
              type="button"
              onClick={toggleMoA}
              className={`p-2 rounded-xl transition-all duration-200 active:scale-95 ${isMoA ? "text-white bg-gradient-to-r from-purple-500 to-indigo-500 shadow-lg shadow-purple-500/20" : "text-muted-foreground hover:text-foreground hover:bg-white/10"}`}
              title={isMoA ? t("chat.input.moa_active") : t("chat.input.moa_enable")}
            >
              {isMoA ? <Users className="w-5 h-5" /> : <Bot className="w-5 h-5" />}
            </button>

            <button
              type="button"
              onClick={() => setShowDebateConfig(true)}
              className="p-2 text-muted-foreground hover:text-orange-500 hover:bg-white/10 rounded-xl transition-all duration-200 active:scale-95"
              title={t("chat.input.start_debate")}
              aria-label={t("chat.input.start_debate")}
            >
              <Swords className="w-5 h-5" aria-hidden="true" />
            </button>
          </div>

          {/* Textarea Auto-resize */}
          <textarea
            ref={inputRef}
            value={inputValue}
            onChange={handleInputChange}
            onKeyDown={(e) => {
              if (showTemplates) {
                // Let the template menu handle navigation keys
                if (["ArrowUp", "ArrowDown", "Enter", "Tab", "Escape"].includes(e.key)) {
                  // We don't prevent default here for all keys, but for Enter/Arrows we might need to
                  // to prevent textarea behavior.
                  // The window listener in PromptTemplates handles the logic, but we need to stop
                  // the textarea from processing it too (e.g. new line on Enter).
                  if (e.key === "Enter" || e.key === "ArrowUp" || e.key === "ArrowDown") {
                    e.preventDefault();
                  }
                  return;
                }
              }

              if (e.key === "Enter" && !e.shiftKey && !showTemplates) {
                e.preventDefault();
                handleSend();
              }
            }}
            placeholder={`${t("chat.input.send_to")} ${currentConversation.title}... (${t("chat.input.templates_hint")})`}
            rows={1}
            className="flex-1 bg-transparent border-none outline-none text-sm min-h-[50px] max-h-40 py-3.5 px-3 resize-none placeholder:text-muted-foreground/40 text-foreground leading-relaxed"
            style={{ minHeight: "52px" }}
            aria-label={t("chat.input_placeholder")}
          />

          {/* Send Button */}
          <button
            type="submit"
            disabled={!inputValue.trim() || isGenerating}
            className="m-1 p-3 bg-primary text-primary-foreground rounded-full hover:bg-primary-hover disabled:opacity-50 disabled:cursor-not-allowed transition-all duration-300 shadow-lg shadow-primary/20 hover:shadow-primary/40 hover:scale-105 active:scale-95 flex items-center justify-center"
          >
            <Send className="w-5 h-5 ml-0.5" />
          </button>
        </form>
      </div>
    </div>
  );
}

/**
 * Main Chat Area container.
 * Wraps the chat interface in an ErrorBoundary for robustness.
 *
 * @component
 */
export default function ChatArea() {
  return (
    <ErrorBoundary FallbackComponent={ChatAreaErrorFallback}>
      <ChatAreaContent />
    </ErrorBoundary>
  );
}
