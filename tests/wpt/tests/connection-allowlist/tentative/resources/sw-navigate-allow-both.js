self.addEventListener('install', e => self.skipWaiting());
self.addEventListener('activate', e => e.waitUntil(self.clients.claim()));

self.addEventListener('message', event => {
  event.waitUntil(async function() {
    let logs = [];
    logs.push("received message");
    const target_url = event.data.url;
    const port = event.data.port;
    try {
      const clients = await self.clients.matchAll({type: 'window'});
      logs.push(`found ${clients.length} clients`);
      for (const client of clients) {
        logs.push(`navigating client ${client.id} to ${target_url}`);
        try {
          const result = await client.navigate(target_url);
          if (result === null) {
            logs.push("navigate resolved to null");
            port.postMessage({result: 'failure', error: 'null', logs: logs});
          } else {
            logs.push("navigate resolved");
            port.postMessage({result: 'success', logs: logs});
          }
        } catch (e) {
          logs.push(`navigate rejected: ${e.name} - ${e.message}`);
          port.postMessage({result: 'failure', error: e.name, logs: logs});
        }
      }
      if (clients.length === 0) {
         port.postMessage({result: 'no_clients', logs: logs});
      }
    } catch (err) {
      logs.push(`matchAll failed: ${err.message}`);
      port.postMessage({result: 'error', error: err.name, logs: logs});
    }
  }());
});
