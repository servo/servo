// META: spec=https://w3c.github.io/payment-method-manifest/#fetch-wam
// META: title=Web App Manifest GET request aborts on 302 redirect
// META: script=/common/utils.js
// META: script=/payment-method-manifest/resources/helpers.js

promise_test(async t => {
  const testId = token();
  const targetWamUrl = createWebAppManifestUrl(testId, {app: 'target'});
  const redirectingWamUrl = createWebAppManifestUrl(testId, {
    redirect_location: targetWamUrl,
  });
  const pmmUrl = createPaymentMethodManifestUrl(testId, {
    body: JSON.stringify({
      default_applications: [redirectingWamUrl],
      supported_origins: [`https://${location.host}`],
    }),
  });
  const pmiUrl = createPaymentMethodIdentifierUrl(testId, {
    link: `<${pmmUrl}>; rel="payment-method-manifest"`,
  });

  const request = new PaymentRequest(
      [{supportedMethods: pmiUrl}],
      {total: {label: 'Total', amount: {currency: 'USD', value: '1.00'}}});

  try {
    await request.canMakePayment();
  } catch (err) {
    // It is fine for this call to fail; server logs are still captured and
    // inspected below.
  }

  // 3 requests expected: HEAD to PMI, GET to PMM, and initial GET to WAM
  // (which returns a 302 redirect that must not be followed).
  const logs = await waitForServerAccessLogs(t, testId, 3);

  assert_equals(
      logs.length, 3,
      'Browser must issue HEAD to PMI, GET to PMM, and initial GET to WAM without following redirect');
  assert_equals(logs[0].endpoint, 'payment-method-identifier',
                'First request must hit PMI URL');
  assert_equals(logs[0].method, 'HEAD', 'PMI request must use HEAD method');

  assert_equals(logs[1].endpoint, 'payment-method-manifest',
                'Second request must hit PMM URL');
  assert_equals(logs[1].method, 'GET', 'PMM request must use GET method');

  assert_equals(logs[2].endpoint, 'web-app-manifest',
                'Third request must hit initial WAM URL');
  assert_equals(logs[2].method, 'GET', 'WAM request must use GET method');
  assert_equals(logs[2].url, redirectingWamUrl,
                'Only the initial redirecting WAM URL must be requested');
}, 'Web app manifest GET request with redirect aborts');
