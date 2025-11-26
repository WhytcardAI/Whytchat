import React, { useState } from 'react';
import { User, Sparkles, Save, Check, X, FileText } from 'lucide-react';
import { cn } from '../../lib/utils';
import { invoke } from '@tauri-apps/api/core';
import { useTranslation } from 'react-i18next';
import toast from 'react-hot-toast';
import { logger } from '../../lib/logger';

export const MessageBubble = React.memo(function MessageBubble({ role, content, sessionId }) {
  const isUser = role === 'user';
  const isSystem = role === 'system';
  const { t } = useTranslation();

  if (isSystem) {
    return (
      <div className="flex w-full mb-4 justify-center animate-fade-in">
        <div className="bg-surface/50 border border-border/50 rounded-full px-4 py-1.5 flex items-center gap-2 text-xs text-muted">
          <Sparkles size={12} className="text-primary/50" />
          <span>{content}</span>
        </div>
      </div>
    );
  }

  // Helper to process inline formatting (bold, code)
  const processInline = (text) => {
    // Split by inline code first
    return text.split(/(`[^`]+`)/g).map((segment, i) => {
      if (segment.startsWith('`') && segment.endsWith('`')) {
        return (
          <code key={i} className="px-1.5 py-0.5 bg-background/60 rounded text-sm font-mono text-primary">
            {segment.slice(1, -1)}
          </code>
        );
      }

      // Then bold
      const withBold = segment.split(/(\*\*[^*]+\*\*)/g).map((subSegment, j) => {
        if (subSegment.startsWith('**') && subSegment.endsWith('**')) {
          return <strong key={j} className="font-bold text-foreground">{subSegment.slice(2, -2)}</strong>;
        }
        return subSegment;
      });

      return <span key={i}>{withBold}</span>;
    });
  };

  // Enhanced markdown-like formatting
  const formatContent = (text) => {
    if (!text) return <span className="text-destructive">Empty content</span>;

    try {
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
          <CodeBlock
            key={index}
            language={language}
            code={code}
            sessionId={sessionId}
            t={t}
          />
        );
      }

      // Text block - process line by line for headers/lists
      const lines = part.split('\n');
      return (
        <div key={index} className="block w-full">
          {lines.map((line, lineIndex) => {
             const trimmed = line.trim();

             // Headers
             if (trimmed.startsWith('### ')) {
                 return <h3 key={lineIndex} className="text-base font-semibold mt-4 mb-2 text-primary">{processInline(trimmed.slice(4))}</h3>;
             }
             if (trimmed.startsWith('## ')) {
                 return <h2 key={lineIndex} className="text-lg font-bold mt-6 mb-3 text-foreground">{processInline(trimmed.slice(3))}</h2>;
             }
             if (trimmed.startsWith('# ')) {
                 return <h1 key={lineIndex} className="text-xl font-bold mt-6 mb-4 text-foreground">{processInline(trimmed.slice(2))}</h1>;
             }

             // Lists (Bullet)
             if (trimmed.startsWith('- ') || trimmed.startsWith('* ')) {
                 return (
                    <div key={lineIndex} className="flex gap-2 ml-2 my-1">
                        <span className="text-primary/70">•</span>
                        <span>{processInline(trimmed.slice(2))}</span>
                    </div>
                 );
             }

             // Numbered Lists
             const numMatch = trimmed.match(/^(\d+)\.\s/);
             if (numMatch) {
                 return (
                    <div key={lineIndex} className="flex gap-2 ml-2 my-1">
                        <span className="text-primary/70 font-mono text-xs pt-1">{numMatch[1]}.</span>
                        <span>{processInline(trimmed.slice(numMatch[0].length))}</span>
                    </div>
                 );
             }

             // Blockquotes
             if (trimmed.startsWith('> ')) {
                 return (
                     <div key={lineIndex} className="border-l-2 border-primary/40 pl-4 py-1 my-2 text-muted italic bg-surface/50 rounded-r">
                         {processInline(trimmed.slice(2))}
                     </div>
                 );
             }

             // Empty lines
             if (trimmed === '') {
                 return <div key={lineIndex} className="h-2" />;
             }

             // Regular text
             return <div key={lineIndex} className="min-h-[1.5em]">{processInline(line)}</div>;
          })}
        </div>
      );
    });
    } catch (err) {
      logger.system.error('formatContent', err);
      return <div className="text-destructive">Error formatting message: {err.message}</div>;
    }
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
          "w-8 h-8 rounded-lg flex items-center justify-center shrink-0 border",
          "transition-transform duration-200",
          isUser
            ? "bg-surface border-border text-foreground"
            : "bg-primary border-primary text-primary-foreground"
        )}>
          {isUser ? <User size={16} /> : <Sparkles size={16} />}
        </div>

        {/* Content */}
        <div className={cn(
          "flex-1 p-4 rounded-2xl text-sm leading-relaxed group relative min-w-0",
          "transition-all duration-200",
          isUser
            ? "bg-muted/30 text-foreground rounded-tr-sm"
            : "bg-surface text-foreground rounded-tl-sm border border-border shadow-sm"
        )}>
          <div className="break-words">
            {content ? formatContent(content) : <span className="text-muted italic">...</span>}
          </div>

          {/* Message Actions (Assistant Only) */}
          {!isUser && (
            <div className="mt-2 pt-2 border-t border-border/50 flex justify-end opacity-0 group-hover:opacity-100 transition-opacity">
               <MessageActions content={content} sessionId={sessionId} t={t} />
            </div>
          )}
        </div>
      </div>
    </div>
  );
});

function MessageActions({ content, sessionId, t }) {
    const [isSaving, setIsSaving] = useState(false);
    const [showInput, setShowInput] = useState(false);
    const [filename, setFilename] = useState('note.md');
    const [saveStatus, setSaveStatus] = useState(null);

    const handleSave = async () => {
        if (!filename || !sessionId) return;
        logger.ui.click('MessageActions:Save', { filename });
        setIsSaving(true);
        try {
            await invoke('save_generated_file', {
                session_id: sessionId,
                file_name: filename,
                content: content
            });
            logger.file.save(filename);
            setSaveStatus('success');
            toast.success(t('common.saved', 'Saved successfully'));
            setTimeout(() => {
                setSaveStatus(null);
                setShowInput(false);
            }, 2000);
        } catch (error) {
            logger.file.uploadError(filename, error);
            setSaveStatus('error');
            toast.error(t('common.error_saving', 'Failed to save file'));
            setTimeout(() => setSaveStatus(null), 2000);
        } finally {
            setIsSaving(false);
        }
    };

    if (showInput) {
        return (
            <div className="flex items-center gap-2 animate-fade-in bg-background/50 p-1 rounded-lg">
                <input
                    type="text"
                    value={filename}
                    onChange={(e) => setFilename(e.target.value)}
                    className="h-6 px-2 text-xs bg-background border border-border rounded focus:outline-none focus:border-primary w-32"
                    placeholder="filename.md"
                    autoFocus
                    onKeyDown={(e) => e.key === 'Enter' && handleSave()}
                />
                <button
                    onClick={handleSave}
                    disabled={isSaving}
                    className={cn(
                        "p-1 rounded transition-colors",
                        saveStatus === 'success' ? "text-success bg-success/10" :
                        saveStatus === 'error' ? "text-destructive bg-destructive/10" :
                        "hover:bg-success/10 text-success"
                    )}
                    title={t('common.save')}
                >
                    {isSaving ? <span className="animate-spin">⌛</span> : saveStatus === 'success' ? <Check size={14} /> : saveStatus === 'error' ? <X size={14} /> : <Check size={14} />}
                </button>
                <button
                    onClick={() => setShowInput(false)}
                    className="p-1 hover:bg-destructive/10 text-destructive rounded transition-colors"
                    title={t('common.cancel')}
                >
                    <X size={14} />
                </button>
            </div>
        );
    }

    return (
        <button
            onClick={() => setShowInput(true)}
            className={cn(
                "flex items-center gap-1.5 px-2 py-1 rounded text-xs font-medium transition-colors",
                saveStatus === 'success' ? "text-success bg-success/10" :
                saveStatus === 'error' ? "text-destructive bg-destructive/10" :
                "text-muted hover:text-primary hover:bg-primary/10"
            )}
            title={t('chat.message.save_as_doc', 'Save as Document')}
        >
            {saveStatus === 'success' ? (
                <>
                    <Check size={12} />
                    <span>{t('common.saved', 'Saved')}</span>
                </>
            ) : saveStatus === 'error' ? (
                <>
                    <X size={12} />
                    <span>{t('common.error', 'Error')}</span>
                </>
            ) : (
                <>
                    <FileText size={12} />
                    <span>{t('common.save_as_doc', 'Save as Doc')}</span>
                </>
            )}
        </button>
    );
}

function CodeBlock({ language, code, sessionId, t }) {
  const [isSaving, setIsSaving] = useState(false);
  const [saveStatus, setSaveStatus] = useState(null); // 'success', 'error'
  const [showInput, setShowInput] = useState(false);
  const [filename, setFilename] = useState('');

  const handleStartSave = () => {
    // Try to guess filename from code or language
    let guess = `snippet.${language || 'txt'}`;
    // Simple heuristic: look for "filename: xxx" or comments
    const firstLine = code.split('\n')[0];
    if (firstLine.includes('filename:')) {
        guess = firstLine.split('filename:')[1].trim();
    } else if (firstLine.startsWith('//') || firstLine.startsWith('#')) {
        const parts = firstLine.split(' ');
        if (parts.length > 1 && parts[1].includes('.')) {
            guess = parts[1];
        }
    }

    setFilename(guess);
    setShowInput(true);
  };

  const handleConfirmSave = async () => {
    if (!filename || !sessionId) return;
    logger.ui.click('CodeBlock:Save', { filename });
    setIsSaving(true);
    try {
      await invoke('save_generated_file', {
        session_id: sessionId,
        file_name: filename,
        content: code
      });
      logger.file.save(filename);
      setSaveStatus('success');
      toast.success(t('common.saved', 'Saved successfully'));
      setTimeout(() => {
          setSaveStatus(null);
          setShowInput(false);
      }, 2000);
    } catch (error) {
      logger.file.uploadError(filename, error);
      setSaveStatus('error');
      toast.error(t('common.error_saving', 'Failed to save file'));
      setTimeout(() => setSaveStatus(null), 2000);
    } finally {
      setIsSaving(false);
    }
  };

  return (
    <div className="my-3 rounded-xl overflow-hidden border border-border/50 bg-background/80">
      <div className="flex items-center justify-between px-4 py-2 bg-surface/50 border-b border-border/50">
        <div className="text-[10px] text-muted font-mono uppercase tracking-wider">
          {language || 'text'}
        </div>
        <div className="flex items-center gap-2">
            {showInput ? (
                <div className="flex items-center gap-2 animate-fade-in">
                    <input
                        type="text"
                        value={filename}
                        onChange={(e) => setFilename(e.target.value)}
                        className="h-6 px-2 text-xs bg-background border border-border rounded focus:outline-none focus:border-primary w-32"
                        placeholder="filename.ext"
                        autoFocus
                        onKeyDown={(e) => e.key === 'Enter' && handleConfirmSave()}
                    />
                    <button
                        onClick={handleConfirmSave}
                        disabled={isSaving}
                        className="p-1 hover:bg-success/10 text-success rounded transition-colors"
                        title={t('common.save')}
                    >
                        {isSaving ? <span className="animate-spin">⌛</span> : <Check size={14} />}
                    </button>
                    <button
                        onClick={() => setShowInput(false)}
                        className="p-1 hover:bg-destructive/10 text-destructive rounded transition-colors"
                        title={t('common.cancel')}
                    >
                        <X size={14} />
                    </button>
                </div>
            ) : (
                <button
                    onClick={handleStartSave}
                    className={cn(
                        "flex items-center gap-1.5 px-2 py-1 rounded text-xs font-medium transition-colors",
                        saveStatus === 'success' ? "text-success bg-success/10" :
                        saveStatus === 'error' ? "text-destructive bg-destructive/10" :
                        "text-muted hover:text-primary hover:bg-primary/10"
                    )}
                    title={t('chat.message.save_to_library', 'Save to Library')}
                >
                    {saveStatus === 'success' ? (
                        <>
                            <Check size={12} />
                            <span>{t('common.saved', 'Saved')}</span>
                        </>
                    ) : saveStatus === 'error' ? (
                        <>
                            <X size={12} />
                            <span>{t('common.error', 'Error')}</span>
                        </>
                    ) : (
                        <>
                            <Save size={12} />
                            <span>{t('common.save', 'Save')}</span>
                        </>
                    )}
                </button>
            )}
        </div>
      </div>
      <pre className="p-4 overflow-x-auto custom-scrollbar">
        <code className="text-sm font-mono text-text">{code}</code>
      </pre>
    </div>
  );
}
