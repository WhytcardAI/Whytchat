import { test, expect } from '@playwright/test';

test.describe('Production Error Handling', () => {
  test.beforeEach(async ({ page }) => {
    // Enable console logging
    page.on('console', msg => console.log(`[Browser] ${msg.text()}`));
    page.on('pageerror', err => console.log(`[Browser Error] ${err}`));

    // Set initial state in localStorage to skip onboarding
    await page.addInitScript(() => {
      window.localStorage.setItem('whytchat-storage', JSON.stringify({
        state: {
          isConfigured: true,
          isSidebarOpen: true,
          currentView: 'knowledge',
          theme: 'light'
        },
        version: 0
      }));
    });

    // Mock Tauri invoke
    await page.addInitScript(() => {
      // Mock for Tauri v2
      window.__TAURI_INTERNALS__ = window.__TAURI_INTERNALS__ || {};

      // Mock metadata for getCurrentWindow
      window.__TAURI_INTERNALS__.metadata = {
        currentWindow: { label: 'main' }
      };

      // Mock transformCallback
      window.__TAURI_INTERNALS__.transformCallback = function(cb) {
          return cb;
      };

      // Mock event internals
      window.__TAURI_INTERNALS__.event = {
          unregisterListener: () => {}
      };

      window.__TAURI_INTERNALS__.invoke = async (cmd, args) => {
        console.log(`[MockInvoke] ${cmd}`, JSON.stringify(args));
        if (cmd === 'run_quick_preflight_check') {
          return { ready_to_start: true, needs_onboarding: false };
        }
        if (cmd === 'initialize_app') {
          return { status: 'ok' };
        }
        if (cmd === 'list_folders') {
          return [];
        }
        if (cmd === 'list_sessions') {
          return [];
        }
        if (cmd === 'list_library_files') {
          return [];
        }
        if (cmd === 'get_app_config') {
            return { theme: 'dark', language: 'en' };
        }
        if (cmd === 'run_diagnostic_category') {
            return [{ name: 'test_mock', passed: true, duration_ms: 10, category: args.category }];
        }

        // Mock Window commands
        if (cmd.startsWith('plugin:window|')) {
            if (cmd.endsWith('is_maximized')) return false;
            return null;
        }

        // Mock Event commands
        if (cmd.startsWith('plugin:event|')) {
            return 123;
        }

        return null;
      };

      // Also mock window.__TAURI__ just in case
      window.__TAURI__ = {
          core: {
              invoke: window.__TAURI_INTERNALS__.invoke
          }
      };
    });

    await page.goto('/');

    // Wait for app to load (MainLayout should render)
    await page.waitForSelector('text=WhytChat', { timeout: 10000 });
  });

  test('should catch crashes and allow recovery', async ({ page }) => {
    // 1. Open Diagnostics Panel via Store (bypass UI to avoid flaky clicks)
    await page.evaluate(() => {
        window.appStore.getState().setDiagnosticsOpen(true);
    });

    // 3. Trigger Crash
    // Wait for diagnostics panel to appear
    await page.waitForSelector('text=System Diagnostics', { timeout: 5000 });

    // Click the simulate crash button
    await page.click('text=Simulate Crash');

    // 4. Verify Error Boundary
    await page.waitForSelector('text=Something went wrong', { timeout: 5000 });

    // Verify the error message is displayed
    await page.waitForSelector('text=This is a simulated crash for testing the Error Boundary');

    // 5. Recover
    await page.click('text=Go Home');

    // 6. Verify App Recovered
    await page.waitForSelector('text=WhytChat', { timeout: 10000 });

    // Ensure error boundary is gone
    const errorTitle = await page.locator('text=Something went wrong').count();
    expect(errorTitle).toBe(0);
  });
});
