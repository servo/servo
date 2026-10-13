// META: spec=https://w3c.github.io/payment-method-manifest/#validate-and-parse
// META: title=Payment Method Manifest supported_origins validation
// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=/payment-method-manifest/resources/helpers.js

promise_test(async t => {
  const testId = token();
  const wamUrl = createWebAppManifestUrl(testId);
  const pmmUrl = createPaymentMethodManifestUrl(testId, {
    body: JSON.stringify({
      default_applications: [wamUrl],
      supported_origins: [
        // Origin without an explicit port (uses default port 443).
        `https://${get_host_info().ORIGINAL_HOST}`,
        // Origins with an explicit port (wptserve's HTTPS port).
        get_host_info().HTTPS_ORIGIN,
        get_host_info().HTTPS_REMOTE_ORIGIN,
        get_host_info().HTTPS_NOTSAMESITE_ORIGIN,
      ],
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

  // The spec does not speak to the supported_origins being fetched by the
  // browser, but we can check whether or not the default_applications was
  // fetched as a proxy for "payment method manifest was parsed as valid".
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
}, 'Multiple valid HTTPS origins in supported_origins are allowed');

promise_test(async t => {
  const testId = token();
  const wamUrl = createWebAppManifestUrl(testId);
  const pmmUrl = createPaymentMethodManifestUrl(testId, {
    body: JSON.stringify({
      default_applications: [wamUrl],
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

  // The spec does not speak to the supported_origins being fetched by the
  // browser, but we can check whether or not the default_applications was
  // fetched as a proxy for "payment method manifest was parsed as valid".
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
}, 'Omitting supported_origins is allowed');

promise_test(async t => {
  const testId = token();
  const wamUrl = createWebAppManifestUrl(testId);
  const pmmUrl = createPaymentMethodManifestUrl(testId, {
    body: JSON.stringify({
      default_applications: [wamUrl],
      supported_origins: get_host_info().HTTPS_ORIGIN,
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

  // The non-array supported_origins should cause payment method manifest
  // validation to fail, so the default_applications should not be fetched.
  assert_equals(
      logs.length, 2,
      'Browser must issue only 2 server requests (HEAD PMI, GET PMM) when supported_origins is a string');
  assert_equals(logs[0].endpoint, 'payment-method-identifier',
                'First request must hit PMI URL');
  assert_equals(logs[1].endpoint, 'payment-method-manifest',
                'Second request must hit PMM URL');
  const wamLogs = logs.filter(l => l.endpoint === 'web-app-manifest');
  assert_equals(
      wamLogs.length, 0,
      'Non-array supported_origins must fail validation and prevent WAM fetch');
}, 'Non-array value for supported_origins fails validation');

promise_test(async t => {
  const testId = token();
  const wamUrl = createWebAppManifestUrl(testId);
  const pmmUrl = createPaymentMethodManifestUrl(testId, {
    body: JSON.stringify({
      default_applications: [wamUrl],
      supported_origins: '*',
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

  // The wildcard supported_origins should cause payment method manifest
  // validation to fail, so the default_applications should not be fetched.
  assert_equals(
      logs.length, 2,
      'Browser must issue only 2 server requests (HEAD PMI, GET PMM) when supported_origins is wildcard "*"');
  assert_equals(logs[0].endpoint, 'payment-method-identifier',
                'First request must hit PMI URL');
  assert_equals(logs[1].endpoint, 'payment-method-manifest',
                'Second request must hit PMM URL');
  const wamLogs = logs.filter(l => l.endpoint === 'web-app-manifest');
  assert_equals(
      wamLogs.length, 0,
      'Wildcard "*" supported_origins must fail validation and prevent WAM fetch');
}, 'Wildcard "*" for supported_origins fails validation');

promise_test(async t => {
  const testId = token();
  const wamUrl = createWebAppManifestUrl(testId);
  const pmmUrl = createPaymentMethodManifestUrl(testId, {
    body: JSON.stringify({
      default_applications: [wamUrl],
      supported_origins: null,
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

  // The null supported_origins should cause payment method manifest validation
  // to fail, so the default_applications should not be fetched.
  assert_equals(
      logs.length, 2,
      'Browser must issue only 2 server requests (HEAD PMI, GET PMM) when supported_origins is null');
  assert_equals(logs[0].endpoint, 'payment-method-identifier',
                'First request must hit PMI URL');
  assert_equals(logs[1].endpoint, 'payment-method-manifest',
                'Second request must hit PMM URL');
  const wamLogs = logs.filter(l => l.endpoint === 'web-app-manifest');
  assert_equals(
      wamLogs.length, 0,
      'Null supported_origins must fail validation and prevent WAM fetch');
}, 'Null value for supported_origins fails validation');

promise_test(async t => {
  const testId = token();
  const wamUrl = createWebAppManifestUrl(testId);
  const pmmUrl = createPaymentMethodManifestUrl(testId, {
    body: JSON.stringify({
      default_applications: [wamUrl],
      supported_origins: [],
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

  // The empty supported_origins list should cause payment method manifest
  // validation to fail, so the default_applications should not be fetched.
  assert_equals(
      logs.length, 2,
      'Browser must issue only 2 server requests (HEAD PMI, GET PMM) when supported_origins is an empty array');
  assert_equals(logs[0].endpoint, 'payment-method-identifier',
                'First request must hit PMI URL');
  assert_equals(logs[1].endpoint, 'payment-method-manifest',
                'Second request must hit PMM URL');
  const wamLogs = logs.filter(l => l.endpoint === 'web-app-manifest');
  assert_equals(
      wamLogs.length, 0,
      'Empty supported_origins array must fail entire manifest validation and prevent WAM fetch');
}, 'Empty array for supported_origins fails validation');

promise_test(async t => {
  const testId = token();
  const wamUrl = createWebAppManifestUrl(testId);
  const pmmUrl = createPaymentMethodManifestUrl(testId, {
    body: JSON.stringify({
      default_applications: [wamUrl],
      supported_origins: [get_host_info().HTTPS_ORIGIN, 123],
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

  // The non-string item in the supported_origins should cause payment method
  // manifest validation to fail, so the default_applications should not be
  // fetched.
  assert_equals(
      logs.length, 2,
      'Browser must issue only 2 server requests (HEAD PMI, GET PMM) when supported_origins contains a non-string');
  assert_equals(logs[0].endpoint, 'payment-method-identifier',
                'First request must hit PMI URL');
  assert_equals(logs[1].endpoint, 'payment-method-manifest',
                'Second request must hit PMM URL');
  const wamLogs = logs.filter(l => l.endpoint === 'web-app-manifest');
  assert_equals(
      wamLogs.length, 0,
      'Non-string item in supported_origins must fail validation and prevent WAM fetch');
}, 'Non-string item in supported_origins fails validation');

promise_test(async t => {
  const testId = token();
  const wamUrl = createWebAppManifestUrl(testId);
  const pmmUrl = createPaymentMethodManifestUrl(testId, {
    body: JSON.stringify({
      default_applications: [wamUrl],
      supported_origins: ['/relative-origin'],
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

  // The relative URL in the supported_origins should cause payment method
  // manifest validation to fail, so the default_applications should not be
  // fetched.
  assert_equals(
      logs.length, 2,
      'Browser must issue only 2 server requests (HEAD PMI, GET PMM) when supported_origins contains a relative URL');
  assert_equals(logs[0].endpoint, 'payment-method-identifier',
                'First request must hit PMI URL');
  assert_equals(logs[1].endpoint, 'payment-method-manifest',
                'Second request must hit PMM URL');
  const wamLogs = logs.filter(l => l.endpoint === 'web-app-manifest');
  assert_equals(
      wamLogs.length, 0,
      'Relative URL in supported_origins must fail validation and prevent WAM fetch');
}, 'Relative URL in supported_origins fails validation');

promise_test(async t => {
  const testId = token();
  const wamUrl = createWebAppManifestUrl(testId);
  const pmmUrl = createPaymentMethodManifestUrl(testId, {
    body: JSON.stringify({
      default_applications: [wamUrl],
      supported_origins: [
        get_host_info().HTTPS_ORIGIN,
        get_host_info().HTTP_ORIGIN,
      ],
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

  // The non-HTTPS origin in the supported_origins should cause payment method
  // manifest validation to fail, so the default_applications should not be
  // fetched.
  assert_equals(
      logs.length, 2,
      'Browser must issue only 2 server requests (HEAD PMI, GET PMM) when supported_origins contains an HTTP origin');
  assert_equals(logs[0].endpoint, 'payment-method-identifier',
                'First request must hit PMI URL');
  assert_equals(logs[1].endpoint, 'payment-method-manifest',
                'Second request must hit PMM URL');
  const wamLogs = logs.filter(l => l.endpoint === 'web-app-manifest');
  assert_equals(
      wamLogs.length, 0,
      'Non-HTTPS origin in supported_origins must fail validation and prevent WAM fetch');
}, 'Non-HTTPS origin in supported_origins fails validation');

promise_test(async t => {
  const testId = token();
  const wamUrl = createWebAppManifestUrl(testId);
  const pmmUrl = createPaymentMethodManifestUrl(testId, {
    body: JSON.stringify({
      default_applications: [wamUrl],
      supported_origins: [get_host_info().HTTPS_ORIGIN_WITH_CREDS],
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

  // The inclusion of a username/password origin in the supported_origins should
  // cause payment method manifest validation to fail, so the
  // default_applications should not be fetched.
  assert_equals(
      logs.length, 2,
      'Browser must issue only 2 server requests (HEAD PMI, GET PMM) when supported_origins contains credentials');
  assert_equals(logs[0].endpoint, 'payment-method-identifier',
                'First request must hit PMI URL');
  assert_equals(logs[1].endpoint, 'payment-method-manifest',
                'Second request must hit PMM URL');
  const wamLogs = logs.filter(l => l.endpoint === 'web-app-manifest');
  assert_equals(
      wamLogs.length, 0,
      'Origin with username/password in supported_origins must fail validation and prevent WAM fetch');
}, 'Origin with username or password in supported_origins fails validation');

promise_test(async t => {
  const testId = token();
  const wamUrl = createWebAppManifestUrl(testId);
  const pmmUrl = createPaymentMethodManifestUrl(testId, {
    body: JSON.stringify({
      default_applications: [wamUrl],
      supported_origins: [`${get_host_info().HTTPS_ORIGIN}/webpay`],
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

  // The inclusion of an origin with a path in the supported_origins should
  // cause payment method manifest validation to fail, so the
  // default_applications should not be fetched.
  assert_equals(
      logs.length, 2,
      'Browser must issue only 2 server requests (HEAD PMI, GET PMM) when supported_origins contains a path');
  assert_equals(logs[0].endpoint, 'payment-method-identifier',
                'First request must hit PMI URL');
  assert_equals(logs[1].endpoint, 'payment-method-manifest',
                'Second request must hit PMM URL');
  const wamLogs = logs.filter(l => l.endpoint === 'web-app-manifest');
  assert_equals(
      wamLogs.length, 0,
      'Origin with non-empty path in supported_origins must fail validation and prevent WAM fetch');
}, 'Origin with path in supported_origins fails validation');

promise_test(async t => {
  const testId = token();
  const wamUrl = createWebAppManifestUrl(testId);
  const pmmUrl = createPaymentMethodManifestUrl(testId, {
    body: JSON.stringify({
      default_applications: [wamUrl],
      supported_origins: [`${get_host_info().HTTPS_ORIGIN}/?action=webpay`],
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

  // The inclusion of an origin with a query in the supported_origins should
  // cause payment method manifest validation to fail, so the
  // default_applications should not be fetched.
  assert_equals(
      logs.length, 2,
      'Browser must issue only 2 server requests (HEAD PMI, GET PMM) when supported_origins contains a query string');
  assert_equals(logs[0].endpoint, 'payment-method-identifier',
                'First request must hit PMI URL');
  assert_equals(logs[1].endpoint, 'payment-method-manifest',
                'Second request must hit PMM URL');
  const wamLogs = logs.filter(l => l.endpoint === 'web-app-manifest');
  assert_equals(
      wamLogs.length, 0,
      'Origin with query in supported_origins must fail validation and prevent WAM fetch');
}, 'Origin with query in supported_origins fails validation');

promise_test(async t => {
  const testId = token();
  const wamUrl = createWebAppManifestUrl(testId);
  const pmmUrl = createPaymentMethodManifestUrl(testId, {
    body: JSON.stringify({
      default_applications: [wamUrl],
      supported_origins: [`${get_host_info().HTTPS_ORIGIN}/#webpay`],
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

  // The inclusion of an origin with a fragment in the supported_origins should
  // cause payment method manifest validation to fail, so the
  // default_applications should not be fetched.
  assert_equals(
      logs.length, 2,
      'Browser must issue only 2 server requests (HEAD PMI, GET PMM) when supported_origins contains a fragment');
  assert_equals(logs[0].endpoint, 'payment-method-identifier',
                'First request must hit PMI URL');
  assert_equals(logs[1].endpoint, 'payment-method-manifest',
                'Second request must hit PMM URL');
  const wamLogs = logs.filter(l => l.endpoint === 'web-app-manifest');
  assert_equals(
      wamLogs.length, 0,
      'Origin with fragment in supported_origins must fail validation and prevent WAM fetch');
}, 'Origin with fragment in supported_origins fails validation');
