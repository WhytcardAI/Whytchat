import { useRef, useState, useEffect } from 'react';
import { Upload, FileText, FolderOpen, Loader2, CheckCircle, AlertCircle } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { useAppStore } from '../../store/appStore';
import { invoke } from '@tauri-apps/api/core';
import i18n from '../../i18n';

export function FilesDropdown({ onClose: _onClose }) {
  const { t } = useTranslation('common');
  const fileInputRef = useRef(null);
  const { currentSessionId, createSession, setCurrentSessionId } = useAppStore();
  const [uploadStatus, setUploadStatus] = useState(null); // null, 'uploading', 'success', 'error'
  const [sessionFiles, setSessionFiles] = useState([]);
  const [isLoading, setIsLoading] = useState(false);

  // Load files for current session
  useEffect(() => {
    const loadSessionFiles = async () => {
      if (!currentSessionId) {
        setSessionFiles([]);
        return;
      }

      setIsLoading(true);
      try {
        const files = await invoke('get_session_files', { session_id: currentSessionId });
        setSessionFiles(files || []);
      } catch (error) {
        console.error('Failed to load session files:', error);
        setSessionFiles([]);
      } finally {
        setIsLoading(false);
      }
    };

    loadSessionFiles();
  }, [currentSessionId, uploadStatus]); // Reload when upload completes

  const handleFileSelect = async (event) => {
    const files = Array.from(event.target.files);

    // Ensure we have a session
    let sessionId = currentSessionId;
    if (!sessionId) {
      try {
        sessionId = await createSession(t('session.title.default'), i18n.language);
        setCurrentSessionId(sessionId);
      } catch (error) {
        console.error('Failed to create session for file upload:', error);
        setUploadStatus('error');
        setTimeout(() => setUploadStatus(null), 3000);
        return;
      }
    }

    setUploadStatus('uploading');

    for (const file of files) {
      try {
        // Read file as array buffer
        const arrayBuffer = await file.arrayBuffer();
        const fileData = Array.from(new Uint8Array(arrayBuffer));

        // Upload to backend
        await invoke('upload_file_for_session', {
          session_id: sessionId,
          file_name: file.name,
          file_data: fileData
        });
      } catch (error) {
        console.error('Failed to upload file:', error);
        setUploadStatus('error');
        setTimeout(() => setUploadStatus(null), 3000);
        event.target.value = '';
        return;
      }
    }

    setUploadStatus('success');
    setTimeout(() => setUploadStatus(null), 3000);
    event.target.value = '';
  };

  const getFileName = (filePath) => {
    // Extract filename from path like "session-id/filename.txt"
    const parts = filePath.split('/');
    return parts[parts.length - 1];
  };

  return (
    <div className="absolute right-0 top-full mt-2 w-72 bg-surface border border-border rounded-xl shadow-xl z-50 overflow-hidden animate-fade-in">
      {/* Header */}
      <div className="px-4 py-3 border-b border-border bg-background/50">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <FolderOpen size={16} className="text-primary" />
            <span className="font-medium text-sm">{t('header.files', 'Context Files')}</span>
          </div>
          <span className="text-xs text-muted bg-background px-2 py-0.5 rounded-full">
            {sessionFiles.length}
          </span>
        </div>
      </div>

      {/* File List */}
      <div className="max-h-48 overflow-y-auto custom-scrollbar">
        {isLoading ? (
          <div className="p-6 text-center">
            <Loader2 size={18} className="animate-spin mx-auto text-muted" />
          </div>
        ) : sessionFiles.length > 0 ? (
          <div className="p-2 space-y-1">
            {sessionFiles.map((file) => (
              <div
                key={file.id}
                className="flex items-center gap-2 p-2 rounded-lg bg-background/50 hover:bg-background group"
              >
                <FileText size={14} className="text-muted shrink-0" />
                <div className="flex-1 min-w-0">
                  <p className="text-xs font-medium truncate">{getFileName(file.file_path)}</p>
                  <p className="text-[10px] text-muted">{file.file_type}</p>
                </div>
              </div>
            ))}
          </div>
        ) : (
          <div className="p-6 text-center">
            <div className="w-10 h-10 rounded-xl bg-background/50 flex items-center justify-center mx-auto mb-2">
              <FileText size={18} className="text-muted/50" />
            </div>
            <p className="text-xs text-muted">{t('panel.no_files', 'No files attached')}</p>
          </div>
        )}
      </div>

      {/* Upload Status */}
      {uploadStatus && (
        <div className="px-3 py-2 border-t border-border">
          <div className={`flex items-center gap-2 p-2 rounded-lg text-xs ${
            uploadStatus === 'success' ? 'bg-green-500/10 text-green-500' :
            uploadStatus === 'error' ? 'bg-red-500/10 text-red-500' :
            'bg-primary/10 text-primary'
          }`}>
            {uploadStatus === 'uploading' && <Loader2 size={14} className="animate-spin" />}
            {uploadStatus === 'success' && <CheckCircle size={14} />}
            {uploadStatus === 'error' && <AlertCircle size={14} />}
            <span>
              {uploadStatus === 'uploading' ? t('upload.uploading', 'Uploading...') :
               uploadStatus === 'success' ? t('upload.success', 'File uploaded!') :
               t('upload.error', 'Upload failed')}
            </span>
          </div>
        </div>
      )}

      {/* Upload Button */}
      <div className="p-3 border-t border-border">
        <input
          ref={fileInputRef}
          type="file"
          multiple
          onChange={handleFileSelect}
          className="hidden"
          accept=".txt,.md,.json,.csv"
        />
        <button
          onClick={() => fileInputRef.current?.click()}
          disabled={uploadStatus === 'uploading'}
          className="w-full flex items-center justify-center gap-2 p-2.5 rounded-lg bg-primary/10 text-primary hover:bg-primary/20 transition-colors text-sm font-medium disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {uploadStatus === 'uploading' ? <Loader2 size={16} className="animate-spin" /> : <Upload size={16} />}
          <span>{uploadStatus === 'uploading' ? t('upload.uploading', 'Uploading...') : t('header.upload_file', 'Add File')}</span>
        </button>
      </div>
    </div>
  );
}
