script = '''
var initPromise = Promise.resolve();

if ('SharedWorkerGlobalScope' in self &&
    self instanceof SharedWorkerGlobalScope) {
  initPromise = new Promise(resolve => {
     self.addEventListener('connect', event => {
          self.postMessage = msg => { event.ports[0].postMessage(msg); };
          resolve();
        });
    });
}

var info = '';

function importCachedScriptTest() {
  return new Promise((resolve, reject) => {
    info = '';
    try {
      importScripts('appcache-worker-import.py?type=cached');
    } catch(e) {
      reject(new Error('Error while importing the cached script: ' +
                       e.toString()));
      return;
    }
    if (info != 'Set by the cached script') {
      reject(new Error('The cached script was not correctly executed'));
    }
    resolve();
  });
}

function importNotInCacheSciptTest() {
  return new Promise((resolve, reject) => {
    try {
      importScripts('appcache-worker-import.py?type=not-in-cache');
      reject(new Error('Importing a non-cached script must fail.'));
    } catch(e) {
      resolve();
    }
  });
}

function importFallbackSciptTest() {
  return new Promise((resolve, reject) => {
    info = '';
    try {
      importScripts('appcache-worker-import.py?type=fallingback');
    } catch(e) {
      reject(new Error('Error while importing the fallingback script: ' +
                       e.toString()));
    }
    if (info != 'Set by the fallbacked script') {
      reject(new Error('The fallingback script was not correctly executed'));
    }
    resolve();
  });
}

function fetchCachedFileTest() {
  return fetch('appcache-worker-data.py?type=cached')
    .then(res => res.text(),
          _ => { throw new Error('Failed to fetch cached file'); })
    .then(text => {
      if (text != 'cached') {
        throw new Error('cached file missmatch');
      }
    });
}

function fetchNotInCacheFileTest() {
  return fetch('appcache-worker-data.py?type=not-in-cache')
    .then(_ => { throw new Error('Fetching not-in-cache file must fail'); },
          _ => {});
}

function fetchFallbackFileTest() {
  return fetch('appcache-worker-data.py?type=fallingback')
    .then(res => res.text(),
          _ => { throw new Error('Failed to fetch fallingback file'); })
    .then(text => {
      if (text != 'fallbacked') {
        throw new Error('fallbacked file miss match');
      }
    });
}

initPromise
  .then(importCachedScriptTest)
  .then(importNotInCacheSciptTest)
  .then(importFallbackSciptTest)
  .then(fetchCachedFileTest)
  .then(fetchNotInCacheFileTest)
  .then(_ => postMessage('Done: %s'),
        error => postMessage(error.toString()));
'''

def main(request, response):
    type = request.GET['type']
    if request.GET['type'] == 'fallingback':
        return 404, [('Content-Type', 'text/plain')], "Page not found"
    return [('Content-Type', 'text/javascript')], script % type
