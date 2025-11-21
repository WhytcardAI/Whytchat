import { memo, useState, useMemo } from "react";
import PropTypes from "prop-types";
import { useTranslation } from "react-i18next";
import { ChevronDown, ChevronRight, BrainCircuit } from "lucide-react";
import { AGENT_REGEX } from "../../lib/constants";

/**
 * Component to display a single chat message.
 * Handles user and AI messages, including special formatting for debate agents and Chain of Thought.
 *
 * @component
 * @param {Object} props
 * @param {string} props.role - The role of the message sender ('user', 'assistant', 'system').
 * @param {string} props.content - The text content of the message.
 */
const MessageBubble = memo(function MessageBubble({ role, content }) {
  const { t } = useTranslation();
  const [isThinkingExpanded, setIsThinkingExpanded] = useState(true);
  const isUser = role === "user";

  // Detection of [Name]: Message format for debate mode
  const agentMatch = !isUser && content ? content.match(AGENT_REGEX) : null;
  const agentName = agentMatch ? agentMatch[1] : null;
  const rawContent = agentMatch ? agentMatch[2] : content;

  // Parse <think> tags for Chain of Thought
  const { thoughtContent, finalContent } = useMemo(() => {
    if (!rawContent) return { thoughtContent: null, finalContent: "" };

    const thinkStart = rawContent.indexOf("<think>");
    if (thinkStart === -1) return { thoughtContent: null, finalContent: rawContent };

    const thinkEnd = rawContent.indexOf("</think>");

    if (thinkEnd !== -1) {
      // Complete thought block
      return {
        thoughtContent: rawContent.substring(thinkStart + 7, thinkEnd).trim(),
        finalContent: (
          rawContent.substring(0, thinkStart) + rawContent.substring(thinkEnd + 8)
        ).trim(),
      };
    } else {
      // Streaming thought block (incomplete)
      return {
        thoughtContent: rawContent.substring(thinkStart + 7).trim(),
        finalContent: rawContent.substring(0, thinkStart).trim(),
      };
    }
  }, [rawContent]);

  return (
    <div
      className={`max-w-4xl mx-auto flex w-full ${isUser ? "justify-end" : "justify-start"} group`}
    >
      <div
        className={`
          relative max-w-[85%] sm:max-w-[75%] rounded-2xl px-5 py-3.5 text-sm leading-7 shadow-sm whitespace-pre-wrap transition-all duration-200
          ${
            isUser
              ? "bg-gradient-to-br from-primary to-orange-600 text-white rounded-tr-none shadow-lg shadow-primary/20 border border-white/10"
              : "glass text-foreground rounded-tl-none border border-white/5 hover:bg-white/5 hover:border-white/10"
          }
        `}
      >
        {/* Agent Header for debate mode */}
        {agentName && (
          <div className="mb-2 flex items-center gap-2 border-b border-white/5 pb-2">
            <div className="w-5 h-5 rounded-full bg-gradient-to-br from-blue-500 to-purple-600 flex items-center justify-center text-[10px] font-bold text-white shadow-inner">
              {agentName.charAt(0)}
            </div>
            <span className="text-xs font-bold text-primary/90 tracking-wide">{agentName}</span>
          </div>
        )}

        {/* Chain of Thought Block */}
        {thoughtContent && (
          <div className="mb-4 rounded-lg bg-black/20 border border-white/5 overflow-hidden">
            <button
              onClick={() => setIsThinkingExpanded(!isThinkingExpanded)}
              className="w-full flex items-center gap-2 px-3 py-2 text-xs font-medium text-muted-foreground hover:text-foreground hover:bg-white/5 transition-colors"
            >
              <BrainCircuit className="w-3.5 h-3.5" />
              <span>{t("chat.orchestrator.thought_process")}</span>
              {isThinkingExpanded ? (
                <ChevronDown className="w-3.5 h-3.5 ml-auto" />
              ) : (
                <ChevronRight className="w-3.5 h-3.5 ml-auto" />
              )}
            </button>
            {isThinkingExpanded && (
              <div className="px-3 py-2 text-xs font-mono text-muted-foreground/80 border-t border-white/5 bg-black/10 animate-in slide-in-from-top-1 duration-200">
                {thoughtContent}
              </div>
            )}
          </div>
        )}

        {/* Small decorative indicator for standard AI */}
        {!isUser && !agentName && (
          <div className="absolute -left-2 -top-2 w-4 h-4 rounded-full bg-white/5 backdrop-blur-md border border-white/10 flex items-center justify-center opacity-0 group-hover:opacity-100 transition-opacity duration-300">
            <div className="w-1.5 h-1.5 bg-primary rounded-full"></div>
          </div>
        )}

        <div className={isUser ? "text-white/95 font-light" : "text-gray-100/90"}>
          {finalContent || (thoughtContent ? "" : displayContent)}
        </div>
      </div>
    </div>
  );
});

MessageBubble.propTypes = {
  role: PropTypes.string.isRequired,
  content: PropTypes.string,
};

export default MessageBubble;
