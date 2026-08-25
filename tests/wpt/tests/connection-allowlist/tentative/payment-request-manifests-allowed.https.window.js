// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/common/get-host-info.sub.js
//
// The test assumes the connection allowlist has been set:
// Connection-Allowlist: (
//   response-origin
//   "*://*:*/connection-allowlist/tentative/resources/pay"
//   "*://*:*/connection-allowlist/tentative/resources/payment-method-manifest.json"
//   "*://*:*/connection-allowlist/tentative/resources/web-app-manifest.json"
//   "*://*:*/web-based-payment-handler/app-simple.js"
//   "*://*:*/images/rgrg-256x256.png"
// )
//
// Note: `response-origin` is required for the test to communicate with the test
// runner (testharness.js and testdriver.js).

const cross_origin = get_host_info().HTTPS_REMOTE_ORIGIN;

// 1. Payment Request API initiates a HEAD request to the payment URL "/pay",
//    which is allowed by the connection allowlist.
// 2. The Link header response initiates a download of the manifest from:
//    "/connection-allowlist/tentative/resources/payment-method-manifest.json",
//    which is allowed by the connection allowlist.
// 3. The manifest requires "web-app-manifest.json" and "rgrg-256x256.png",
//    both are allowed by the connection allowlist.
const pay_url = cross_origin + '/connection-allowlist/tentative/resources/pay';

const defaultDetails = {
  total: {
    label: 'Total',
    amount: {
      currency: 'USD',
      value: '0.01',
    },
  },
};

promise_test(async t => {
  const request =
      new PaymentRequest([{supportedMethods: pay_url}], defaultDetails);
  const result = await request.canMakePayment();
  assert_true(result,
              'All Payment Request API manifest requests are allowed by ' +
                  'connection allowlist.');
}, 'Payment Request API manifest download is allowed.');
