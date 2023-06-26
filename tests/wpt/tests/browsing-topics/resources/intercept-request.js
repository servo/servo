self.addEventListener('fetch', event => {
  event.waitUntil(async function () {
    if (!event.clientId) return;
    const client = await clients.get(event.clientId);
    if (!client) return;

    client.postMessage({
      fetchUrl: event.request.url,
      topicsHeader: String(event.request.headers.get("Sec-Browsing-Topics"))
    });
  }());
});
