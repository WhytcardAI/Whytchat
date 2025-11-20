import { useEffect, useState, useRef } from "react";
import { api } from "../../lib/api";
import { useLearningStore } from "../../stores/learningStore";
import { Brain, ThumbsUp, ThumbsDown, Sparkles, Terminal, ArrowDown } from "lucide-react";

export default function OrchestrationVisualizer({ isEmbedded = false }) {
  const [thoughts, setThoughts] = useState([]);
  const scrollRef = useRef(null);
  const addInsight = useLearningStore((state) => state.addInsight);

  useEffect(() => {
    const unlistenStart = api.on("agent-thinking-start", (event) => {
      setThoughts((prev) => [
        ...prev,
        {
          id: Date.now(),
          agent: event.payload.agent,
          status: "thinking",
          content: "",
          timestamp: new Date(),
        },
      ]);
    });

    const unlistenThought = api.on("agent-thought", (event) => {
      setThoughts((prev) => {
        const lastIndex = prev.findLastIndex(
          (t) => t.agent === event.payload.agent && t.status === "thinking"
        );

        if (lastIndex !== -1) {
          const newThoughts = [...prev];
          newThoughts[lastIndex] = {
            ...newThoughts[lastIndex],
            status: "completed",
            content: event.payload.content,
            timestamp: new Date(event.payload.timestamp),
          };
          return newThoughts;
        } else {
          return [
            ...prev,
            {
              id: Date.now(),
              agent: event.payload.agent,
              status: "completed",
              content: event.payload.content,
              timestamp: new Date(event.payload.timestamp),
            },
          ];
        }
      });
    });

    return () => {
      unlistenStart.then((f) => f());
      unlistenThought.then((f) => f());
    };
  }, []);

  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [thoughts]);

  const handleFeedback = (thoughtId, isGood) => {
    const thought = thoughts.find((t) => t.id === thoughtId);
    if (thought) {
      addInsight(thought.agent, thought.content, isGood);
      setThoughts((prev) =>
        prev.map((t) =>
          t.id === thoughtId ? { ...t, feedback: isGood ? "up" : "down" } : t
        )
      );
    }
  };

  if (thoughts.length === 0 && !isEmbedded) return null;

  const containerClass = isEmbedded
    ? "flex flex-col h-full w-full relative"
    : "fixed top-20 right-4 w-80 max-h-[80vh] flex flex-col z-40 pointer-events-none";

  const scrollClass = isEmbedded
    ? "flex-1 overflow-y-auto pr-2 scrollbar-hide pb-10 pl-4"
    : "flex flex-col overflow-y-auto pr-2 pointer-events-auto scrollbar-hide pb-10 pl-4";

  return (
    <div className={containerClass}>
      {isEmbedded && thoughts.length === 0 && (
          <div className="flex flex-col items-center justify-center h-full text-muted-foreground opacity-20">
              <Brain className="w-12 h-12 mb-2" />
              <p className="text-xs text-center">En attente de raisonnement...</p>
          </div>
      )}
      <div
        ref={scrollRef}
        className={scrollClass}
      >
        {/* Ligne de flux verticale */}
        <div className="absolute left-8 top-4 bottom-0 w-0.5 bg-gradient-to-b from-transparent via-white/10 to-transparent -z-10" />

        {thoughts.map((thought, index) => (
          <div key={thought.id} className="relative mb-6 group last:mb-0">
            {/* Connecteur point */}
            <div className={`absolute -left-4 top-5 w-3 h-3 rounded-full border-2 border-background ${getAgentColorBg(thought.agent)} shadow-[0_0_10px_rgba(0,0,0,0.5)] z-10`} />
            
            {/* Connecteur ligne vers la carte */}
            {/* <div className="absolute -left-3 top-6 w-4 h-[1px] bg-white/20" /> */}

            <div
              className={`
                relative overflow-hidden rounded-xl border backdrop-blur-xl transition-all duration-500
                ${
                  thought.status === "thinking"
                    ? "bg-yellow-500/5 border-yellow-500/20 animate-pulse"
                    : "bg-black/40 border-white/10 shadow-lg hover:bg-black/60"
                }
              `}
            >
              {/* Header Compact */}
              <div className="flex items-center justify-between p-2 border-b border-white/5 bg-white/5">
                <span className={`text-[10px] font-bold tracking-widest uppercase ${getAgentColorText(thought.agent)}`}>
                  {thought.agent}
                </span>
                 <div className="flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                  {thought.status === "completed" && (
                    <>
                      <button
                        onClick={() => handleFeedback(thought.id, true)}
                        className={`p-1 rounded hover:bg-white/10 ${thought.feedback === 'up' ? 'text-green-400' : 'text-muted-foreground'}`}
                      >
                        <ThumbsUp className="w-3 h-3" />
                      </button>
                      <button
                        onClick={() => handleFeedback(thought.id, false)}
                         className={`p-1 rounded hover:bg-white/10 ${thought.feedback === 'down' ? 'text-red-400' : 'text-muted-foreground'}`}
                      >
                        <ThumbsDown className="w-3 h-3" />
                      </button>
                    </>
                  )}
                </div>
              </div>

              {/* Content Ultra Compact */}
              <div className="p-3 text-[11px] font-sans leading-snug text-muted-foreground/80">
                {thought.status === "thinking" ? (
                  <div className="flex items-center gap-2 text-yellow-500/70">
                    <Terminal className="w-3 h-3 animate-spin" />
                    <span>Generating insight...</span>
                  </div>
                ) : (
                  thought.content
                )}
              </div>
            </div>
            
            {/* Flèche de flux vers le suivant (sauf dernier) */}
            {index < thoughts.length - 1 && (
                 <div className="absolute -bottom-4 left-1/2 -translate-x-1/2 text-white/10">
                    <ArrowDown className="w-3 h-3" />
                 </div>
            )}
          </div>
        ))}
      </div>
    </div>
  );
}

function getAgentColorBg(name) {
  switch (name?.toLowerCase()) {
    case "logicien": return "bg-blue-500";
    case "créatif": return "bg-purple-500";
    case "critique": return "bg-orange-500";
    case "chercheur": return "bg-emerald-500";
    case "fusion context a": return "bg-cyan-500";
    case "fusion context b": return "bg-pink-500";
    default: return "bg-gray-500";
  }
}

function getAgentColorText(name) {
  switch (name?.toLowerCase()) {
    case "logicien": return "text-blue-400";
    case "créatif": return "text-purple-400";
    case "critique": return "text-orange-400";
    case "chercheur": return "text-emerald-400";
    case "fusion context a": return "text-cyan-400";
    case "fusion context b": return "text-pink-400";
    default: return "text-gray-400";
  }
}