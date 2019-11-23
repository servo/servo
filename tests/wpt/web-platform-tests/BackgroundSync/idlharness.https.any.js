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
      idlArray.add_objects({
        ServiceWorkerGlobalScope: ['self', 'onsync'],
        ServiceWorkerRegistration: ['registration'],
        SyncManager: ['registration.sync'],
        SyncEvent: ['new SyncEvent("tag", "lastChance")'],
      });
  }
}
);
