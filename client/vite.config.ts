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
          // Three.js is ~800KB — split it into its own chunk for better caching
          'three': ['three'],
          // React vendor chunk for long-term caching
          'vendor-react': ['react', 'react-dom'],
        },
      },
    },
  }
})
