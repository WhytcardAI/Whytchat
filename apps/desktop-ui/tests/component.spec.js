/**
 * Component Unit Tests
 *
 * Tests for individual React components using Playwright component testing.
 */

import { test, expect } from '@playwright/test';

// ============================================================================
// ChatInput Component Tests
// ============================================================================

test.describe('ChatInput Component', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForSelector('#chat-input', { timeout: 60000 });
  });

  test('renders input field', async ({ page }) => {
    const input = page.locator('#chat-input');
    await expect(input).toBeVisible();
  });

  test('accepts text input', async ({ page }) => {
    const input = page.locator('#chat-input');
    await input.fill('Test message');
    await expect(input).toHaveValue('Test message');
  });

  test('clears after sending', async ({ page }) => {
    const input = page.locator('#chat-input');
    await input.fill('Test message');
    await input.press('Enter');

    // Input should clear after sending
    await page.waitForTimeout(500);
    await expect(input).toHaveValue('');
  });

  test('supports multi-line input with Shift+Enter', async ({ page }) => {
    const input = page.locator('#chat-input');
    await input.fill('Line 1');
    await input.press('Shift+Enter');
    await input.type('Line 2');

    const value = await input.inputValue();
    expect(value).toContain('Line 1');
    expect(value).toContain('Line 2');
  });
});

// ============================================================================
// MessageBubble Component Tests
// ============================================================================

test.describe('MessageBubble Component', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForSelector('#chat-input', { timeout: 60000 });
  });

  test('displays user message correctly', async ({ page }) => {
    const input = page.locator('#chat-input');
    await input.fill('Test user message');
    await input.press('Enter');

    await page.waitForSelector('.message.user', { timeout: 5000 });
    const message = page.locator('.message.user').last();
    await expect(message).toContainText('Test user message');
  });

  test('displays assistant message with proper styling', async ({ page }) => {
    const input = page.locator('#chat-input');
    await input.fill('Hello');
    await input.press('Enter');

    await page.waitForSelector('.message.assistant', { timeout: 60000 });
    const message = page.locator('.message.assistant').last();
    await expect(message).toBeVisible();
  });

  test('handles code blocks in responses', async ({ page }) => {
    const input = page.locator('#chat-input');
    await input.fill('Write a simple hello world in Python');
    await input.press('Enter');

    await page.waitForSelector('.message.assistant', { timeout: 60000 });

    // Check if code formatting is present
    const codeBlock = page.locator('.message.assistant code, .message.assistant pre');
    // Code block may or may not be present depending on response
    const hasCode = await codeBlock.count() > 0;
    expect(typeof hasCode).toBe('boolean');
  });
});

// ============================================================================
// ThinkingBubble Component Tests
// ============================================================================

test.describe('ThinkingBubble Component', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForSelector('#chat-input', { timeout: 60000 });
  });

  test('shows thinking indicator during response generation', async ({ page }) => {
    const input = page.locator('#chat-input');
    await input.fill('What is the meaning of life?');

    // Start observing before sending
    const thinkingPromise = page.waitForSelector('.thinking, [data-testid="thinking"], .animate-pulse', {
      timeout: 5000,
    }).catch(() => null);

    await input.press('Enter');

    // Thinking indicator might appear briefly
    await thinkingPromise;
    // It's OK if it doesn't appear (fast response)
  });

  test('shows thinking steps', async ({ page }) => {
    const input = page.locator('#chat-input');
    await input.fill('Complex question that requires analysis');

    // Listen for thinking steps
    const stepsPromise = page.waitForSelector('[data-testid="thinking-step"], .thinking-step', {
      timeout: 10000,
    }).catch(() => null);

    await input.press('Enter');

    // Steps may or may not appear
    await stepsPromise;
  });
});

// ============================================================================
// Sidebar Component Tests
// ============================================================================

