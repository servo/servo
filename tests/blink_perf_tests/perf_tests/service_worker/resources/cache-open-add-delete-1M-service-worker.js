importScripts('/resources/worker-test-helper.js');

// Send the result to all clients to finish the test.
function sendTestResultToClients(result) {
  clients.matchAll({includeUncontrolled: true}).then((allClients) => {
    for (const client of allClients) {
      client.postMessage(result);
    }
  });
}

const CACHE_NAME = 'test-cache';

async function cacheOpenAddDelete() {
  const cache = await caches.open(CACHE_NAME);
  await cache.add('/service_worker/resources/data/1M.txt');
  await cache.delete(CACHE_NAME);
}

async function test() {
  const result = await self.workerTestHelper.measureRunsPerSecond({
    run: cacheOpenAddDelete
  });
  sendTestResultToClients(result);
}

self.addEventListener('install', function(event) {
  event.waitUntil(test());
});
