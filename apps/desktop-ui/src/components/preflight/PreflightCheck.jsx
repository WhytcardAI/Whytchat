import { CheckCircle, XCircle, AlertTriangle, RefreshCw } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { cn } from '../../lib/utils';

/**
 * Component to display preflight check results when checks fail.
 * Shows detailed status of each system component.
 */
export function PreflightCheck({ report, onRetry }) {
  const { t } = useTranslation('common');

  if (!report) return null;

  const criticalChecks = ['directories', 'model_file', 'llama_server_binary', 'database'];

  const getCheckIcon = (check) => {
    if (check.passed) {
      return <CheckCircle className="w-5 h-5 text-green-500" />;
    }
    if (criticalChecks.includes(check.name)) {
      return <XCircle className="w-5 h-5 text-red-500" />;
    }
    return <AlertTriangle className="w-5 h-5 text-yellow-500" />;
  };

  const getCheckLabel = (name) => {
    const labels = {
      directories: t('preflight.checks.directories', 'System Directories'),
      model_file: t('preflight.checks.model_file', 'AI Model (GGUF)'),
      llama_server_binary: t('preflight.checks.llama_server', 'Inference Server'),
      embeddings: t('preflight.checks.embeddings', 'Embedding Model'),
      database: t('preflight.checks.database', 'Database'),
      vectors_dir: t('preflight.checks.vectors', 'Vector Storage'),
      llama_server_startup: t('preflight.checks.llama_startup', 'Server Startup Test'),
      embeddings_load: t('preflight.checks.embeddings_load', 'Embeddings Load Test'),
    };
    return labels[name] || name;
  };

  return (
    <div className="h-screen w-screen flex items-center justify-center bg-background p-8">
      <div className="w-full max-w-2xl bg-surface rounded-2xl shadow-xl border border-border overflow-hidden">
        {/* Header */}
        <div className="bg-red-500/10 border-b border-red-500/20 p-6">
          <div className="flex items-center gap-3">
            <XCircle className="w-8 h-8 text-red-500" />
            <div>
              <h1 className="text-xl font-bold text-text">
                {t('preflight.title', 'System Check Failed')}
              </h1>
              <p className="text-sm text-muted mt-1">
                {report.summary}
              </p>
            </div>
          </div>
        </div>

        {/* Checks List */}
        <div className="p-6 space-y-3 max-h-[400px] overflow-y-auto">
          {report.checks.map((check, index) => (
            <div
              key={index}
              className={cn(
                "flex items-start gap-3 p-3 rounded-lg border",
                check.passed
                  ? "bg-green-500/5 border-green-500/20"
                  : criticalChecks.includes(check.name)
                  ? "bg-red-500/5 border-red-500/20"
                  : "bg-yellow-500/5 border-yellow-500/20"
              )}
            >
              {getCheckIcon(check)}
              <div className="flex-1 min-w-0">
                <div className="flex items-center justify-between">
                  <span className="font-medium text-text">
                    {getCheckLabel(check.name)}
                  </span>
                  <span
                    className={cn(
                      "text-xs px-2 py-0.5 rounded-full",
                      check.passed
                        ? "bg-green-500/20 text-green-700"
                        : "bg-red-500/20 text-red-700"
                    )}
                  >
                    {check.passed ? t('preflight.passed', 'OK') : t('preflight.failed', 'FAILED')}
                  </span>
                </div>
                <p className="text-sm text-muted mt-1">{check.message}</p>
                {check.details && (
                  <p className="text-xs text-muted/70 mt-1 font-mono bg-background/50 p-2 rounded">
                    {check.details}
                  </p>
                )}
              </div>
            </div>
          ))}
        </div>

        {/* Actions */}
        <div className="border-t border-border p-6 bg-background/50">
          <div className="flex items-center justify-between">
            <p className="text-sm text-muted">
              {t('preflight.hint', 'Fix the issues above and try again.')}
            </p>
            <button
              onClick={onRetry}
              className="flex items-center gap-2 px-4 py-2 bg-primary text-white rounded-lg hover:bg-primary/90 transition-colors"
            >
              <RefreshCw className="w-4 h-4" />
              {t('preflight.retry', 'Retry')}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
