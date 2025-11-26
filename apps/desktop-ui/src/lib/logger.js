/**
 * Centralized Logger for WhytChat Desktop UI
 *
 * Categories:
 * - UI: User interface interactions (clicks, focus, etc.)
 * - CHAT: Chat-related actions (send message, stream, etc.)
 * - SESSION: Session management (create, delete, switch)
 * - FILE: File operations (upload, delete, analyze)
 * - STORE: State management actions
 * - NAVIGATION: View changes and routing
 * - SYSTEM: App initialization, preflight, etc.
 *
 * Levels: DEBUG, INFO, WARN, ERROR
 */

const LOG_LEVELS = {
  DEBUG: 0,
  INFO: 1,
  WARN: 2,
  ERROR: 3,
};

const CATEGORY_COLORS = {
  UI: '#3B82F6',       // Blue
  CHAT: '#10B981',     // Green
  SESSION: '#8B5CF6',  // Purple
  FILE: '#F59E0B',     // Amber
  STORE: '#EC4899',    // Pink
  NAVIGATION: '#06B6D4', // Cyan
  SYSTEM: '#6B7280',   // Gray
};

// Configure minimum log level (can be changed at runtime)
let minLogLevel = import.meta.env.DEV ? LOG_LEVELS.DEBUG : LOG_LEVELS.INFO;

/**
 * Set the minimum log level
 * @param {'DEBUG' | 'INFO' | 'WARN' | 'ERROR'} level
 */
export function setLogLevel(level) {
  minLogLevel = LOG_LEVELS[level] ?? LOG_LEVELS.INFO;
}

/**
 * Format timestamp
 */
function getTimestamp() {
  return new Date().toISOString().split('T')[1].slice(0, 12);
}

/**
 * Core log function
 * @param {'DEBUG' | 'INFO' | 'WARN' | 'ERROR'} level
 * @param {string} category
 * @param {string} action
 * @param {object} [data]
 */
function log(level, category, action, data = null) {
  if (LOG_LEVELS[level] < minLogLevel) return;

  const timestamp = getTimestamp();
  const color = CATEGORY_COLORS[category] || '#888';
  const prefix = `%c[${timestamp}] [${category}] ${action}`;

  const style = `color: ${color}; font-weight: bold;`;
  const consoleMethod = console[level.toLowerCase()] || console.log;

  if (data) {
    consoleMethod(prefix, style, data);
  } else {
    consoleMethod(prefix, style);
  }
}

/**
 * Logger object with category-specific methods
 */
