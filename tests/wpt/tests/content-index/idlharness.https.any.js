// META: global=window,worker
// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js
// META: script=/service-workers/service-worker/resources/test-helpers.sub.js

'use strict';

// https://wicg.github.io/content-index/spec/

idl_test(
  ['content-index'],
  ['service-workers', 'html', 'dom'],
  async (idl_array, t) => {
    // TODO: Handle other worker types.
    if (self.GLOBAL.isWindow()) {
      idl_array.add_objects({
        ServiceWorkerRegistration: ['registration'],
        ContentIndex: ['registration.index'],
      });
      self.registration = await service_worker_unregister_and_register(
          t, 'resources/sw.js', 'resources/does/not/exist');
      t.add_cleanup(() => registration.unregister());
    }
    else if (self.ServiceWorkerGlobalScope) {
      idl_array.add_objects({
        ServiceWorkerGlobalScope: ['self'],
        ServiceWorkerRegistration: ['registration'],
        ContentIndex: ['registration.index'],
        ContentIndexEvent: ['new ContentIndexEvent("type", {id: "foo"})'],
      });
    }
  }
);
