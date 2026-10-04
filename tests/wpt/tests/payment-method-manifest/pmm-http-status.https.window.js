// META: spec=https://w3c.github.io/payment-method-manifest/#fetch-pmm
// META: title=HTTP response status handling for Payment Method Manifest requests
// META: script=/common/utils.js
// META: script=/payment-method-manifest/resources/helpers.js

promise_test(async t => {
  const testId = token();
  const wamUrl = createWebAppManifestUrl(testId);
  const pmmUrl = createPaymentMethodManifestUrl(testId, {
    status: 404,
    body: JSON.stringify({
      default_applications: [wamUrl],
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

  const logs = await waitForServerAccessLogs(t, testId, 2);

  assert_equals(
      logs.length, 2,
      'Browser must issue only HEAD to PMI and GET to PMM on HTTP 404');
  assert_equals(logs[0].endpoint, 'payment-method-identifier',
                'First request must hit PMI URL');
  assert_equals(logs[0].method, 'HEAD', 'PMI request must use HEAD method');
  assert_equals(logs[1].endpoint, 'payment-method-manifest',
                'Second request must hit PMM URL');
  assert_equals(logs[1].method, 'GET', 'PMM request must use GET method');

  const wamLogs = logs.filter(l => l.endpoint === 'web-app-manifest');
  assert_equals(
      wamLogs.length, 0,
      'HTTP 404 response on PMM must abort ingestion; WAM GET must not occur');
}, 'HTTP 404 response on Payment Method Manifest GET request aborts ingestion and does not fetch web app manifest');

promise_test(async t => {
  const testId = token();
  const wamUrl = createWebAppManifestUrl(testId);
  const pmmUrl = createPaymentMethodManifestUrl(testId, {
    status: 500,
    body: JSON.stringify({
      default_applications: [wamUrl],
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

  const logs = await waitForServerAccessLogs(t, testId, 2);

  assert_equals(
      logs.length, 2,
      'Browser must issue only HEAD to PMI and GET to PMM on HTTP 500');
  assert_equals(logs[0].endpoint, 'payment-method-identifier',
                'First request must hit PMI URL');
  assert_equals(logs[0].method, 'HEAD', 'PMI request must use HEAD method');
  assert_equals(logs[1].endpoint, 'payment-method-manifest',
                'Second request must hit PMM URL');
  assert_equals(logs[1].method, 'GET', 'PMM request must use GET method');

  const wamLogs = logs.filter(l => l.endpoint === 'web-app-manifest');
  assert_equals(
      wamLogs.length, 0,
      'HTTP 500 response on PMM must abort ingestion; WAM GET must not occur');
}, 'HTTP 500 response on Payment Method Manifest GET request aborts ingestion and does not fetch web app manifest');
