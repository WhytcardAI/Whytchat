import { useState, useEffect, useRef } from "react";
import { useTranslation } from "react-i18next";
import { open } from "@tauri-apps/plugin-dialog";
import { readTextFile } from "@tauri-apps/plugin-fs";
import useAppStore from "../../lib/store";
import MessageBubble from "../MessageBubble";
import { Send, Paperclip, Bot, Users, Settings } from "lucide-react";

export default function ChatArea() {
  const { t } = useTranslation();
  const {
    getCurrentConversation,
    startChat,
    isGenerating,
    updateConversationConfig
  } = useAppStore();
  
  const [inputValue, setInputValue] = useState("");
  const [showConfig, setShowConfig] = useState(false);
  const scrollRef = useRef(null);

  const currentConversation = getCurrentConversation();
  const isMoA = currentConversation?.config?.useMoA || false;

  const handleSend = (e) => {
    e?.preventDefault();
    const text = inputValue.trim();
    if (!text || isGenerating) return;
    startChat(text);
    setInputValue("");
  };

  const handleAttachFile = async () => {
    try {
      const selectedPath = await open({ multiple: false });
      if (typeof selectedPath === "string") {
        const content = await readTextFile(selectedPath);
        setInputValue((prev) => prev + `\n\n[${t("chat.input.file_attached")} ${selectedPath}]\n${content}\n[/Fichier]`);
      }
    } catch (error) {
      console.error("File error:", error);
    }
  };

  const toggleMoA = () => {
    if (currentConversation) {
      updateConversationConfig(currentConversation.id, { useMoA: !isMoA });
    }
  };

  // Auto-scroll to bottom
  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [currentConversation?.messages, isGenerating]);

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
          <p className="text-sm text-muted-foreground max-w-md leading-relaxed">{t("chat.input.select_create")}</p>
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
                <h2 className="font-medium text-sm truncate text-foreground/90">{currentConversation.title}</h2>
             </div>
             <div className="flex items-center gap-1 border-l border-white/10 pl-3 ml-3">
               <button
                 onClick={() => setShowConfig(!showConfig)}
                 className={`p-1.5 rounded-full transition-all duration-300 ${showConfig ? 'bg-primary text-primary-foreground shadow-lg shadow-primary/25' : 'hover:bg-white/10 text-muted-foreground hover:text-foreground'}`}
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
                  <button onClick={() => setShowConfig(false)} className="text-xs text-muted-foreground hover:text-primary transition-colors">
                      {t("settings.close")}
                  </button>
              </div>
              
              <div className="space-y-4">
                  <div className="space-y-1.5">
                      <label className="text-[10px] uppercase tracking-wider font-bold text-muted-foreground/70">{t("settings.system_prompt")}</label>
                      <textarea
                        className="w-full h-24 bg-black/20 text-xs p-3 rounded-xl border border-white/5 resize-none outline-none focus:border-primary/50 focus:ring-1 focus:ring-primary/50 transition-all placeholder:text-white/20"
                        value={currentConversation.config.systemPrompt}
                        onChange={(e) => updateConversationConfig(currentConversation.id, { systemPrompt: e.target.value })}
                        placeholder="You are a helpful assistant..."
                      />
                  </div>
                  <div className="space-y-1.5">
                      <label className="text-[10px] uppercase tracking-wider font-bold text-muted-foreground/70">{t("settings.user_persona")}</label>
                      <textarea
                        className="w-full h-16 bg-black/20 text-xs p-3 rounded-xl border border-white/5 resize-none outline-none focus:border-primary/50 focus:ring-1 focus:ring-primary/50 transition-all placeholder:text-white/20"
                        value={currentConversation.config.userPersona || ""}
                        onChange={(e) => updateConversationConfig(currentConversation.id, { userPersona: e.target.value })}
                        placeholder="Ex: Senior Developer..."
                      />
                  </div>
                  
                  <div className="grid grid-cols-2 gap-4 pt-2">
                      <div className="space-y-1.5">
                          <label className="text-[10px] uppercase tracking-wider font-bold text-muted-foreground/70">{t("settings.models.temperature")}</label>
                          <input
                            type="number"
                            step="0.1"
                            min="0.1"
                            max="2.0"
                            className="w-full bg-black/20 text-xs p-2.5 rounded-xl border border-white/5 outline-none focus:border-primary/50 focus:ring-1 focus:ring-primary/50 text-center font-mono"
                            value={currentConversation.config.temperature || 0.7}
                            onChange={(e) => updateConversationConfig(currentConversation.id, { temperature: parseFloat(e.target.value) })}
                          />
                      </div>
                      <div className="space-y-1.5">
                          <label className="text-[10px] uppercase tracking-wider font-bold text-muted-foreground/70">{t("settings.models.max_tokens")}</label>
                          <input
                            type="number"
                            step="128"
                            min="128"
                            className="w-full bg-black/20 text-xs p-2.5 rounded-xl border border-white/5 outline-none focus:border-primary/50 focus:ring-1 focus:ring-primary/50 text-center font-mono"
                            value={currentConversation.config.maxTokens || 4096}
                            onChange={(e) => updateConversationConfig(currentConversation.id, { maxTokens: parseInt(e.target.value) })}
                          />
                      </div>
                  </div>
              </div>
          </div>
      )}

      {/* Messages Area */}
      <div ref={scrollRef} className="flex-1 overflow-y-auto px-4 sm:px-8 lg:px-12 py-20 space-y-8 scrollbar-thin scrollbar-thumb-white/5 hover:scrollbar-thumb-white/10 scrollbar-track-transparent">
        {currentConversation.messages
          .filter((m) => m.role !== "system")
          .map((m, i) => (
            <div key={m.timestamp || i} className="animate-in fade-in slide-in-from-bottom-4 duration-500 fill-mode-backwards" style={{ animationDelay: `${i * 50}ms` }}>
                <MessageBubble role={m.role} content={m.content} />
            </div>
          ))}
        {isGenerating && (
            <div className="flex justify-start animate-pulse pl-4">
                <div className="glass rounded-2xl px-5 py-3 text-sm text-muted-foreground flex items-center gap-2 shadow-sm">
                    <div className="w-2 h-2 bg-primary rounded-full animate-bounce [animation-delay:-0.3s]"></div>
                    <div className="w-2 h-2 bg-primary rounded-full animate-bounce [animation-delay:-0.15s]"></div>
                    <div className="w-2 h-2 bg-primary rounded-full animate-bounce"></div>
                    <span className="ml-2 text-xs font-medium opacity-70">{t("chat.orchestrator.thinking")}</span>
                </div>
            </div>
        )}
      </div>

      {/* Input Area (Floating Dock) */}
      <div className="p-6 pb-6 flex justify-center relative z-20">
        <form
            onSubmit={handleSend}
            className="relative w-full max-w-3xl glass-strong rounded-[2rem] shadow-2xl p-1.5 pl-4 flex items-end gap-2 transition-all duration-300 focus-within:shadow-[0_0_30px_rgba(var(--primary),0.15)] focus-within:border-primary/30 border border-white/10"
        >
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
                    className={`p-2 rounded-xl transition-all duration-200 active:scale-95 ${isMoA ? 'text-white bg-gradient-to-r from-purple-500 to-indigo-500 shadow-lg shadow-purple-500/20' : 'text-muted-foreground hover:text-foreground hover:bg-white/10'}`}
                    title={isMoA ? t("chat.input.moa_active") : t("chat.input.moa_enable")}
                >
                    {isMoA ? <Users className="w-5 h-5" /> : <Bot className="w-5 h-5" />}
                </button>
            </div>

            {/* Textarea Auto-resize */}
            <textarea
                value={inputValue}
                onChange={(e) => setInputValue(e.target.value)}
                onKeyDown={(e) => {
                    if (e.key === "Enter" && !e.shiftKey) {
                        e.preventDefault();
                        handleSend();
                    }
                }}
                placeholder={`${t("chat.input.send_to")} ${currentConversation.title}...`}
                rows={1}
                className="flex-1 bg-transparent border-none outline-none text-sm min-h-[50px] max-h-40 py-3.5 px-3 resize-none placeholder:text-muted-foreground/40 text-foreground leading-relaxed"
                style={{ minHeight: '52px' }}
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