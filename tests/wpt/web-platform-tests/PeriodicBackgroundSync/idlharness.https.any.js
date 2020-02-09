// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

'use strict';

// https://github.com/WICG/BackgroundSync/blob/master/spec/PeriodicBackgroundSync-index.html

const idl = `
partial interface ServiceWorkerGlobalScope {
    attribute EventHandler onperiodicsync;
};
[
  Exposed=(Window,Worker)
] partial interface ServiceWorkerRegistration {
    readonly attribute PeriodicSyncManager periodicSync;
    readonly attribute SyncManager sync;
};
dictionary PeriodicSyncEventInit : ExtendableEventInit {
    required DOMString tag;
};
[
  Constructor(DOMString type, PeriodicSyncEventInit init),
  Exposed=ServiceWorker
] interface PeriodicSyncEvent : ExtendableEvent {
    readonly attribute DOMString tag;
};
`;

test(t => {
  const idlArray = new IdlArray();
  idlArray.add_idls(idl);
  idlArray.add_objects({
    ServiceWorkerGlobalScope: ['self', 'onperiodicsync'],
    ServiceWorkerRegistration: ['registration'],
    PeriodicSyncManager: ['registration.periodicSync'],
    PeriodicSyncEvent: ['new PeriodicSyncEvent("tag")'],
  });
}, 'IDL test for Periodic Background Sync');
