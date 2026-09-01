// META: spec=https://w3c.github.io/payment-method-manifest/#fetch-pmm
// META: title=Link header targeting non-HTTPS URL is aborted
// META: script=/common/utils.js
// META: script=/payment-method-manifest/resources/helpers.js

promise_test(async t => {
  const testId = token();
  const httpManifestUrl = `http://{{host}}:{{ports[http][0]}}/payment-method-manifest/resources/payment-method-manifest.py?id=${testId}`;
  const pmiUrl = createPaymentMethodIdentifierUrl(testId, { link: `<${httpManifestUrl}>; rel="payment-method-manifest"` });

  const request = new PaymentRequest(
    [{ supportedMethods: pmiUrl }],
    { total: { label: 'Total', amount: { currency: 'USD', value: '1.00' } } }
  );

  try {
    await request.canMakePayment();
  } catch (err) {}

  const logs = await waitForServerAccessLogs(t, testId, 1);

  assert_equals(logs.length, 1, 'Browser must issue only 1 server request (HEAD to PMI)');
  assert_equals(logs[0].endpoint, 'payment-method-identifier', 'Request must hit PMI URL');
  assert_equals(logs[0].method, 'HEAD', 'PMI request must use HEAD method');
}, 'Link header targeting non-HTTPS URL is aborted');
