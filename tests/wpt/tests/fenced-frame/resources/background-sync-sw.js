import {getOneShotSyncPromise, getPeriodicSyncPromise} from './background-sync-helper.js';

self.addEventListener('install', e => e.waitUntil(skipWaiting()));
self.addEventListener('activate', e => e.waitUntil(clients.claim()));

self.addEventListener('message', async e => {
  const {method, isPeriodic} = e.data;
  const promise = isPeriodic ?
      getPeriodicSyncPromise(self.registration, method) :
      getOneShotSyncPromise(self.registration, method);
  const message =
      await promise
          .then(() => {
            return `[background synnc ${method}] Unexpectedly started`;
          })
          .catch((e) => {
            return e.message;
          });

  e.source.postMessage(message);
});
