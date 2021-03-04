addEventListener("fetch", event => {
  event.waitUntil(async function () {
    if (!event.clientId) return;
    const client = await clients.get(event.clientId);
    if (!client) return;

    client.postMessage({
      "dest": event.request.headers.get("sec-fetch-dest"),
      "mode": event.request.headers.get("sec-fetch-mode"),
      "site": event.request.headers.get("sec-fetch-site"),
      "user": event.request.headers.get("sec-fetch-user")
    });
  }());
});
