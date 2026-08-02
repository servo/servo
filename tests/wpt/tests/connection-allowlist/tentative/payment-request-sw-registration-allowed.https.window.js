// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
//
// The test assumes the connection allowlist has been set:
// Connection-Allowlist: (response-origin)
//
// It allows same-origin network requests, so the JIT payment app installation
// (manifest, icon, service worker registration) is allowed. It also allows the
// test to communicate with the test runner (testharness.js and testdriver.js)
//
// 1. Trigger PaymentRequest JIT installation with same-origin payment manifest.
// 2. JIT succeeds, payment app registers a same-origin service worker.
// 3. Verify the service worker is registered.

const pay_url = window.location.origin +
    '/connection-allowlist/tentative/payment-app-manifest.json';

const defaultDetails = {
  total: {
    label: 'Total',
    amount: {
      currency: 'USD',
      value: '0.01',
    },
  },
};

promise_test(
    async t => {
      // Ensure any registered service workers are cleaned up after the test,
      // regardless of whether the test passes, fails, or throws.
      t.add_cleanup(async () => {
        const regs = await navigator.serviceWorker.getRegistrations();
        for (const reg of regs) {
          if (reg.scope.includes('connection-allowlist/tentative/')) {
            await reg.unregister();
          }
        }
      });

      const request =
          new PaymentRequest([{supportedMethods: pay_url}], defaultDetails);

      const response =
          await test_driver.bless('installing a payment app', () => {
            return request.show();
          });
      await response.complete('success');

      const regs = await navigator.serviceWorker.getRegistrations();
      const found = regs.some(
          reg => reg.scope.includes('connection-allowlist/tentative/'));
      assert_true(found, 'Service worker should be registered');
    },
    'Payment Request API payment app service worker registration is allowed ' +
        'by the connection allowlist.');
