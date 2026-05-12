import './i18n'
import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import './index.css'
import 'react-grid-layout/css/styles.css'
import 'react-resizable/css/styles.css'
import App from './App'
import { toast } from './components/Toast'

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 30_000,
      gcTime: 5 * 60_000,
      refetchOnWindowFocus: false,
      retry: 1,
    },
    mutations: {
      onError: (err: any) => {
        toast.error(err.message || 'Request failed')
      },
    },
  },
})

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <QueryClientProvider client={queryClient}>
      <App />
    </QueryClientProvider>
  </StrictMode>
)

// Register service worker for push notifications
import { registerServiceWorker } from './sw-register'
registerServiceWorker()
