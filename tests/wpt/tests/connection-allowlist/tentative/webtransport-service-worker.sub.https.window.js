// META: script=/service-workers/service-worker/resources/test-helpers.sub.js
//
// The following tests assume no Connection-Allowlist is set on this document.
// The script loaded at service-worker-webtransport-connector.https.js has its
// own allowlist.

const wt_port = '{{ports[webtransport-h3][0]}}';
const SUCCESS = true;
const FAILURE = false;

async function service_worker_webtransport_test(
    t, host, expectation, description) {
  const scope = 'resources/';
  const registration = await service_worker_unregister_and_register(
      t, 'resources/service-worker-webtransport-connector.https.js', scope);
  t.add_cleanup(() => registration.unregister()); // Ensure cleanup on failure
  await wait_for_state(t, registration.installing, 'activated');

  const wt_url = `https://${host}:${
      wt_port}/webtransport/handlers/custom-response.py?:status=200`;
  const controller = registration.active;

  const result = await new Promise((resolve) => {
    navigator.serviceWorker.onmessage = (e) => resolve(e.data.success);
    controller.postMessage(wt_url);
  });

  if (expectation === SUCCESS) {
    assert_true(result, `WebTransport connection to ${host} should succeed.`);
  } else {
    assert_false(
        result, `WebTransport connection to ${host} should be blocked.`);
  }
}

promise_test(async t => {
  await service_worker_webtransport_test(
      t, '{{hosts[][]}}', SUCCESS,
      'Same-origin WebTransport connection should succeed.');
});

promise_test(async t => {
  await service_worker_webtransport_test(
      t, '{{hosts[alt][]}}', FAILURE,
      'Cross-origin WebTransport connection should fail.');
});
