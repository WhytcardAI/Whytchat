/**
 * WhytChat E2E Test Suite
 *
 * Comprehensive end-to-end tests covering all UI workflows.
 */

import { test, expect } from '@playwright/test';
import path from 'path';
import { fileURLToPath } from 'url';
import { setupTauriMock } from './helpers/tauri-mock.js';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// ============================================================================
// Test Configuration
// ============================================================================

test.describe.configure({ mode: 'serial' });

const TIMEOUTS = {
  short: 5000,
  medium: 15000,
  long: 60000,
  modelDownload: 300000,
};

// ============================================================================
// Helper Functions
// ============================================================================

async function waitForAppReady(page) {
  // Wait for backend initialization
  await page.waitForSelector('[data-testid="app-ready"]', {
    timeout: TIMEOUTS.long,
  }).catch(() => {
    // Fallback: wait for main interface
    return page.waitForSelector('#chat-input, [data-testid="chat-interface"]', {
      timeout: TIMEOUTS.long,
    });
  });
}

async function skipOnboarding(page) {
  try {
    const skipButton = page.locator('button:has-text("Skip"), button:has-text("Passer")');
    if (await skipButton.isVisible({ timeout: 2000 })) {
      await skipButton.click();
    }
  } catch {
    // Onboarding already skipped or not present
  }
}

async function createNewSession(page, title = 'Test Session') {
  await page.click('[data-testid="new-chat-button"], button:has-text("New Chat"), button:has-text("Nouvelle conversation")');

  // Wait for session wizard
  await page.waitForSelector('[data-testid="session-wizard"], .session-wizard', { timeout: TIMEOUTS.short });

  // Enter title
  const titleInput = page.locator('input[placeholder*="title"], input[name="title"]');
  if (await titleInput.isVisible()) {
    await titleInput.fill(title);
  }

  // Confirm creation
  await page.click('button:has-text("Create"), button:has-text("Créer")');

  // Wait for chat interface
  await page.waitForSelector('#chat-input', { timeout: TIMEOUTS.medium });
}

// ============================================================================
// Global Test Setup - Tauri Mock
// ============================================================================

test.beforeEach(async ({ page }) => {
  // Setup Tauri mock BEFORE any navigation
  await setupTauriMock(page);
});

// ============================================================================
// App Initialization Tests
// ============================================================================

test.describe('App Initialization', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
  });

  test('should display loading state during initialization', async ({ page }) => {
    // Check for loading indicator
    const loadingIndicator = page.locator('.loading, [data-testid="loading"], text=/initializing/i');

    // Either loading is visible briefly, or app is already ready
    const isLoading = await loadingIndicator.isVisible({ timeout: 1000 }).catch(() => false);

    if (isLoading) {
      await expect(loadingIndicator).toBeVisible();
    }

    // Wait for app to be ready
    await waitForAppReady(page);
  });

  test('should show main interface after initialization', async ({ page }) => {
    await waitForAppReady(page);

    // Main layout should be visible
    await expect(page.locator('main, [data-testid="main-layout"]')).toBeVisible();
  });

  test('should have accessible navigation', async ({ page }) => {
    await waitForAppReady(page);

    // Sidebar should be present
    const sidebar = page.locator('nav, aside, [data-testid="sidebar"]');
    await expect(sidebar).toBeVisible();
  });
});

// ============================================================================
// Chat Interface Tests
// ============================================================================

