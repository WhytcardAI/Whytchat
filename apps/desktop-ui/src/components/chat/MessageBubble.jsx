import React from 'react';
import { User, Sparkles } from 'lucide-react';
import { cn } from '../../lib/utils';

export const MessageBubble = React.memo(function MessageBubble({ role, content }) {
  const isUser = role === 'user';

  // Simple markdown-like formatting
  const formatContent = (text) => {
    if (!text) return null;

    // Split by code blocks first
    const parts = text.split(/(```[\s\S]*?```)/g);

    return parts.map((part, index) => {
      // Code block
      if (part.startsWith('```') && part.endsWith('```')) {
        const codeContent = part.slice(3, -3);
        const firstLine = codeContent.indexOf('\n');
        const language = firstLine > 0 ? codeContent.slice(0, firstLine).trim() : '';
        const code = firstLine > 0 ? codeContent.slice(firstLine + 1) : codeContent;

        return (
          <pre key={index} className="my-3 p-4 bg-background/80 rounded-xl overflow-x-auto border border-border/50">
            {language && (
              <div className="text-[10px] text-muted mb-2 font-mono uppercase tracking-wider">{language}</div>
            )}
            <code className="text-sm font-mono text-text">{code}</code>
          </pre>
        );
      }

      // Inline code
      const withInlineCode = part.split(/(`[^`]+`)/g).map((segment, i) => {
        if (segment.startsWith('`') && segment.endsWith('`')) {
          return (
            <code key={i} className="px-1.5 py-0.5 bg-background/60 rounded text-sm font-mono text-primary">
              {segment.slice(1, -1)}
            </code>
          );
        }
        return segment;
      });

      return <span key={index}>{withInlineCode}</span>;
    });
  };

  return (
    <div className={cn(
      "flex w-full mb-4 animate-fade-in",
      isUser ? "justify-end" : "justify-start"
    )}>
      <div className={cn(
        "flex max-w-[85%] lg:max-w-3xl gap-3",
        isUser ? "flex-row-reverse" : "flex-row"
      )}>
        {/* Avatar */}
        <div className={cn(
          "w-9 h-9 rounded-xl flex items-center justify-center shrink-0 shadow-md",
          "transition-transform duration-200 hover:scale-105",
          isUser
            ? "bg-gradient-to-br from-primary to-primary/80 text-white"
            : "bg-gradient-to-br from-accent to-accent/80 text-white"
        )}>
          {isUser ? <User size={18} /> : <Sparkles size={18} />}
        </div>

        {/* Content */}
        <div className={cn(
          "flex-1 p-4 rounded-2xl text-sm leading-relaxed",
          "transition-all duration-200",
          isUser
            ? "bg-primary/15 text-text rounded-tr-sm border border-primary/20"
            : "bg-surface text-text rounded-tl-sm border border-border shadow-sm"
        )}>
          <div className="whitespace-pre-wrap break-words">
            {formatContent(content)}
          </div>
        </div>
      </div>
    </div>
  );
});
