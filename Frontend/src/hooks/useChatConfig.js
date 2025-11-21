import { useState, useEffect } from "react";

/**
 * Hook to manage chat configuration panels visibility.
 *
 * @param {string} conversationId - The ID of the current conversation.
 */
export function useChatConfig(conversationId) {
  const [showConfig, setShowConfig] = useState(false);
  const [showDebateConfig, setShowDebateConfig] = useState(false);

  // Close config panels when switching conversations
  useEffect(() => {
    setShowConfig(false); // eslint-disable-line react-hooks/set-state-in-effect
    setShowDebateConfig(false);
  }, [conversationId]);

  return {
    showConfig,
    setShowConfig,
    showDebateConfig,
    setShowDebateConfig,
  };
}
