// META: global=window,worker
// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js
// META: script=/service-workers/service-worker/resources/test-helpers.sub.js

// https://w3c.github.io/push-api/

idl_test(
  ['push-api'],
  ['service-workers', 'html', 'dom'],
  async (idl_array, t) => {
    const isServiceWorker = 'ServiceWorkerGlobalScope' in self
      && self instanceof ServiceWorkerGlobalScope;
    if (isServiceWorker) {
      idl_array.add_objects({
        ServiceWorkerGlobalScope: ['self'],
        PushEvent: ['new PushEvent("type")'],
        PushSubscriptionChangeEvent: [
          'new PushSubscriptionChangeEvent("pushsubscriptionchange")'
        ],
      })
    }
    if (GLOBAL.isWindow() || isServiceWorker) {
      idl_array.add_objects({
        // self.registration set for window below, and registration is already
        // part of ServiceWorkerGlobalScope.
        ServiceWorkerRegistration: ['registration'],
        PushManager: ['registration.pushManager'],
      });
    }

    if (GLOBAL.isWindow()) {
      const scope = '/service-workers/service-worker/resources/';
      const worker = `${scope}empty-worker.js`;
      return service_worker_unregister_and_register(t, worker, scope)
        .then(registration => {
          self.registration = registration;
          t.add_cleanup(function () { registration.unregister(); });
        });
    }
  }
);
