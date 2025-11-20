import { useState, useEffect, useRef, useMemo } from "react";
import { useTranslation } from "react-i18next";
import useAppStore from "../../lib/store";
import { Plus, MessageSquare, Users, Edit2, Calendar } from "lucide-react";
import { format, isToday, isYesterday, subDays } from "date-fns";
import { fr as frLocale, enUS, de, es, it, pt, nl } from "date-fns/locale";

export default function Sidebar() {
  const { t, i18n } = useTranslation();
  const {
    conversations,
    currentConversationId,
    setCurrentConversation,
    createConversation,
    renameConversation,
    selectedConversationIds,
    toggleConversationSelection,
    startMeeting
  } = useAppStore();

  const [editingId, setEditingId] = useState(null);
  const [editTitle, setEditTitle] = useState("");
  const editInputRef = useRef(null);

  // Gestion des dates locales pour date-fns
  const dateLocale = useMemo(() => {
    switch(i18n.language) {
      case 'fr': return frLocale;
      case 'en': return enUS;
      case 'de': return de;
      case 'es': return es;
      case 'it': return it;
      case 'pt': return pt;
      case 'nl': return nl;
      default: return enUS;
    }
  }, [i18n.language]);

  const startEditing = (e, conv) => {
      e.stopPropagation();
      setEditingId(conv.id);
      setEditTitle(conv.title);
  };

  const saveTitle = () => {
      if (editingId && editTitle.trim()) {
          renameConversation(editingId, editTitle.trim());
      }
      setEditingId(null);
  };

  const handleKeyDown = (e) => {
      if (e.key === "Enter") saveTitle();
      if (e.key === "Escape") setEditingId(null);
  };

  useEffect(() => {
      if (editingId && editInputRef.current) {
          editInputRef.current.focus();
      }
  }, [editingId]);

  // Groupement des conversations par date
  const groupedConversations = useMemo(() => {
    const groups = {
      today: [],
      yesterday: [],
      last7days: [],
      older: []
    };

    const sorted = [...conversations].sort((a, b) => new Date(b.createdAt) - new Date(a.createdAt));

    sorted.forEach(conv => {
      const date = new Date(conv.createdAt);
      if (isToday(date)) {
        groups.today.push(conv);
      } else if (isYesterday(date)) {
        groups.yesterday.push(conv);
      } else if (date > subDays(new Date(), 7)) {
        groups.last7days.push(conv);
      } else {
        groups.older.push(conv);
      }
    });

    return groups;
  }, [conversations]);

  const renderGroup = (title, items) => {
    if (items.length === 0) return null;
    return (
      <div className="mb-6 animate-in fade-in duration-500 slide-in-from-left-2">
        <h3 className="text-[10px] font-bold text-muted-foreground/50 px-4 mb-2 uppercase tracking-widest font-mono">
          {title}
        </h3>
        <div className="space-y-0.5 px-2">
          {items.map(conv => (
            <div key={conv.id}
                 className={`group relative flex items-center gap-3 px-3 py-2 rounded-lg transition-all duration-200 cursor-pointer
                 ${currentConversationId === conv.id
                    ? 'bg-white/5 text-foreground shadow-soft border border-white/5'
                    : 'text-muted-foreground hover:bg-white/5 hover:text-foreground border border-transparent'}`}
                 onClick={() => setCurrentConversation(conv.id)}
            >
               {/* Checkbox pour Meeting (visible au survol ou si sélectionné) */}
               <div className={`absolute left-3 z-10 ${selectedConversationIds.includes(conv.id) ? 'opacity-100' : 'opacity-0 group-hover:opacity-100'} transition-opacity duration-200`} onClick={(e) => e.stopPropagation()}>
                  <input
                     type="checkbox"
                     checked={selectedConversationIds.includes(conv.id)}
                     onChange={() => toggleConversationSelection(conv.id)}
                     className="w-3.5 h-3.5 rounded border-muted-foreground/50 bg-transparent checked:bg-primary checked:border-primary transition-colors cursor-pointer appearance-none border"
                  />
                  {/* Custom checkmark fallback handled by CSS or simple icon if needed, standard checkbox style for now */}
               </div>

               {/* Icone (masquée si checkbox visible au survol) */}
               <div className={`shrink-0 transition-opacity duration-200 ${selectedConversationIds.includes(conv.id) ? 'opacity-0' : 'group-hover:opacity-0'}`}>
                  {conv.type === 'meeting' ? (
                      <Users className="w-4 h-4 text-purple-400" />
                  ) : (
                      <MessageSquare className={`w-4 h-4 ${currentConversationId === conv.id ? 'text-primary' : 'text-muted-foreground/70 group-hover:text-foreground'}`} />
                  )}
               </div>
               
               <div className="flex-1 overflow-hidden min-w-0" onDoubleClick={(e) => startEditing(e, conv)}>
                   {editingId === conv.id ? (
                       <input
                         ref={editInputRef}
                         type="text"
                         value={editTitle}
                         onChange={(e) => setEditTitle(e.target.value)}
                         onBlur={saveTitle}
                         onKeyDown={handleKeyDown}
                         className="w-full bg-background/50 text-foreground text-xs px-2 py-1 rounded border border-primary/50 outline-none focus:ring-1 focus:ring-primary"
                         onClick={(e) => e.stopPropagation()}
                         autoFocus
                       />
                   ) : (
                       <div className="flex flex-col gap-0.5">
                         <span className={`text-xs truncate transition-colors ${currentConversationId === conv.id ? 'font-medium' : 'font-normal'}`}>
                             {conv.title}
                         </span>
                         {/* Date cachée par défaut pour plus de propreté, visible au hover si besoin, ou très discret */}
                       </div>
                   )}
               </div>
               
               {/* Bouton Edit au survol - Style minimaliste */}
               <button
                 onClick={(e) => startEditing(e, conv)}
                 className="opacity-0 group-hover:opacity-100 p-1 rounded text-muted-foreground/50 hover:text-foreground hover:bg-white/10 transition-all"
                 title={t("conversations.rename")}
               >
                   <Edit2 className="w-3 h-3" />
               </button>
            </div>
          ))}
        </div>
      </div>
    );
  };
  
  return (
    <div className="flex flex-col w-full h-full bg-transparent">
      <div className="p-3 pb-0">
        <button
          onClick={() => createConversation()}
          className="w-full group flex items-center justify-between bg-primary/90 hover:bg-primary text-primary-foreground px-3 py-2.5 rounded-xl transition-all shadow-lg shadow-primary/20 active:scale-[0.98] border border-white/10"
        >
          <span className="text-xs font-medium flex items-center gap-2">
            <Plus className="w-4 h-4 group-hover:rotate-90 transition-transform duration-300" />
            {t("conversations.new")}
          </span>
          <div className="w-5 h-5 rounded-full bg-white/20 flex items-center justify-center">
             <span className="text-[10px] font-bold">⌘N</span>
          </div>
        </button>
      </div>

      <div className="flex-1 overflow-y-auto py-4 scrollbar-thin scrollbar-thumb-white/5 hover:scrollbar-thumb-white/10 scrollbar-track-transparent px-1">
        {conversations.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-40 text-muted-foreground/30 px-6 text-center animate-in fade-in zoom-in-95 duration-500">
             <div className="w-12 h-12 rounded-full bg-white/5 flex items-center justify-center mb-3 border border-white/5">
                <MessageSquare className="w-5 h-5" />
             </div>
             <p className="text-xs">{t("conversations.empty")}</p>
          </div>
        ) : (
          <>
            {renderGroup(t("date.today") || "Aujourd'hui", groupedConversations.today)}
            {renderGroup(t("date.yesterday") || "Hier", groupedConversations.yesterday)}
            {renderGroup(t("date.last7days") || "7 derniers jours", groupedConversations.last7days)}
            {renderGroup(t("date.older") || "Plus ancien", groupedConversations.older)}
          </>
        )}
      </div>

      {/* Footer Actions (Meeting) - Flottant au dessus du fond */}
      {selectedConversationIds.length >= 2 && (
          <div className="p-3 pt-0 animate-in slide-in-from-bottom-2">
              <button
                  onClick={startMeeting}
                  className="w-full flex items-center justify-center gap-2 bg-gradient-to-r from-purple-600 to-indigo-600 text-white px-3 py-2.5 rounded-xl hover:brightness-110 text-xs font-bold uppercase tracking-wider shadow-lg shadow-purple-500/25 border border-white/10 transition-all"
              >
                  <Users className="w-4 h-4" />
                  {t("chat.input.create_meeting")} <span className="bg-white/20 px-1.5 rounded text-[10px]">{selectedConversationIds.length}</span>
              </button>
          </div>
      )}
    </div>
  );
}