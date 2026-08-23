// META: spec=https://w3c.github.io/payment-method-manifest/#ingest
// META: title=Payment Method Manifest (PMM) fetch and parsing end-to-end success
// META: script=/common/utils.js
// META: script=/payment-method-manifest/resources/helpers.js

promise_test(async t => {
  const testId = token();
  const pmiUrl = createPaymentMethodIdentifierUrl(testId);

  const request = new PaymentRequest(
    [{ supportedMethods: pmiUrl }],
    { total: { label: 'Total', amount: { currency: 'USD', value: '1.00' } } }
  );

  // Trigger Payment Method Manifest (PMM) ingestion pipeline
  try {
    await request.canMakePayment();
  } catch (err) {
    // It is fine for this call to fail; server logs are still captured and
    // inspected below.
  }

  const logs = await waitForServerAccessLogs(t, testId);

  assert_equals(logs.length, 2, 'Browser must issue exactly 2 server requests (HEAD for PMI, GET for PMM)');
  assert_equals(logs[0].endpoint, 'pmi', 'First request must hit PMI URL');
  assert_equals(logs[0].method, 'HEAD', 'PMI request must use HEAD method');

  assert_equals(logs[1].endpoint, 'payment-method-manifest', 'Second request must hit PMM URL');
  assert_equals(logs[1].method, 'GET', 'PMM request must use GET method');
}, 'Payment Method Manifest (PMM) fetch and parsing end-to-end success');
