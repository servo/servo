// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

'use strict';

// https://wicg.github.io/BackgroundSync/spec/

idl_test(
  ['BackgroundSync'],
  ['service-workers', 'html', 'dom'],
  idlArray => {
    const isServiceWorker = location.pathname.includes('.serviceworker.');
    if (isServiceWorker) {
      idl_array.add_objects({
        ServiceWorkerGlobalScope: ['self', 'onsync', 'onperiodicsync'],
        ServiceWorkerRegistration: ['registration'],
        SyncManager: ['registration.sync'],
        PeriodicSyncManager: ['registration.periodicSync'],
        SyncEevnt: ['new SyncEvent("tag", "lastChance")'],
        PeriodicSyncEevnt: ['new PeriodicSyncEvent("tag")'],
      });
  }
}
);
