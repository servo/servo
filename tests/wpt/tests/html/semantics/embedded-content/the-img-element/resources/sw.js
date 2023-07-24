addEventListener('install', (event) => {
  skipWaiting();
});

addEventListener('activate', (event) => {
  event.waitUntil(clients.claim());
});

async function broadcast(msg) {
  const allClients = await clients.matchAll();
  for (const client of allClients) {
    client.postMessage(msg);
  }
}

addEventListener('fetch', (event) => {
  event.waitUntil(
    broadcast({ url: event.request.url, mode: event.request.mode })
  )
});
