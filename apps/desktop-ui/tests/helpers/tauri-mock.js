/**
 * Tauri Mock Setup for Playwright E2E Tests
 *
 * This module provides mock implementations of Tauri APIs for testing
 * in a browser environment where the actual Tauri runtime is not available.
 *
 * Key features:
 * - Full IPC invoke mocking for Tauri commands
 * - Event system support (listen/emit/unlisten)
 * - Session and file management simulation
 * - Chat streaming with token events
 */

/**
 * Injects Tauri mock APIs into the page context.
 * Call this in beforeEach BEFORE navigating in Playwright tests.
 *
 * @param {import('@playwright/test').Page} page - Playwright page object
 */
export async function setupTauriMock(page) {
  // Set initial state in localStorage to skip onboarding
  await page.addInitScript(() => {
    window.localStorage.setItem('whytchat-storage', JSON.stringify({
      state: {
        isConfigured: true,
        isSidebarOpen: true,
        currentView: 'chat',
        currentSessionId: null,
        theme: 'light'
      },
      version: 0
    }));
  });

  await page.addInitScript(() => {
    // =========================================
    // Mock Data Storage
    // =========================================
    const mockSessions = [];
    let sessionCounter = 0;
    const mockLibraryFiles = [];
    const mockFolders = [];

    // =========================================
    // Event System Mock
    // =========================================
    // Registry: Map<eventName, Set<handlerId>>
    const eventListenerRegistry = new Map();
    let callbackIdCounter = 0;

    /**
     * Emit a mock event to all registered listeners
     * @param {string} eventName - The event name (e.g., 'chat-token')
     * @param {any} payload - The event payload
     */
    function emitMockEvent(eventName, payload) {
      const handlers = eventListenerRegistry.get(eventName);
      if (handlers && handlers.size > 0) {
        handlers.forEach(handlerId => {
          const callback = window[`_${handlerId}`];
          if (typeof callback === 'function') {
            console.log(`[Mock Event] Emitting "${eventName}" to handler ${handlerId}`, payload);
            callback({ event: eventName, payload, id: Date.now() });
          }
        });
      } else {
        console.log(`[Mock Event] No listeners for "${eventName}"`, payload);
      }
    }

    // Expose emitMockEvent globally for debugging
    window.__TAURI_MOCK__ = { emitMockEvent, eventListenerRegistry };

    // Mock preflight report
    const mockPreflightReport = {
      all_passed: true,
      ready_to_start: true,
      needs_onboarding: false,
      summary: 'All systems operational',
      checks: [
        { name: 'directories', passed: true, message: 'All directories exist' },
        { name: 'model_file', passed: true, message: 'Model file found' },
        { name: 'llama_server_binary', passed: true, message: 'Inference server ready' },
        { name: 'database', passed: true, message: 'Database connected' },
        { name: 'embeddings', passed: true, message: 'Embeddings model loaded' },
      ],
    };

    // =========================================
    // __TAURI_INTERNALS__ Setup (Core Tauri Runtime Mock)
    // =========================================
    window.__TAURI_INTERNALS__ = window.__TAURI_INTERNALS__ || {};

    // Metadata for getCurrentWindow (Tauri v2)
    window.__TAURI_INTERNALS__.metadata = {
      currentWindow: { label: 'main' },
      currentWebview: { label: 'main', windowLabel: 'main' }
    };

    /**
     * Transform a callback function into a stored reference
     * Tauri uses this to pass callbacks from JS to Rust
     */
    window.__TAURI_INTERNALS__.transformCallback = function(callback, once = false) {
      const id = callbackIdCounter++;
      const wrappedCallback = once
        ? (result) => {
            callback(result);
            delete window[`_${id}`];
          }
        : callback;
      window[`_${id}`] = wrappedCallback;
      return id;
    };

    // Mock event internals for unlisten
    window.__TAURI_INTERNALS__.event = {
      unregisterListener: (id) => {
        console.log(`[Mock Event] Unregistering listener ${id}`);
        // Remove from all event registries
        eventListenerRegistry.forEach((handlers) => {
          handlers.delete(id);
        });
      }
    };

    // Main invoke mock
    window.__TAURI_INTERNALS__.invoke = async (command, args = {}) => {
      console.log(`[MockInvoke] ${command}`, JSON.stringify(args));

      switch (command) {
        // Preflight & Initialization
        case 'check_preflight':
        case 'run_quick_preflight_check':
          return mockPreflightReport;

        case 'initialize_app':
          return { success: true, status: 'ok' };

        case 'check_model_exists':
          return true;

        case 'get_initialization_status':
          return { initialized: true, error: null };

        case 'get_app_config':
          return { theme: 'dark', language: 'en' };

        // Sessions
        case 'list_sessions':
          return mockSessions;

        case 'create_session': {
          const session = {
            id: `session-${++sessionCounter}`,
            title: args.title || 'New Session',
            created_at: Date.now(),
            updated_at: Date.now(),
            model_config: {
              model_id: 'mock-model',
              temperature: args.temperature || 0.7,
              system_prompt: args.systemPrompt || args.system_prompt || '',
            },
            is_favorite: false,
            folder_id: null,
            sort_order: 0,
          };
          mockSessions.push(session);
          return session.id;
        }

        case 'get_session_messages':
          // Return empty array for new sessions
          return [];

        case 'delete_session': {
          const sessionId = args.sessionId || args.session_id;
          const index = mockSessions.findIndex(s => s.id === sessionId);
          if (index > -1) mockSessions.splice(index, 1);
          return null;
        }

        case 'toggle_session_favorite': {
          const sessionId = args.sessionId || args.session_id;
          const session = mockSessions.find(s => s.id === sessionId);
          if (session) {
            session.is_favorite = !session.is_favorite;
            return session.is_favorite;
          }
          return false;
        }

        case 'update_session':
          return null;

        case 'move_session_to_folder':
          return null;

        // Chat
        case 'debug_chat': {
          // Simulate streaming response by emitting tokens via the event system
          const response = `mock response`;

          // Emit tokens with delays to simulate streaming
          // The response is split into tokens and emitted one by one
          setTimeout(() => {
            const tokens = response.split(' ');
            let tokenIndex = 0;

            function emitNextToken() {
              if (tokenIndex < tokens.length) {
                const token = tokens[tokenIndex] + (tokenIndex < tokens.length - 1 ? ' ' : '');
                emitMockEvent('chat-token', token);
                tokenIndex++;
                setTimeout(emitNextToken, 30);
              } else {
                // Signal end of stream with empty token
                emitMockEvent('chat-token', '');
                console.log('[Mock Chat] Stream complete');
              }
            }

            emitNextToken();
          }, 100);

          return { success: true };
        }

        // Files & Library
        case 'list_library_files':
          return mockLibraryFiles;

        case 'get_session_files':
          return [];

        case 'upload_file_for_session': {
          const file = {
            id: `file-${Date.now()}`,
            name: args.fileName || args.file_name,
            path: `/mock/path/${args.fileName || args.file_name}`,
            file_type: 'text/plain',
            size: (args.fileData || args.file_data || []).length,
            created_at: Date.now(),
            folder_id: null,
          };
          mockLibraryFiles.push(file);
          return file;
        }

        case 'delete_file': {
          const fileId = args.fileId || args.file_id;
          const index = mockLibraryFiles.findIndex(f => f.id === fileId);
          if (index > -1) mockLibraryFiles.splice(index, 1);
          return null;
        }

        case 'reindex_library':
          return { indexed: mockLibraryFiles.length, errors: 0 };

        case 'move_file_to_folder':
          return null;

        // Folders
        case 'list_folders':
          return mockFolders;

        case 'create_folder': {
          const folder = {
            id: `folder-${Date.now()}`,
            name: args.name,
            color: args.color || '#6366f1',
            sort_order: 0,
            created_at: Date.now(),
            folder_type: args.folderType || args.folder_type || 'session',
          };
          mockFolders.push(folder);
          return folder;
        }

        case 'delete_folder': {
          const folderId = args.folderId || args.folder_id;
          const index = mockFolders.findIndex(f => f.id === folderId);
          if (index > -1) mockFolders.splice(index, 1);
          return null;
        }

        // Diagnostics
        case 'run_diagnostics':
          return {
            total_tests: 10,
            passed: 10,
            failed: 0,
            total_duration_ms: 1234,
            results: [],
            categories: [],
          };

        case 'run_category_tests':
        case 'run_diagnostic_category':
          return [{ name: 'test_mock', passed: true, duration_ms: 10, category: args.category }];

        // Model
        case 'download_model':
          return { success: true };

        case 'get_download_progress':
          return { progress: 100, status: 'complete' };

        default:
          // Handle plugin commands
          if (command.startsWith('plugin:window|')) {
            if (command.endsWith('is_maximized')) return false;
            if (command.endsWith('is_minimized')) return false;
            if (command.endsWith('is_fullscreen')) return false;
            if (command.endsWith('is_focused')) return true;
            if (command.endsWith('is_visible')) return true;
            return null;
          }

          // Event system plugin commands
          if (command.startsWith('plugin:event|')) {
            const eventCommand = command.replace('plugin:event|', '');

            if (eventCommand === 'listen') {
              // Register event listener
              // args contains: { event: string, handler: number }
              const eventName = args.event;
              const handlerId = args.handler;

              console.log(`[Mock Event] Registering listener for "${eventName}" with handler ${handlerId}`);

              if (!eventListenerRegistry.has(eventName)) {
                eventListenerRegistry.set(eventName, new Set());
              }
              eventListenerRegistry.get(eventName).add(handlerId);

              // Return a listener ID (can be the same as handler for simplicity)
              return handlerId;
            }

            if (eventCommand === 'unlisten') {
              // Unregister event listener
              const listenerId = args.event; // In Tauri, the event arg contains the listener ID
              console.log(`[Mock Event] Unregistering listener ${listenerId}`);
              eventListenerRegistry.forEach((handlers) => {
                handlers.delete(listenerId);
              });
              return null;
            }

            if (eventCommand === 'emit') {
              // Emit event to all listeners
              const eventName = args.event;
              const payload = args.payload;
              emitMockEvent(eventName, payload);
              return null;
            }

            return null;
          }

          console.warn(`[Tauri Mock] Unhandled command: ${command}`);
          return null;
      }
    };

    // Set up window.__TAURI__ for compatibility
    // Note: @tauri-apps/api v2 primarily uses __TAURI_INTERNALS__
    // but we expose __TAURI__ for any direct access patterns
    window.__TAURI__ = {
      core: {
        invoke: window.__TAURI_INTERNALS__.invoke
      },
      event: {
        listen: async (eventName, handler) => {
          // This version delegates to the IPC system
          const handlerId = window.__TAURI_INTERNALS__.transformCallback(handler);
          return window.__TAURI_INTERNALS__.invoke('plugin:event|listen', {
            event: eventName,
            handler: handlerId
          }).then(() => {
            return () => {
              eventListenerRegistry.forEach((handlers) => {
                handlers.delete(handlerId);
              });
              delete window[`_${handlerId}`];
            };
          });
        },
        emit: async (eventName, payload) => {
          return window.__TAURI_INTERNALS__.invoke('plugin:event|emit', {
            event: eventName,
            payload
          });
        },
      },
    };

    console.log('[Tauri Mock] Initialized successfully');
  });
}

/**
 * Waits for the app to be fully initialized after Tauri mock setup.
 *
 * @param {import('@playwright/test').Page} page - Playwright page object
 * @param {number} timeout - Timeout in milliseconds
 */
export async function waitForAppReady(page, timeout = 30000) {
  // Wait for either chat input or onboarding to appear
  await page.waitForSelector('#chat-input, [data-testid="onboarding"]', {
    timeout,
  });
}
