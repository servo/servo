self.addEventListener('securitypolicyviolation', e => {
  let context = 'DedicatedWorker';
  if (typeof SharedWorkerGlobalScope !== 'undefined' && self instanceof SharedWorkerGlobalScope) {
    context = 'SharedWorker';
  } else if (typeof ServiceWorkerGlobalScope !== 'undefined' && self instanceof ServiceWorkerGlobalScope) {
    context = 'ServiceWorker';
  }

  const msg = {
    type: 'violation',
    blockedURI: e.blockedURI,
    urlHash: e.urlHash,
    evalHash: e.evalHash
  };

  if (context === 'DedicatedWorker') {
    self.postMessage(msg);
  } else if (context === 'SharedWorker') {
    if (self.sharedPorts) {
      self.sharedPorts.forEach(port => port.postMessage(msg));
    }
  } else if (context === 'ServiceWorker') {
    self.clients.matchAll({ includeUncontrolled: true }).then(clients => {
      clients.forEach(client => client.postMessage(msg));
    });
  }
});

// The import is blocked by the CSP of this worker. Catch the resulting
// NetworkError, so that the worker's script evaluation doesn't fail; for a
// service worker that would make its registration fail, which can tear down
// the worker before it has reported the violation.
function importBlockedScript() {
  try {
    importScripts('externalScript.js');
  } catch {}
}

if (typeof SharedWorkerGlobalScope !== 'undefined' && self instanceof SharedWorkerGlobalScope) {
  self.sharedPorts = [];
  self.addEventListener('connect', e => {
    self.sharedPorts.push(e.ports[0]);
    importBlockedScript();
  });
} else {
  importBlockedScript();
}
