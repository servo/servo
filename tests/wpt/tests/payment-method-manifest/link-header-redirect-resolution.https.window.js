// META: spec=https://w3c.github.io/payment-method-manifest/#fetch-pmm
// META: title=Relative Link header target is resolved against final post-redirect response URL
// META: script=/common/utils.js
// META: script=/payment-method-manifest/resources/helpers.js

promise_test(async t => {
  const testId = token();
  const targetUrl = createPaymentMethodIdentifierUrl(testId, {
    link: `<payment-method-manifest.py?id=${
        testId}>; rel="payment-method-manifest"`,
  });
  const pmiUrl =
      createPaymentMethodIdentifierUrl(testId, {redirect_location: targetUrl});

  const request = new PaymentRequest(
      [{supportedMethods: pmiUrl}],
      {total: {label: 'Total', amount: {currency: 'USD', value: '1.00'}}});

  try {
    await request.canMakePayment();
  } catch (err) {
    // It is fine for this call to fail; server logs are still captured and
    // inspected below.
  }

  // 3 requests expected:
  // 1. HEAD to initial PMI URL (redirects 302 to targetUrl)
  // 2. HEAD to targetUrl (returns 200 + relative Link header)
  // 3. GET to manifest URL resolved relative to targetUrl, not initial URL
  const logs = await waitForServerAccessLogs(t, testId, 3);

  assert_equals(
      logs.length, 3,
      'Browser must follow redirect and fetch manifest via relative Link header');
  assert_equals(logs[0].endpoint, 'payment-method-identifier',
                'First request must hit initial PMI URL');
  assert_equals(logs[0].method, 'HEAD', 'First request must use HEAD method');

  assert_equals(logs[1].endpoint, 'payment-method-identifier',
                'Second request must hit post-redirect target PMI URL');
  assert_equals(logs[1].method, 'HEAD', 'Second request must use HEAD method');

  assert_equals(logs[2].endpoint, 'payment-method-manifest',
                'Third request must fetch manifest');
  assert_equals(logs[2].method, 'GET', 'Manifest request must use GET method');
  const expectedManifestUrl = createPaymentMethodManifestUrl(testId);
  assert_equals(
      logs[2].url, expectedManifestUrl,
      'Manifest URL must be resolved relative to final post-redirect PMI URL');
}, 'Relative Link header target is resolved against final post-redirect response URL');
