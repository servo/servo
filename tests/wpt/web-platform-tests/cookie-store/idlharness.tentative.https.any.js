// META: global=window,serviceworker
// META: timeout=long
// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js
// META: script=/service-workers/service-worker/resources/test-helpers.sub.js
'use strict';

// https://wicg.github.io/cookie-store/

idl_test(
  ['cookie-store'],
  ['service-workers', 'html', 'dom'],
  async (idl_array, t) => {
    const isServiceWorker = 'ServiceWorkerGlobalScope' in self
      && self instanceof ServiceWorkerGlobalScope;

    if (isServiceWorker) {
      idl_array.add_objects({
        ExtendableCookieChangeEvent: [
            'new ExtendableCookieChangeEvent("cookiechange")'],
        ServiceWorkerGlobalScope: ['self'],
      });
    } else {
      const registration = await service_worker_unregister_and_register(
          t, 'resources/empty_sw.js', 'resources/does/not/exist');
      t.add_cleanup(() => registration.unregister());

      // Global property referenced by idl_array.add_objects().
      self.registration = registration;

      idl_array.add_objects({
        CookieChangeEvent: ['new CookieChangeEvent("change")'],
        Window: ['self'],
      });
    }

    idl_array.add_objects({
      CookieStore: ['self.cookieStore'],
      CookieStoreManager: ['self.registration.cookies'],
      ServiceWorkerRegistration: ['self.registration'],
    });
  }
);
