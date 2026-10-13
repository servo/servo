// META: spec=https://w3c.github.io/payment-method-manifest/#fetch-pmm
// META: title=Payment method identifier redirect origin restrictions
// META: script=/common/utils.js
// META: script=/payment-method-manifest/resources/helpers.js

promise_test(async t => {
  const testId = token();
  const targetUrl = createPaymentMethodIdentifierUrl(
      testId, {host: '{{hosts[][www]}}:{{ports[https][0]}}'});
  const pmiUrl = createPaymentMethodIdentifierUrl(testId, {
    host: '{{hosts[][]}}:{{ports[https][0]}}',
    redirect_location: targetUrl,
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

  // 3 requests expected:
  // 1. HEAD to initial PMI endpoint (redirects 302 to targetUrl)
  // 2. HEAD to same-site subdomain PMI endpoint (returns 200 + Link header)
  // 3. GET to manifest endpoint (returns 200)
  const logs = await waitForServerAccessLogs(t, testId, 3);

  assert_equals(logs.length, 3,
                'Browser must follow same-site redirect and fetch manifest');
  assert_equals(logs[0].endpoint, 'payment-method-identifier',
                'First request must hit initial PMI URL');
  assert_equals(logs[0].method, 'HEAD', 'First request must use HEAD method');

  assert_equals(
      logs[1].endpoint, 'payment-method-identifier',
      'Second request must hit redirected PMI URL on same-site subdomain');
  assert_equals(logs[1].method, 'HEAD', 'Second request must use HEAD method');

  assert_equals(logs[2].endpoint, 'payment-method-manifest',
                'Third request must fetch manifest');
  assert_equals(logs[2].method, 'GET', 'Manifest request must use GET method');
}, 'Same-site cross-origin redirects on initial HEAD request are permitted');

promise_test(async t => {
  const testId = token();
  const targetUrl = createPaymentMethodIdentifierUrl(
      testId, {host: '{{hosts[alt][]}}:{{ports[https][0]}}'});
  const pmiUrl = createPaymentMethodIdentifierUrl(testId, {
    host: '{{hosts[][]}}:{{ports[https][0]}}',
    redirect_location: targetUrl,
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

  // Wait for initial HEAD request
  const logs = await waitForServerAccessLogs(t, testId, 1);

  assert_equals(logs.length, 1,
                'Browser must issue only 1 server request before aborting');
  assert_equals(logs[0].endpoint, 'payment-method-identifier',
                'Sole request must hit PMI URL');
  assert_equals(logs[0].method, 'HEAD', 'Sole request must use HEAD method');
}, 'Cross-site redirects on initial HEAD request are prohibited and abort fetch');
