// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js
// META: timeout=long

'use strict';

// https://wicg.github.io/periodic-background-sync/

idl_test(
  ['periodic-background-sync'],
  ['service-workers', 'html', 'dom'],
  async idl_array => {
    idl_array.add_objects({
      ServiceWorkerGlobalScope: ['self', 'onperiodicsync'],
      ServiceWorkerRegistration: ['registration'],
      PeriodicSyncManager: ['registration.periodicSync'],
      PeriodicSyncEvent: ['new PeriodicSyncEvent("tag")'],
    });
  }
);
