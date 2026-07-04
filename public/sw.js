/**
 * Trix Audio Converter — Service Worker
 *
 * Handles offline caching with stale-while-revalidate strategy.
 * Manages three cache buckets: static assets, fonts, and general resources.
 * Supports background sync for queued operations.
 *
 * @author João Vitor de Melo <joaovmelo259@gmail.com>
 * @version 2.0.0
 * @license MIT
 */

const CACHE_NAME = 'trix-audio-converter-v2';
const CACHE_STATIC = 'trix-static-v2';
const CACHE_FONTS = 'trix-fonts-v1';
const STATIC_ASSETS = [
  '/',
  '/trix_logo_sunset.png',
  '/manifest.json',
];

// Install event - cache static assets
self.addEventListener('install', (event) => {
  event.waitUntil(
    caches.open(CACHE_STATIC).then((cache) => {
      return cache.addAll(STATIC_ASSETS);
    }).then(() => caches.open(CACHE_NAME))
  );
  self.skipWaiting();
});

// Activate event - clean up old caches
self.addEventListener('activate', (event) => {
  const KEEP_CACHES = new Set([CACHE_NAME, CACHE_STATIC, CACHE_FONTS]);
  event.waitUntil(
    caches.keys().then((cacheNames) => {
      return Promise.all(
        cacheNames
          .filter((name) => !KEEP_CACHES.has(name))
          .map((name) => caches.delete(name))
      );
    })
  );
  self.clients.claim();
});

// Fetch event - serve from cache, fallback to network
self.addEventListener('fetch', (event) => {
  if (event.request.method !== 'GET') return;
  if (event.request.url.includes('/api/')) return;
  if (event.request.url.startsWith('ws://') || event.request.url.startsWith('wss://')) return;

  const url = new URL(event.request.url);

  // Fonts: cache-first with network fallback
  if (url.hostname.includes('fonts.googleapis.com') || url.hostname.includes('fonts.gstatic.com')) {
    event.respondWith(
      caches.open(CACHE_FONTS).then((cache) => {
        return cache.match(event.request).then((cached) => {
          if (cached) return cached;
          return fetch(event.request).then((response) => {
            if (response.ok) cache.put(event.request, response.clone());
            return response;
          }).catch(() => new Response('', { status: 503, statusText: 'Offline' }));
        });
      })
    );
    return;
  }

  // Static assets (JS/CSS from /assets/): cache-first
  if (url.pathname.startsWith('/assets/')) {
    event.respondWith(
      caches.open(CACHE_STATIC).then((cache) => {
        return cache.match(event.request).then((cached) => {
          if (cached) return cached;
          return fetch(event.request).then((response) => {
            if (response.ok) cache.put(event.request, response.clone());
            return response;
          }).catch(() => new Response('', { status: 503, statusText: 'Offline' }));
        });
      })
    );
    return;
  }

  // Navigation and other: stale-while-revalidate
  event.respondWith(
    caches.match(event.request).then((cachedResponse) => {
      const fetchPromise = fetch(event.request).then((networkResponse) => {
        if (networkResponse && networkResponse.status === 200 && networkResponse.type === 'basic') {
          const responseToCache = networkResponse.clone();
          caches.open(CACHE_NAME).then((cache) => {
            cache.put(event.request, responseToCache);
          });
        }
        return networkResponse;
      }).catch(() => {
        // Offline fallback for navigation requests
        if (event.request.mode === 'navigate') {
          return caches.match('/');
        }
        return new Response('Offline', { status: 503 });
      });

      return cachedResponse || fetchPromise;
    })
  );
});

// Background sync for offline operations
self.addEventListener('sync', (event) => {
  if (event.tag === 'sync-conversions') {
    event.waitUntil(syncConversions());
  }
});

// Offline queue via IndexedDB
const DB_NAME = 'trix-offline-queue';
const DB_VERSION = 1;
const STORE_NAME = 'pending-conversions';

function openDB() {
  return new Promise((resolve, reject) => {
    const req = indexedDB.open(DB_NAME, DB_VERSION);
    req.onupgradeneeded = () => {
      const db = req.result;
      if (!db.objectStoreNames.contains(STORE_NAME)) {
        db.createObjectStore(STORE_NAME, { keyPath: 'id', autoIncrement: true });
      }
    };
    req.onsuccess = () => resolve(req.result);
    req.onerror = () => reject(req.error);
  });
}

async function addToOfflineQueue(options) {
  const db = await openDB();
  return new Promise((resolve, reject) => {
    const tx = db.transaction(STORE_NAME, 'readwrite');
    tx.objectStore(STORE_NAME).add({ options, timestamp: Date.now() });
    tx.oncomplete = () => resolve();
    tx.onerror = () => reject(tx.error);
  });
}

async function getOfflineQueue() {
  const db = await openDB();
  return new Promise((resolve, reject) => {
    const tx = db.transaction(STORE_NAME, 'readonly');
    const req = tx.objectStore(STORE_NAME).getAll();
    req.onsuccess = () => resolve(req.result);
    req.onerror = () => reject(req.error);
  });
}

async function clearOfflineQueue() {
  const db = await openDB();
  return new Promise((resolve, reject) => {
    const tx = db.transaction(STORE_NAME, 'readwrite');
    tx.objectStore(STORE_NAME).clear();
    tx.oncomplete = () => resolve();
    tx.onerror = () => reject(tx.error);
  });
}

async function syncConversions() {
  try {
    const pending = await getOfflineQueue();
    if (pending.length === 0) return;

    for (const item of pending) {
      try {
        const tokenMeta = document.querySelector('meta[name="api-token"]');
        const token = tokenMeta ? tokenMeta.getAttribute('content') || '' : '';
        const resp = await fetch('/api/start', {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
            'Authorization': `Bearer ${token}`,
          },
          body: JSON.stringify(item.options),
        });
        if (!resp.ok) throw new Error(`HTTP ${resp.status}`);
      } catch (err) {
        console.error('[SW] Failed to sync item:', err);
      }
    }
    await clearOfflineQueue();
    // Notify clients that sync completed
    const clients = await self.clients.matchAll();
    clients.forEach(client => client.postMessage({ type: 'offline-sync-done' }));
  } catch (err) {
    console.error('[SW] syncConversions error:', err);
  }
}

// Listen for messages from the main thread
self.addEventListener('message', (event) => {
  if (event.data && event.data.type === 'add-offline-queue') {
    addToOfflineQueue(event.data.options).then(() => {
      console.log('[SW] Added to offline queue');
    });
  }
});