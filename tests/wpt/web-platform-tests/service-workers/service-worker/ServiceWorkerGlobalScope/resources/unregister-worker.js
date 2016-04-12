function matchQuery(query) {
  return self.location.href.indexOf(query) != -1;
}

if (matchQuery('?evaluation'))
  self.registration.unregister();

self.addEventListener('install', function(e) {
    if (matchQuery('?install'))
      self.registration.unregister();
  });

self.addEventListener('activate', function(e) {
    if (matchQuery('?activate'))
      self.registration.unregister();
  });

self.addEventListener('message', function(e) {
    self.registration.unregister()
      .then(function(result) {
          e.data.port.postMessage({result: result});
        });
  });
