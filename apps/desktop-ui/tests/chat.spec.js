import { test, expect } from '@playwright/test';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

test.beforeEach(async ({ page }) => {
    await page.goto('');
    // Wait for the onboarding to finish, or for the main chat interface to be visible
    await page.waitForSelector('#chat-input', { timeout: 100000 });
});

test('chat prompt', async ({ page }) => {
  await page.fill('#chat-input', 'Hello, world!');
  await page.press('#chat-input', 'Enter');
  await page.waitForSelector('.message.assistant');
  const assistantMessages = await page.locator('.message.assistant').all();
  const lastMessage = assistantMessages[assistantMessages.length - 1];
  const messageContent = await lastMessage.textContent();
  expect(messageContent).not.toBe('');
  expect(messageContent).not.toContain('répétitions');
});

test('file upload and RAG', async ({ page }) => {
    // Onboarding is handled by beforeEach

    // 1. Upload a file
    await page.click('button[aria-label="Fichiers"]'); // Assuming this is the files button
    const fileChooserPromise = page.waitForEvent('filechooser');
    await page.click('button:has-text("Upload")'); // Or whatever the upload button is
    const fileChooser = await fileChooserPromise;
    const filePath = path.join(__dirname, 'fixtures', 'test-file.txt');
    await fileChooser.setFiles(filePath);

    // Verify file is listed
    await page.waitForSelector('text=test-file.txt');

    // 2. Ask a question that can only be answered by the file
    await page.fill('#chat-input', 'What is the secret code?');
    await page.press('#chat-input', 'Enter');

    // 3. Verify the RAG response
    await page.waitForSelector('.message.assistant');
    const assistantMessages = await page.locator('.message.assistant').all();
    const lastMessage = assistantMessages[assistantMessages.length - 1];
    const messageContent = await lastMessage.textContent();
    expect(messageContent).toContain('42');
});
