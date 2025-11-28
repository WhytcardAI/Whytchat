import { useState, useCallback, useRef, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useTranslation } from 'react-i18next';
import {
  Play,
  CheckCircle,
  XCircle,
  Clock,
  AlertTriangle,
  Terminal,
  RotateCcw,
  ChevronDown,
  ChevronRight,
  Activity,
  Database,
  Brain,
  Cpu,
  HardDrive,
  Search,
  Link,
  Bug,
} from 'lucide-react';
import { cn } from '../../lib/utils';

// Category icons mapping
const categoryIcons = {
  database: Database,
  rag: Search,
  brain: Brain,
  llm: Cpu,
  filesystem: HardDrive,
  integration: Link,
  debug: Bug,
};

// Component that throws an error for testing ErrorBoundary
function BuggyComponent() {
  throw new Error("This is a simulated crash for testing the Error Boundary.");
}

// Test status component
function TestStatus({ status, size = 16 }) {
  if (status === 'running') {
    return (
      <Activity
        className="animate-pulse text-blue-400"
        size={size}
      />
    );
  }
  if (status === 'passed') {
    return (
      <CheckCircle
        className="text-green-500"
        size={size}
      />
    );
  }
  if (status === 'failed') {
    return (
      <XCircle
        className="text-red-500"
        size={size}
      />
    );
  }
  return (
    <Clock
      className="text-gray-400"
      size={size}
    />
  );
}

// Individual test result row
function TestRow({ result, isExpanded, onToggle }) {
  return (
    <div className={cn('border-b border-gray-700/50', !result.passed && 'bg-red-900/10')}>
      <div
        className="flex items-center gap-3 px-3 py-2 cursor-pointer hover:bg-gray-700/30 transition-colors"
        onClick={onToggle}
      >
        {result.details ? (
          isExpanded ? (
            <ChevronDown
              size={14}
              className="text-gray-400"
            />
          ) : (
            <ChevronRight
              size={14}
              className="text-gray-400"
            />
          )
        ) : (
          <span className="w-[14px]" />
        )}

        <TestStatus status={result.passed ? 'passed' : 'failed'} />

        <span className="font-mono text-sm text-gray-300 flex-1">{result.name}</span>

        <span className={cn('text-xs', result.passed ? 'text-gray-400' : 'text-red-400')}>
          {result.message}
        </span>

        <span className="text-xs text-gray-500 font-mono w-16 text-right">{result.duration_ms}ms</span>
      </div>

      {isExpanded && result.details && (
        <div className="px-10 py-2 bg-gray-800/50 text-xs font-mono text-gray-400 whitespace-pre-wrap">
          {result.details}
        </div>
      )}
    </div>
  );
}

// Category section
function CategorySection({ category, results, categoryStatus, defaultExpanded = false }) {
  const { t } = useTranslation();
  const [isExpanded, setIsExpanded] = useState(defaultExpanded);
  const [expandedTests, setExpandedTests] = useState({});

  const CategoryIcon = categoryIcons[category] || Activity;
  const passed = results.filter((r) => r.passed).length;
  const failed = results.filter((r) => !r.passed).length;

  const toggleTest = (testName) => {
    setExpandedTests((prev) => ({
      ...prev,
      [testName]: !prev[testName],
    }));
  };

  return (
    <div className="mb-4">
      <div
        className="flex items-center gap-3 px-3 py-2 bg-gray-800/50 rounded-t-lg cursor-pointer hover:bg-gray-700/50 transition-colors"
        onClick={() => setIsExpanded(!isExpanded)}
      >
        {isExpanded ? <ChevronDown size={16} /> : <ChevronRight size={16} />}

        <CategoryIcon
          size={18}
          className="text-blue-400"
        />

        <span className="font-medium text-gray-200 capitalize flex-1">{t(`diagnostics.categories.${category}`, category)}</span>

        {categoryStatus === 'running' ? (
          <Activity
            className="animate-pulse text-blue-400"
            size={16}
          />
        ) : (
          <div className="flex items-center gap-2 text-sm">
            {passed > 0 && <span className="text-green-500">{passed} ✓</span>}
            {failed > 0 && <span className="text-red-500">{failed} ✗</span>}
          </div>
        )}
      </div>

      {isExpanded && (
        <div className="border border-gray-700/50 border-t-0 rounded-b-lg overflow-hidden">
          {results.map((result) => (
            <TestRow
              key={result.name}
              result={result}
              isExpanded={expandedTests[result.name]}
              onToggle={() => toggleTest(result.name)}
            />
          ))}
        </div>
      )}
    </div>
  );
}