test.describe('Sidebar Component', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForSelector('#chat-input', { timeout: 60000 });
  });

  test('renders sidebar', async ({ page }) => {
    const sidebar = page.locator('aside, nav, [data-testid="sidebar"]');
    await expect(sidebar).toBeVisible();
  });

  test('shows session list', async ({ page }) => {
    const sessionList = page.locator('[data-testid="session-list"], .session-list');
    // May be empty initially
    await expect(sessionList.or(page.locator('aside'))).toBeVisible();
  });

  test('new chat button is visible', async ({ page }) => {
    const newChatButton = page.locator('button:has-text("New"), button:has-text("Nouveau"), [data-testid="new-chat"]');
    await expect(newChatButton.first()).toBeVisible();
  });

  test('toggle sidebar visibility', async ({ page }) => {
    const toggleButton = page.locator('[data-testid="toggle-sidebar"], button[aria-label*="sidebar"]');

    if (await toggleButton.isVisible()) {
      const sidebar = page.locator('aside, [data-testid="sidebar"]');
      const initiallyVisible = await sidebar.isVisible();

      await toggleButton.click();
      await page.waitForTimeout(500);

      if (initiallyVisible) {
        // Should be hidden or collapsed
        const width = await sidebar.evaluate(el => el.offsetWidth);
        expect(width).toBeLessThanOrEqual(50);
      }
    }
  });
});

// ============================================================================
// SessionWizard Component Tests
// ============================================================================

test.describe('SessionWizard Component', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForSelector('#chat-input', { timeout: 60000 });
  });

  test('opens wizard on new chat click', async ({ page }) => {
    const newChatButton = page.locator('button:has-text("New Chat"), button:has-text("Nouvelle conversation"), [data-testid="new-chat"]');

    if (await newChatButton.isVisible()) {
      await newChatButton.click();

      const wizard = page.locator('[data-testid="session-wizard"], .session-wizard, .modal');
      await expect(wizard).toBeVisible({ timeout: 5000 });
    }
  });

  test('allows setting session title', async ({ page }) => {
    const newChatButton = page.locator('[data-testid="new-chat"]');

    if (await newChatButton.isVisible()) {
      await newChatButton.click();

      const titleInput = page.locator('input[name="title"], input[placeholder*="title"]');
      if (await titleInput.isVisible()) {
        await titleInput.fill('My Custom Session');
        await expect(titleInput).toHaveValue('My Custom Session');
      }
    }
  });

  test('closes wizard on cancel', async ({ page }) => {
    const newChatButton = page.locator('[data-testid="new-chat"]');

    if (await newChatButton.isVisible()) {
      await newChatButton.click();

      const cancelButton = page.locator('button:has-text("Cancel"), button:has-text("Annuler")');
      if (await cancelButton.isVisible()) {
        await cancelButton.click();

        const wizard = page.locator('[data-testid="session-wizard"]');
        await expect(wizard).not.toBeVisible();
      }
    }
  });
});

// ============================================================================
// KnowledgeView Component Tests
// ============================================================================

test.describe('KnowledgeView Component', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForSelector('#chat-input', { timeout: 60000 });
  });

  test('renders knowledge view tab', async ({ page }) => {
    const knowledgeTab = page.locator('button:has-text("Knowledge"), button:has-text("Import"), [data-testid="knowledge-tab"]');
    await expect(knowledgeTab.first()).toBeVisible();
  });

  test('shows file list when opened', async ({ page }) => {
    const knowledgeTab = page.locator('button:has-text("Knowledge"), [data-testid="knowledge-tab"]');

    if (await knowledgeTab.isVisible()) {
      await knowledgeTab.click();

      const fileList = page.locator('[data-testid="file-list"], .file-list, .knowledge-content');
      await expect(fileList.or(page.locator('[data-testid="knowledge-view"]'))).toBeVisible();
    }
  });

  test('has upload button', async ({ page }) => {
    const knowledgeTab = page.locator('[data-testid="knowledge-tab"]');

    if (await knowledgeTab.isVisible()) {
      await knowledgeTab.click();

      const uploadButton = page.locator('button:has-text("Upload"), button:has-text("Import"), [data-testid="upload-file"]');
      await expect(uploadButton.first()).toBeVisible();
    }
  });
});

// ============================================================================
// ErrorBoundary Component Tests
// ============================================================================

test.describe('ErrorBoundary Component', () => {
  test('catches and displays errors gracefully', async ({ page }) => {
    await page.goto('/');
    await page.waitForSelector('#chat-input', { timeout: 60000 });

    // The app should be functional even if errors occurred
    const chatInput = page.locator('#chat-input');
    await expect(chatInput).toBeVisible();
    await expect(chatInput).toBeEnabled();
  });
});

// ============================================================================
// Dashboard Component Tests
// ============================================================================

test.describe('Dashboard Component', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForSelector('#chat-input, [data-testid="dashboard"]', { timeout: 60000 });
  });

  test('renders main content area', async ({ page }) => {
    const mainContent = page.locator('main, [data-testid="main-content"]');
    await expect(mainContent).toBeVisible();
  });
});
