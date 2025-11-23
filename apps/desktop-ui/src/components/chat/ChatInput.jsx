import React, { useState } from 'react';
import { Send, Paperclip, Bot, Swords } from 'lucide-react';
import { cn } from '../../lib/utils';
import { useTranslation } from 'react-i18next';

export function ChatInput({ onSend, onFileUpload, disabled }) {
  const [input, setInput] = useState('');
  const { t } = useTranslation('common');

  const handleSubmit = (e) => {
    e.preventDefault();
    if (!input.trim() || disabled) return;
    onSend(input, false); // Web toggle temporarily removed for visual match
    setInput('');
  };

  const handleKeyDown = (e) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      handleSubmit(e);
    }
  };

  const handleFileClick = () => {
    const input = document.createElement('input');
    input.type = 'file';
    input.onchange = (e) => {
      const file = e.target.files[0];
      if (file && onFileUpload) {
        onFileUpload(file);
      }
    };
    input.click();
  };

  return (
    <div className="w-full max-w-3xl mx-auto px-4 pb-6">
      <form
        onSubmit={handleSubmit}
        className="relative bg-surface rounded-[2rem] shadow-xl shadow-black/5 border border-border flex items-end p-2 transition-all focus-within:ring-2 focus-within:ring-primary/20"
      >
        {/* Left Actions */}
        <div className="flex items-center gap-1 pb-2 pl-2">
          <button type="button" onClick={handleFileClick} className="p-2 text-muted hover:text-text hover:bg-background rounded-full transition-colors">
            <Paperclip size={20} />
          </button>
          <button type="button" className="p-2 bg-secondary text-white rounded-xl hover:opacity-90 transition-opacity shadow-sm shadow-secondary/30">
            <Bot size={20} />
          </button>
          <button type="button" className="p-2 text-muted hover:text-text hover:bg-background rounded-full transition-colors">
            <Swords size={20} />
          </button>
        </div>

        {/* Input */}
        <textarea
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder={t('chat.placeholder', 'Envoyer un message à WhytChat...')}
          className="flex-1 bg-transparent text-text p-3 min-h-[50px] max-h-[150px] resize-none focus:outline-none text-sm scrollbar-hide mb-1"
          disabled={disabled}
          rows={1}
        />

        {/* Send Button */}
        <div className="pb-1 pr-1">
          <button
            type="submit"
            disabled={!input.trim() || disabled}
            className="p-3 bg-primary text-white rounded-full hover:bg-primary/90 disabled:opacity-50 disabled:cursor-not-allowed transition-all shadow-lg shadow-primary/30"
          >
            <Send size={20} className={cn(input.trim() && "ml-0.5")} />
          </button>
        </div>
      </form>
    </div>
  );
}
