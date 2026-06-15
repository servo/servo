// META: script=/service-workers/service-worker/resources/test-helpers.sub.js
//
// The following tests assume no Connection-Allowlist is set on this document.
// The script loaded at service-worker-websocket-connector.https.js has its own
// allowlist.

const ws_port = '{{ports[wss][0]}}';
const SUCCESS = true;
const FAILURE = false;

async function service_worker_websocket_test(
    t, host, expectation, description) {
  const scope = 'resources/';
  const registration = await service_worker_unregister_and_register(
      t, 'resources/service-worker-websocket-connector.https.js', scope);
  t.add_cleanup(() => registration.unregister()); // Ensure cleanup on failure
  await wait_for_state(t, registration.installing, 'activated');

  const ws_url = `wss://${host}:${ws_port}/echo`;
  const controller = registration.active;

  const result = await new Promise((resolve) => {
    navigator.serviceWorker.onmessage = (e) => resolve(e.data.success);
    controller.postMessage(ws_url);
  });

  if (expectation === SUCCESS) {
    assert_true(result, `WebSocket connection to ${host} should succeed.`);
  } else {
    assert_false(result, `WebSocket connection to ${host} should be blocked.`);
  }
}

promise_test(async t => {
  await service_worker_websocket_test(
      t, '{{hosts[][]}}', SUCCESS,
      'Same-origin WebSocket connection should succeed.');
});

promise_test(async t => {
  await service_worker_websocket_test(
      t, '{{hosts[alt][]}}', FAILURE,
      'Cross-origin WebSocket connection should fail.');
});
