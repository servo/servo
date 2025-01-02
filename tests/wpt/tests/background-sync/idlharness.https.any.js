// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js
// META: timeout=long

'use strict';

// https://wicg.github.io/background-sync/spec/

idl_test(
  ['background-sync'],
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
