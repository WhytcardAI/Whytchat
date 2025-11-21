import { motion } from "framer-motion";
import { useTranslation } from "react-i18next";
import { BrainCircuit } from "lucide-react";

/**
 * Component displaying an animated thinking indicator.
 * Used when the AI is generating a response or processing a request.
 *
 * @component
 */
export default function ThinkingIndicator() {
  const { t } = useTranslation();

  return (
    <div className="flex items-center gap-3 px-4 py-3">
      <div className="relative flex items-center justify-center w-8 h-8">
        {/* Pulsing background circle */}
        <motion.div
          className="absolute inset-0 bg-primary/20 rounded-full"
          animate={{
            scale: [1, 1.5, 1],
            opacity: [0.5, 0, 0.5],
          }}
          transition={{
            duration: 2,
            repeat: Infinity,
            ease: "easeInOut",
          }}
        />
        
        {/* Icon with subtle rotation/pulse */}
        <motion.div
          animate={{
            rotate: [0, 5, -5, 0],
          }}
          transition={{
            duration: 4,
            repeat: Infinity,
            ease: "easeInOut",
          }}
        >
          <BrainCircuit className="w-5 h-5 text-primary" />
        </motion.div>
      </div>

      <div className="flex flex-col gap-0.5">
        <span className="text-sm font-medium text-foreground/80">
          {t("chat.orchestrator.thinking")}
        </span>
        <div className="flex gap-1">
          {[0, 1, 2].map((i) => (
            <motion.div
              key={i}
              className="w-1 h-1 rounded-full bg-primary/60"
              animate={{
                y: [0, -3, 0],
                opacity: [0.4, 1, 0.4],
              }}
              transition={{
                duration: 0.8,
                repeat: Infinity,
                delay: i * 0.2,
                ease: "easeInOut",
              }}
            />
          ))}
        </div>
      </div>
    </div>
  );
}