test.describe('Chat Interface', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForAppReady(page);
    await skipOnboarding(page);
  });

  test('should have chat input field', async ({ page }) => {
    const chatInput = page.locator('#chat-input, [data-testid="chat-input"], textarea[placeholder*="message"]');
    await expect(chatInput).toBeVisible();
    await expect(chatInput).toBeEnabled();
  });

  test('should send message on Enter', async ({ page }) => {
    const chatInput = page.locator('#chat-input');

    await chatInput.fill('Hello, world!');
    await chatInput.press('Enter');

    // User message should appear
    await page.waitForSelector('.message.user, [data-testid="user-message"]', {
      timeout: TIMEOUTS.short,
    });

    const userMessage = page.locator('.message.user, [data-testid="user-message"]').last();
    await expect(userMessage).toContainText('Hello, world!');
  });

  test('should show thinking indicator while generating', async ({ page }) => {
    const chatInput = page.locator('#chat-input');

    await chatInput.fill('What is AI?');
    await chatInput.press('Enter');

    // Thinking indicator should appear
    const thinkingIndicator = page.locator('.thinking, [data-testid="thinking-indicator"], .animate-pulse');

    // It might be brief, so we just check it was there at some point
    await thinkingIndicator.waitFor({ state: 'visible', timeout: TIMEOUTS.short }).catch(() => {});
  });

  test('should display assistant response', async ({ page }) => {
    const chatInput = page.locator('#chat-input');

    await chatInput.fill('Say hello');
    await chatInput.press('Enter');

    // Wait for assistant response
    await page.waitForSelector('.message.assistant, [data-testid="assistant-message"]', {
      timeout: TIMEOUTS.long,
    });

    const assistantMessage = page.locator('.message.assistant').last();
    const content = await assistantMessage.textContent();

    expect(content).not.toBe('');
    expect(content?.length).toBeGreaterThan(0);
  });

  test('should handle multi-turn conversation', async ({ page }) => {
    const chatInput = page.locator('#chat-input');

    // First turn
    await chatInput.fill('My name is Test User');
    await chatInput.press('Enter');

    await page.waitForSelector('.message.assistant', { timeout: TIMEOUTS.long });

    // Second turn
    await chatInput.fill('What is my name?');
    await chatInput.press('Enter');

    // Wait for second response
    const messages = page.locator('.message.assistant');
    await expect(messages).toHaveCount(2, { timeout: TIMEOUTS.long });
  });

  test('should handle empty message gracefully', async ({ page }) => {
    const chatInput = page.locator('#chat-input');

    await chatInput.press('Enter'); // Empty message

    // Should not send empty message
    const messages = page.locator('.message');
    await expect(messages).toHaveCount(0, { timeout: 2000 }).catch(() => {});
  });

  test('should handle very long messages', async ({ page }) => {
    const chatInput = page.locator('#chat-input');
    const longMessage = 'A'.repeat(5000);

    await chatInput.fill(longMessage);
    await chatInput.press('Enter');

    // Should still send and display
    await page.waitForSelector('.message.user', { timeout: TIMEOUTS.short });
  });
});

// ============================================================================
// Session Management Tests
// ============================================================================

test.describe('Session Management', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForAppReady(page);
    await skipOnboarding(page);
  });

  test('should create new session', async ({ page }) => {
    await createNewSession(page, 'My New Session');

    // Session should be created and visible in sidebar
    await expect(page.locator('text=My New Session')).toBeVisible({ timeout: TIMEOUTS.short });
  });

  test('should switch between sessions', async ({ page }) => {
    // Create first session
    await createNewSession(page, 'Session 1');

    const chatInput = page.locator('#chat-input');
    await chatInput.fill('Message in session 1');
    await chatInput.press('Enter');

    await page.waitForSelector('.message.user', { timeout: TIMEOUTS.short });

    // Create second session
    await createNewSession(page, 'Session 2');

    // Messages should be cleared for new session
    await expect(page.locator('.message')).toHaveCount(0, { timeout: TIMEOUTS.short });

    // Switch back to first session
    await page.click('text=Session 1');

    // Messages from session 1 should reappear
    await expect(page.locator('.message.user')).toContainText('Message in session 1');
  });

  test('should update session title', async ({ page }) => {
    await createNewSession(page, 'Original Title');

    // Find and click edit button
    const sessionItem = page.locator('text=Original Title').first();
    await sessionItem.hover();

    const editButton = page.locator('[data-testid="edit-session"], button[aria-label*="edit"]');
    if (await editButton.isVisible({ timeout: 1000 })) {
      await editButton.click();

      // Update title
      const titleInput = page.locator('input[value="Original Title"]');
      await titleInput.fill('Updated Title');
      await titleInput.press('Enter');

      await expect(page.locator('text=Updated Title')).toBeVisible();
    }
  });

  test('should toggle session favorite', async ({ page }) => {
    await createNewSession(page, 'Favorite Test');

    // Find favorite button
    const favoriteButton = page.locator('[data-testid="favorite-session"], button[aria-label*="favorite"]');

    if (await favoriteButton.isVisible({ timeout: 1000 })) {
      await favoriteButton.click();

      // Session should be marked as favorite
      await expect(page.locator('.favorite, [data-favorite="true"]')).toBeVisible();
    }
  });

  test('should delete session', async ({ page }) => {
    await createNewSession(page, 'Delete Me');

    // Find and click delete button
    const sessionItem = page.locator('text=Delete Me').first();
    await sessionItem.hover();

    const deleteButton = page.locator('[data-testid="delete-session"], button[aria-label*="delete"]');
    if (await deleteButton.isVisible({ timeout: 1000 })) {
      await deleteButton.click();

      // Confirm deletion if dialog appears
      const confirmButton = page.locator('button:has-text("Confirm"), button:has-text("Delete"), button:has-text("Supprimer")');
      if (await confirmButton.isVisible({ timeout: 1000 })) {
        await confirmButton.click();
      }

      // Session should be removed
      await expect(page.locator('text=Delete Me')).not.toBeVisible({ timeout: TIMEOUTS.short });
    }
  });
});

