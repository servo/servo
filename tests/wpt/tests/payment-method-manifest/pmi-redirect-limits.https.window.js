// META: spec=https://w3c.github.io/payment-method-manifest/#fetch-pmm
// META: title=Payment method identifier maximum redirects
// META: script=/common/utils.js
// META: script=/payment-method-manifest/resources/helpers.js

promise_test(async t => {
  const testId = token();
  const pmiUrl = createPaymentMethodIdentifierUrl(testId, {num_redirects: 3});

  const request = new PaymentRequest(
      [{supportedMethods: pmiUrl}],
      {total: {label: 'Total', amount: {currency: 'USD', value: '1.00'}}});

  try {
    await request.canMakePayment();
  } catch (err) {
    // It is fine for this call to fail; server logs are still captured and
    // inspected below.
  }

  // 5 requests expected: 4 HEAD requests (num_redirects=3, 2, 1, 0) + 1
  // manifest GET request
  const logs = await waitForServerAccessLogs(t, testId, 5);

  assert_equals(logs.length, 5,
                'Browser must complete 3 redirects and fetch manifest');

  const pmiLogs = logs.filter(l => l.endpoint === 'payment-method-identifier');
  assert_equals(pmiLogs.length, 4,
                'Must perform 4 HEAD requests during 3 redirects');
  pmiLogs.forEach((log, index) => {
    assert_equals(log.method, 'HEAD',
                  `PMI request ${index + 1} must use HEAD method`);
  });

  const manifestLogs =
      logs.filter(l => l.endpoint === 'payment-method-manifest');
  assert_equals(manifestLogs.length, 1,
                'Must fetch manifest after 3 redirects');
  assert_equals(manifestLogs[0].method, 'GET',
                'Manifest request must use GET method');
}, 'Up to 3 redirects for the Link header HEAD request are allowed');

promise_test(async t => {
  const testId = token();
  const pmiUrl = createPaymentMethodIdentifierUrl(testId, {num_redirects: 4});

  const request = new PaymentRequest(
      [{supportedMethods: pmiUrl}],
      {total: {label: 'Total', amount: {currency: 'USD', value: '1.00'}}});

  try {
    await request.canMakePayment();
  } catch (err) {
    // It is fine for this call to fail; server logs are still captured and
    // inspected below.
  }

  // 4 requests expected: 4 HEAD requests then an error on the 5th.
  const logs = await waitForServerAccessLogs(t, testId, 4);

  assert_equals(logs.length, 4,
                'Browser must perform 4 HEAD requests before aborting');
  logs.forEach((log, index) => {
    assert_equals(log.endpoint, 'payment-method-identifier',
                  `Request ${index + 1} must hit PMI URL`);
    assert_equals(log.method, 'HEAD',
                  `Request ${index + 1} must use HEAD method`);
  });

  const manifestLogs =
      logs.filter(l => l.endpoint === 'payment-method-manifest');
  assert_equals(
      manifestLogs.length, 0,
      'Exceeding 3 redirects (URL list size > 4) must abort fetch; manifest GET must not occur');
}, 'More than 3 redirects for the Link header HEAD request is not allowed');
