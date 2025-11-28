import { useState, useEffect, useCallback, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { useAppStore } from '../store/appStore';
import { useTranslation } from 'react-i18next';
import toast from 'react-hot-toast';
import { logger } from '../lib/logger';

// Global listener state - prevents duplicate listeners in React Strict Mode
let globalListenersSetup = false;
let messageHandler = null;
let thinkingHandler = null;
let unlistenFunctions = [];

// Counter for generating unique message IDs
let messageIdCounter = 0;
const generateMessageId = () => `msg_${Date.now()}_${++messageIdCounter}`;

/**
 * Hook to handle chat streaming logic, message history, and Tauri events.
 * @param {string} sessionId - The current session ID.
 * @returns {Object} - { messages, sendMessage, refreshMessages }
 */
export function useChatStream(sessionId) {
  const { t } = useTranslation();
  const [messages, setMessages] = useState([]);
  const { setThinking, addThinkingStep, clearThinkingSteps } = useAppStore();

  // Refs to store component mounted state
  const isMountedRef = useRef(true);

  // Update mounted state
  useEffect(() => {
    isMountedRef.current = true;
    return () => {
      isMountedRef.current = false;
    };
  }, []);

  // Load messages for current session
  useEffect(() => {
    let isActive = true;

    const loadMessages = async () => {
      // Clear messages immediately on session change to prevent ghosting
      setMessages([]);

      if (sessionId && isMountedRef.current) {
        try {
          // Tauri auto-converts camelCase to snake_case
          const sessionMessages = await invoke('get_session_messages', {
            sessionId: sessionId
          });
          if (isMountedRef.current && isActive) {
            const formattedMessages = sessionMessages.map((msg) => ({
              id: msg.id || generateMessageId(),
              role: msg.role,
              content: msg.content
            }));
            logger.chat.loadMessages(sessionId, formattedMessages.length);
            setMessages(formattedMessages);
          }
        } catch (error) {
          logger.chat.error(error);
          toast.error(t('chat.error_loading', 'Failed to load messages'));
        }
      }
    };

    loadMessages();

    return () => {
      isActive = false;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [sessionId]); // 't' intentionally excluded - translation changes shouldn't reload messages

  // Register handlers that will be called by global listeners
  useEffect(() => {
    // Token counter for logging
    let tokenCount = 0;

    // Update the global handlers to point to current state setters
    messageHandler = (token) => {
      tokenCount++;

      if (isMountedRef.current) {
        setMessages(prev => {
          const lastMsg = prev[prev.length - 1];
          // Ensure token is a string to prevent concatenation errors
          const safeToken = String(token || '');

          if (lastMsg && lastMsg.role === 'assistant') {
            // Append to existing assistant message
            const newContent = (lastMsg.content || '') + safeToken;

            // Log every 50 tokens to avoid spam
            if (tokenCount % 50 === 0) {
              logger.chat.receiveToken(tokenCount);
            }

            return [
              ...prev.slice(0, -1),
              { ...lastMsg, content: newContent }
            ];
          } else {
            // Create new assistant message, but only if token is not empty
            // This prevents ghost bubbles if the first token is empty
            if (safeToken && safeToken.length > 0) {
              logger.chat.streamStart(sessionId);
              return [...prev, { id: generateMessageId(), role: 'assistant', content: safeToken }];
            }
            return prev;
          }
        });
      }
    };

    thinkingHandler = (step) => {
      if (isMountedRef.current) {
        addThinkingStep(step);
      }
    };
  }, [addThinkingStep, sessionId]);

  // Setup global listeners ONCE with proper cleanup
  useEffect(() => {
    async function setupGlobalListeners() {
      if (globalListenersSetup) {
        return;
      }

      globalListenersSetup = true;
      logger.system.init('Setting up chat event listeners');

      try {
        // Setup thinking listener and store unlisten function
        const unlistenThinking = await listen('thinking-step', (event) => {
          if (thinkingHandler) {
            thinkingHandler(event.payload);
          }
        });
        unlistenFunctions.push(unlistenThinking);

        // Setup token listener and store unlisten function
        const unlistenToken = await listen('chat-token', (event) => {
          if (messageHandler) {
            messageHandler(event.payload);
          }
        });
        unlistenFunctions.push(unlistenToken);

        logger.system.init('Chat event listeners ready');
      } catch (error) {
        logger.system.error('setupListeners', error);
        globalListenersSetup = false;
      }
    }

    setupGlobalListeners();

    // Cleanup function for HMR and app unmount
    return () => {
      // Only cleanup if we're truly unmounting (not just re-rendering)
      // This is handled by checking if there are no more mounted instances
    };
  }, []);

  const sendMessage = useCallback(async (text, activeSessionId, isHidden = false) => {
      // Use provided activeSessionId (in case it was just created) or prop sessionId
      const targetSessionId = activeSessionId || sessionId;

      if (!targetSessionId) {
          logger.chat.error('No session ID provided');
          return;
      }

      // 1. Add User Message Locally (unless hidden)
      if (!isHidden) {
        const userMsg = { id: generateMessageId(), role: 'user', content: text };
        setMessages(prev => [...prev, userMsg]);
      }

      // 2. Set Thinking State (Global)
      setThinking(true);
      clearThinkingSteps();

      try {
        // 3. Call Backend (Real Thinking Mode)
        // The backend will emit 'chat-token' events for the response
        // Tauri auto-converts camelCase to snake_case
        await invoke('debug_chat', {
          sessionId: targetSessionId,
          message: text
        });

        logger.chat.streamEnd(targetSessionId, null);
        setThinking(false);

      } catch (error) {
        logger.chat.error(error);
        setMessages(prev => [...prev, { id: generateMessageId(), role: 'assistant', content: `${t('chat.error')}: ${error}` }]);
        setThinking(false);
      }
  }, [sessionId, setThinking, clearThinkingSteps, t]);

  const refreshMessages = useCallback(async () => {
      if (sessionId) {
          try {
            // Tauri auto-converts camelCase to snake_case
            const sessionMessages = await invoke('get_session_messages', {
              sessionId: sessionId
            });
            const formattedMessages = sessionMessages.map(msg => ({
              id: msg.id || generateMessageId(),
              role: msg.role,
              content: msg.content
            }));
            logger.chat.loadMessages(sessionId, formattedMessages.length);
            setMessages(formattedMessages);
          } catch (error) {
            logger.chat.error(error);
            toast.error('Failed to refresh messages');
          }
      }
  }, [sessionId]); // Note: 't' removed - use static string for error

  const addSystemMessage = useCallback((text) => {
    setMessages(prev => [...prev, { id: generateMessageId(), role: 'system', content: text }]);
  }, []);

  // Export cleanup function for testing/HMR
  const cleanupListeners = useCallback(() => {
    unlistenFunctions.forEach(unlisten => unlisten());
    unlistenFunctions = [];
    globalListenersSetup = false;
    messageHandler = null;
    thinkingHandler = null;
    logger.system.init('Chat event listeners cleaned up');
  }, []);

  return {
    messages,
    sendMessage,
    refreshMessages,
    addSystemMessage,
    cleanupListeners
  };
}
