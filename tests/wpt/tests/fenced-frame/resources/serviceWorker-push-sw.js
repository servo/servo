self.addEventListener('install', e => e.waitUntil(skipWaiting()));
self.addEventListener('activate', e => e.waitUntil(clients.claim()));

self.addEventListener('message', async e => {
  const method = e.data;

  const promise = method === 'subscribe' ?
      self.registration.pushManager.subscribe({userVisibleOnly: true}) :
      Promise.resolve();
  const message = await promise
                      .then(() => {
                        return `${method}: Unexpectedly started`;
                      })
                      .catch((e) => {
                        return e.message;
                      });

  e.source.postMessage(message);
});
