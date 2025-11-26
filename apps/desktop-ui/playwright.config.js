import { defineConfig } from '@playwright/test';

export default defineConfig({
  testDir: './tests',
  use: {
    baseURL: 'http://localhost:1420',
    browserName: 'webkit',
    headless: false,
    launchOptions: {
      executablePath: './src-tauri/target/release/whytchat.exe',
    },
  },
});