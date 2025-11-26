import { useEffect, useRef, useState } from 'react';
import { useAppStore } from '../../store/appStore';
import { useTranslation } from 'react-i18next';
import toast from 'react-hot-toast';
import { logger } from '../../lib/logger';
import {
  Upload,
  FileText,
  Trash2,
  Brain,
  Database,
  FolderOpen,
  Clock,
  HardDrive,
  Plus,
  Folder,
  ChevronRight,
  ChevronDown,
  RefreshCw
} from 'lucide-react';

export function KnowledgeView() {
  const { t } = useTranslation('common');
  const {
    libraryFiles,
    loadLibraryFiles,
    currentSessionId,
    uploadFile,
    setQuickAction,
    setView,
    folders,
    loadFolders,
    createFolder,
    deleteFolder,
    moveFileToFolder,
    deleteFile,
    reindexLibrary
  } = useAppStore();

  const fileInputRef = useRef(null);
  const [isCreateFolderOpen, setIsCreateFolderOpen] = useState(false);
  const [newFolderName, setNewFolderName] = useState('');
  const [expandedFolders, setExpandedFolders] = useState({});
  const [isReindexing, setIsReindexing] = useState(false);

  useEffect(() => {
    loadLibraryFiles();
    loadFolders();
  }, [loadLibraryFiles, loadFolders]);

  const handleUploadClick = () => {
    logger.ui.click('KnowledgeView:Upload');
    fileInputRef.current?.click();
  };

  const handleReindex = async () => {
    if (isReindexing) return;
    logger.ui.click('KnowledgeView:Reindex');

    toast((toastInstance) => (
      <div className="flex flex-col gap-2 min-w-[200px]">
        <span className="font-medium text-sm">{t('knowledge.reindex_confirm', 'This will re-process all files in your library. It may take some time. Continue?')}</span>
        <div className="flex gap-2 justify-end">
          <button
            className="px-3 py-1.5 bg-surface border border-border rounded-lg text-xs hover:bg-background transition-colors"
            onClick={() => toast.dismiss(toastInstance.id)}
          >
            {t('common.cancel', 'Cancel')}
          </button>
          <button
            className="px-3 py-1.5 bg-primary text-primary-foreground rounded-lg text-xs hover:opacity-90 transition-colors"
            onClick={() => {
              toast.dismiss(toastInstance.id);
              performReindex();
            }}
          >
            {t('common.continue', 'Continue')}
          </button>
        </div>
      </div>
    ), { duration: 8000, icon: '⚠️' });
  };

  const performReindex = async () => {
    setIsReindexing(true);
    logger.file.reindex();
    const toastId = toast.loading(t('knowledge.reindexing', 'Reindexing...'));
    try {
        const result = await reindexLibrary();
        if (result.success) {
            logger.file.reindexComplete(result);
            toast.success(result.message, { id: toastId });
        } else {
            logger.store.error('reindexLibrary', result.error);
            toast.error(t('knowledge.reindex_failed', 'Reindexing failed: ') + result.error, { id: toastId });
        }
    } catch (error) {
        logger.store.error('reindexLibrary', error);
        toast.error(t('knowledge.reindex_error', 'An error occurred during reindexing'), { id: toastId });
    } finally {
        setIsReindexing(false);
    }
  };

  const handleCreateFolder = async () => {
    if (!newFolderName.trim()) return;
    logger.ui.click('KnowledgeView:CreateFolder', { name: newFolderName });
    try {
      await createFolder(newFolderName, '#6366f1', 'document');
      setNewFolderName('');
      setIsCreateFolderOpen(false);
    } catch (error) {
      logger.store.error('createFolder', error);
    }
  };

  const handleDeleteFile = (fileId) => {
    logger.ui.click('KnowledgeView:DeleteFile', { fileId });
    toast((toastInstance) => (
      <div className="flex flex-col gap-2 min-w-[200px]">
        <span className="font-medium text-sm">{t('data.delete_confirm', 'Delete this file?')}</span>
        <div className="flex gap-2 justify-end">
          <button
            className="px-3 py-1.5 bg-surface border border-border rounded-lg text-xs hover:bg-background transition-colors"
            onClick={() => toast.dismiss(toastInstance.id)}
          >
            {t('common.cancel', 'Cancel')}
          </button>
          <button
            className="px-3 py-1.5 bg-destructive text-destructive-foreground rounded-lg text-xs hover:opacity-90 transition-colors"
            onClick={async () => {
              toast.dismiss(toastInstance.id);
              try {
                await deleteFile(fileId);
                logger.file.delete(fileId);
                toast.success(t('common.deleted', 'File deleted'));
              } catch (error) {
                logger.store.error('deleteFile', error);
                toast.error(t('common.error', 'Failed to delete file'));
              }
            }}
          >
            {t('common.delete', 'Delete')}
          </button>
        </div>
      </div>
    ), { duration: 5000, icon: '🗑️' });
  };

  const toggleFolder = (folderId) => {
    logger.ui.toggle('KnowledgeFolder', expandedFolders[folderId] ? 'collapse' : 'expand');
    setExpandedFolders(prev => ({
      ...prev,
      [folderId]: !prev[folderId]
    }));
  };

  const handleFileChange = async (e) => {
    const files = Array.from(e.target.files || []);
    if (files.length === 0) return;

    if (!currentSessionId) {
        toast.error(t('data.select_session_first', 'Please select or create a session to upload files.'));
        return;
    }

    const fileCount = files.length;
    logger.file.upload(`${fileCount} files`, currentSessionId);

    // Show loading toast
    const toastId = toast.loading(
      t('knowledge.uploading_files', 'Uploading {{count}} file(s)...', { count: fileCount })
    );

    let successCount = 0;
    let failCount = 0;

    // Upload files sequentially to avoid overwhelming the backend
    for (const file of files) {
      try {
        await uploadFile(currentSessionId, file);
        logger.file.uploadSuccess(file.name);
        successCount++;
      } catch (error) {
        logger.file.uploadError(file.name, error);
        failCount++;
      }
    }

    // Show result toast
    if (failCount === 0) {
      toast.success(
        t('knowledge.upload_success', '{{count}} file(s) uploaded successfully', { count: successCount }),
        { id: toastId }
      );
    } else if (successCount === 0) {
      toast.error(
        t('knowledge.upload_error', 'Failed to upload {{count}} file(s)', { count: failCount }),
        { id: toastId }
      );
    } else {
      toast.success(
        t('knowledge.upload_partial', '{{success}} uploaded, {{failed}} failed', { success: successCount, failed: failCount }),
        { id: toastId }
      );
    }

    loadLibraryFiles();
    if (fileInputRef.current) fileInputRef.current.value = '';
  };

  const handleAnalyze = (fileName) => {
    logger.file.analyze(fileName);
    setQuickAction({
      type: 'analyze',
      payload: fileName,
      prompt: t('rag.analyze_prompt', 'Can you analyze the file {{fileName}} and tell me what it is about?', { fileName })
    });
    setView('chat');
  };

  const formatSize = (bytes) => {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
  };

  // Filter folders for documents
  const documentFolders = folders.filter(f => f.folder_type === 'document' || f.type === 'document');

  // Group files by folder
  const filesByFolder = libraryFiles.reduce((acc, file) => {
    const folderId = file.folder_id || 'unfiled';
    if (!acc[folderId]) acc[folderId] = [];
    acc[folderId].push(file);
    return acc;
  }, {});

  const unfiledFiles = filesByFolder['unfiled'] || [];

  const renderFileItem = (file) => (
    <div key={file.id} className="p-3 hover:bg-surface/50 transition-colors flex items-center gap-3 group border-b border-border/50 last:border-0">
        <div className="w-8 h-8 rounded-lg bg-surface border border-border flex items-center justify-center text-muted group-hover:text-primary group-hover:border-primary/30 transition-colors">
            <FileText size={16} />
        </div>

        <div className="flex-1 min-w-0">
            <h4 className="font-medium text-text truncate text-sm">{file.name}</h4>
            <div className="flex items-center gap-2 text-[10px] text-muted">
                <span>{formatSize(file.size)}</span>
                <span>•</span>
                <span className="uppercase">{file.file_type.split('/').pop()}</span>
            </div>
        </div>

        <div className="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
            <button
                onClick={() => handleAnalyze(file.name)}
                className="p-1.5 hover:bg-primary/10 hover:text-primary rounded-lg transition-colors"
                title={t('rag.analyze', 'Analyze')}
            >
                <Brain size={14} />
            </button>

            <div className="relative group/menu">
                <button className="p-1.5 hover:bg-surface hover:text-text rounded-lg transition-colors">
                    <FolderOpen size={14} />
                </button>
                <div className="absolute right-0 top-full mt-1 w-48 bg-surface border border-border rounded-lg shadow-lg z-50 hidden group-hover/menu:block">
                    <div className="p-1">
                        <div className="text-xs font-medium text-muted px-2 py-1">Move to...</div>
                        <button
                            onClick={() => moveFileToFolder(file.id, null).then(loadLibraryFiles)}
                            className="w-full text-left px-2 py-1.5 text-sm hover:bg-primary/10 hover:text-primary rounded-md flex items-center gap-2"
                        >
                            <Database size={12} />
                            Root
                        </button>
                        {documentFolders.map(folder => (
                            <button
                                key={folder.id}
                                onClick={() => moveFileToFolder(file.id, folder.id).then(loadLibraryFiles)}
                                className="w-full text-left px-2 py-1.5 text-sm hover:bg-primary/10 hover:text-primary rounded-md flex items-center gap-2"
                            >
                                <Folder size={12} style={{ color: folder.color }} />
                                {folder.name}
                            </button>
                        ))}
                    </div>
                </div>
            </div>

            <button
                onClick={() => handleDeleteFile(file.id)}
                className="p-1.5 hover:bg-destructive/10 hover:text-destructive rounded-lg transition-colors"
                title={t('common.delete', 'Delete')}
            >
                <Trash2 size={14} />
            </button>
        </div>
    </div>
  );

  return (
    <div className="flex-1 h-full bg-background flex flex-col overflow-hidden">
      {/* Header */}
      <header className="h-16 border-b border-border flex items-center justify-between px-6 bg-surface">
        <div>
          <h1 className="text-xl font-bold text-text flex items-center gap-2">
            <Database className="text-primary" size={24} />
            {t('knowledge.title', 'Knowledge Base')}
          </h1>
          <p className="text-xs text-muted">{t('knowledge.subtitle', 'Manage your documents and data sources')}</p>
        </div>

        <div className="flex items-center gap-3">
            <button
                onClick={handleReindex}
                disabled={isReindexing}
                className={`flex items-center gap-2 px-3 py-2 bg-surface border border-border text-text rounded-xl hover:bg-surface/80 transition-all text-sm font-medium ${isReindexing ? 'opacity-50 cursor-not-allowed' : ''}`}
                title={t('knowledge.reindex_tooltip', 'Re-process all files to update search index')}
            >
                <RefreshCw size={16} className={isReindexing ? 'animate-spin' : ''} />
                {isReindexing ? t('knowledge.reindexing', 'Reindexing...') : t('knowledge.reindex', 'Re-index')}
            </button>
            <button
                onClick={() => setIsCreateFolderOpen(true)}
                className="flex items-center gap-2 px-3 py-2 bg-surface border border-border text-text rounded-xl hover:bg-surface/80 transition-all text-sm font-medium"
            >
                <Plus size={16} />
                {t('knowledge.new_folder', 'New Folder')}
            </button>
            <button
                onClick={handleUploadClick}
                className="flex items-center gap-2 px-4 py-2 bg-primary text-primary-foreground dark:text-zinc-900 rounded-xl hover:opacity-90 transition-all font-medium text-sm"
            >
                <Upload size={16} />
                {t('data.import', 'Import Data')}
            </button>
        </div>
      </header>

      {/* Create Folder Modal/Input */}
      {isCreateFolderOpen && (
        <div className="px-6 py-4 bg-surface border-b border-border flex items-center gap-3 animate-in slide-in-from-top-2">
            <Folder size={20} className="text-primary" />
            <input
                autoFocus
                type="text"
                placeholder="Folder Name"
                className="flex-1 bg-background border border-border rounded-lg px-3 py-1.5 text-sm focus:outline-none focus:ring-2 focus:ring-primary/50"
                value={newFolderName}
                onChange={(e) => setNewFolderName(e.target.value)}
                onKeyDown={(e) => e.key === 'Enter' && handleCreateFolder()}
            />
            <button onClick={handleCreateFolder} className="text-primary hover:underline text-sm font-medium">Create</button>
            <button onClick={() => setIsCreateFolderOpen(false)} className="text-muted hover:text-text text-sm">Cancel</button>
        </div>
      )}

      {/* Dashboard Grid */}
      <div className="flex-1 overflow-y-auto p-6">
        {/* Stats Cards */}
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-8">
             <div className="p-5 rounded-2xl bg-surface border border-border flex flex-col gap-4">
                <div className="flex items-center justify-between">
                    <div className="p-2 rounded-lg bg-blue-500/10 text-blue-500">
                        <FileText size={24} />
                    </div>
                    <span className="text-2xl font-bold text-text">{libraryFiles.length}</span>
                </div>
                <div>
                    <h3 className="font-medium text-text">{t('knowledge.stats.documents', 'Documents')}</h3>
                    <p className="text-xs text-muted">{t('knowledge.stats.documents_desc', 'Indexed in local database')}</p>
                </div>
            </div>

            <div className="p-5 rounded-2xl bg-surface border border-border flex flex-col gap-4">
                <div className="flex items-center justify-between">
                    <div className="p-2 rounded-lg bg-purple-500/10 text-purple-500">
                        <HardDrive size={24} />
                    </div>
                    <span className="text-2xl font-bold text-text">
                        {formatSize(libraryFiles.reduce((acc, file) => acc + (file.size || 0), 0))}
                    </span>
                </div>
                <div>
                    <h3 className="font-medium text-text">{t('knowledge.stats.storage', 'Local Storage')}</h3>
                    <p className="text-xs text-muted">{t('knowledge.stats.storage_desc', 'Total size on disk')}</p>
                </div>
            </div>

            <div className="p-5 rounded-2xl bg-surface border border-border flex flex-col gap-4">
                <div className="flex items-center justify-between">
                    <div className="p-2 rounded-lg bg-emerald-500/10 text-emerald-500">
                        <Brain size={24} />
                    </div>
                    <span className="text-2xl font-bold text-text">Ready</span>
                </div>
                <div>
                    <h3 className="font-medium text-text">{t('knowledge.stats.rag', 'RAG Engine')}</h3>
                    <p className="text-xs text-muted">{t('knowledge.stats.rag_desc', 'Vector database active')}</p>
                </div>
            </div>
        </div>

        {/* File Browser */}
        <div className="bg-surface rounded-2xl border border-border overflow-hidden flex flex-col">
            <div className="p-4 border-b border-border flex items-center justify-between">
                <h2 className="font-semibold text-text flex items-center gap-2">
                    <Clock size={18} className="text-muted" />
                    {t('knowledge.library', 'Library')}
                </h2>
            </div>

            <div className="flex-1 overflow-y-auto min-h-[300px]">
                {/* Folders */}
                {documentFolders.map(folder => (
                    <div key={folder.id} className="border-b border-border/50 last:border-0">
                        <div
                            className="flex items-center gap-2 p-3 hover:bg-surface/80 cursor-pointer select-none group"
                            onClick={() => toggleFolder(folder.id)}
                        >
                            {expandedFolders[folder.id] ? <ChevronDown size={16} className="text-muted" /> : <ChevronRight size={16} className="text-muted" />}
                            <Folder size={18} style={{ color: folder.color }} className="fill-current opacity-20" />
                            <span className="font-medium text-text flex-1">{folder.name}</span>
                            <span className="text-xs text-muted">{(filesByFolder[folder.id] || []).length} files</span>

                            <button
                                onClick={(e) => { e.stopPropagation(); deleteFolder(folder.id); }}
                                className="p-1.5 text-muted hover:text-destructive opacity-0 group-hover:opacity-100 transition-opacity"
                            >
                                <Trash2 size={14} />
                            </button>
                        </div>

                        {expandedFolders[folder.id] && (
                            <div className="pl-8 bg-background/30 border-t border-border/30">
                                {(filesByFolder[folder.id] || []).length === 0 ? (
                                    <div className="p-3 text-xs text-muted italic">Empty folder</div>
                                ) : (
                                    (filesByFolder[folder.id] || []).map(renderFileItem)
                                )}
                            </div>
                        )}
                    </div>
                ))}

                {/* Unfiled Files */}
                {unfiledFiles.length > 0 && (
                    <div className="border-t border-border/50">
                        <div className="p-2 bg-surface/50 text-xs font-medium text-muted uppercase tracking-wider px-4">
                            Unfiled Documents
                        </div>
                        {unfiledFiles.map(renderFileItem)}
                    </div>
                )}

                {libraryFiles.length === 0 && (
                    <div className="p-12 text-center flex flex-col items-center gap-3 text-muted">
                        <div className="w-16 h-16 rounded-full bg-surface border-2 border-dashed border-border flex items-center justify-center">
                            <Upload size={24} className="opacity-50" />
                        </div>
                        <p>{t('knowledge.no_files', 'No files in your library yet')}</p>
                        <button onClick={handleUploadClick} className="text-primary hover:underline text-sm">
                            {t('knowledge.upload_first', 'Upload your first document')}
                        </button>
                    </div>
                )}
            </div>
        </div>
      </div>

      <input
        type="file"
        ref={fileInputRef}
        onChange={handleFileChange}
        className="hidden"
        accept=".txt,.md,.csv,.json,.pdf,.docx,.doc"
        multiple
      />
    </div>
  );
}
