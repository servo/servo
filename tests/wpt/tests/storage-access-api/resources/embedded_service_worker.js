"use strict";

// Replies to commands sent via postMessage. Used by the tests that check
// whether a ServiceWorker's requests inherit the registering frame's storage
// access.
self.addEventListener("install", () => self.skipWaiting());
self.addEventListener("activate", (event) => event.waitUntil(self.clients.claim()));

self.addEventListener("message", async (message) => {
  const reply = (data) => message.source.postMessage({data});
  switch (message.data.command) {
  case "fetch": {
    const data = await fetch(message.data.url, {mode: 'cors', credentials: 'include'})
      .then((resp) => resp.text());
    reply(data);
    break;
  }
  case "websocket": {
    const data = await new Promise((resolve, reject) => {
      const ws = new WebSocket(message.data.origin + '/echo-cookie');
      ws.onmessage = (event) => resolve(event.data);
      ws.onerror = () => reject(new Error('Unexpected error event'));
      ws.onclose = (event) => reject('Unexpected close event: ' + JSON.stringify(event));
    }).catch((error) => "error: " + error);
    reply(data);
    break;
  }
  default:
  }
});
