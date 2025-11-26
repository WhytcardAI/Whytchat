import { useState, useRef, useEffect } from 'react';
import { Send, Loader2 } from 'lucide-react';
import { cn } from '../../lib/utils';
import { useTranslation } from 'react-i18next';
import { logger } from '../../lib/logger';

export function ChatInput({ onSend, disabled }) {
  const [input, setInput] = useState('');
  const [isFocused, setIsFocused] = useState(false);
  const textareaRef = useRef(null);
  const { t } = useTranslation('common');

  // Auto-resize textarea
  useEffect(() => {
    if (textareaRef.current) {
      textareaRef.current.style.height = 'auto';
      textareaRef.current.style.height = Math.min(textareaRef.current.scrollHeight, 150) + 'px';
    }
  }, [input]);

  const handleSubmit = (e) => {
    e.preventDefault();
    if (!input.trim() || disabled) return;
    logger.ui.submit('ChatInput', { messageLength: input.length });
    onSend(input, false);
    setInput('');
    // Reset textarea height
    if (textareaRef.current) {
      textareaRef.current.style.height = 'auto';
    }
  };

  const handleKeyDown = (e) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      logger.ui.keypress('Enter', 'ChatInput');
      handleSubmit(e);
    }
  };

  const canSend = input.trim().length > 0 && !disabled;

  return (
    <div className="w-full max-w-3xl mx-auto px-4 pb-6">
      <form
        onSubmit={handleSubmit}
        className={cn(
          "relative rounded-2xl transition-all duration-200",
          "bg-surface/90 backdrop-blur-xl",
          "border shadow-lg",
          isFocused
            ? "border-primary/50 shadow-glow ring-2 ring-primary/20"
            : "border-border shadow-black/5"
        )}
      >
        <div className="flex items-end p-2 gap-2">
          {/* Input Area */}
          <textarea
            ref={textareaRef}
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyDown={handleKeyDown}
            onFocus={() => setIsFocused(true)}
            onBlur={() => setIsFocused(false)}
            placeholder={t('chat.placeholder', 'Send a message to WhytChat...')}
            className={cn(
              "flex-1 bg-transparent text-text py-3 px-3",
              "min-h-[48px] max-h-[150px] resize-none",
              "focus:outline-none text-sm leading-relaxed",
              "placeholder:text-muted/60",
              "scrollbar-hide"
            )}
            disabled={disabled}
            rows={1}
          />

          {/* Send Button */}
          <button
            type="submit"
            disabled={!canSend}
            className={cn(
              "p-3 rounded-xl transition-all duration-200",
              "focus:outline-none focus:ring-2 focus:ring-primary/30",
              canSend
                ? "bg-primary text-primary-foreground dark:text-zinc-900 shadow-lg shadow-primary/30 hover:opacity-90 hover:shadow-glow active:scale-95"
                : "bg-muted/20 text-muted cursor-not-allowed"
            )}
          >
            {disabled ? (
              <Loader2 size={20} className="animate-spin" />
            ) : (
              <Send size={20} className={cn(canSend && "translate-x-0.5")} />
            )}
          </button>
        </div>

        {/* Character hint */}
        {input.length > 0 && (
          <div className="absolute -bottom-5 right-4 text-[10px] text-muted/60">
            {input.length} {t('chat.characters', 'chars')}
          </div>
        )}
      </form>

      {/* Keyboard hint */}
      <p className="text-center text-[10px] text-muted/50 mt-3">
        {t('chat.hint', 'Press Enter to send, Shift+Enter for new line')}
      </p>
    </div>
  );
}