// Main TestConsole component
export function TestConsole({ onComplete, autoStart = false, className }) {
  const { t } = useTranslation();
  const [status, setStatus] = useState('idle'); // idle, running, complete
  const [results, setResults] = useState([]);
  const [currentCategory, setCurrentCategory] = useState(null);
  const [summary, setSummary] = useState(null);
  const [simulateCrash, setSimulateCrash] = useState(false);
  const consoleRef = useRef(null);

  // Group results by category
  const groupedResults = results.reduce((acc, result) => {
    if (!acc[result.category]) {
      acc[result.category] = [];
    }
    acc[result.category].push(result);
    return acc;
  }, {});

  // Auto-scroll to bottom when new results arrive
  useEffect(() => {
    if (consoleRef.current) {
      consoleRef.current.scrollTop = consoleRef.current.scrollHeight;
    }
  }, [results]);

  // Run all tests
  const runAllTests = useCallback(async () => {
    setStatus('running');
    setResults([]);
    setSummary(null);

    const categories = ['database', 'rag', 'brain', 'llm', 'filesystem', 'integration'];

    for (const category of categories) {
      setCurrentCategory(category);

      try {
        const categoryResults = await invoke('run_diagnostic_category', { category });
        setResults((prev) => [...prev, ...categoryResults]);
      } catch (error) {
        console.error(`Error running ${category} tests:`, error);
        setResults((prev) => [
          ...prev,
          {
            name: `${category}_error`,
            category,
            passed: false,
            duration_ms: 0,
            message: `Failed to run ${category} tests`,
            details: error.toString(),
          },
        ]);
      }
    }

    setCurrentCategory(null);
    setStatus('complete');

    // Calculate summary from collected results
    // Note: We need to use a ref or function update to get latest results
  }, []);

  // Calculate summary when results change
  useEffect(() => {
    if (status === 'complete' && results.length > 0) {
      const passed = results.filter((r) => r.passed).length;
      const failed = results.filter((r) => !r.passed).length;

      setSummary({
        total: passed + failed,
        passed,
        failed,
      });

      if (onComplete) {
        onComplete({
          passed,
          failed,
          results,
        });
      }
    }
  }, [status, results, onComplete]);

  // Auto-start if prop is set
  useEffect(() => {
    if (autoStart && status === 'idle') {
      runAllTests();
    }
  }, [autoStart, status, runAllTests]);

  return (
    <div className={cn('flex flex-col bg-gray-900 rounded-xl overflow-hidden', className)}>
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-3 bg-gray-800/50 border-b border-gray-700">
        <div className="flex items-center gap-3">
          <Terminal
            size={20}
            className="text-blue-400"
          />
          <h3 className="font-semibold text-gray-200">{t('diagnostics.title', 'System Diagnostics')}</h3>
        </div>

        <div className="flex items-center gap-2">
          {status === 'complete' && summary && (
            <div className="flex items-center gap-3 mr-4 text-sm">
              <span className="text-green-500 flex items-center gap-1">
                <CheckCircle size={14} />
                {summary.passed}
              </span>
              <span className="text-red-500 flex items-center gap-1">
                <XCircle size={14} />
                {summary.failed}
              </span>
            </div>
          )}

          <button
            onClick={runAllTests}
            disabled={status === 'running'}
            className={cn(
              'flex items-center gap-2 px-3 py-1.5 rounded-lg text-sm font-medium transition-colors',
              status === 'running' ? 'bg-gray-700 text-gray-400 cursor-not-allowed' : 'bg-blue-600 hover:bg-blue-500 text-white',
            )}
          >
            {status === 'running' ? (
              <>
                <Activity
                  className="animate-spin"
                  size={16}
                />
                {t('diagnostics.running', 'Running...')}
              </>
            ) : status === 'complete' ? (
              <>
                <RotateCcw size={16} />
                {t('diagnostics.rerun', 'Re-run')}
              </>
            ) : (
              <>
                <Play size={16} />
                {t('diagnostics.runAll', 'Run All Tests')}
              </>
            )}
          </button>
        </div>
      </div>

      {/* Console output */}
      <div
        ref={consoleRef}
        className="flex-1 overflow-y-auto p-4 min-h-[300px] max-h-[500px]"
      >
        {status === 'idle' && (
          <div className="flex flex-col items-center justify-center h-full text-gray-400">
            <AlertTriangle
              size={48}
              className="mb-4 text-yellow-500/50"
            />
            <p className="text-center">{t('diagnostics.notStarted', 'Click "Run All Tests" to start diagnostics')}</p>
          </div>
        )}

        {(status === 'running' || status === 'complete') && (
          <div className="space-y-4">
            {['database', 'rag', 'brain', 'llm', 'filesystem', 'integration'].map((category) => {
              const categoryResults = groupedResults[category] || [];
              const isCurrentCategory = currentCategory === category;

              if (categoryResults.length === 0 && !isCurrentCategory) {
                return null;
              }

              return (
                <CategorySection
                  key={category}
                  category={category}
                  results={categoryResults}
                  categoryStatus={isCurrentCategory ? 'running' : 'complete'}
                  defaultExpanded={isCurrentCategory || categoryResults.some(r => !r.passed)}
                />
              );
            })}
          </div>
        )}
      </div>

      {/* Debug / Error Simulation */}
      <div className="px-4 py-3 bg-gray-800/30 border-t border-gray-700 flex items-center justify-between">
        <div className="flex items-center gap-2 text-xs text-gray-400">
          <Bug size={14} />
          <span>{t('diagnostics.debugTools', 'Debug Tools')}</span>
        </div>
        <button
          onClick={() => setSimulateCrash(true)}
          className="px-3 py-1.5 bg-destructive/20 text-destructive hover:bg-destructive/30 border border-destructive/30 rounded-lg text-xs font-medium transition-colors flex items-center gap-2"
        >
          <AlertTriangle size={12} />
          {t('diagnostics.simulateCrash', 'Simulate Crash')}
        </button>
      </div>

      {simulateCrash && <BuggyComponent />}

      {/* Footer with summary */}
      {status === 'complete' && summary && (
        <div
          className={cn(
            'px-4 py-3 border-t border-gray-700',
            summary.failed === 0 ? 'bg-green-900/20' : 'bg-red-900/20',
          )}
        >
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              {summary.failed === 0 ? (
                <>
                  <CheckCircle
                    size={20}
                    className="text-green-500"
                  />
                  <span className="text-green-400 font-medium">{t('diagnostics.allPassed', 'All tests passed!')}</span>
                </>
              ) : (
                <>
                  <AlertTriangle
                    size={20}
                    className="text-yellow-500"
                  />
                  <span className="text-yellow-400 font-medium">
                    {t('diagnostics.someFailures', '{{count}} test(s) failed', { count: summary.failed })}
                  </span>
                </>
              )}
            </div>

            <span className="text-sm text-gray-400">
              {t('diagnostics.summary', '{{passed}}/{{total}} tests passed', {
                passed: summary.passed,
                total: summary.total,
              })}
            </span>
          </div>
        </div>
      )}
    </div>
  );
}

export default TestConsole;
