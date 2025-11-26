import { useRef } from 'react';
import { useAppStore } from '../../store/appStore';
import { Upload, FolderPlus, FileText, Trash2, Brain, FileSearch } from 'lucide-react';
import { useTranslation } from 'react-i18next';

export function DataSidebar() {
  const { t } = useTranslation('common');
  const { sessionFiles, uploadFile, currentSessionId, setQuickAction } = useAppStore();
  const fileInputRef = useRef(null);

  const handleUploadClick = () => {
    fileInputRef.current?.click();
  };

  const handleFileChange = async (e) => {
    const file = e.target.files?.[0];
    if (!file || !currentSessionId) return;

    try {
      await uploadFile(currentSessionId, file);
    } catch (error) {
      console.error("Upload failed", error);
    }
    // Reset input
    if (fileInputRef.current) fileInputRef.current.value = '';
  };

  const handleAnalyze = (fileName) => {
    setQuickAction({
      type: 'analyze',
      payload: fileName,
      prompt: t('rag.analyze_prompt', 'Can you analyze the file {{fileName}} and tell me what it is about?', { fileName })
    });
  };

  const handleSummarize = (fileName) => {
    setQuickAction({
      type: 'summarize',
      payload: fileName,
      prompt: t('rag.summarize_prompt', 'Please provide a concise summary of {{fileName}}.', { fileName })
    });
  };

  return (
    <aside className="w-80 bg-surface/80 backdrop-blur-xl border-l border-border flex flex-col shrink-0 h-full transition-all duration-300">
      <div className="p-4 border-b border-border/50">
        <h2 className="font-semibold text-text mb-1">{t('data.title', 'Data & Knowledge')}</h2>
        <p className="text-xs text-muted">{t('data.subtitle', 'Manage session context')}</p>
      </div>

      <div className="p-3 grid grid-cols-2 gap-2">
        <button
          onClick={handleUploadClick}
          className="flex flex-col items-center justify-center gap-2 p-3 rounded-xl bg-primary/10 text-primary hover:bg-primary/20 transition-colors border border-primary/20"
        >
          <Upload size={20} />
          <span className="text-xs font-medium">{t('data.import', 'Import')}</span>
        </button>
        <button className="flex flex-col items-center justify-center gap-2 p-3 rounded-xl bg-surface border border-border hover:bg-surface/80 transition-colors text-muted hover:text-text">
          <FolderPlus size={20} />
          <span className="text-xs font-medium">{t('data.new_group', 'New Group')}</span>
        </button>
      </div>

      <div className="flex-1 overflow-y-auto p-3 space-y-4 custom-scrollbar">
        {/* Files Section */}
        <div className="space-y-2">
          <h3 className="text-[10px] font-semibold text-muted uppercase tracking-wider px-1">
            {t('data.files', 'Files')} ({sessionFiles.length})
          </h3>

          {sessionFiles.length === 0 ? (
            <div className="text-center py-8 text-muted/50 text-xs border-2 border-dashed border-border/50 rounded-xl">
              {t('data.no_files', 'No files uploaded')}
            </div>
          ) : (
            <div className="space-y-1">
              {sessionFiles.map((file, index) => {
                const fileName = (file.file_path || file.path || '').split('/').pop();
                return (
                  <div key={file.id || index} className="group flex items-center gap-3 p-2 rounded-lg hover:bg-surface/80 transition-colors border border-transparent hover:border-border/50">
                    <div className="w-8 h-8 rounded-lg bg-blue-500/10 text-blue-500 flex items-center justify-center shrink-0">
                      <FileText size={16} />
                    </div>
                    <div className="flex-1 min-w-0">
                      <p className="text-sm font-medium text-text truncate">{fileName}</p>
                      <p className="text-[10px] text-muted truncate">{file.file_type}</p>
                    </div>

                    {/* Quick Actions */}
                    <div className="flex items-center opacity-0 group-hover:opacity-100 transition-opacity">
                      <button
                        onClick={() => handleAnalyze(fileName)}
                        title={t('rag.analyze', 'Analyze')}
                        className="p-1.5 text-muted hover:text-primary transition-colors"
                      >
                        <Brain size={14} />
                      </button>
                      <button
                        onClick={() => handleSummarize(fileName)}
                        title={t('rag.summarize', 'Summarize')}
                        className="p-1.5 text-muted hover:text-primary transition-colors"
                      >
                        <FileSearch size={14} />
                      </button>
                      <button className="p-1.5 text-muted hover:text-destructive transition-colors">
                        <Trash2 size={14} />
                      </button>
                    </div>
                  </div>
                );
              })}
            </div>
          )}
        </div>

        {/* Groups Placeholder */}
        <div className="space-y-2">
           <h3 className="text-[10px] font-semibold text-muted uppercase tracking-wider px-1">
            {t('data.groups', 'Groups')}
          </h3>
          <div className="text-center py-4 text-muted/40 text-xs italic">
            {t('data.no_groups', 'No groups created')}
          </div>
        </div>
      </div>

      <input
        type="file"
        ref={fileInputRef}
        onChange={handleFileChange}
        className="hidden"
        accept=".txt,.md,.csv,.json"
      />
    </aside>
  );
}
