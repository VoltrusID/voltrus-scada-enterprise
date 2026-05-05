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
    rollupOptions: {
      output: {
        manualChunks: {
          // Three.js (~730KB) — only loaded when 3D twin view is opened
          'three': ['three'],
          // React vendor chunk for long-term caching
          'vendor-react': ['react', 'react-dom'],
          // Fabric.js (~300KB) — only used by SCADA canvas
          'fabric': ['fabric'],
          // TradingView lightweight-charts (~190KB) — only used by TvDetailChart
          'lightweight-charts': ['lightweight-charts'],
        },
      },
    },
  }
})
