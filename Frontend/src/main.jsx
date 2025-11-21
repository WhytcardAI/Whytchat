import "./shims/disableViewTransitions";
import React from "react";
import ReactDOM from "react-dom/client";
import { ErrorBoundary } from "react-error-boundary";
import App from "./App.jsx";
import GlobalErrorFallback from "./components/GlobalErrorFallback";
import "./index.css";
import "./i18n";

ReactDOM.createRoot(document.getElementById("root")).render(
  <React.StrictMode>
    <ErrorBoundary FallbackComponent={GlobalErrorFallback}>
      <App />
    </ErrorBoundary>
  </React.StrictMode>
);
