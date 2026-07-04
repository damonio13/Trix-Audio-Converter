/**
 * Trix Audio Converter — Vite Configuration
 *
 * @author João Vitor de Melo <joaovmelo259@gmail.com>
 * @version 1.0.0
 * @license MIT
 */

import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import path from 'path'

export default defineConfig({
  plugins: [react()],

  resolve: {
    alias: {
      // Maps `@/` imports to `./src/` so components can use `@/utils/api`
      // instead of relative `../../utils/api` paths.
      '@': path.resolve(__dirname, './src'),
    },
  },

  server: {
    // Fixed port so run.bat can reliably detect the Vite dev server via TCP.
    port: 8888,
    strictPort: true,
    proxy: {
      // Forwards all `/api/*` calls from the dev server to the Rust backend
      // (port 3939). This avoids CORS issues during local development.
      '/api': {
        target: 'http://localhost:3939',
        changeOrigin: true,
      },
    },
  },

  build: {
    // Output directory consumed by the Rust backend's `ServeDir` static file server.
    outDir: 'dist',
    // Source maps disabled in production to reduce bundle size and prevent source exposure.
    sourcemap: false,
  },
})
