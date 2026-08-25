// META: script=/common/get-host-info.sub.js
//
// This document has no Connection-Allowlist, but it loads a Shared Worker
// script from resources/shared-worker-websocket.https.js, which maintains
// its own allowlist. The worker content will attempt to connect via
// WebSocket and post the result back to this document.

const ws_port = '{{ports[wss][0]}}';
const SUCCESS = true;
const FAILURE = false;

function shared_worker_websocket_test(host, expectation, description) {
  promise_test(async t => {
    const worker =
        new SharedWorker('resources/shared-worker-websocket.https.js');

    // Start the message port via onmessage.
    const msgEvent = new Promise((resolve, reject) => {
      worker.port.onmessage = resolve;
    });

    // Tell the SharedWorker to initiate a WebSocket connection to `host`.
    const ws_url = `wss://${host}:${ws_port}/echo`;
    worker.port.postMessage(ws_url);

    // Wait for the SharedWorker to reply.
    const result = await msgEvent;

    if (expectation === SUCCESS) {
      assert_true(result.data.success, `WebSocket to ${host} should succeed.`);
    } else {
      assert_false(
          result.data.success, `WebSocket to ${host} should be blocked.`);
    }
  }, description);
}

// Same-origin WebSocket from the worker should succeed (allowlisted via
// explicit pattern).
shared_worker_websocket_test(
    '{{hosts[][]}}', SUCCESS,
    'Same-origin WebSocket from a shared worker succeeds.');

// Same-site but cross-origin subdomains should fail.
shared_worker_websocket_test(
    '{{hosts[][www]}}', FAILURE,
    'Cross-origin same-site WebSocket (www) from a shared worker is blocked.');

shared_worker_websocket_test(
    '{{hosts[][www1]}}', FAILURE,
    'Cross-origin same-site WebSocket (www1) from a shared worker is blocked.');

// Cross-site origins should fail.
shared_worker_websocket_test(
    '{{hosts[alt][]}}', FAILURE,
    'Cross-site WebSocket from a shared worker is blocked.');

shared_worker_websocket_test(
    '{{hosts[alt][www]}}', FAILURE,
    'Cross-site WebSocket (www subdomain) from a shared worker is blocked.');
