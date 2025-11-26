import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import { invoke } from '@tauri-apps/api/core';
import { toast } from 'react-hot-toast';

// Global app state store
export const useAppStore = create(
  persist(
    (set, get) => ({
    // UI State
    isSidebarOpen: true,
    toggleSidebar: function() {
      return set(function(state) {
        return { isSidebarOpen: !state.isSidebarOpen };
      });
    },
    isRightSidebarOpen: false,
    toggleRightSidebar: function() {
      return set(function(state) {
        return { isRightSidebarOpen: !state.isRightSidebarOpen };
      });
    },

    // Theme State
    theme: 'light', // 'light' | 'dark'
    toggleTheme: function() {
      return set(function(state) {
        const newTheme = state.theme === 'light' ? 'dark' : 'light';
        if (newTheme === 'dark') {
          document.documentElement.classList.add('dark');
        } else {
          document.documentElement.classList.remove('dark');
        }
        return { theme: newTheme };
      });
    },
    setTheme: function(theme) {
      if (theme === 'dark') {
        document.documentElement.classList.add('dark');
      } else {
        document.documentElement.classList.remove('dark');
      }
      return set({ theme: theme });
    },

    // Session State
    sessions: [],
    sessionFiles: [],
    currentSessionId: null,
    setCurrentSessionId: function(id) {
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
          set({ sessions: sessions });
          resolve();
        }).catch(function(error) {
          console.error('Failed to load sessions:', error);
          get().showError('Failed to load sessions.');
          reject(error);
        });
      });
    },

    loadSessionFiles: function(sessionId) {
      return new Promise(function(resolve, reject) {
        invoke('get_session_files', { session_id: sessionId, sessionId: sessionId }).then(function(files) {
          set({ sessionFiles: files });
          resolve(files);
        }).catch(function(error) {
          console.error('Failed to load session files:', error);
          reject(error);
        });
      });
    },

    uploadFile: function(sessionId, file) {
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

            // Reload files
            await get().loadSessionFiles(sessionId);
            resolve();
          } catch (error) {
            console.error('Failed to upload file:', error);
            get().showError('Failed to upload file: ' + (error.message || error));
            reject(error);
          }
        };
        reader.onerror = function(error) {
          reject(error);
        };
        reader.readAsArrayBuffer(file);
      });
    },

    // Create new session
    createSession: function(title, language, systemPrompt, temperature) {
      return new Promise(function(resolve, reject) {
        invoke('create_session', {
          title: title,
          language: language,
          system_prompt: systemPrompt,
          systemPrompt: systemPrompt,
          temperature: temperature
        }).then(function(sessionId) {
          // Reload sessions to get the new one
          get().loadSessions().then(function() {
            set({ currentSessionId: sessionId });
            resolve(sessionId);
          }).catch(reject);
        }).catch(function(error) {
          console.error('Failed to create session:', error);
          get().showError('Failed to create session.');
          reject(error);
        });
      });
    },

    updateSession: function(sessionId, title, modelConfig) {
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
          console.error('Failed to update session:', error);
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
          console.error('Failed to toggle favorite:', error);
          get().showError('Failed to update favorite.');
          reject(error);
        });
      });
    },

    // Delete a session
    deleteSession: function(sessionId) {
      console.log('[AppStore] Requesting delete for session:', sessionId);
      if (!sessionId) {
        console.error('[AppStore] Invalid session ID for delete');
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
          console.error('Failed to delete session:', error);
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
          set({ folders: folders });
          resolve();
        }).catch(function(error) {
          console.error('Failed to load folders:', error);
          reject(error);
        });
      });
    },

    createFolder: function(name, color) {
      return new Promise(function(resolve, reject) {
        invoke('create_folder', { name: name, color: color }).then(function(folder) {
          set(function(state) {
            return { folders: state.folders.concat([folder]) };
          });
          resolve(folder);
        }).catch(function(error) {
          console.error('Failed to create folder:', error);
          get().showError('Failed to create folder.');
          reject(error);
        });
      });
    },

    deleteFolder: function(folderId) {
      return new Promise(function(resolve, reject) {
        // Send both snake_case and camelCase to satisfy Tauri bindings
        invoke('delete_folder', { folder_id: folderId, folderId: folderId }).then(function() {
          set(function(state) {
            return { folders: state.folders.filter(function(f) { return f.id !== folderId; }) };
          });
          // Reload sessions as their folder_id may have changed
          get().loadSessions().then(resolve).catch(reject);
        }).catch(function(error) {
          console.error('Failed to delete folder:', error);
          get().showError('Failed to delete folder.');
          reject(error);
        });
      });
    },

    moveSessionToFolder: function(sessionId, folderId) {
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
          console.error('Failed to move session:', error);
          get().showError('Failed to move session.');
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
    initializeApp: async function() {
      try {
        set({ isBackendInitializing: true });
        await invoke('initialize_app');
        set({ isBackendInitialized: true, isBackendInitializing: false });
      } catch (error) {
        console.error('Failed to initialize backend:', error);
        set({ isBackendInitializing: false }); // Stop loading on error
        get().showError('Failed to initialize backend. Please restart the application.');
      }
    },

    // Diagnostics
    isDiagnosticsOpen: false,
    setDiagnosticsOpen: function(isOpen) {
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
  }),
  {
    name: 'whytchat-storage',
    partialize: (state) => ({
      isConfigured: state.isConfigured,
      isSidebarOpen: state.isSidebarOpen,
      isRightSidebarOpen: state.isRightSidebarOpen,
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