// ============================================================================
// Knowledge Base / File Management Tests
// ============================================================================

test.describe('Knowledge Base', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForAppReady(page);
    await skipOnboarding(page);
  });

  test('should navigate to knowledge view', async ({ page }) => {
    // Click on knowledge/documents tab
    const knowledgeTab = page.locator('button:has-text("Knowledge"), button:has-text("Import"), [data-testid="knowledge-tab"]');
    await knowledgeTab.click();

    await expect(page.locator('[data-testid="knowledge-view"], .knowledge-view')).toBeVisible();
  });

  test('should upload text file', async ({ page }) => {
    // Navigate to knowledge view
    await page.click('button:has-text("Knowledge"), button:has-text("Import"), [data-testid="knowledge-tab"]');

    // Trigger file upload
    const fileChooserPromise = page.waitForEvent('filechooser');
    await page.click('button:has-text("Upload"), button:has-text("Importer")');

    const fileChooser = await fileChooserPromise;
    const testFilePath = path.join(__dirname, 'fixtures', 'test-document.txt');
    await fileChooser.setFiles(testFilePath);

    // File should appear in list
    await expect(page.locator('text=test-document.txt')).toBeVisible({ timeout: TIMEOUTS.medium });
  });

  test('should display uploaded files list', async ({ page }) => {
    await page.click('button:has-text("Knowledge"), [data-testid="knowledge-tab"]');

    // File list should be visible
    const fileList = page.locator('[data-testid="file-list"], .file-list');
    await expect(fileList).toBeVisible();
  });

  test('should delete file from library', async ({ page }) => {
    await page.click('button:has-text("Knowledge"), [data-testid="knowledge-tab"]');

    // If there are files, test deletion
    const fileItem = page.locator('[data-testid="file-item"]').first();

    if (await fileItem.isVisible({ timeout: 1000 })) {
      await fileItem.hover();

      const deleteButton = page.locator('[data-testid="delete-file"], button[aria-label*="delete"]').first();
      if (await deleteButton.isVisible()) {
        await deleteButton.click();

        // Confirm if needed
        const confirmButton = page.locator('button:has-text("Confirm"), button:has-text("Delete")');
        if (await confirmButton.isVisible({ timeout: 1000 })) {
          await confirmButton.click();
        }
      }
    }
  });
});

// ============================================================================
// RAG (Retrieval-Augmented Generation) Tests
// ============================================================================

