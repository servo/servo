// META: script=/common/get-host-info.sub.js
//
// The following tests assume the policy
// `Connection-Allowlist: (response-origin
// "https://{{host}}:{{ports[webtransport-h3][0]}}")` has been set. The WPT
// WebTransport server runs on a different port than HTTPS, so the allowlist
// explicitly includes the same-host WebTransport origin.

const wt_port = '{{ports[webtransport-h3][0]}}';
const SUCCESS = true;
const FAILURE = false;

// The worker content will attempt to connect via WebTransport and post the
// result back.
const worker_content = `
  onmessage = async (e) => {
    const url = e.data;
    try {
      const wt = new WebTransport(url);
      await wt.ready;
      wt.close();
      postMessage({ url: url, success: true });
    } catch (err) {
      postMessage({ url: url, success: false, error: err.name });
    }
  };
`;
const dataUrl = 'data:text/javascript,' + encodeURIComponent(worker_content);

function worker_webtransport_test(host, expectation, description) {
  promise_test(async t => {
    const worker = new Worker(dataUrl, {type: 'module'});
    const wt_url = `https://${host}:${
        wt_port}/webtransport/handlers/custom-response.py?:status=200`;

    worker.postMessage(wt_url);

    const msgEvent = await new Promise((resolve, reject) => {
      worker.onmessage = resolve;
      worker.onerror = (e) => reject(new Error('Worker Error'));
    });

    if (expectation === SUCCESS) {
      assert_true(
          msgEvent.data.success, `WebTransport to ${host} should succeed.`);
    } else {
      assert_false(
          msgEvent.data.success, `WebTransport to ${host} should be blocked.`);
    }
  }, description);
}

// Same-origin WebTransport from the worker should succeed (allowlisted via
// explicit pattern).
worker_webtransport_test(
    '{{hosts[][]}}', SUCCESS,
    'Same-origin WebTransport from a dedicated worker succeeds.');

// Same-site but cross-origin subdomains should fail.
worker_webtransport_test(
    '{{hosts[][www]}}', FAILURE,
    'Cross-origin same-site WebTransport (www) from a dedicated worker is blocked.');

worker_webtransport_test(
    '{{hosts[][www1]}}', FAILURE,
    'Cross-origin same-site WebTransport (www1) from a dedicated worker is blocked.');

// Cross-site origins should fail.
worker_webtransport_test(
    '{{hosts[alt][]}}', FAILURE,
    'Cross-site WebTransport from a dedicated worker is blocked.');

worker_webtransport_test(
    '{{hosts[alt][www]}}', FAILURE,
    'Cross-site WebTransport (www subdomain) from a dedicated worker is blocked.');
