import React from 'react'
import ReactDOM from 'react-dom/client'
import './i18n'
import './index.css'
import App from './App'

ReactDOM.createRoot(document.getElementById('root')).render(
  <React.StrictMode>
    <React.Suspense fallback={<div className="p-4 text-white">Loading...</div>}>
      <App />
    </React.Suspense>
  </React.StrictMode>
)
