// META: spec=https://w3c.github.io/payment-method-manifest/#validate-and-parse
// META: title=Payment Method Manifest default_applications validation and parsing
// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=/payment-method-manifest/resources/helpers.js

promise_test(async t => {
  const testId = token();
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

  // 3 requests expected: HEAD to PMI, GET to PMM, GET to WAM
  const logs = await waitForServerAccessLogs(t, testId, 3);

  assert_equals(
      logs.length, 3,
      'Browser must perform 3 server requests (HEAD PMI, GET PMM, GET WAM)');
  assert_equals(logs[0].endpoint, 'payment-method-identifier',
                'First request must hit PMI URL');
  assert_equals(logs[0].method, 'HEAD', 'PMI request must use HEAD method');

  assert_equals(logs[1].endpoint, 'payment-method-manifest',
                'Second request must hit PMM URL');
  assert_equals(logs[1].method, 'GET', 'PMM request must use GET method');

  assert_equals(logs[2].endpoint, 'web-app-manifest',
                'Third request must hit WAM URL');
  assert_equals(logs[2].method, 'GET', 'WAM request must use GET method');
}, 'Web app manifest fetching end-to-end success');

promise_test(async t => {
  const testId = token();
  const wamUrl1 = createWebAppManifestUrl(testId, {app: '1'});
  const wamUrl2 = createWebAppManifestUrl(testId, {app: '2'});
  const pmmUrl = createPaymentMethodManifestUrl(testId, {
    body: JSON.stringify({
      default_applications: [wamUrl1, wamUrl2],
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

  // 4 requests expected: HEAD to PMI, GET to PMM, GET to WAM1, GET to WAM2
  const logs = await waitForServerAccessLogs(t, testId, 4);

  assert_equals(
      logs.length, 4,
      'Browser must perform 4 server requests (HEAD PMI, GET PMM, GET WAM1, GET WAM2)');
  assert_equals(logs[0].endpoint, 'payment-method-identifier',
                'First request must hit PMI URL');
  assert_equals(logs[0].method, 'HEAD', 'PMI request must use HEAD method');

  assert_equals(logs[1].endpoint, 'payment-method-manifest',
                'Second request must hit PMM URL');
  assert_equals(logs[1].method, 'GET', 'PMM request must use GET method');

  const wamLogs = logs.filter(l => l.endpoint === 'web-app-manifest');
  assert_equals(wamLogs.length, 2, 'Must fetch both web app manifests');
  wamLogs.forEach((log, index) => {
    assert_equals(log.method, 'GET',
                  `WAM request ${index + 1} must use GET method`);
  });
  assert_array_equals(wamLogs.map(l => l.url).sort(), [wamUrl1, wamUrl2].sort(),
                      'Both distinct WAM URLs must be fetched');
}, 'Multiple default_applications web app manifests are fetched');

promise_test(async t => {
  const testId = token();
  // Setup a payment method manifest that lists three web app manifests, two of
  // whom are the same (wamUrl1 is duplicated).
  const wamUrl1 = createWebAppManifestUrl(testId, {app: '1'});
  const wamUrl2 = createWebAppManifestUrl(testId, {app: '2'});
  const pmmUrl = createPaymentMethodManifestUrl(testId, {
    body: JSON.stringify({
      default_applications: [wamUrl1, wamUrl1, wamUrl2],
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

  // Expecting exactly 4 requests: HEAD to PMI, GET to PMM, GET to wamUrl1
  // (deduplicated), and GET to wamUrl2.
  const logs = await waitForServerAccessLogs(t, testId, 4);

  assert_equals(
      logs.length, 4,
      'Browser must perform 4 requests (HEAD PMI, GET PMM, and 1 GET per unique WAM URL)');
  assert_equals(logs[0].endpoint, 'payment-method-identifier',
                'First request must hit PMI URL');
  assert_equals(logs[0].method, 'HEAD', 'PMI request must use HEAD method');

  assert_equals(logs[1].endpoint, 'payment-method-manifest',
                'Second request must hit PMM URL');
  assert_equals(logs[1].method, 'GET', 'PMM request must use GET method');

  const wamLogs = logs.filter(l => l.endpoint === 'web-app-manifest');
  assert_equals(wamLogs.length, 2,
                'Duplicate WAM URL must be deduplicated and fetched only once');
  wamLogs.forEach(log => {
    assert_equals(log.method, 'GET', 'WAM request must use GET method');
  });
  assert_array_equals(wamLogs.map(l => l.url).sort(), [wamUrl1, wamUrl2].sort(),
                      'Each unique WAM URL must be fetched exactly once');
}, 'Duplicate URLs in default_applications are deduplicated');

promise_test(async t => {
  const testId = token();
  const relativeWamUrl = `web-app-manifest.py?id=${testId}`;
  const pmmUrl = createPaymentMethodManifestUrl(testId, {
    body: JSON.stringify({
      default_applications: [relativeWamUrl],
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

  // 3 requests expected: HEAD to PMI, GET to PMM, GET to resolved WAM URL
  const logs = await waitForServerAccessLogs(t, testId, 3);

  assert_equals(logs.length, 3, 'Browser must perform 3 server requests');
  assert_equals(logs[0].endpoint, 'payment-method-identifier',
                'First request must hit PMI URL');
  assert_equals(logs[1].endpoint, 'payment-method-manifest',
                'Second request must hit PMM URL');
  assert_equals(logs[2].endpoint, 'web-app-manifest',
                'Third request must hit resolved WAM URL');
  assert_equals(logs[2].method, 'GET', 'WAM request must use GET method');

  const expectedWamUrl = createWebAppManifestUrl(testId);
  assert_equals(logs[2].url, expectedWamUrl,
                'WAM URL must be resolved relative to PMM URL');
}, 'Relative default_applications URL resolves against payment method manifest URL');

promise_test(async t => {
  const testId = token();
  const wamUrl = createWebAppManifestUrl(testId);
  const pmmUrl = createPaymentMethodManifestUrl(testId, {
    body: JSON.stringify({
      default_applications: [wamUrl],
      supported_origins: [`https://${location.host}`],
      created_by: 'Alice',
      created_in: 'Wonderland',
      extra_list: [1, 2, 3],
      extra_object: {nested: true},
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

  assert_equals(
      logs.length, 3,
      'Browser must perform 3 server requests (HEAD PMI, GET PMM, GET WAM)');
  assert_equals(logs[0].endpoint, 'payment-method-identifier',
                'First request must hit PMI URL');
  assert_equals(logs[0].method, 'HEAD', 'PMI request must use HEAD method');
  assert_equals(logs[1].endpoint, 'payment-method-manifest',
                'Second request must hit PMM URL');
  assert_equals(logs[1].method, 'GET', 'PMM request must use GET method');
  assert_equals(logs[2].endpoint, 'web-app-manifest',
                'Third request must hit WAM URL');
  assert_equals(logs[2].method, 'GET', 'WAM request must use GET method');
}, 'Unrecognized members in a payment method manifest are ignored');

promise_test(async t => {
  const testId = token();
  const wamUrl = createWebAppManifestUrl(testId);
  // Note that default_applications here is just a string, not a URL as
  // is required by the spec.
  const manifestUrl = createPaymentMethodManifestUrl(testId, {
    body: JSON.stringify({
      default_applications: wamUrl,
      supported_origins: [`https://${location.host}`],
    }),
  });
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

  assert_equals(
      logs.length, 2,
      'Browser must issue only 2 server requests (HEAD PMI, GET PMM) and ignore non-array default_applications');
  assert_equals(logs[0].endpoint, 'payment-method-identifier',
                'First request must hit PMI URL');
  assert_equals(logs[0].method, 'HEAD', 'PMI request must use HEAD method');
  assert_equals(logs[1].endpoint, 'payment-method-manifest',
                'Second request must hit manifest URL');
  assert_equals(logs[1].method, 'GET', 'Manifest request must use GET method');
  const wamLogs = logs.filter(l => l.endpoint === 'web-app-manifest');
  assert_equals(
      wamLogs.length, 0,
      'Non-array default_applications must not trigger any WAM fetch');
}, 'Payment method manifest with non-array default_applications fails parsing cleanly and does not fetch web app manifest');

promise_test(async t => {
  const testId = token();
  const wamUrl = createWebAppManifestUrl(testId);
  const pmmUrl = createPaymentMethodManifestUrl(testId, {
    body: JSON.stringify({
      default_applications: [wamUrl, 123],
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
      'Browser must issue only 2 server requests (HEAD PMI, GET PMM) when default_applications contains a non-string');
  assert_equals(logs[0].endpoint, 'payment-method-identifier',
                'First request must hit PMI URL');
  assert_equals(logs[0].method, 'HEAD', 'PMI request must use HEAD method');
  assert_equals(logs[1].endpoint, 'payment-method-manifest',
                'Second request must hit PMM URL');
  assert_equals(logs[1].method, 'GET', 'PMM request must use GET method');
  const wamLogs = logs.filter(l => l.endpoint === 'web-app-manifest');
  assert_equals(
      wamLogs.length, 0,
      'Non-string item in default_applications must fail validation before fetching any WAM');
}, 'Non-string item in default_applications fails parsing cleanly and does not fetch web app manifest');

promise_test(async t => {
  const testId = token();
  const wamUrl = createWebAppManifestUrl(testId);
  const pmmUrl = createPaymentMethodManifestUrl(testId, {
    body: JSON.stringify({
      default_applications: [wamUrl, 'https://'],
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
      'Browser must issue only 2 server requests (HEAD PMI, GET PMM) when default_applications contains an unparseable URL');
  assert_equals(logs[0].endpoint, 'payment-method-identifier',
                'First request must hit PMI URL');
  assert_equals(logs[0].method, 'HEAD', 'PMI request must use HEAD method');
  assert_equals(logs[1].endpoint, 'payment-method-manifest',
                'Second request must hit PMM URL');
  assert_equals(logs[1].method, 'GET', 'PMM request must use GET method');
  const wamLogs = logs.filter(l => l.endpoint === 'web-app-manifest');
  assert_equals(
      wamLogs.length, 0,
      'Unparseable URL in default_applications must fail validation before fetching any WAM');
}, 'Unparseable URL in default_applications fails parsing cleanly and does not fetch web app manifest');

promise_test(async t => {
  const testId = token();
  const insecureWamUrl = createWebAppManifestUrl(testId, {
    origin: get_host_info().HTTP_ORIGIN,
  });
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
  const insecureWamUrl = createWebAppManifestUrl(testId, {
    origin: get_host_info().HTTP_ORIGIN,
    app: 'insecure',
  });
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
