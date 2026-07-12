// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/common/get-host-info.sub.js
//
// The test assumes the connection allowlist has been set:
// Connection-Allowlist: (
//   response-origin
//   "*://*:*/common/redirect.py*"
//   "*://*:*/connection-allowlist/tentative/resources/pay"
//   "*://*:*/connection-allowlist/tentative/resources/payment-method-manifest.json"
//   "*://*:*/connection-allowlist/tentative/resources/web-app-manifest.json"
//   "*://*:*/web-based-payment-handler/app-simple.js"
//   "*://*:*/images/rgrg-256x256.png"
// ); redirects=block
//
// Note: `response-origin` is required for the test to communicate with the test
// runner (testharness.js and testdriver.js).

const cross_origin = get_host_info().HTTPS_REMOTE_ORIGIN;

// 1. Payment Request API initiates a HEAD request to the payment URL "/pay",
//    which is allowed by the connection allowlist.
// 2. The redirect URL redirects to the payment URL "/pay". The redirect is
//    blocked because the connection allowlist specifies `redirects=block`.
const pay_url = cross_origin + '/connection-allowlist/tentative/resources/pay';
const redirect_url = cross_origin +
    '/common/redirect.py?status=302&location=' + encodeURIComponent(pay_url);

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
      new PaymentRequest([{supportedMethods: redirect_url}], defaultDetails);
  const result = await request.canMakePayment();
  assert_false(result, 'Manifest redirect is blocked by connection allowlist.');
}, 'Payment Request API manifest download redirect is blocked.');
