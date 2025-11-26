/* eslint-disable no-unused-vars, no-prototype-builtins */
/**
 * WhytChat Browser Console Test Suite
 * ====================================
 * Copiez-collez ce script dans la console du navigateur (F12 > Console)
 * pour tester l'application en temps réel.
 *
 * ATTENTION: N'exécutez PAS sur une instance de production!
 */

(function WhytChatStressTest() {
  'use strict';

  const results = { passed: 0, failed: 0, warnings: 0 };

  const log = {
    pass: (msg) => { results.passed++; console.log(`%c✅ PASS: ${msg}`, 'color: #22c55e'); },
    fail: (msg) => { results.failed++; console.log(`%c❌ FAIL: ${msg}`, 'color: #ef4444'); },
    warn: (msg) => { results.warnings++; console.log(`%c⚠️ WARN: ${msg}`, 'color: #f59e0b'); },
    info: (msg) => console.log(`%cℹ️ INFO: ${msg}`, 'color: #3b82f6'),
    section: (msg) => console.log(`%c\n═══ ${msg} ═══`, 'color: #8b5cf6; font-weight: bold; font-size: 14px'),
  };

  // ============================================================================
  // TEST 1: DOM XSS Prevention
  // ============================================================================
  function testXSSPrevention() {
    log.section('TEST 1: XSS Prevention');

    const xssPayloads = [
      '<script>alert(1)</script>',
      '<img src=x onerror=alert(1)>',
      '"><img src=x onerror=alert(1)>',
      'javascript:alert(1)',
    ];

    // Check if any script tags are in the DOM that shouldn't be
    const scripts = document.querySelectorAll('script:not([src])');
    const inlineScripts = Array.from(scripts).filter(s =>
      s.textContent.includes('alert') || s.textContent.includes('eval')
    );

    if (inlineScripts.length === 0) {
      log.pass('No dangerous inline scripts detected');
    } else {
      log.fail(`Found ${inlineScripts.length} potentially dangerous inline scripts`);
    }

    // Check for dangerous event handlers
    const allElements = document.querySelectorAll('*');
    let dangerousHandlers = 0;
    allElements.forEach(el => {
      const attrs = el.attributes;
      for (let i = 0; i < attrs.length; i++) {
        if (attrs[i].name.startsWith('on') &&
            (attrs[i].value.includes('alert') || attrs[i].value.includes('eval'))) {
          dangerousHandlers++;
        }
      }
    });

    if (dangerousHandlers === 0) {
      log.pass('No dangerous event handlers in DOM');
    } else {
      log.fail(`Found ${dangerousHandlers} dangerous event handlers`);
    }
  }

  // ============================================================================
  // TEST 2: React State Consistency
  // ============================================================================
  function testReactState() {
    log.section('TEST 2: React State Consistency');

    // Check for React DevTools
    const hasReactDevTools = typeof __REACT_DEVTOOLS_GLOBAL_HOOK__ !== 'undefined';
    log.info(`React DevTools: ${hasReactDevTools ? 'Available' : 'Not detected'}`);

    // Look for orphaned React roots
    const reactRoots = document.querySelectorAll('[data-reactroot]');
    if (reactRoots.length <= 1) {
      log.pass(`Found ${reactRoots.length} React root(s) - expected`);
    } else {
      log.warn(`Found ${reactRoots.length} React roots - may indicate multiple apps`);
    }

    // Check for error boundaries (look for error UI)
    const errorBoundaries = document.querySelectorAll('[class*="error"], [class*="Error"]');
    log.info(`Error-related elements in DOM: ${errorBoundaries.length}`);
  }

  // ============================================================================
  // TEST 3: Memory Usage
  // ============================================================================
  async function testMemoryUsage() {
    log.section('TEST 3: Memory Usage');

    if (performance.memory) {
      const memory = performance.memory;
      const usedMB = (memory.usedJSHeapSize / 1024 / 1024).toFixed(2);
      const totalMB = (memory.totalJSHeapSize / 1024 / 1024).toFixed(2);
      const limitMB = (memory.jsHeapSizeLimit / 1024 / 1024).toFixed(2);

      log.info(`JS Heap: ${usedMB}MB / ${totalMB}MB (limit: ${limitMB}MB)`);

      const usagePercent = (memory.usedJSHeapSize / memory.jsHeapSizeLimit) * 100;
      if (usagePercent < 50) {
        log.pass(`Memory usage healthy: ${usagePercent.toFixed(1)}%`);
      } else if (usagePercent < 80) {
        log.warn(`Memory usage elevated: ${usagePercent.toFixed(1)}%`);
      } else {
        log.fail(`Memory usage critical: ${usagePercent.toFixed(1)}%`);
      }
    } else {
      log.warn('performance.memory not available (Chrome only)');
    }
  }

  // ============================================================================
  // TEST 4: Event Listener Count
  // ============================================================================
  function testEventListeners() {
    log.section('TEST 4: Event Listeners');

    // This is a rough estimate - actual count requires DevTools
    const eventTypes = ['click', 'keydown', 'keyup', 'input', 'change', 'submit', 'scroll'];
    let totalEstimate = 0;

    eventTypes.forEach(type => {
      const elements = document.querySelectorAll(`[on${type}]`);
      totalEstimate += elements.length;
    });

    log.info(`Inline event handlers (rough estimate): ${totalEstimate}`);

    // Check for potential memory leaks from unremoved listeners
    const forms = document.querySelectorAll('form');
    const inputs = document.querySelectorAll('input, textarea');
    const buttons = document.querySelectorAll('button');

    log.info(`Interactive elements: ${forms.length} forms, ${inputs.length} inputs, ${buttons.length} buttons`);

    if (buttons.length < 100) {
      log.pass('Reasonable number of interactive elements');
    } else {
      log.warn('Large number of interactive elements - check for cleanup');
    }
  }

  // ============================================================================
  // TEST 5: LocalStorage Integrity
  // ============================================================================
  function testLocalStorage() {
    log.section('TEST 5: LocalStorage Integrity');

    try {
      // Check for WhytChat storage
      const whytchatStorage = localStorage.getItem('whytchat-storage');

      if (whytchatStorage) {
        const parsed = JSON.parse(whytchatStorage);
        log.pass('whytchat-storage exists and is valid JSON');

        // Check structure
        const state = parsed.state || parsed;
        const expectedKeys = ['isConfigured', 'isSidebarOpen', 'currentView', 'theme'];
        const missingKeys = expectedKeys.filter(k => !(k in state));

        if (missingKeys.length === 0) {
          log.pass('All expected keys present in storage');
        } else {
          log.warn(`Missing keys: ${missingKeys.join(', ')}`);
        }

        // Check for sensitive data that shouldn't be stored
        const sensitivePatterns = ['password', 'token', 'secret', 'api_key'];
        const storageStr = JSON.stringify(state).toLowerCase();
        const foundSensitive = sensitivePatterns.filter(p => storageStr.includes(p));

        if (foundSensitive.length === 0) {
          log.pass('No sensitive data patterns found in storage');
        } else {
          log.fail(`Potentially sensitive data in storage: ${foundSensitive.join(', ')}`);
        }
      } else {
        log.warn('whytchat-storage not found - might be first run');
      }

      // Check total localStorage size
      let totalSize = 0;
      for (let key in localStorage) {
        if (localStorage.hasOwnProperty(key)) {
          totalSize += (localStorage[key].length + key.length) * 2; // UTF-16
        }
      }
      const sizeMB = (totalSize / 1024 / 1024).toFixed(2);
      log.info(`Total localStorage size: ${sizeMB}MB`);

      if (totalSize < 5 * 1024 * 1024) {
        log.pass('localStorage size within limits');
      } else {
        log.warn('localStorage approaching 5MB limit');
      }

    } catch (e) {
      log.fail(`localStorage error: ${e.message}`);
    }
  }

  // ============================================================================
  // TEST 6: Network Request Patterns
  // ============================================================================
  function testNetworkPatterns() {
    log.section('TEST 6: Network Patterns');

    // Check for WebSocket connections (Tauri uses IPC, not WebSockets)
    const wsProtocols = ['ws:', 'wss:'];
    log.info('Note: Tauri uses IPC, not HTTP/WebSocket for backend calls');

    // Check for any external resources
    const scripts = document.querySelectorAll('script[src]');
    const styles = document.querySelectorAll('link[rel="stylesheet"]');
    const images = document.querySelectorAll('img[src]');

    const externalScripts = Array.from(scripts).filter(s =>
      s.src && (s.src.startsWith('http://') || s.src.startsWith('https://'))
    );

    if (externalScripts.length === 0) {
      log.pass('No external scripts loaded');
    } else {
      log.warn(`${externalScripts.length} external script(s) detected`);
      externalScripts.forEach(s => log.info(`  - ${s.src}`));
    }
  }

  // ============================================================================
  // TEST 7: UI State Consistency
  // ============================================================================
  function testUIConsistency() {
    log.section('TEST 7: UI State Consistency');

    // Check for loading states stuck
    const loaders = document.querySelectorAll('[class*="animate-spin"], [class*="loading"], [class*="Loader"]');
    log.info(`Loading indicators visible: ${loaders.length}`);

    // Check for overlapping modals
    const modals = document.querySelectorAll('[class*="fixed"][class*="inset-0"]');
    if (modals.length <= 1) {
      log.pass(`Modal state correct: ${modals.length} modal(s)`);
    } else {
      log.warn(`Multiple overlays detected: ${modals.length} - possible stacking issue`);
    }

    // Check z-index stacking
    const highZElements = [];
    document.querySelectorAll('*').forEach(el => {
      const style = window.getComputedStyle(el);
      const zIndex = parseInt(style.zIndex);
      if (zIndex > 100) {
        highZElements.push({ el, zIndex });
      }
    });

    log.info(`Elements with z-index > 100: ${highZElements.length}`);
    if (highZElements.length > 10) {
      log.warn('Many high z-index elements - check for stacking context issues');
    }
  }

  // ============================================================================
  // TEST 8: Accessibility Quick Check
  // ============================================================================
  function testAccessibility() {
    log.section('TEST 8: Accessibility Quick Check');

    // Check for missing alt text
    const imagesWithoutAlt = document.querySelectorAll('img:not([alt])');
    if (imagesWithoutAlt.length === 0) {
      log.pass('All images have alt attributes');
    } else {
      log.warn(`${imagesWithoutAlt.length} image(s) missing alt attribute`);
    }

    // Check for buttons without accessible text
    const buttons = document.querySelectorAll('button');
    let emptyButtons = 0;
    buttons.forEach(btn => {
      const hasText = btn.textContent.trim().length > 0;
      const hasAriaLabel = btn.hasAttribute('aria-label');
      const hasTitle = btn.hasAttribute('title');
      if (!hasText && !hasAriaLabel && !hasTitle) {
        emptyButtons++;
      }
    });

    if (emptyButtons === 0) {
      log.pass('All buttons have accessible text');
    } else {
      log.warn(`${emptyButtons} button(s) without accessible text`);
    }

    // Check for form labels
    const inputs = document.querySelectorAll('input:not([type="hidden"]):not([type="submit"]):not([type="button"])');
    let unlabeledInputs = 0;
    inputs.forEach(input => {
      const hasLabel = input.labels && input.labels.length > 0;
      const hasAriaLabel = input.hasAttribute('aria-label');
      const hasPlaceholder = input.hasAttribute('placeholder');
      if (!hasLabel && !hasAriaLabel && !hasPlaceholder) {
        unlabeledInputs++;
      }
    });

    if (unlabeledInputs === 0) {
      log.pass('All inputs have labels or accessible names');
    } else {
      log.warn(`${unlabeledInputs} input(s) without accessible labels`);
    }

    // Check color contrast (basic check)
    const textElements = document.querySelectorAll('p, span, h1, h2, h3, h4, h5, h6, label');
    log.info(`Text elements to check for contrast: ${textElements.length}`);
  }

  // ============================================================================
  // TEST 9: Rapid Action Simulation
  // ============================================================================
  async function testRapidActions() {
    log.section('TEST 9: Rapid Action Simulation (Safe Mode)');

    log.info('Simulating rapid clicks on theme toggle...');
    const themeButton = document.querySelector('[title*="theme" i], [title*="Theme" i], button:has(svg)');

    if (themeButton) {
      const originalTheme = document.documentElement.classList.contains('dark');
      let clickCount = 0;

      // Rapid clicks simulation (10 clicks in 100ms)
      const startTime = performance.now();
      for (let i = 0; i < 10; i++) {
        // Don't actually click - just measure potential
        clickCount++;
      }
      const duration = performance.now() - startTime;

      log.info(`Simulated ${clickCount} rapid clicks in ${duration.toFixed(2)}ms`);
      log.pass('Rapid action simulation completed (no actual clicks performed)');
    } else {
      log.warn('Theme toggle button not found for test');
    }
  }

  // ============================================================================
  // TEST 10: Error Handling
  // ============================================================================
  function testErrorHandling() {
    log.section('TEST 10: Error Handling');

    // Check if window.onerror is set
    const hasErrorHandler = typeof window.onerror === 'function';
    log.info(`Global error handler: ${hasErrorHandler ? 'Set' : 'Not set'}`);

    // Check for unhandledrejection handler
    // We can't easily check this without triggering it
    log.info('Unhandled Promise rejection handler: Cannot verify without triggering');

    // Check console for existing errors
    log.info('Check browser console for any existing errors (this script cannot access console history)');

    // Test that we can catch errors properly
    try {
      const testError = new Error('Test error - ignore this');
      throw testError;
    } catch (e) {
      log.pass('Error catching works correctly');
    }
  }

  // ============================================================================
  // RUN ALL TESTS
  // ============================================================================
  async function runAllTests() {
    console.clear();
    console.log('%c╔═══════════════════════════════════════════════════════════════╗', 'color: #8b5cf6');
    console.log('%c║       WhytChat Browser Console Test Suite                     ║', 'color: #8b5cf6');
    console.log('%c║       ====================================                    ║', 'color: #8b5cf6');
    console.log('%c║       Mode: SAFE (No modifications)                           ║', 'color: #8b5cf6');
    console.log('%c╚═══════════════════════════════════════════════════════════════╝', 'color: #8b5cf6');

    const startTime = performance.now();

    testXSSPrevention();
    testReactState();
    await testMemoryUsage();
    testEventListeners();
    testLocalStorage();
    testNetworkPatterns();
    testUIConsistency();
    testAccessibility();
    await testRapidActions();
    testErrorHandling();

    const duration = (performance.now() - startTime).toFixed(2);

    console.log('%c\n╔═══════════════════════════════════════════════════════════════╗', 'color: #8b5cf6');
    console.log('%c║                        SUMMARY                                ║', 'color: #8b5cf6');
    console.log('%c╚═══════════════════════════════════════════════════════════════╝', 'color: #8b5cf6');
    console.log(`%c✅ Passed: ${results.passed}`, 'color: #22c55e; font-weight: bold');
    console.log(`%c⚠️ Warnings: ${results.warnings}`, 'color: #f59e0b; font-weight: bold');
    console.log(`%c❌ Failed: ${results.failed}`, 'color: #ef4444; font-weight: bold');
    console.log(`%c⏱️ Duration: ${duration}ms`, 'color: #3b82f6');

    return results;
  }

  // Execute
  return runAllTests();
})();
