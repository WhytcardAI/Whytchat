import { motion } from "framer-motion";
import { Mic } from "lucide-react";
import PropTypes from "prop-types";

/**
 * Component representing an agent in the debate stage.
 *
 * @component
 * @param {Object} props
 * @param {Object} props.agent - The agent object containing name and other details.
 * @param {boolean} props.isActive - Whether the agent is currently speaking.
 * @param {string} props.color - The color theme for the agent ('blue' or 'red').
 * @param {string} props.roleLabel - The label describing the agent's role (e.g., 'Thesis').
 */
export default function AgentAvatar({ agent, isActive, color, roleLabel }) {
  const colorClasses = {
    blue: "from-blue-500 to-cyan-500 shadow-blue-500/20",
    red: "from-red-500 to-orange-500 shadow-red-500/20",
  };

  return (
    <motion.div
      className={`flex flex-col items-center gap-3 transition-all duration-500 ${isActive ? "opacity-100 scale-105" : "opacity-50 scale-95 grayscale-[0.5]"}`}
      animate={{
        y: isActive ? [0, -5, 0] : 0,
      }}
      transition={{
        y: {
          duration: 2,
          repeat: isActive ? Infinity : 0,
          ease: "easeInOut",
        },
      }}
      aria-label={`${agent.name} (${roleLabel})`}
    >
      <div className="relative">
        {/* Glow Effect when active */}
        {isActive && (
          <motion.div
            layoutId="activeGlow"
            className={`absolute inset-0 rounded-full bg-gradient-to-r ${colorClasses[color]} blur-xl opacity-40`}
            transition={{ duration: 0.3 }}
          />
        )}

        {/* Avatar Circle */}
        <div
          className={`relative w-16 h-16 md:w-20 md:h-20 rounded-full bg-gradient-to-br ${colorClasses[color]} p-0.5 shadow-lg`}
        >
          <div className="w-full h-full rounded-full bg-black/90 flex items-center justify-center overflow-hidden border-2 border-transparent">
            {/* Placeholder Icon or Initials */}
            <div className="text-xl font-bold text-white/90" aria-hidden="true">
              {agent.name.charAt(0).toUpperCase()}
            </div>
          </div>

          {/* Speaking Indicator */}
          {isActive && (
            <motion.div
              initial={{ scale: 0 }}
              animate={{ scale: 1 }}
              className="absolute -bottom-1 -right-1 bg-white text-black p-1.5 rounded-full shadow-lg z-10"
              aria-label="Speaking"
            >
              <Mic className="w-3 h-3" />
            </motion.div>
          )}
        </div>
      </div>

      <div className="text-center space-y-0.5">
        <h3 className={`text-sm font-bold ${isActive ? "text-white" : "text-muted-foreground"}`}>
          {agent.name}
        </h3>
        <p className="text-[10px] uppercase tracking-wider text-white/40 font-medium">
          {roleLabel}
        </p>
      </div>
    </motion.div>
  );
}

AgentAvatar.propTypes = {
  agent: PropTypes.shape({
    name: PropTypes.string.isRequired,
  }).isRequired,
  isActive: PropTypes.bool.isRequired,
  side: PropTypes.oneOf(["left", "right"]).isRequired,
  color: PropTypes.oneOf(["blue", "red"]).isRequired,
  roleLabel: PropTypes.string.isRequired,
};
