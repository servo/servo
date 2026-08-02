// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
//
// The test assumes the connection allowlist has been set:
// Connection-Allowlist: (
//   "*://*:*/connection-allowlist/tentative/payment-app-manifest.json"
//   "*://*:*/images/rgrg-256x256.png"
//   "*://*:*/connection-allowlist/tentative/payment-request-sw-registration-blocked.https.window.js"
//   "*://*:*/resources/testharness.js"
//   "*://*:*/resources/testharnessreport.js"
//   "*://*:*/resources/testdriver.js"
//   "*://*:*/resources/testdriver-vendor.js"
// )
//
// Note: The allowlist explicitly allows required test runner scripts, the
// payment app manifest, and the icon, but does not allow the service worker
// script ("payment-app-sw.js").
//
// 1. Trigger PaymentRequest JIT installation with same-origin payment manifest.
// 2. The manifest and icon downloads are allowed, but the service worker script
//    is not allowed by the connection allowlist.
// 3. Verify the service worker registration fails.

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

      test_driver.bless('installing a payment app', () => {
        // request.show() initiates JIT payment app installation asynchronously.
        // We do not await this promise because if JIT installation fails, an
        // error dialog is displayed that keeps the promise pending indefinitely
        // until closed.
        request.show().catch(() => {});
      });

      // Wait for some time to allow Payment Request API to attempt fetching and
      // registering `payment-app-sw.js` which gets blocked by the connection
      // allowlist.
      await new Promise(resolve => t.step_timeout(resolve, 2000));

      const regs = await navigator.serviceWorker.getRegistrations();
      let found = false;
      for (const reg of regs) {
        if (reg.scope.includes('connection-allowlist/tentative/')) {
          found = true;
        }
      }
      assert_false(found, 'Service worker should not be registered');
    },
    'Payment Request API payment app service worker registration is blocked ' +
        'by the connection allowlist.');
