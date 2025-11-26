import { defineConfig } from '@playwright/test';

export default defineConfig({
  timeout: 120000, // 2 minutes
  use: {
    baseURL: 'http://localhost:1420',
  },
  webServer: {
    command: 'npm run dev',
    url: 'http://localhost:1420',
    reuseExistingServer: !process.env.CI,
    timeout: 120000, // 2 minutes
  },
});