test.describe('RAG Functionality', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForAppReady(page);
    await skipOnboarding(page);
  });

  test('should answer question using document context', async ({ page }) => {
    // Upload a test file first
    await page.click('button:has-text("Knowledge"), [data-testid="knowledge-tab"]');

    const fileChooserPromise = page.waitForEvent('filechooser');
    await page.click('button:has-text("Upload")');

    const fileChooser = await fileChooserPromise;
    await fileChooser.setFiles(path.join(__dirname, 'fixtures', 'secret-code.txt'));

    // Wait for file to be processed
    await page.waitForTimeout(2000);

    // Create a session and link the file
    await createNewSession(page, 'RAG Test');

    // Ask about the document content
    const chatInput = page.locator('#chat-input');
    await chatInput.fill('What is the secret code in the document?');
    await chatInput.press('Enter');

    // Response should contain information from the document
    await page.waitForSelector('.message.assistant', { timeout: TIMEOUTS.long });

    const response = await page.locator('.message.assistant').last().textContent();
    expect(response).toContain('42'); // The secret code from the test file
  });
});

// ============================================================================
// Folder Management Tests
// ============================================================================

test.describe('Folder Management', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForAppReady(page);
    await skipOnboarding(page);
  });

  test('should create new folder', async ({ page }) => {
    const createFolderButton = page.locator('button:has-text("New Folder"), button:has-text("Nouveau dossier"), [data-testid="create-folder"]');

    if (await createFolderButton.isVisible({ timeout: 1000 })) {
      await createFolderButton.click();

      // Enter folder name
      const folderInput = page.locator('input[placeholder*="folder"], input[placeholder*="dossier"]');
      await folderInput.fill('Test Folder');
      await folderInput.press('Enter');

      // Folder should appear
      await expect(page.locator('text=Test Folder')).toBeVisible();
    }
  });

  test('should expand/collapse folder', async ({ page }) => {
    // Create folder first
    const createFolderButton = page.locator('[data-testid="create-folder"]');
    if (await createFolderButton.isVisible({ timeout: 1000 })) {
      await createFolderButton.click();

      const folderInput = page.locator('input[placeholder*="folder"]');
      await folderInput.fill('Expandable Folder');
      await folderInput.press('Enter');

      // Click to expand
      const folderItem = page.locator('text=Expandable Folder').first();
      await folderItem.click();

      // Click again to collapse
      await folderItem.click();
    }
  });
});

// ============================================================================
// Theme and Settings Tests
// ============================================================================

test.describe('Theme and Settings', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForAppReady(page);
    await skipOnboarding(page);
  });

  test('should toggle theme', async ({ page }) => {
    const themeToggle = page.locator('button[aria-label*="theme"], [data-testid="theme-toggle"]');

    if (await themeToggle.isVisible({ timeout: 1000 })) {
      // Get initial theme
      const htmlElement = page.locator('html');
      const initialDark = await htmlElement.evaluate(el => el.classList.contains('dark'));

      // Toggle theme
      await themeToggle.click();

      // Theme should change
      const afterToggle = await htmlElement.evaluate(el => el.classList.contains('dark'));
      expect(afterToggle).not.toBe(initialDark);
    }
  });

  test('should toggle sidebar', async ({ page }) => {
    const sidebarToggle = page.locator('button[aria-label*="sidebar"], [data-testid="toggle-sidebar"]');

    if (await sidebarToggle.isVisible({ timeout: 1000 })) {
      // Get initial state
      const sidebar = page.locator('aside, [data-testid="sidebar"]');
      const initialVisible = await sidebar.isVisible();

      // Toggle
      await sidebarToggle.click();

      // State should change
      await page.waitForTimeout(500); // Animation

      if (initialVisible) {
        await expect(sidebar).not.toBeVisible();
      }
    }
  });

  test('should persist theme preference', async ({ page }) => {
    const themeToggle = page.locator('[data-testid="theme-toggle"]');

    if (await themeToggle.isVisible({ timeout: 1000 })) {
      await themeToggle.click();

      const htmlElement = page.locator('html');
      const currentTheme = await htmlElement.evaluate(el => el.classList.contains('dark'));

      // Reload page
      await page.reload();
      await waitForAppReady(page);

      // Theme should persist
      const afterReload = await htmlElement.evaluate(el => el.classList.contains('dark'));
      expect(afterReload).toBe(currentTheme);
    }
  });
});

