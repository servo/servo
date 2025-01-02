self.addEventListener('install', e => e.waitUntil(skipWaiting()));
self.addEventListener('activate', e => e.waitUntil(clients.claim()));

self.addEventListener('message', async event => {
  const method = event.data;
  const {index} = self.registration;
  const id = 'fenced-frame-id-sw';

  let promise;
  if (method === 'add') {
    promise = index.add({
      id,
      title: 'same title',
      description: 'same description',
      url: 'resources/'
    });
  } else if (method === 'delete') {
    promise = index.delete(id);
  } else if (method === 'getAll') {
    promise = index.getAll();
  } else {
    promise = Promise.resolve();
  }

  const message = await promise.then(() => 'success').catch(e => e.message);

  event.source.postMessage(message);
});
