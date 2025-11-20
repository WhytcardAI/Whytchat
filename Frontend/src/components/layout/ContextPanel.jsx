import { useState } from "react";
import OrchestrationVisualizer from "../OrchestrationVisualizer";
import KnowledgePanel from "./KnowledgePanel";
import { Activity, Database } from "lucide-react";

export default function ContextPanel() {
  const [activeTab, setActiveTab] = useState("orchestration");

  return (
    <div className="w-full h-full flex flex-col relative overflow-hidden bg-transparent">
      {/* Tabs Navigation - Style Pill/Segmented Control plus moderne */}
      <div className="p-3 pb-0">
        <div className="flex items-center bg-black/20 p-1 rounded-xl border border-white/5">
          <button
            onClick={() => setActiveTab("orchestration")}
            className={`flex-1 flex items-center justify-center gap-2 py-1.5 rounded-lg text-xs font-medium transition-all duration-300 ${
              activeTab === "orchestration"
                ? "bg-primary/10 text-primary shadow-sm"
                : "text-muted-foreground hover:text-foreground hover:bg-white/5"
            }`}
          >
            <Activity className="w-3.5 h-3.5" />
            Orchestration
          </button>
          <button
            onClick={() => setActiveTab("knowledge")}
            className={`flex-1 flex items-center justify-center gap-2 py-1.5 rounded-lg text-xs font-medium transition-all duration-300 ${
              activeTab === "knowledge"
                ? "bg-primary/10 text-primary shadow-sm"
                : "text-muted-foreground hover:text-foreground hover:bg-white/5"
            }`}
          >
            <Database className="w-3.5 h-3.5" />
            Connaissances
          </button>
        </div>
      </div>

      <div className="flex-1 overflow-hidden relative pointer-events-auto">
        <div className="p-3 h-full overflow-y-auto scrollbar-thin scrollbar-thumb-white/5 hover:scrollbar-thumb-white/10">
          {activeTab === "orchestration" ? (
            <div className="animate-in fade-in slide-in-from-right-4 duration-300">
               <OrchestrationVisualizer isEmbedded={true} />
            </div>
          ) : (
            <div className="animate-in fade-in slide-in-from-right-4 duration-300">
               <KnowledgePanel />
            </div>
          )}
        </div>
      </div>
    </div>
  );
}