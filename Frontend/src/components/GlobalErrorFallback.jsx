import PropTypes from "prop-types";

export default function GlobalErrorFallback({ error }) {
  return (
    <div className="flex flex-col items-center justify-center h-screen w-screen bg-black text-white p-8 text-center">
      <h1 className="text-3xl font-bold text-red-500 mb-4">Something went wrong</h1>
      <p className="text-muted-foreground mb-6">The application encountered a critical error.</p>
      <pre className="bg-white/10 p-4 rounded-lg text-left text-xs overflow-auto max-w-2xl mb-6 border border-white/10">
        {error.message}
      </pre>
      <button
        onClick={() => window.location.reload()}
        className="px-6 py-3 bg-primary text-white rounded-full hover:bg-primary/90 transition-colors font-medium"
      >
        Reload Application
      </button>
    </div>
  );
}

GlobalErrorFallback.propTypes = {
  error: PropTypes.shape({
    message: PropTypes.string.isRequired,
  }).isRequired,
};
