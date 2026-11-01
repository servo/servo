// META: spec=https://w3c.github.io/payment-method-manifest/#fetch-pmm
// META: title=Link header target URL resolution and validation
// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=/payment-method-manifest/resources/helpers.js

promise_test(async t => {
  const testId = token();
  const httpManifestUrl = createPaymentMethodManifestUrl(testId, {
    origin: get_host_info().HTTP_ORIGIN,
  });
  const pmiUrl = createPaymentMethodIdentifierUrl(
      testId, {link: `<${httpManifestUrl}>; rel="payment-method-manifest"`});

  const request = new PaymentRequest(
      [{supportedMethods: pmiUrl}],
      {total: {label: 'Total', amount: {currency: 'USD', value: '1.00'}}});

  try {
    await request.canMakePayment();
  } catch (err) {
  }

  const logs = await waitForServerAccessLogs(t, testId, 1);

  assert_equals(logs.length, 1,
                'Browser must issue only 1 server request (HEAD to PMI)');
  assert_equals(logs[0].endpoint, 'payment-method-identifier',
                'Request must hit PMI URL');
  assert_equals(logs[0].method, 'HEAD', 'PMI request must use HEAD method');
}, 'Link header targeting non-HTTPS URL is aborted');

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
