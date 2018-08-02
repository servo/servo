// META: global=window,worker
// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

// https://w3c.github.io/push-api/

idl_test(
  ['push-api'],
  ['service-workers', 'html', 'dom'],
  idl_array => {
    // TODO: ServiceWorkerRegistration objects
    if ('ServiceWorkerGlobalScope' in self
        && self instanceof ServiceWorkerGlobalScope) {
      idl_array.add_objects({
        PushSubscriptionChangeEvent: [
          'new PushSubscriptionChangeEvent("pushsubscriptionchange")'
        ],
      })
    }
  },
  'push-api interfaces'
);
