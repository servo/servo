// The source to post setup and completion results to.
let source = null;

function sendMessageToDocument(msg) {
  source.postMessage(msg);
}

// This is needed to create a local javascript object identical to the
// one returned by a BackgroundFetchEvent, so that it can be serialized
// and transmitted from the service worker context to the document.
function cloneRegistration(registration) {
  function deepCopy(src) {
    if (typeof src !== 'object' || src === null)
        return src;
    var dst = Array.isArray(src) ? [] : {};
    for (var property in src) {
        if (typeof src[property] === 'function')
            continue;
        dst[property] = deepCopy(src[property]);
    }
    return dst;
  }

  return deepCopy(registration);
}

// Notify the document that the SW is registered and ready.
self.addEventListener('message', event => {
  source = event.source;
  sendMessageToDocument('ready');
});
