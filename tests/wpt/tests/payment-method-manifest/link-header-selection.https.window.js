// META: spec=https://w3c.github.io/payment-method-manifest/#fetch-pmm
// META: title=Link header manifest selection rules
// META: script=/common/utils.js
// META: script=/payment-method-manifest/resources/helpers.js

promise_test(async t => {
  const testId = token();
  const pmiUrl = createPaymentMethodIdentifierUrl(testId);

  const request = new PaymentRequest(
      [{supportedMethods: pmiUrl}],
      {total: {label: 'Total', amount: {currency: 'USD', value: '1.00'}}});

  try {
    await request.canMakePayment();
  } catch (err) {
  }

  const logs = await waitForServerAccessLogs(t, testId, 2);

  assert_equals(
      logs.length, 2,
      'Browser must issue exactly 2 server requests (HEAD for PMI, GET for PMM)');
  assert_equals(logs[0].endpoint, 'payment-method-identifier',
                'First request must hit PMI URL');
  assert_equals(logs[0].method, 'HEAD', 'PMI request must use HEAD method');
  assert_equals(logs[1].endpoint, 'payment-method-manifest',
                'Second request must hit PMM URL');
  assert_equals(logs[1].method, 'GET', 'PMM request must use GET method');
}, 'Link header with payment-method-manifest rel successfully initiates manifest fetch');

promise_test(async t => {
  const testId = token();
  const pmiUrl = createPaymentMethodIdentifierUrl(testId, {link: 'none'});

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
}, 'Payment Method Manifest fetch fails if Link header is missing');

promise_test(async t => {
  const testId = token();
  const manifestUrl = createPaymentMethodManifestUrl(testId);
  const pmiUrl = createPaymentMethodIdentifierUrl(
      testId, {link: `<${manifestUrl}>; rel="stylesheet"`});

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
}, 'Link header with incorrect rel attribute is ignored');

promise_test(async t => {
  const testId = token();
  const manifestUrl = createPaymentMethodManifestUrl(testId);
  const otherUrl = new URL('resources/other.json', location.href).href;
  const pmiUrl = createPaymentMethodIdentifierUrl(testId, {
    link: [
      `<${otherUrl}>; rel="other"`,
      `<${manifestUrl}>; rel="payment-method-manifest"`,
    ],
  });

  const request = new PaymentRequest(
      [{supportedMethods: pmiUrl}],
      {total: {label: 'Total', amount: {currency: 'USD', value: '1.00'}}});

  try {
    await request.canMakePayment();
  } catch (err) {
  }

  const logs = await waitForServerAccessLogs(t, testId, 2);

  assert_equals(
      logs.length, 2,
      'Browser must issue exactly 2 server requests (HEAD for PMI, GET for Manifest)');
  assert_equals(logs[0].endpoint, 'payment-method-identifier',
                'First request must hit PMI URL');
  assert_equals(logs[0].method, 'HEAD', 'PMI request must use HEAD method');
  assert_equals(logs[1].endpoint, 'payment-method-manifest',
                'Second request must hit correct manifest URL');
  assert_equals(logs[1].method, 'GET', 'Manifest request must use GET method');
}, 'Link header selecting payment-method-manifest link from multiple entries');

promise_test(async t => {
  const testId = token();
  const manifestUrl1 = createPaymentMethodManifestUrl(testId);
  const manifestUrl2 = createPaymentMethodManifestUrl(testId);
  const pmiUrl = createPaymentMethodIdentifierUrl(testId, {
    link: [
      `<${manifestUrl1}>; rel="payment-method-manifest"`,
      `<${manifestUrl2}>; rel="payment-method-manifest"`,
    ],
  });

  const request = new PaymentRequest(
      [{supportedMethods: pmiUrl}],
      {total: {label: 'Total', amount: {currency: 'USD', value: '1.00'}}});

  try {
    await request.canMakePayment();
  } catch (err) {
  }

  const logs = await waitForServerAccessLogs(t, testId, 1);

  assert_equals(
      logs.length, 1,
      'Browser must issue only 1 server request (HEAD to PMI); duplicate manifest links abort fetch');
  assert_equals(logs[0].endpoint, 'payment-method-identifier',
                'Request must hit PMI URL');
  assert_equals(logs[0].method, 'HEAD', 'PMI request must use HEAD method');
}, 'Multiple rel="payment-method-manifest" link headers cause fetch to abort');
