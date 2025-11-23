import React, { useState, useRef, useEffect } from 'react';
import { MessageBubble } from './MessageBubble';
import { ThinkingBubble } from './ThinkingBubble';
import { ChatInput } from './ChatInput';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { useAppStore } from '../../store/appStore';

export function ChatInterface() {
  const [messages, setMessages] = useState([]);
  const [uploadStatus, setUploadStatus] = useState(null); // null, 'uploading', 'success', 'error'
  const { setThinking, addThinkingStep, clearThinkingSteps, isThinking, thinkingSteps } = useAppStore();
  const messagesEndRef = useRef(null);

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  };

  useEffect(() => {
    scrollToBottom();
  }, [messages]);

  // Listen for backend thinking events and streaming tokens
  useEffect(() => {
    let unlistenThinking;
    let unlistenToken;

    async function setupListeners() {
      unlistenThinking = await listen('thinking-step', (event) => {
        addThinkingStep(event.payload);
      });

      unlistenToken = await listen('chat-token', (event) => {
        const token = event.payload;
        setMessages(prev => {
          const lastMsg = prev[prev.length - 1];
          if (lastMsg && lastMsg.role === 'assistant') {
            // Append to existing assistant message
            return [
              ...prev.slice(0, -1),
              { ...lastMsg, content: lastMsg.content + token }
            ];
          } else {
            // Create new assistant message if none exists (or last was user)
            return [...prev, { role: 'assistant', content: token }];
          }
        });
      });
    }

    setupListeners();

    return () => {
      if (unlistenThinking) unlistenThinking();
      if (unlistenToken) unlistenToken();
    };
  }, []);

  const handleSend = async (text, isWebEnabled) => {
    // 1. Add User Message
    const userMsg = { role: 'user', content: text };
    setMessages(prev => [...prev, userMsg]);

    // 2. Set Thinking State (Global)
    setThinking(true);
    clearThinkingSteps();

    try {
      // 3. Call Backend (Real Thinking Mode)
      // The backend will emit 'chat-token' events for the response
      // We await the final result just to ensure completion, but the UI updates via events
      await invoke('debug_chat', { message: text });

      setThinking(false);

    } catch (error) {
      console.error("Backend Error:", error);
      setMessages(prev => [...prev, { role: 'assistant', content: `Error: ${error}` }]);
      setThinking(false);
    }
  };

  const handleFileUpload = async (file) => {
    setUploadStatus('uploading');
    try {
      const reader = new FileReader();
      reader.onload = async (e) => {
        const arrayBuffer = e.target.result;
        const fileData = Array.from(new Uint8Array(arrayBuffer));
        await invoke('upload_file_for_session', {
          session_id: 'default-session',
          file_name: file.name,
          file_data: fileData
        });
        setUploadStatus('success');
        setTimeout(() => setUploadStatus(null), 3000); // Reset after 3 seconds
      };
      reader.onerror = () => {
        setUploadStatus('error');
        setTimeout(() => setUploadStatus(null), 3000);
      };
      reader.readAsArrayBuffer(file);
    } catch (error) {
      console.error('Upload error:', error);
      setUploadStatus('error');
      setTimeout(() => setUploadStatus(null), 3000);
    }
  };

  return (
    <div className="flex flex-col h-full w-full relative">
      {/* Messages Area */}
      <div className="flex-1 overflow-y-auto p-4 space-y-6 scrollbar-thin scrollbar-thumb-border scrollbar-track-transparent pb-32">
        {messages.length === 0 && (
          <div className="h-full flex flex-col items-center justify-center text-muted opacity-50">
            <p>Commencez une nouvelle conversation.</p>
          </div>
        )}

        {messages.map((msg, idx) => (
          <MessageBubble key={idx} role={msg.role} content={msg.content} />
        ))}

        {/* Thinking Bubble - Shows during generation or if steps exist */}
        {(isThinking || thinkingSteps.length > 0) && (
          <ThinkingBubble steps={thinkingSteps} />
        )}

        <div ref={messagesEndRef} />
      </div>

      {/* Input Area (Floating) */}
      <div className="absolute bottom-0 left-0 right-0 bg-gradient-to-t from-background via-background to-transparent pt-10 z-10">
        {uploadStatus && (
          <div className={`mb-2 text-center text-sm ${uploadStatus === 'success' ? 'text-green-500' : uploadStatus === 'error' ? 'text-red-500' : 'text-blue-500'}`}>
            {uploadStatus === 'uploading' ? 'Uploading...' : uploadStatus === 'success' ? 'File uploaded successfully' : 'Upload error'}
          </div>
        )}
        <ChatInput onSend={handleSend} onFileUpload={handleFileUpload} disabled={isThinking} />
        <ChatInput onSend={handleSend} disabled={isThinking} />
      </div>
    </div>
  );
}