export const logger = {
  // ========== UI Actions ==========
  ui: {
    click: (element, details = null) =>
      log('INFO', 'UI', `Click: ${element}`, details),
    focus: (element) =>
      log('DEBUG', 'UI', `Focus: ${element}`),
    blur: (element) =>
      log('DEBUG', 'UI', `Blur: ${element}`),
    change: (element, value) =>
      log('DEBUG', 'UI', `Change: ${element}`, { value }),
    submit: (form, details = null) =>
      log('INFO', 'UI', `Submit: ${form}`, details),
    toggle: (element, state) =>
      log('INFO', 'UI', `Toggle: ${element}`, { state }),
    drag: (action, details = null) =>
      log('DEBUG', 'UI', `Drag: ${action}`, details),
    keypress: (key, element) =>
      log('DEBUG', 'UI', `Keypress: ${key} on ${element}`),
  },

  // ========== Chat Actions ==========
  chat: {
    sendMessage: (sessionId, messagePreview) =>
      log('INFO', 'CHAT', 'Send message', { sessionId, preview: messagePreview?.slice(0, 50) }),
    receiveToken: (tokenCount) =>
      log('DEBUG', 'CHAT', 'Receive token', { tokenCount }),
    streamStart: (sessionId) =>
      log('INFO', 'CHAT', 'Stream started', { sessionId }),
    streamEnd: (sessionId, totalTokens) =>
      log('INFO', 'CHAT', 'Stream ended', { sessionId, totalTokens }),
    error: (error) =>
      log('ERROR', 'CHAT', 'Chat error', { error }),
    loadMessages: (sessionId, count) =>
      log('DEBUG', 'CHAT', 'Load messages', { sessionId, count }),
  },

  // ========== Session Actions ==========
  session: {
    create: (title) =>
      log('INFO', 'SESSION', 'Create session', { title }),
    createSuccess: (sessionId) =>
      log('INFO', 'SESSION', 'Session created', { sessionId }),
    delete: (sessionId) =>
      log('INFO', 'SESSION', 'Delete session', { sessionId }),
    select: (sessionId) =>
      log('INFO', 'SESSION', 'Select session', { sessionId }),
    favorite: (sessionId, isFavorite) =>
      log('INFO', 'SESSION', 'Toggle favorite', { sessionId, isFavorite }),
    moveToFolder: (sessionId, folderId) =>
      log('INFO', 'SESSION', 'Move to folder', { sessionId, folderId }),
    load: (count) =>
      log('DEBUG', 'SESSION', 'Load sessions', { count }),
  },

  // ========== File Actions ==========
  file: {
    upload: (fileName, sessionId) =>
      log('INFO', 'FILE', 'Upload file', { fileName, sessionId }),
    uploadSuccess: (fileName) =>
      log('INFO', 'FILE', 'Upload success', { fileName }),
    uploadError: (fileName, error) =>
      log('ERROR', 'FILE', 'Upload failed', { fileName, error }),
    delete: (fileId) =>
      log('INFO', 'FILE', 'Delete file', { fileId }),
    analyze: (fileName) =>
      log('INFO', 'FILE', 'Analyze file', { fileName }),
    select: (fileName) =>
      log('DEBUG', 'FILE', 'Select file', { fileName }),
    save: (fileName) =>
      log('INFO', 'FILE', 'Save file', { fileName }),
    reindex: () =>
      log('INFO', 'FILE', 'Reindex library'),
    reindexComplete: (result) =>
      log('INFO', 'FILE', 'Reindex complete', result),
    link: (fileId, sessionId) =>
      log('INFO', 'FILE', 'Link file to session', { fileId, sessionId }),
    unlink: (fileId, sessionId) =>
      log('INFO', 'FILE', 'Unlink file from session', { fileId, sessionId }),
  },

  // ========== Store Actions ==========
  store: {
    setState: (key, value) =>
      log('DEBUG', 'STORE', `Set ${key}`, { value }),
    action: (actionName, params = null) =>
      log('DEBUG', 'STORE', `Action: ${actionName}`, params),
    error: (actionName, error) =>
      log('ERROR', 'STORE', `Action failed: ${actionName}`, { error }),
  },

  // ========== Navigation Actions ==========
  navigation: {
    viewChange: (from, to) =>
      log('INFO', 'NAVIGATION', 'View change', { from, to }),
    openModal: (modalName) =>
      log('INFO', 'NAVIGATION', 'Open modal', { modal: modalName }),
    closeModal: (modalName) =>
      log('INFO', 'NAVIGATION', 'Close modal', { modal: modalName }),
    toggleSidebar: (isOpen) =>
      log('DEBUG', 'NAVIGATION', 'Toggle sidebar', { isOpen }),
  },

  // ========== System Actions ==========
  system: {
    init: (step) =>
      log('INFO', 'SYSTEM', `Init: ${step}`),
    preflight: (status, details = null) =>
      log('INFO', 'SYSTEM', `Preflight: ${status}`, details),
    ready: () =>
      log('INFO', 'SYSTEM', 'Application ready'),
    error: (context, error) =>
      log('ERROR', 'SYSTEM', `Error in ${context}`, { error }),
    themeChange: (theme) =>
      log('DEBUG', 'SYSTEM', 'Theme change', { theme }),
  },

  // ========== Generic methods ==========
  debug: (category, message, data = null) =>
    log('DEBUG', category, message, data),
  info: (category, message, data = null) =>
    log('INFO', category, message, data),
  warn: (category, message, data = null) =>
    log('WARN', category, message, data),
  error: (category, message, data = null) =>
    log('ERROR', category, message, data),
};

export default logger;
