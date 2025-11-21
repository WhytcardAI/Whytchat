import { motion, AnimatePresence } from "framer-motion";
import { Swords } from "lucide-react";
import { useTranslation } from "react-i18next";
import { ErrorBoundary } from "react-error-boundary";
import useAppStore from "../../lib/store";
import AgentAvatar from "./AgentAvatar";

/**
 * Fallback component for DebateStage errors.
 */
function DebateErrorFallback() {
  const { t } = useTranslation();
  return (
    <div className="w-full max-w-4xl mx-auto mb-6 px-4 p-4 bg-red-500/10 border border-red-500/20 rounded-xl text-center">
      <p className="text-red-400 text-sm">
        {t("debate.error_loading") || "Error loading debate stage"}
      </p>
    </div>
  );
}

/**
 * Component displaying the debate stage with two agents and progress.
 *
 * @component
 */
function DebateStageContent() {
  const { t } = useTranslation();
  const { getCurrentConversation } = useAppStore();
  const conversation = getCurrentConversation();

  // Defensive check for nested properties
  if (!conversation?.config?.debateConfig?.isDebating) {
    return null;
  }

  const { debateConfig } = conversation.config;
  const { agentA, agentB, currentSpeaker, currentRound, rounds, topic } = debateConfig;

  // Ensure agents exist before rendering
  if (!agentA || !agentB) {
    return null;
  }

  const isSpeakerA = currentSpeaker === "A";

  return (
    <div
      className="w-full max-w-4xl mx-auto mb-6 px-4"
      role="region"
      aria-label={t("debate.mode_title")}
    >
      <motion.div
        initial={{ opacity: 0, y: -20 }}
        animate={{ opacity: 1, y: 0 }}
        className="relative bg-black/40 backdrop-blur-xl border border-white/10 rounded-3xl p-6 overflow-hidden shadow-2xl"
      >
        {/* Background Effects */}
        <div className="absolute inset-0 bg-gradient-to-r from-blue-500/5 via-transparent to-red-500/5" />

        {/* Header Info */}
        <div className="relative z-10 flex flex-col items-center mb-8">
          <div className="flex items-center gap-2 text-xs font-bold uppercase tracking-widest text-muted-foreground mb-2">
            <Swords className="w-3 h-3" aria-hidden="true" />
            <span>{t("debate.mode_title")}</span>
          </div>
          <h2 className="text-lg font-medium text-white text-center max-w-2xl leading-relaxed">
            {topic}
          </h2>
        </div>

        {/* Stage Area */}
        <div className="relative z-10 flex items-center justify-between gap-8 md:gap-16">
          {/* Agent A (Left) */}
          <AgentAvatar
            agent={agentA}
            isActive={isSpeakerA}
            side="left"
            color="blue"
            roleLabel={t("debate.role.thesis")}
          />

          {/* Center Status */}
          <div className="flex-1 flex flex-col items-center gap-3">
            <div className="flex items-center gap-2 px-3 py-1 rounded-full bg-white/5 border border-white/5">
              <span className="text-xs font-mono text-muted-foreground" aria-live="polite">
                {t("debate.round_progress", { current: currentRound, total: rounds })}
              </span>
            </div>

            {/* Progress Bar */}
            <div
              className="w-full h-1.5 bg-white/5 rounded-full overflow-hidden"
              role="progressbar"
              aria-valuenow={currentRound}
              aria-valuemin={0}
              aria-valuemax={rounds}
              aria-label={t("debate.round_progress", { current: currentRound, total: rounds })}
            >
              <motion.div
                className="h-full bg-gradient-to-r from-blue-500 via-purple-500 to-red-500"
                initial={{ width: 0 }}
                animate={{ width: `${(currentRound / rounds) * 100}%` }}
                transition={{ duration: 0.5 }}
              />
            </div>

            <AnimatePresence mode="wait">
              <motion.div
                key={currentSpeaker}
                initial={{ opacity: 0, y: 5 }}
                animate={{ opacity: 1, y: 0 }}
                exit={{ opacity: 0, y: -5 }}
                className="text-xs text-white/50 font-medium"
                aria-live="polite"
              >
                {isSpeakerA ? t("debate.turn.thesis") : t("debate.turn.antithesis")}
              </motion.div>
            </AnimatePresence>
          </div>

          {/* Agent B (Right) */}
          <AgentAvatar
            agent={agentB}
            isActive={!isSpeakerA}
            side="right"
            color="red"
            roleLabel={t("debate.role.antithesis")}
          />
        </div>
      </motion.div>
    </div>
  );
}

/**
 * Main container for the Debate Stage feature.
 * Wraps the content in an ErrorBoundary to prevent crashes from affecting the entire app.
 *
 * @component
 */
export default function DebateStage() {
  return (
    <ErrorBoundary FallbackComponent={DebateErrorFallback}>
      <DebateStageContent />
    </ErrorBoundary>
  );
}