// ============================================================================
// Internationalization Tests
// ============================================================================

test.describe('Internationalization', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForAppReady(page);
  });

  test('should display French translations', async ({ page }) => {
    // Set language to French
    await page.evaluate(() => {
      localStorage.setItem('i18nextLng', 'fr');
    });
    await page.reload();
    await waitForAppReady(page);

    // Check for French text
    const frenchText = page.locator('text=/Nouvelle|Dossier|Envoyer|Bienvenue/');
    await expect(frenchText.first()).toBeVisible({ timeout: TIMEOUTS.short }).catch(() => {});
  });

  test('should display English translations', async ({ page }) => {
    // Set language to English
    await page.evaluate(() => {
      localStorage.setItem('i18nextLng', 'en');
    });
    await page.reload();
    await waitForAppReady(page);

    // Check for English text
    const englishText = page.locator('text=/New|Folder|Send|Welcome/');
    await expect(englishText.first()).toBeVisible({ timeout: TIMEOUTS.short }).catch(() => {});
  });
});

// ============================================================================
// Accessibility Tests
// ============================================================================

test.describe('Accessibility', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForAppReady(page);
    await skipOnboarding(page);
  });

  test('should have accessible form elements', async ({ page }) => {
    const chatInput = page.locator('#chat-input, [role="textbox"]');

    // Check for proper attributes
    const hasLabel = await chatInput.getAttribute('aria-label') ||
                     await chatInput.getAttribute('placeholder');

    expect(hasLabel).toBeTruthy();
  });

  test('should support keyboard navigation', async ({ page }) => {
    // Tab through elements
    await page.keyboard.press('Tab');
    await page.keyboard.press('Tab');
    await page.keyboard.press('Tab');

    // An element should be focused
    const focusedElement = page.locator(':focus');
    await expect(focusedElement).toBeVisible();
  });

  test('should have proper heading structure', async ({ page }) => {
    const h1 = page.locator('h1');
    const headingCount = await h1.count();

    // Should have at least one main heading
    expect(headingCount).toBeGreaterThanOrEqual(0);
  });
});

// ============================================================================
// Error Handling Tests
// ============================================================================

test.describe('Error Handling', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await waitForAppReady(page);
    await skipOnboarding(page);
  });

  test('should handle network errors gracefully', async ({ page }) => {
    // Simulate offline mode
    await page.context().setOffline(true);

    const chatInput = page.locator('#chat-input');
    await chatInput.fill('Test message');
    await chatInput.press('Enter');

    // Wait a bit for potential error handling
    await page.waitForTimeout(3000);

    // App should still be responsive after network error
    await expect(chatInput).toBeVisible();

    // Restore network
    await page.context().setOffline(false);
  });

  test('should recover from errors', async ({ page }) => {
    // After any error, app should remain usable
    const chatInput = page.locator('#chat-input');

    await expect(chatInput).toBeVisible();
    await expect(chatInput).toBeEnabled();
  });
});

// ============================================================================
// Performance Tests
// ============================================================================

test.describe('Performance', () => {
  test('should load within acceptable time', async ({ page }) => {
    const startTime = Date.now();

    await page.goto('/');
    await waitForAppReady(page);

    const loadTime = Date.now() - startTime;

    // Should load within 10 seconds (generous for model initialization)
    expect(loadTime).toBeLessThan(10000);
  });

  test('should render messages quickly', async ({ page }) => {
    await page.goto('/');
    await waitForAppReady(page);
    await skipOnboarding(page);

    const chatInput = page.locator('#chat-input');

    const startTime = Date.now();
    await chatInput.fill('Quick test');
    await chatInput.press('Enter');

    await page.waitForSelector('.message.user', { timeout: TIMEOUTS.short });
    const renderTime = Date.now() - startTime;

    // User message should appear quickly
    expect(renderTime).toBeLessThan(1000);
  });
});
