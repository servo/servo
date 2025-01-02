self.addEventListener('install', e => e.waitUntil(skipWaiting()));
self.addEventListener('activate', e => e.waitUntil(clients.claim()));

self.addEventListener('message', async event => {
  const method = event.data;

  if (method === 'constructor') {
    try {
      new Notification('test');
    } catch (e) {
      event.source.postMessage(e.message);
    }
  } else if (method === 'showNotification') {
    try {
      await self.registration.showNotification('test', {body: 'test'});
    } catch (e) {
      event.source.postMessage(e.message);
    }
  }
});
