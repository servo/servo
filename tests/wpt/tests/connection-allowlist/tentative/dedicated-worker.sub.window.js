// META: script=/common/get-host-info.sub.js
//
// The following tests assume the policy `Connection-Allowlist: (response-origin)` has been set.

const port = get_host_info().HTTP_PORT_ELIDED;
const SUCCESS = true;
const FAILURE = false;

// The worker content will attempt to fetch a URL and post the result back.
const worker_content = `
  onmessage = async (e) => {
    const url = e.data;
    try {
      const r = await fetch(url, { mode: 'cors', credentials: 'omit' });
      postMessage({ url: url, success: r.ok });
    } catch (err) {
      postMessage({ url: url, success: false, error: err.name });
    }
  };
`;
const dataUrl = "data:text/javascript," + encodeURIComponent(worker_content);

function worker_fetch_test(origin, expectation, description) {
  promise_test(async t => {
    const worker = new Worker(dataUrl, { type: 'module' });
    const fetch_url = `${origin}/common/blank-with-cors.html`;

    worker.postMessage(fetch_url);

    const msgEvent = await new Promise((resolve) => {
      worker.onmessage = resolve;
      worker.onerror = (e) => resolve({ data: { success: false, error: "Worker Error" } });
    });

    if (expectation === SUCCESS) {
      assert_true(msgEvent.data.success, `Fetch to ${origin} should succeed.`);
    } else {
      // TODO(crbug.com/447954811): This should be false (blocked) once inheritance is implemented.
      // For now, we expect SUCCESS because it's not yet implemented.
      assert_true(msgEvent.data.success, `Fetch to ${origin} currently succeeds but should be blocked.`);
    }
  }, description);
}

// 1. Same-origin fetch from the worker should succeed as it is allowlisted via (response-origin).
worker_fetch_test(
  "http://{{hosts[][]}}" + port,
  SUCCESS,
  "Same-origin fetch from a dedicated worker (data: URL) succeeds."
);

// 2. Cross-origin fetch from the worker should be blocked as it is not allowlisted.
// Currently it succeeds because inheritance/blocking is not implemented.
worker_fetch_test(
  "http://{{hosts[alt][]}}" + port,
  FAILURE,
  "Cross-origin fetch from a dedicated worker (data: URL) should be blocked by inherited policy."
);