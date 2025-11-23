import { User, Bot } from 'lucide-react';
import { cn } from '../../lib/utils';

export function MessageBubble({ role, content }) {
  const isUser = role === 'user';

  return (
    <div className={cn(
      "flex w-full mb-6",
      isUser ? "justify-end" : "justify-start"
    )}>
      <div className={cn(
        "flex max-w-3xl w-full gap-4",
        isUser ? "flex-row-reverse" : "flex-row"
      )}>
        {/* Avatar */}
        <div className={cn(
          "w-8 h-8 rounded-full flex items-center justify-center shrink-0",
          isUser ? "bg-primary text-white" : "bg-accent text-white"
        )}>
          {isUser ? <User size={16} /> : <Bot size={16} />}
        </div>

        {/* Content */}
        <div className={cn(
          "flex-1 p-4 rounded-2xl text-sm leading-relaxed shadow-sm",
          isUser
            ? "bg-primary/10 text-slate-100 rounded-tr-none"
            : "bg-surface text-slate-200 rounded-tl-none border border-slate-700"
        )}>
          <div className="whitespace-pre-wrap">{content}</div>
        </div>
      </div>
    </div>
  );
}
