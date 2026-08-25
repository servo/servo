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

if (typeof SharedWorkerGlobalScope !== 'undefined' && self instanceof SharedWorkerGlobalScope) {
  self.sharedPorts = [];
  self.addEventListener('connect', e => {
    self.sharedPorts.push(e.ports[0]);
    importScripts('externalScript.js');
  });
} else {
  importScripts('externalScript.js');
}
