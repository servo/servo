// The source to post setup and completion results to.
let source = null;

function sendMessageToDocument(msg) {
  source.postMessage(msg);
}

// Notify the document that the SW is registered and ready.
self.addEventListener('message', event => {
  source = event.source;
  sendMessageToDocument('ready');
});

self.addEventListener('periodicsync', event => {
  sendMessageToDocument('periodicsync event received!');
});
