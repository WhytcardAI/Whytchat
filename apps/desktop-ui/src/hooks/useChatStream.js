import { useState, useEffect, useCallback, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { useAppStore } from '../store/appStore';
import { useTranslation } from 'react-i18next';

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
            setMessages(formattedMessages);
          }
        } catch (error) {
          console.error('Failed to load messages:', error);
        }
      }
    };

    loadMessages();

    return () => {
      isActive = false;
    };
  }, [sessionId]);

  // Register handlers that will be called by global listeners
  useEffect(() => {
    // Update the global handlers to point to current state setters
    messageHandler = (token) => {
      if (isMountedRef.current) {
        setMessages(prev => {
          const lastMsg = prev[prev.length - 1];
          if (lastMsg && lastMsg.role === 'assistant') {
            // Append to existing assistant message
            return [
              ...prev.slice(0, -1),
              { ...lastMsg, content: lastMsg.content + token }
            ];
          } else {
            // Create new assistant message
            return [...prev, { role: 'assistant', content: token }];
          }
        });
      }
    };

    thinkingHandler = (step) => {
      if (isMountedRef.current) {
        addThinkingStep(step);
      }
    };
  }, [addThinkingStep]);

  // Setup global listeners ONCE
  useEffect(() => {
    async function setupGlobalListeners() {
      if (globalListenersSetup) {
        console.log('[DEBUG] Global listeners already setup, skipping');
        return;
      }

      console.log('[DEBUG] Setting up global listeners (first time)');
      globalListenersSetup = true;

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

        console.log('[DEBUG] Global listeners set up successfully');

      } catch (error) {
        console.error('Error setting up listeners:', error);
        globalListenersSetup = false;
      }
    }

    setupGlobalListeners();

    // No cleanup here - listeners are truly global and persist across component mounts
  }, []);

  const sendMessage = useCallback(async (text, activeSessionId) => {
      // Use provided activeSessionId (in case it was just created) or prop sessionId
      const targetSessionId = activeSessionId || sessionId;

      if (!targetSessionId) {
          console.error("Cannot send message: No session ID provided.");
          return;
      }

      // 1. Add User Message Locally
      const userMsg = { role: 'user', content: text };
      setMessages(prev => [...prev, userMsg]);

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

        setThinking(false);

      } catch (error) {
        console.error("Backend Error:", error);
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
            setMessages(formattedMessages);
          } catch (error) {
            console.error('Failed to refresh messages:', error);
          }
      }
  }, [sessionId]);

  return {
    messages,
    sendMessage,
    refreshMessages
  };
}
