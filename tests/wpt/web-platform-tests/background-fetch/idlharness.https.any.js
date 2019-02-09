// META: global=window,worker
// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

'use strict';

// https://wicg.github.io/background-fetch/

idl_test(
  ['background-fetch'],
  ['service-workers', 'html', 'dom'],
  idl_array => {
    const isServiceWorker = location.pathname.includes('.serviceworker.');
    if (isServiceWorker) {
      idl_array.add_objects({
        ServiceWorkerGlobalScope: ['self'],
        ServiceWorkerRegistration: ['registration'],
        BackgroundFetchManager: ['registration.backgroundFetch'],
        BackgroundFetchEvent: ['new BackgroundFetchEvent("type")'],
        BackgroundFetchUpdateEvent: ['new BackgroundFetchUpdateEvent("type")'],
      });
    }
  }
);
