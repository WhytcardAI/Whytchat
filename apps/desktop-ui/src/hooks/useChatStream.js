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
          // Send both snake_case and camelCase to satisfy Tauri bindings
          const sessionMessages = await invoke('get_session_messages', {
            session_id: sessionId,
            sessionId: sessionId
          });
          if (isMountedRef.current && isActive) {
            const formattedMessages = sessionMessages.map(msg => ({
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
  }, [sessionId, t]);

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
                return [...prev, { role: 'assistant', content: safeToken }];
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

  // Setup global listeners ONCE
  useEffect(() => {
    async function setupGlobalListeners() {
      if (globalListenersSetup) {
        return;
      }

      globalListenersSetup = true;
      logger.system.init('Setting up chat event listeners');

      try {
        // Setup thinking listener
        await listen('thinking-step', (event) => {
          if (thinkingHandler) {
            thinkingHandler(event.payload);
          }
        });

        // Setup token listener
        await listen('chat-token', (event) => {
          if (messageHandler) {
            messageHandler(event.payload);
          }
        });

        logger.system.init('Chat event listeners ready');
      } catch (error) {
        logger.system.error('setupListeners', error);
        globalListenersSetup = false;
      }
    }

    setupGlobalListeners();

    // No cleanup here - listeners are truly global and persist across component mounts
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
        const userMsg = { role: 'user', content: text };
        setMessages(prev => [...prev, userMsg]);
      }

      // 2. Set Thinking State (Global)
      setThinking(true);
      clearThinkingSteps();

      try {
        // 3. Call Backend (Real Thinking Mode)
        // The backend will emit 'chat-token' events for the response
        await invoke('debug_chat', {
          session_id: targetSessionId,
          message: text
        });

        logger.chat.streamEnd(targetSessionId, null);
        setThinking(false);

      } catch (error) {
        logger.chat.error(error);
        setMessages(prev => [...prev, { role: 'assistant', content: `${t('chat.error')}: ${error}` }]);
        setThinking(false);
      }
  }, [sessionId, setThinking, clearThinkingSteps, t]);

  const refreshMessages = useCallback(async () => {
      if (sessionId) {
          try {
            // Send both snake_case and camelCase to satisfy Tauri bindings
            const sessionMessages = await invoke('get_session_messages', {
              session_id: sessionId,
              sessionId: sessionId
            });
            const formattedMessages = sessionMessages.map(msg => ({
              role: msg.role,
              content: msg.content
            }));
            logger.chat.loadMessages(sessionId, formattedMessages.length);
            setMessages(formattedMessages);
          } catch (error) {
            logger.chat.error(error);
            toast.error(t('chat.error_refreshing', 'Failed to refresh messages'));
          }
      }
  }, [sessionId, t]);

  const addSystemMessage = useCallback((text) => {
    setMessages(prev => [...prev, { role: 'system', content: text }]);
  }, []);

  return {
    messages,
    sendMessage,
    refreshMessages,
    addSystemMessage
  };
}
