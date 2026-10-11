// META: spec=https://w3c.github.io/payment-method-manifest/#fetch-pmm
// META: spec=https://w3c.github.io/payment-method-manifest/#fetch-wam
// META: title=Fetch options and headers for PMI, PMM, and WAM requests
// META: script=/common/utils.js
// META: script=/payment-method-manifest/resources/helpers.js

promise_test(async t => {
  const testId = token();
  const cookieName = `wpt_pmi_cookie_${testId.replace(/-/g, '_')}`;
  document.cookie =
      `${cookieName}=secret_pmi_value; path=/; secure; samesite=none`;
  t.add_cleanup(() => {
    document.cookie =
        `${cookieName}=; path=/; max-age=0; secure; samesite=none`;
  });

  const pmiUrl = createPaymentMethodIdentifierUrl(testId);

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

  assert_equals(logs.length, 2,
                'Browser must issue HEAD to PMI and GET to manifest');
  const pmiLog = logs[0];
  assert_equals(pmiLog.endpoint, 'payment-method-identifier',
                'First request must hit PMI URL');
  assert_equals(pmiLog.method, 'HEAD', 'PMI request must use HEAD method');

  // Verify credentials omission (credentials: omit)
  const cookieHeader = pmiLog.headers['cookie'];
  assert_true(
      !cookieHeader || !cookieHeader.includes(cookieName),
      'PMI HEAD request must omit credentials/cookies (credentials: "omit")');
  assert_equals(pmiLog.headers['authorization'], undefined,
                'PMI HEAD request must not send Authorization header');
}, 'Payment method identifier HEAD request omits credentials (cookies and authorization)');

promise_test(async t => {
  const testId = token();
  const cookieName = `wpt_pmm_cookie_${testId.replace(/-/g, '_')}`;
  document.cookie =
      `${cookieName}=secret_pmm_value; path=/; secure; samesite=none`;
  t.add_cleanup(() => {
    document.cookie =
        `${cookieName}=; path=/; max-age=0; secure; samesite=none`;
  });

  const manifestUrl = createPaymentMethodManifestUrl(testId);
  const pmiUrl = createPaymentMethodIdentifierUrl(testId, {
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

  assert_equals(logs.length, 2,
                'Browser must issue HEAD to PMI and GET to manifest');
  assert_equals(logs[0].endpoint, 'payment-method-identifier',
                'First request must hit PMI URL');
  const manifestLog = logs[1];
  assert_equals(manifestLog.endpoint, 'payment-method-manifest',
                'Second request must hit manifest URL');
  assert_equals(manifestLog.method, 'GET',
                'Manifest request must use GET method');

  // Verify credentials omission (credentials: omit)
  const cookieHeader = manifestLog.headers['cookie'];
  assert_true(
      !cookieHeader || !cookieHeader.includes(cookieName),
      'Manifest GET request must omit credentials/cookies (credentials: "omit")');
  assert_equals(manifestLog.headers['authorization'], undefined,
                'Manifest GET request must not send Authorization header');
}, 'Payment method manifest GET request omits credentials (cookies and authorization)');

promise_test(async t => {
  const testId = token();
  const manifestUrl = createPaymentMethodManifestUrl(testId);
  const pmiUrl = createPaymentMethodIdentifierUrl(testId, {
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

  assert_equals(logs.length, 2,
                'Browser must issue HEAD to PMI and GET to manifest');
  assert_equals(logs[0].endpoint, 'payment-method-identifier',
                'First request must hit PMI URL');
  assert_equals(logs[0].method, 'HEAD', 'PMI request must use HEAD method');
  assert_equals(logs[1].endpoint, 'payment-method-manifest',
                'Second request must hit manifest URL');
  assert_equals(logs[1].method, 'GET', 'Manifest request must use GET method');

  // Per PMM § 3.3 step 8: referrer is paymentMethod
  assert_true(logs[1].headers['referer'] !== undefined,
              'Manifest GET request must include Referer header');
  assert_true(logs[1].headers['referer'] === pmiUrl,
              'Manifest GET request Referer header must match PMI URL');
}, 'Payment method manifest GET request sets Referer header to match PMI URL');

promise_test(async t => {
  const testId = token();
  const cookieName = `wpt_wam_cookie_${testId.replace(/-/g, '_')}`;
  document.cookie =
      `${cookieName}=secret_wam_value; path=/; secure; samesite=none`;
  t.add_cleanup(() => {
    document.cookie =
        `${cookieName}=; path=/; max-age=0; secure; samesite=none`;
  });

  const wamUrl = createWebAppManifestUrl(testId);
  const pmmUrl = createPaymentMethodManifestUrl(testId, {
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

  const logs = await waitForServerAccessLogs(t, testId, 3);

  assert_equals(logs.length, 3,
                'Browser must issue HEAD to PMI, GET to PMM, and GET to WAM');
  assert_equals(logs[0].endpoint, 'payment-method-identifier',
                'First request must hit PMI URL');
  assert_equals(logs[1].endpoint, 'payment-method-manifest',
                'Second request must hit PMM URL');
  const wamLog = logs[2];
  assert_equals(wamLog.endpoint, 'web-app-manifest',
                'Third request must hit WAM URL');
  assert_equals(wamLog.method, 'GET', 'WAM request must use GET method');

  // Verify credentials omission (credentials: omit)
  const cookieHeader = wamLog.headers['cookie'];
  assert_true(
      !cookieHeader || !cookieHeader.includes(cookieName),
      'WAM GET request must omit credentials/cookies (credentials: "omit")');
  assert_equals(wamLog.headers['authorization'], undefined,
                'WAM GET request must not send Authorization header');
}, 'Web app manifest GET request omits credentials (cookies and authorization)');
