import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import { invoke } from '@tauri-apps/api/core';
import { toast } from 'react-hot-toast';
import { logger } from '../lib/logger';

// Global app state store
export const useAppStore = create(
  persist(
    (set, get) => ({
    // UI State
    currentView: 'knowledge', // 'knowledge' | 'chat'
    setView: function(view) {
      logger.navigation.viewChange(get().currentView, view);
      return set({ currentView: view });
    },
    isSidebarOpen: true,
    toggleSidebar: function() {
      return set(function(state) {
        logger.navigation.toggleSidebar(!state.isSidebarOpen);
        return { isSidebarOpen: !state.isSidebarOpen };
      });
    },
    // isRightSidebarOpen removed in favor of Knowledge View

    // Theme State
    theme: 'light', // 'light' | 'dark'
    toggleTheme: function() {
      return set(function(state) {
        const newTheme = state.theme === 'light' ? 'dark' : 'light';
        logger.system.themeChange(newTheme);
        if (newTheme === 'dark') {
          document.documentElement.classList.add('dark');
        } else {
          document.documentElement.classList.remove('dark');
        }
        return { theme: newTheme };
      });
    },
    setTheme: function(theme) {
      logger.system.themeChange(theme);
      if (theme === 'dark') {
        document.documentElement.classList.add('dark');
      } else {
        document.documentElement.classList.remove('dark');
      }
      return set({ theme: theme });
    },

    // Session Creation Wizard State
    isCreatingSession: false,
    setIsCreatingSession: function(isOpen) {
      if (isOpen) {
        logger.navigation.openModal('SessionWizard');
      } else {
        logger.navigation.closeModal('SessionWizard');
      }
      return set({ isCreatingSession: isOpen });
    },

    // Session State
    sessions: [],
    sessionFiles: [],
    currentSessionId: null,
    setCurrentSessionId: function(id) {
      logger.session.select(id);
      // When changing session, ensure we reset thinking state and load files
      if (id) {
        get().loadSessionFiles(id);
      } else {
        set({ sessionFiles: [] });
      }
      return set({
        currentSessionId: id,
        isThinking: false,
        thinkingSteps: []
      });
    },
    setSessions: function(sessions) {
      return set({ sessions: sessions });
    },

    // Load sessions from backend
    loadSessions: function() {
      return new Promise(function(resolve, reject) {
        invoke('list_sessions').then(function(sessions) {
          logger.session.load(sessions.length);
          set({ sessions: sessions });
          resolve();
        }).catch(function(error) {
          logger.store.error('loadSessions', error);
          get().showError('Failed to load sessions.');
          reject(error);
        });
      });
    },

    loadSessionFiles: function(sessionId) {
      return new Promise(function(resolve, reject) {
        invoke('get_session_files', { session_id: sessionId, sessionId: sessionId }).then(function(files) {
          logger.store.action('loadSessionFiles', { sessionId, count: files.length });
          set({ sessionFiles: files });
          resolve(files);
        }).catch(function(error) {
          logger.store.error('loadSessionFiles', error);
          reject(error);
        });
      });
    },

    libraryFiles: [],
    loadLibraryFiles: function() {
      return new Promise(function(resolve, reject) {
        invoke('list_library_files').then(function(files) {
          logger.store.action('loadLibraryFiles', { count: files.length });
          set({ libraryFiles: files });
          resolve(files);
        }).catch(function(error) {
          logger.store.error('loadLibraryFiles', error);
          reject(error);
        });
      });
    },

    uploadFile: function(sessionId, file) {
      logger.file.upload(file.name, sessionId);
      return new Promise(function(resolve, reject) {
        const reader = new FileReader();
        reader.onload = async function(e) {
          try {
            const content = new Uint8Array(e.target.result);
            // Convert Uint8Array to regular array for Tauri
            const fileData = Array.from(content);

            await invoke('upload_file_for_session', {
              session_id: sessionId,
              sessionId: sessionId,
              file_name: file.name,
              fileName: file.name,
              file_data: fileData,
              fileData: fileData
            });

            logger.file.uploadSuccess(file.name);
            // Reload files
            await get().loadSessionFiles(sessionId);
            resolve();
          } catch (error) {
            logger.file.uploadError(file.name, error);
            get().showError('Failed to upload file: ' + (error.message || error));
            reject(error);
          }
        };
        reader.onerror = function(error) {
          logger.file.uploadError(file.name, error);
          reject(error);
        };
        reader.readAsArrayBuffer(file);
      });
    },

    deleteFile: function(fileId) {
      logger.file.delete(fileId);
      return new Promise(function(resolve, reject) {
        invoke('delete_file', { file_id: fileId, fileId: fileId }).then(function() {
          set(function(state) {
            return { libraryFiles: state.libraryFiles.filter(function(f) { return f.id !== fileId; }) };
          });
          // Also reload session files if we have a current session
          if (get().currentSessionId) {
            get().loadSessionFiles(get().currentSessionId).catch(function(err) { logger.store.error('loadSessionFiles', err); });
          }
          resolve();
        }).catch(function(error) {
          logger.store.error('deleteFile', error);
          get().showError('Failed to delete file.');
          reject(error);
        });
      });
    },

    reindexLibrary: function() {
      logger.file.reindex();
      return new Promise(function(resolve, reject) {
        invoke('reindex_library').then(function(result) {
          logger.file.reindexComplete(result);
          get().loadLibraryFiles().catch(function(err) { logger.store.error('loadLibraryFiles', err); });
          resolve(result);
        }).catch(function(error) {
          logger.store.error('reindexLibrary', error);
          get().showError('Failed to reindex library.');
          reject(error);
        });
      });
    },

    // Create new session
    createSession: function(title, language, systemPrompt, temperature) {
      logger.session.create(title);
      return new Promise(function(resolve, reject) {
        invoke('create_session', {
          title: title,
          language: language,
          system_prompt: systemPrompt,
          systemPrompt: systemPrompt,
          temperature: temperature
        }).then(function(sessionId) {
          logger.session.createSuccess(sessionId);
          // Reload sessions to get the new one
          get().loadSessions().then(function() {
            set({ currentSessionId: sessionId });
            resolve(sessionId);
          }).catch(reject);
        }).catch(function(error) {
          logger.store.error('createSession', error);
          get().showError('Failed to create session.');
          reject(error);
        });
      });
    },

    updateSession: function(sessionId, title, modelConfig) {
      logger.store.action('updateSession', { sessionId, title });
      return new Promise(function(resolve, reject) {
        invoke('update_session', {
          session_id: sessionId,
          sessionId: sessionId,
          title: title,
          model_config: modelConfig,
          modelConfig: modelConfig
        }).then(function() {
          // Reload sessions to get updated data
          get().loadSessions().then(resolve).catch(reject);
        }).catch(function(error) {
          logger.store.error('updateSession', error);
          get().showError('Failed to update session.');
          reject(error);
        });
      });
    },

    // Toggle favorite status
    toggleFavorite: function(sessionId) {
      return new Promise(function(resolve, reject) {
        // Send both snake_case and camelCase to satisfy Tauri bindings
        invoke('toggle_session_favorite', { session_id: sessionId, sessionId: sessionId }).then(function(isFavorite) {
          logger.session.favorite(sessionId, isFavorite);
          // Update local state
          set(function(state) {
            return {
              sessions: state.sessions.map(function(s) {
                return s.id === sessionId ? { ...s, is_favorite: isFavorite } : s;
              }).sort(function(a, b) {
                // Sort: favorites first, then by updated_at
                if (a.is_favorite !== b.is_favorite) return b.is_favorite ? 1 : -1;
                return (b.updated_at || b.created_at) - (a.updated_at || a.created_at);
              })
            };
          });
          resolve(isFavorite);
        }).catch(function(error) {
          logger.store.error('toggleFavorite', error);
          get().showError('Failed to update favorite.');
          reject(error);
        });
      });
    },

    // Delete a session
    deleteSession: function(sessionId) {
      logger.session.delete(sessionId);
      if (!sessionId) {
        logger.store.error('deleteSession', 'Invalid session ID');
        return Promise.reject('Invalid session ID');
      }

      return new Promise(function(resolve, reject) {
        // Try both snake_case and camelCase to be safe with Tauri 2.0 bindings
        invoke('delete_session', { sessionId: sessionId, session_id: sessionId }).then(function() {
          set(function(state) {
            // Remove from sessions list
            var newSessions = state.sessions.filter(function(s) { return s.id !== sessionId; });
            // If deleted session was current, select another or null
            var newCurrentId = state.currentSessionId === sessionId
              ? (newSessions.length > 0 ? newSessions[0].id : null)
              : state.currentSessionId;
            return {
              sessions: newSessions,
              currentSessionId: newCurrentId
            };
          });
          resolve();
        }).catch(function(error) {
          logger.store.error('deleteSession', error);
          var errorMsg = typeof error === 'string' ? error : (error.message || 'Unknown error');
          get().showError('Failed to delete session: ' + errorMsg);
          reject(error);
        });
      });
    },

    // Folders
    folders: [],
    loadFolders: function() {
      return new Promise(function(resolve, reject) {
        invoke('list_folders').then(function(folders) {
          logger.store.action('loadFolders', { count: folders.length });
          set({ folders: folders });
          resolve();
        }).catch(function(error) {
          logger.store.error('loadFolders', error);
          reject(error);
        });
      });
    },

    createFolder: function(name, color, folderType) {
      logger.store.action('createFolder', { name, folderType });
      return new Promise(function(resolve, reject) {
        invoke('create_folder', { name: name, color: color, folder_type: folderType, folderType: folderType }).then(function(folder) {
          set(function(state) {
            return { folders: state.folders.concat([folder]) };
          });
          resolve(folder);
        }).catch(function(error) {
          logger.store.error('createFolder', error);
          get().showError('Failed to create folder.');
          reject(error);
        });
      });
    },

    deleteFolder: function(folderId) {
      logger.store.action('deleteFolder', { folderId });
      return new Promise(function(resolve, reject) {
        // Send both snake_case and camelCase to satisfy Tauri bindings
        invoke('delete_folder', { folder_id: folderId, folderId: folderId }).then(function() {
          set(function(state) {
            return { folders: state.folders.filter(function(f) { return f.id !== folderId; }) };
          });
          // Reload sessions as their folder_id may have changed
          get().loadSessions().then(resolve).catch(reject);
          // Reload library files as their folder_id may have changed
          get().loadLibraryFiles().catch(function(err) { logger.store.error('loadLibraryFiles', err); });
        }).catch(function(error) {
          logger.store.error('deleteFolder', error);
          get().showError('Failed to delete folder.');
          reject(error);
        });
      });
    },

    moveSessionToFolder: function(sessionId, folderId) {
      logger.session.moveToFolder(sessionId, folderId);
      return new Promise(function(resolve, reject) {
        // Send both snake_case and camelCase to satisfy Tauri bindings
        invoke('move_session_to_folder', {
          session_id: sessionId,
          sessionId: sessionId,
          folder_id: folderId,
          folderId: folderId
        }).then(function() {
          // Update local state
          set(function(state) {
            return {
              sessions: state.sessions.map(function(s) {
                return s.id === sessionId ? { ...s, folder_id: folderId } : s;
              })
            };
          });
          resolve();
        }).catch(function(error) {
          logger.store.error('moveSessionToFolder', error);
          get().showError('Failed to move session.');
          reject(error);
        });
      });
    },

    moveFileToFolder: function(fileId, folderId) {
      logger.store.action('moveFileToFolder', { fileId, folderId });
      return new Promise(function(resolve, reject) {
        invoke('move_file_to_folder', {
          file_id: fileId,
          fileId: fileId,
          folder_id: folderId,
          folderId: folderId
        }).then(function() {
          // Update local state
          set(function(state) {
            return {
              libraryFiles: state.libraryFiles.map(function(f) {
                return f.id === fileId ? { ...f, folder_id: folderId } : f;
              })
            };
          });
          resolve();
        }).catch(function(error) {
          logger.store.error('moveFileToFolder', error);
          get().showError('Failed to move file.');
          reject(error);
        });
      });
    },

    // Thinking State (Orchestration)
    isThinking: false,
    thinkingSteps: [],
    setThinking: function(isThinking) {
      return set({ isThinking: isThinking });
    },
    addThinkingStep: function(step) {
      return set(function(state) {
        return { thinkingSteps: state.thinkingSteps.concat([step]) };
      });
    },
    clearThinkingSteps: function() {
      return set({ thinkingSteps: [] });
    },

    // Onboarding & Configuration
    isConfigured: false, // Set to true once model is downloaded
    setIsConfigured: function(value) {
      return set({ isConfigured: value });
    },
    completeOnboarding: function() {
      return set({ isConfigured: true });
    },

    // Backend initialization state
    isBackendInitialized: false,
    isBackendInitializing: true, // Assume initializing at start
    initializationError: null,
    initializeApp: async function() {
      try {
        logger.system.init('backend');
        set({ isBackendInitializing: true, initializationError: null });
        await invoke('initialize_app');
        logger.system.ready();
        set({ isBackendInitialized: true, isBackendInitializing: false });
      } catch (error) {
        logger.system.error('initializeApp', error);
        const errorMessage = typeof error === 'string' ? error : (error.message || JSON.stringify(error));
        set({
          isBackendInitializing: false,
          initializationError: errorMessage
        });
        get().showError('Failed to initialize backend: ' + errorMessage);
      }
    },

    // Diagnostics
    isDiagnosticsOpen: false,
    setDiagnosticsOpen: function(isOpen) {
      if (isOpen) {
        logger.navigation.openModal('Diagnostics');
      } else {
        logger.navigation.closeModal('Diagnostics');
      }
      return set({ isDiagnosticsOpen: isOpen });
    },

    // Error Handling
    error: null,
    setError: function(error) {
      return set({ error: error });
    },
    showError: function(message) {
      toast.error(message);
      set({ error: message });
    },

    // Quick Actions (for RAG/File interactions)
    quickAction: null,
    setQuickAction: function(action) {
      return set({ quickAction: action });
    },
    clearQuickAction: function() {
      return set({ quickAction: null });
    },
  }),
  {
    name: 'whytchat-storage',
    partialize: (state) => ({
      isConfigured: state.isConfigured,
      isSidebarOpen: state.isSidebarOpen,
      currentView: state.currentView,
      currentSessionId: state.currentSessionId,
      theme: state.theme
    }),
    onRehydrateStorage: () => (state) => {
      if (state && state.theme === 'dark') {
        document.documentElement.classList.add('dark');
      }
    }
  }
 )
);
