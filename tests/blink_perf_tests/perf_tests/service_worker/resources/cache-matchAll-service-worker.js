importScripts('/resources/worker-test-helper.js');

// Send the result to all clients to finish the test.
function sendTestResultToClients(result) {
  clients.matchAll({includeUncontrolled: true}).then((allClients) => {
    for (const client of allClients) {
      client.postMessage(result);
    }
  });
}

const FILES = [];
for (let i = 0; i < 100; i++) {
  FILES.push(`/service_worker/resources/data/1K_${i}.txt`);
}

let cache = null;
async function setup() {
  cache = await caches.open('test-cache');
  await cache.addAll(FILES);
}

async function cacheMatchAll() {
  await cache.matchAll();
}

async function test() {
  const result = await self.workerTestHelper.measureTime({
    setup: setup,
    run: cacheMatchAll,
    iterationCount: 10
  });
  sendTestResultToClients(result);
}

self.addEventListener('install', function(event) {
  event.waitUntil(test());
});
