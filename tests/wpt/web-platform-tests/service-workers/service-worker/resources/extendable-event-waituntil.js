var pendingPorts = [];
var portResolves = [];

onmessage = function(e) {
  var message = e.data;
  if ('port' in message) {
    var resolve = self.portResolves.shift();
    if (resolve)
      resolve(message.port);
    else
      self.pendingPorts.push(message.port);
  }
};

function fulfillPromise() {
  return new Promise(function(resolve) {
      // Make sure the oninstall/onactivate callback returns first.
      Promise.resolve().then(function() {
          var port = self.pendingPorts.shift();
          if (port)
            resolve(port);
          else
            self.portResolves.push(resolve);
        });
    }).then(function(port) {
        port.postMessage('SYNC');
        return new Promise(function(resolve) {
            port.onmessage = function(e) {
              if (e.data == 'ACK')
                resolve();
            };
          });
      });
}

function rejectPromise() {
  return new Promise(function(resolve, reject) {
      // Make sure the oninstall/onactivate callback returns first.
      Promise.resolve().then(reject);
    });
}

function stripScopeName(url) {
  return url.split('/').slice(-1)[0];
}

oninstall = function(e) {
  switch (stripScopeName(self.location.href)) {
    case 'install-fulfilled':
      e.waitUntil(fulfillPromise());
      break;
    case 'install-rejected':
      e.waitUntil(rejectPromise());
      break;
    case 'install-multiple-fulfilled':
      e.waitUntil(fulfillPromise());
      e.waitUntil(fulfillPromise());
      break;
    case 'install-reject-precedence':
      e.waitUntil(fulfillPromise());
      e.waitUntil(rejectPromise());
      break;
  }
};

onactivate = function(e) {
  switch (stripScopeName(self.location.href)) {
    case 'activate-fulfilled':
      e.waitUntil(fulfillPromise());
      break;
    case 'activate-rejected':
      e.waitUntil(rejectPromise());
      break;
  }
};
