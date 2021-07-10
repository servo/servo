var info = '';

var initPromise = new Promise(resolve => {
  self.addEventListener('connect', event => {
    self.postMessage = msg => { event.ports[0].postMessage(msg); };
    resolve();
  });
});

function importNotInCacheSciptTest() {
  return new Promise((resolve, reject) => {
    try {
      importScripts('appcache-worker-import.py?type=not-in-cache');
    } catch(e) {
      reject(new Error('Error while importing the not-in-cache script: ' +
                       e.toString()));
      return;
    }
    if (info != 'Set by the not-in-cache script') {
      reject(new Error('The not-in-cache script was not correctly executed'));
    }
    resolve();
  });
}

function importFallbackSciptTest() {
  return new Promise((resolve, reject) => {
    try {
      importScripts('appcache-worker-import.py?type=fallingback');
      reject(new Error('Importing a fallback script must fail.'));
    } catch(e) {
      resolve();
    }
  });
}

function fetchNotInCacheFileTest() {
  return fetch('appcache-worker-data.py?type=not-in-cache')
    .then(res => res.text(),
          _ => { throw new Error('Failed to fetch not-in-cache file'); })
    .then(text => {
      if (text != 'not-in-cache') {
        throw new Error('not-in-cache file mismatch');
      }
    })
}

function fetchFallbackFileTest() {
  return fetch('appcache-worker-data.py?type=fallingback')
    .then(res => {
            if (res.status != 404) {
              throw new Error(
                  'Fetching fallback file must resolve with 404 response');
            }
          },
          _ => { throw new Error('Fetching fallback file must not fail'); });
}

initPromise
  .then(importNotInCacheSciptTest)
  .then(importFallbackSciptTest)
  .then(fetchNotInCacheFileTest)
  .then(fetchFallbackFileTest)
  .then(_ => postMessage('Done'), error => postMessage(error.toString()));
