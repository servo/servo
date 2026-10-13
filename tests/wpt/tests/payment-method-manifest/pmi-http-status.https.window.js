// META: spec=https://w3c.github.io/payment-method-manifest/#fetch-pmm
// META: title=HTTP response status handling for PMI requests
// META: script=/common/utils.js
// META: script=/payment-method-manifest/resources/helpers.js

promise_test(async t => {
  const testId = token();
  const manifestUrl = createPaymentMethodManifestUrl(testId);
  const pmiUrl = createPaymentMethodIdentifierUrl(testId, {
    status: 204,
    link: `<${manifestUrl}>; rel="payment-method-manifest"`,
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
      'Browser must issue HEAD to PMI and GET to manifest on HTTP 204');
  assert_equals(logs[0].endpoint, 'payment-method-identifier',
                'First request must hit PMI URL');
  assert_equals(logs[0].method, 'HEAD', 'PMI request must use HEAD method');
  assert_equals(logs[1].endpoint, 'payment-method-manifest',
                'Second request must hit manifest URL');
  assert_equals(logs[1].method, 'GET', 'Manifest request must use GET method');
}, 'HTTP 204 No Content response on initial PMI HEAD request succeeds');

promise_test(async t => {
  const testId = token();
  const pmiUrl = createPaymentMethodIdentifierUrl(testId, {status: 404});

  const request = new PaymentRequest(
      [{supportedMethods: pmiUrl}],
      {total: {label: 'Total', amount: {currency: 'USD', value: '1.00'}}});

  try {
    await request.canMakePayment();
  } catch (err) {
    // It is fine for this call to fail; server logs are still captured and
    // inspected below.
  }

  const logs = await waitForServerAccessLogs(t, testId, 1);

  const pmiLogs = logs.filter(l => l.endpoint === 'payment-method-identifier');
  assert_true(pmiLogs.length >= 1, 'Must issue at least one PMI HEAD request');
  pmiLogs.forEach(log => {
    assert_equals(log.method, 'HEAD', 'PMI request must use HEAD method');
  });

  const manifestLogs =
      logs.filter(l => l.endpoint === 'payment-method-manifest');
  assert_equals(
      manifestLogs.length, 0,
      'HTTP 404 response on PMI must abort manifest fetching; manifest GET must not occur');
}, 'HTTP 404 response on initial PMI HEAD request aborts manifest fetching');

promise_test(async t => {
  const testId = token();
  const pmiUrl = createPaymentMethodIdentifierUrl(testId, {status: 500});

  const request = new PaymentRequest(
      [{supportedMethods: pmiUrl}],
      {total: {label: 'Total', amount: {currency: 'USD', value: '1.00'}}});

  try {
    await request.canMakePayment();
  } catch (err) {
    // It is fine for this call to fail; server logs are still captured and
    // inspected below.
  }

  const logs = await waitForServerAccessLogs(t, testId, 1);

  const pmiLogs = logs.filter(l => l.endpoint === 'payment-method-identifier');
  assert_true(pmiLogs.length >= 1, 'Must issue at least one PMI HEAD request');
  pmiLogs.forEach(log => {
    assert_equals(log.method, 'HEAD', 'PMI request must use HEAD method');
  });

  const manifestLogs =
      logs.filter(l => l.endpoint === 'payment-method-manifest');
  assert_equals(
      manifestLogs.length, 0,
      'HTTP 500 response on PMI must abort manifest fetching; manifest GET must not occur');
}, 'HTTP 500 response on initial PMI HEAD request aborts manifest fetching');
