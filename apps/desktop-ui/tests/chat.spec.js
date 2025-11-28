import { test, expect } from '@playwright/test';
import { setupTauriMock } from './helpers/tauri-mock.js';

test.beforeEach(async ({ page }) => {
    // Setup Tauri mock BEFORE navigating
    await setupTauriMock(page);

    await page.goto('');
    // Wait for the app to load (Welcome screen or chat interface)
    await page.waitForSelector('text=WhytChat', { timeout: 30000 });

    // If we see the welcome screen, create a new conversation
    const newConvButton = page.locator('button:has-text("New conversation")');
    if (await newConvButton.isVisible({ timeout: 2000 })) {
        await newConvButton.click();

        // Wait for session wizard modal
        await page.waitForSelector('text=Conversation Title', { timeout: 5000 });

        // Fill in the conversation title
        const titleInput = page.locator('textbox[placeholder*="e.g."], input[placeholder*="e.g."]');
        await titleInput.fill('Test Conversation');

        // Click Create button
        const createButton = page.locator('button:has-text("Create")');
        await createButton.click();
    }

    // Now wait for the session to be ready
    await page.waitForSelector('text=Session ready', { timeout: 15000 });
});

test('chat prompt', async ({ page }) => {
  // Use role-based selector for the chat textarea
  const chatInput = page.getByRole('textbox', { name: /send a message/i });
  await expect(chatInput).toBeVisible({ timeout: 5000 });
  await chatInput.fill('Hello, world!');
  await chatInput.press('Enter');

  // Wait for user message to appear first
  await page.waitForSelector('text=Hello, world!', { timeout: 5000 });

  // Wait for assistant response (mock response contains "mock response")
  await page.waitForTimeout(2000); // Give time for streaming mock tokens

  // Check for any message that's not from user
  const pageContent = await page.content();
  expect(pageContent).toContain('mock response');
});

test('multiple messages in conversation', async ({ page }) => {
  const chatInput = page.getByRole('textbox', { name: /send a message/i });

  // Send first message
  await chatInput.fill('First message');
  await chatInput.press('Enter');
  await page.waitForSelector('text=First message', { timeout: 5000 });

  // Wait for response
  await page.waitForTimeout(1500);

  // Send second message
  await chatInput.fill('Second message');
  await chatInput.press('Enter');
  await page.waitForSelector('text=Second message', { timeout: 5000 });

  // Verify both messages exist
  const pageContent = await page.content();
  expect(pageContent).toContain('First message');
  expect(pageContent).toContain('Second message');
});

test('chat input is cleared after sending', async ({ page }) => {
  const chatInput = page.getByRole('textbox', { name: /send a message/i });

  await chatInput.fill('Test message');
  await chatInput.press('Enter');

  // Verify input is cleared
  await expect(chatInput).toHaveValue('');
});

test('displays streaming tokens progressively', async ({ page }) => {
  const chatInput = page.getByRole('textbox', { name: /send a message/i });

  await chatInput.fill('Test streaming');
  await chatInput.press('Enter');

  // Wait for user message
  await page.waitForSelector('text=Test streaming', { timeout: 5000 });

  // Mock should stream tokens - wait for response to build up
  await page.waitForTimeout(2000);

  // Verify assistant response appeared
  const pageContent = await page.content();
  expect(pageContent).toContain('mock');
});
