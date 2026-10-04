// META: spec=https://w3c.github.io/payment-method-manifest/#validate-and-parse
// META: title=Non-HTTPS Web App Manifest URL in default_applications is not fetched
// META: script=/common/utils.js
// META: script=/payment-method-manifest/resources/helpers.js

promise_test(async t => {
  const testId = token();
  const insecureWamUrl =
      `http://{{host}}:{{ports[http][0]}}/payment-method-manifest/resources/web-app-manifest.py?id=${
          testId}`;
  const pmmUrl = createPaymentMethodManifestUrl(testId, {
    body: JSON.stringify({
      default_applications: [insecureWamUrl],
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

  // 2 requests expected: HEAD to PMI and GET to PMM. WAM GET must NOT be
  // issued.
  const logs = await waitForServerAccessLogs(t, testId, 2);

  assert_equals(
      logs.length, 2,
      'Browser must perform only 2 server requests (HEAD PMI and GET PMM)');
  assert_equals(logs[0].endpoint, 'payment-method-identifier',
                'First request must hit PMI URL');
  assert_equals(logs[1].endpoint, 'payment-method-manifest',
                'Second request must hit PMM URL');

  const wamLogs = logs.filter(l => l.endpoint === 'web-app-manifest');
  assert_equals(wamLogs.length, 0,
                'Insecure non-HTTPS WAM URL must not be fetched');
}, 'Non-HTTPS web app manifest URL in default_applications is not fetched');

promise_test(async t => {
  const testId = token();
  const validHttpsWamUrl = createWebAppManifestUrl(testId, {app: 'valid'});
  const insecureWamUrl =
      `http://{{host}}:{{ports[http][0]}}/payment-method-manifest/resources/web-app-manifest.py?id=${
          testId}&app=insecure`;
  const pmmUrl = createPaymentMethodManifestUrl(testId, {
    body: JSON.stringify({
      default_applications: [validHttpsWamUrl, insecureWamUrl],
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

  // Per section 3.4 step 5.4.3, a non-HTTPS URL in default_applications causes
  // the entire manifest validation to return failure before any WAM is fetched.
  const logs = await waitForServerAccessLogs(t, testId, 2);

  assert_equals(
      logs.length, 2,
      'Browser must perform only 2 server requests (HEAD PMI and GET PMM)');
  assert_equals(logs[0].endpoint, 'payment-method-identifier',
                'First request must hit PMI URL');
  assert_equals(logs[1].endpoint, 'payment-method-manifest',
                'Second request must hit PMM URL');

  const wamLogs = logs.filter(l => l.endpoint === 'web-app-manifest');
  assert_equals(
      wamLogs.length, 0,
      'Validation failure must prevent any WAM in default_applications from being fetched');
}, 'Non-HTTPS URL in default_applications fails entire manifest validation without fetching valid HTTPS web app manifest');
