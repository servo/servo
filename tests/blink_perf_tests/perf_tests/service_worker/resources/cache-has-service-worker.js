importScripts('/resources/worker-test-helper.js');

// Send the result to all clients to finish the test.
function sendTestResultToClients(result) {
  clients.matchAll({includeUncontrolled: true}).then((allClients) => {
    for (const client of allClients) {
      client.postMessage(result);
    }
  });
}

async function setup() {
  for (let i = 0; i < 100; i++) {
    const cache = await caches.open(`test_cache_${i}`);
    await cache.add(`/service_worker/resources/data/1K_${i}.txt`);
  }
}

async function cacheStorageHas() {
  await caches.has('test_cache_50');
  await caches.has('test_cache_101');
}

async function test() {
  const result = await self.workerTestHelper.measureRunsPerSecond({
    setup: setup,
    run: cacheStorageHas,
  });
  sendTestResultToClients(result);
}

self.addEventListener('install', function(event) {
  event.waitUntil(test());
});
