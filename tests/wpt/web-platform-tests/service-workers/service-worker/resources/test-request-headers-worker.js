// Add a unique timestamp per request to induce service worker script update.
// Time stamp: %TIMESTAMP%

// The server injects the request headers here as a JSON string.
const headersAsJson = `%HEADERS%`;
const headers = JSON.parse(headersAsJson);

self.addEventListener('message', async (e) => {
  e.source.postMessage(headers);
});
