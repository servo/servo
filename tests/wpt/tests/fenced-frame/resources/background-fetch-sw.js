self.addEventListener('install', e => e.waitUntil(skipWaiting()));
self.addEventListener('activate', e => e.waitUntil(clients.claim()));

self.addEventListener('message', async e => {
  const method = e.data;

  let promise;
  switch (method) {
    case 'fetch':
      promise = self.registration.backgroundFetch.fetch(
          'test-fetch', ['background-fetch-inner.https.html.headers'],
          {title: 'Background Fetch'});
      break;
    case 'get':
      promise = self.registration.backgroundFetch.get('test-fetch')
      break;
    case 'getIds':
      promise = registration.backgroundFetch.getIds();
      break;
    default:
      promise = Promise.resolve();
      break;
  }

  const message =
      await promise
          .then(() => {
            return `[backgroundFetch.${method}] Unexpectedly started`;
          })
          .catch((e) => {
            return `[backgroundFetch.${
                method}] Failed inside fencedframe as expected`;
          });

  e.source.postMessage(message);
});
