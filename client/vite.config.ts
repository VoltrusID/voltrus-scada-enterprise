import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

export default defineConfig({
  plugins: [react()],
  server: {
    proxy: {
      // Server serves the built client same-origin in production, so every
      // API path (/api/v1, /api/license, /api/simulation, ...) must reach it
      // in dev too. ws: true forwards the /api/v1/ws live-data socket.
      '/api': {
        target: 'http://localhost:3100',
        ws: true,
      }
    }
  },
  build: {
    outDir: 'dist',
    sourcemap: false,
    minify: true,
    rolldown: {
      drop: ['console', 'debugger'],
    },
    rollupOptions: {
      output: {
        manualChunks(id) {
          if (id.includes('/node_modules/three/')) return 'three'
          if (id.includes('/node_modules/react/') || id.includes('/node_modules/react-dom/')) return 'vendor-react'
          if (id.includes('/node_modules/lightweight-charts/')) return 'lightweight-charts'
        },
      },
    },
  }
})
