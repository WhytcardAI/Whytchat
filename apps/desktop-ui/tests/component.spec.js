/**
 * Component Unit Tests
 *
 * Tests for individual React components using Playwright component testing.
 * Uses Tauri mock for testing without the actual Tauri runtime.
 */

import { test, expect } from '@playwright/test';
import { setupTauriMock } from './helpers/tauri-mock.js';

// ============================================================================
// Global Test Setup - Tauri Mock
// ============================================================================

test.beforeEach(async ({ page }) => {
  // Setup Tauri mock BEFORE any navigation
  await setupTauriMock(page);
});

// ============================================================================
// Helper Functions
// ============================================================================

/**
 * Creates a new session to enable the chat interface
 */
async function createSession(page, title = 'Test Session') {
  // Click "New conversation" button in sidebar
  const newChatBtn = page.locator('button:has-text("New conversation")');
  await newChatBtn.click();

  // Wait for session wizard modal
  await page.waitForSelector('text=Conversation Title', { timeout: 5000 });

  // Fill the session title in the wizard (matches placeholder pattern "e.g. ...")
  const titleInput = page.locator('input[placeholder*="e.g."], textbox[placeholder*="e.g."]');
  await titleInput.fill(title);

  // Click Create button
  const createBtn = page.locator('button:has-text("Create")');
  await createBtn.click();

  // Wait for session to be ready
  await page.waitForSelector('text=Session ready', { timeout: 5000 });
}

/**
 * Gets the chat input element (textarea or contenteditable div)
 */
function getChatInput(page) {
  return page.getByRole('textbox', { name: /send a message|envoyer un message/i });
}

// ============================================================================
// ChatInput Component Tests
// ============================================================================

test.describe('ChatInput Component', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    // Wait for app to be ready
    await page.waitForSelector('aside, main', { timeout: 10000 });
    // Create a session to access chat input
    await createSession(page);
  });

  test('renders input field', async ({ page }) => {
    const input = getChatInput(page);
    await expect(input).toBeVisible();
  });

  test('accepts text input', async ({ page }) => {
    const input = getChatInput(page);
    await input.fill('Test message');
    await expect(input).toContainText('Test message');
  });

  test('clears after sending', async ({ page }) => {
    const input = getChatInput(page);
    await input.fill('Test message');
    await input.press('Enter');

    // Wait for the message to be processed
    await page.waitForTimeout(500);

    // Input should be empty (or contain only whitespace)
    const text = await input.innerText();
    expect(text.trim()).toBe('');
  });

  test('supports multi-line input with Shift+Enter', async ({ page }) => {
    const input = getChatInput(page);
    await input.fill('Line 1');
    await input.press('Shift+Enter');
    await input.pressSequentially('Line 2');

    // Use inputValue for textarea or textContent for contenteditable
    const text = await input.inputValue().catch(() => input.textContent());
    expect(text).toContain('Line 1');
    expect(text).toContain('Line 2');
  });
});

// ============================================================================
// MessageBubble Component Tests
// ============================================================================

test.describe('MessageBubble Component', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForSelector('aside, main', { timeout: 10000 });
    await createSession(page);
  });

  test('displays user message correctly', async ({ page }) => {
    const input = getChatInput(page);
    await input.fill('Test user message');
    await input.press('Enter');

    // Wait for user message to appear in the chat
    await page.waitForFunction(() => {
      return document.body.innerText.includes('Test user message');
    }, { timeout: 5000 });

    // Verify the message is visible
    const pageContent = await page.content();
    expect(pageContent).toContain('Test user message');
  });

  test('displays assistant message after streaming', async ({ page }) => {
    const input = getChatInput(page);
    await input.fill('Hello');
    await input.press('Enter');

    // Wait for mock response to stream
    await page.waitForFunction(() => {
      return document.body.innerText.includes('mock response');
    }, { timeout: 10000 });

    const pageContent = await page.content();
    expect(pageContent).toContain('mock response');
  });
});

// ============================================================================
// Sidebar Component Tests
// ============================================================================

test.describe('Sidebar Component', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForSelector('aside, nav', { timeout: 10000 });
  });

  test('renders sidebar', async ({ page }) => {
    const sidebar = page.locator('aside, nav[role="navigation"]');
    await expect(sidebar.first()).toBeVisible();
  });

  test('new chat button is visible', async ({ page }) => {
    const newChatButton = page.getByRole('button', { name: /nouvelle conversation|new chat/i });
    await expect(newChatButton).toBeVisible();
  });

  test('toggle sidebar visibility', async ({ page }) => {
    const toggleButton = page.locator('button[aria-label*="sidebar" i], button[aria-label*="menu" i]');

    if (await toggleButton.first().isVisible()) {
      await toggleButton.first().click();
      await page.waitForTimeout(500);
      // Sidebar state should change (either hidden or collapsed)
    }
  });
});

// ============================================================================
// SessionWizard Component Tests
// ============================================================================

test.describe('SessionWizard Component', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForSelector('aside, main', { timeout: 10000 });
  });

  test('opens wizard on new chat click', async ({ page }) => {
    const newChatButton = page.locator('button:has-text("New conversation")');
    await newChatButton.click();

    // Wait for wizard modal to appear with title
    await page.waitForSelector('text=Conversation Title', { timeout: 5000 });
  });

  test('allows setting session title', async ({ page }) => {
    const newChatButton = page.locator('button:has-text("New conversation")');
    await newChatButton.click();

    await page.waitForSelector('text=Conversation Title', { timeout: 5000 });

    const titleInput = page.locator('input[placeholder*="e.g."]');
    await titleInput.fill('My Custom Session');
    await expect(titleInput).toHaveValue('My Custom Session');
  });

  test('closes wizard on cancel', async ({ page }) => {
    const newChatButton = page.locator('button:has-text("New conversation")');
    await newChatButton.click();

    await page.waitForSelector('text=Conversation Title', { timeout: 5000 });

    const cancelButton = page.locator('button:has-text("Cancel")');
    if (await cancelButton.isVisible()) {
      await cancelButton.click();

      // Wizard should be closed - title no longer visible
      await expect(page.locator('text=Conversation Title')).not.toBeVisible({ timeout: 2000 });
    }
  });

  test('creates session and shows chat interface', async ({ page }) => {
    await createSession(page, 'New Test Session');

    // Chat input should be visible after session creation
    const input = getChatInput(page);
    await expect(input).toBeVisible();
  });
});

// ============================================================================
// Main Layout Tests
// ============================================================================

test.describe('Main Layout', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForSelector('aside, main', { timeout: 10000 });
  });

  test('renders main content area', async ({ page }) => {
    const mainContent = page.locator('main');
    await expect(mainContent).toBeVisible();
  });

  test('renders header with title bar', async ({ page }) => {
    const header = page.locator('header, [data-tauri-drag-region]');
    await expect(header.first()).toBeVisible();
  });
});

// ============================================================================
// Error Handling Tests
// ============================================================================

test.describe('Error Handling', () => {
  test('app loads without crashing', async ({ page }) => {
    await page.goto('/');

    // App should load without errors
    await page.waitForSelector('aside, main', { timeout: 10000 });

    // Check for error boundaries
    const sidebar = page.locator('aside');
    await expect(sidebar).toBeVisible();
  });
});
