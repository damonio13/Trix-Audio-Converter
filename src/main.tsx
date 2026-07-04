/**
 * Trix Audio Converter � main
 *
 * @author Jo�o Vitor de Melo <joaovmelo259@gmail.com>
 * @version 1.0.0
 * @license MIT
 */
/**
 * React application entry point
 *
 * @author João Vitor de Melo <joaovmelo259@gmail.com>
 * @version 1.0.0
 * @license MIT
 */
import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import App from './App'
import './index.css'

/**
 * Suppresses the browser's native context menu so the app feels native
 * and prevents users from accessing browser inspect tools via right-click.
 */
function disableContextMenu(e: MouseEvent) {
  e.preventDefault()
}

/**
 * Blocks common browser developer-tool keyboard shortcuts to prevent
 * casual inspection of the WebView source in the packaged desktop app.
 *
 * Blocked shortcuts:
 * - F12 — DevTools toggle
 * - Ctrl/⌘+Shift+I/J/C/K — DevTools panels
 * - Ctrl/⌘+U — View Source
 * - Ctrl/⌘+R / Ctrl/⌘+Shift+R — browser reload
 * - Ctrl/⌘+S — Save page
 * - Ctrl/⌘+P — Print
 */
function disableShortcuts(e: KeyboardEvent) {
  const isCtrl = e.ctrlKey || e.metaKey
  const isShift = e.shiftKey

  if (e.key === 'F12') {
    e.preventDefault()
    return
  }

  if (isCtrl && isShift && ['I', 'J', 'C', 'K'].includes(e.key.toUpperCase())) {
    e.preventDefault()
    return
  }

  if (isCtrl && ['U', 'R', 'S', 'P'].includes(e.key.toUpperCase())) {
    e.preventDefault()
    return
  }

  if (isCtrl && isShift && e.key.toUpperCase() === 'R') {
    e.preventDefault()
    return
  }
}

document.addEventListener('contextmenu', disableContextMenu)
document.addEventListener('keydown', disableShortcuts)

// Register the Service Worker for offline-first PWA support.
// When a new SW version is found during `updatefound`, it logs a notice
// so the developer knows a manual refresh will apply the update.
if ('serviceWorker' in navigator) {
  window.addEventListener('load', () => {
    navigator.serviceWorker.register('/sw.js')
      .then((registration) => {
        console.log('[SW] Registered:', registration.scope);
        
        registration.addEventListener('updatefound', () => {
          const newWorker = registration.installing;
          if (newWorker) {
            newWorker.addEventListener('statechange', () => {
              if (newWorker.state === 'installed' && navigator.serviceWorker.controller) {
                console.log('[SW] New version available, please refresh');
              }
            });
          }
        });
      })
      .catch((error) => {
        console.error('[SW] Registration failed:', error);
      });
  });
}

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <App />
  </StrictMode>,
)
