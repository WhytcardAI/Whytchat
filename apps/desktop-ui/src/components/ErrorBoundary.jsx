import React from 'react';
import { AlertTriangle, RefreshCw, Home } from 'lucide-react';

export class ErrorBoundary extends React.Component {
  constructor(props) {
    super(props);
    this.state = { hasError: false, error: null, errorInfo: null };
  }

  static getDerivedStateFromError(error) {
    // Update state so the next render will show the fallback UI.
    return { hasError: true, error };
  }

  componentDidCatch(error, errorInfo) {
    // You can also log the error to an error reporting service
    console.error("Uncaught error:", error, errorInfo);
    this.setState({ errorInfo });
  }

  handleReload = () => {
    window.location.reload();
  };

  handleGoHome = () => {
    // Attempt to recover by clearing state/storage if needed, or just reload to root
    window.location.href = '/';
  };

  render() {
    if (this.state.hasError) {
      return (
        <div className="h-screen w-screen flex flex-col items-center justify-center bg-background p-6 text-center">
          <div className="w-20 h-20 rounded-full bg-destructive/10 flex items-center justify-center mb-6 animate-in zoom-in duration-300">
            <AlertTriangle className="w-10 h-10 text-destructive" />
          </div>

          <h1 className="text-2xl font-bold text-foreground mb-2">Something went wrong</h1>
          <p className="text-muted-foreground max-w-md mb-8">
            The application encountered an unexpected error. We&apos;ve logged the issue and you can try reloading to fix it.
          </p>

          {this.state.error && (
            <div className="w-full max-w-lg bg-surface border border-border rounded-lg p-4 mb-8 text-left overflow-auto max-h-48">
              <p className="font-mono text-xs text-destructive mb-2 font-semibold">
                {this.state.error.toString()}
              </p>
              {this.state.errorInfo && (
                <pre className="font-mono text-[10px] text-muted-foreground whitespace-pre-wrap">
                  {this.state.errorInfo.componentStack}
                </pre>
              )}
            </div>
          )}

          <div className="flex gap-4">
            <button
              onClick={this.handleReload}
              className="flex items-center gap-2 px-4 py-2 bg-primary text-primary-foreground rounded-lg hover:opacity-90 transition-colors"
            >
              <RefreshCw size={16} />
              Reload Application
            </button>
            <button
              onClick={this.handleGoHome}
              className="flex items-center gap-2 px-4 py-2 bg-surface border border-border text-foreground rounded-lg hover:bg-muted transition-colors"
            >
              <Home size={16} />
              Go Home
            </button>
          </div>
        </div>
      );
    }

    return this.props.children;
  }
}
