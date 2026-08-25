// META: script=/common/get-host-info.sub.js
//
// This document has no Connection-Allowlist, but it loads a Shared Worker
// script from resources/shared-worker-webtransport.https.js, which maintains
// its own allowlist. The worker content will attempt to connect via
// WebTransport and post the result back to this document.

const wt_port = '{{ports[webtransport-h3][0]}}';
const SUCCESS = true;
const FAILURE = false;

function shared_worker_webtransport_test(host, expectation, description) {
  promise_test(async t => {
    const worker =
        new SharedWorker('resources/shared-worker-webtransport.https.js');

    // Start the message port via onmessage.
    const msgEvent = new Promise((resolve, reject) => {
      worker.port.onmessage = resolve;
    });

    // Tell the SharedWorker to initiate a WebTransport connection to `host`.
    const wt_url = `https://${host}:${
        wt_port}/webtransport/handlers/custom-response.py?:status=200`;
    worker.port.postMessage(wt_url);

    // Wait for the SharedWorker to reply.
    const result = await msgEvent;

    if (expectation === SUCCESS) {
      assert_true(
          result.data.success, `WebTransport to ${host} should succeed.`);
    } else {
      assert_false(
          result.data.success, `WebTransport to ${host} should be blocked.`);
    }
  }, description);
}

// Same-origin WebTransport from the worker should succeed (allowlisted via
// explicit pattern).
shared_worker_webtransport_test(
    '{{hosts[][]}}', SUCCESS,
    'Same-origin WebTransport from a shared worker succeeds.');

// Same-site but cross-origin subdomains should fail.
shared_worker_webtransport_test(
    '{{hosts[][www]}}', FAILURE,
    'Cross-origin same-site WebTransport (www) from a shared worker is blocked.');

shared_worker_webtransport_test(
    '{{hosts[][www1]}}', FAILURE,
    'Cross-origin same-site WebTransport (www1) from a shared worker is blocked.');

// Cross-site origins should fail.
shared_worker_webtransport_test(
    '{{hosts[alt][]}}', FAILURE,
    'Cross-site WebTransport from a shared worker is blocked.');

shared_worker_webtransport_test(
    '{{hosts[alt][www]}}', FAILURE,
    'Cross-site WebTransport (www subdomain) from a shared worker is blocked.');
