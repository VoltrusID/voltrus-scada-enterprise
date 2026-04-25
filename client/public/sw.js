// Service worker shell for Web Push Notifications (groundwork)
self.addEventListener('install', () => {
  console.log('[SW] Installed');
  self.skipWaiting();
});

self.addEventListener('activate', () => {
  console.log('[SW] Activated');
});

self.addEventListener('push', (event) => {
  console.log('[SW] Push received:', event.data?.text());
  // TODO: show notification using self.registration.showNotification()
});
