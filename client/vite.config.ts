import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

export default defineConfig({
  plugins: [react()],
  server: {
    proxy: {
      '/api/v1': 'http://localhost:3100'
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
