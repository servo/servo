importScripts("helpers.js");

// Send the result to the clients of this service worker.
async function ping(message) {
  const clients = await self.clients.matchAll({ includeUncontrolled: true });
  for (const client of clients) {
    client.postMessage(message);
  }
  for (const n of await registration.getNotifications()) {
    n.close();
  }
}

(async () => {
  await untilActivate();

  const shown = await registration.showNotification("serviceworker").then(() => true, err => false);
  ping({ shown });
})();
