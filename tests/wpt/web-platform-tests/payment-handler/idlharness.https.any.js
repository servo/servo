// META: global=window,worker
// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js
// META: script=/service-workers/service-worker/resources/test-helpers.sub.js
// META: timeout=long

'use strict';

// https://w3c.github.io/payment-handler/

idl_test(
  ['payment-handler'],
  ['service-workers', 'html', 'dom'],
  async (idl_array, t) => {
    const isWindow = self.GLOBAL.isWindow();
    const isServiceWorker = 'ServiceWorkerGlobalScope' in self;
    const hasRegistration = isServiceWorker || isWindow;

    if (hasRegistration) {
      idl_array.add_objects({
        ServiceWorkerRegistration: ['registration'],
        PaymentManager: ['paymentManager'],
        PaymentInstruments: ['instruments'],
      });
    }
    if (isServiceWorker) {
      idl_array.add_objects({
        ServiceWorkerGlobalScope: ['self'],
        CanMakePaymentEvent: ['new CanMakePaymentEvent("type")'],
        PaymentRequestEvent: ['new PaymentRequestEvent("type")'],
      })
    }

    if (isWindow) {
      const scope = '/service-workers/service-worker/resources/';
      const reg = await service_worker_unregister_and_register(
        t, '/service-workers/service-worker/resources/empty-worker.js', scope);
      self.registration = reg;
      await wait_for_state(t, reg.installing, "activated");
      add_completion_callback(() => reg.unregister());
    }
    if (hasRegistration) {
      self.paymentManager = self.registration.paymentManager;
      self.instruments = self.paymentManager.instruments;
    }
  }
);
