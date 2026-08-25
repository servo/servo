// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/common/get-host-info.sub.js
//
// The test assumes the connection allowlist has been set:
// Connection-Allowlist: (
//   response-origin
//   "*://*:*/connection-allowlist/tentative/resources/payment-method-manifest.json"
//   "*://*:*/connection-allowlist/tentative/resources/web-app-manifest.json"
//   "*://*:*/web-based-payment-handler/app-simple.js"
//   "*://*:*/images/rgrg-256x256.png"
// )
//
// Note: `response-origin` is required for the test to communicate with the test
// runner (testharness.js and testdriver.js).

const cross_origin = get_host_info().HTTPS_REMOTE_ORIGIN;

// Payment Request API initiates a HEAD request to the payment URL "/pay", which
// is blocked by the connection allowlist. Thus the manifest download fails,
// even though the payment manifest and the app manifest are allowed.
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
  assert_false(result,
               'Payment Request API Initial HEAD request is blocked by ' +
                   'connection allowlist.');
}, 'Payment Request API initial HEAD is blocked.');
