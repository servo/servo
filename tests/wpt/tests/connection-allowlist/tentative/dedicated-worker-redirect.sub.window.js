// META: script=/common/get-host-info.sub.js
//
// The following tests assume the policy `Connection-Allowlist: (response-origin)` has been set.

const port = get_host_info().HTTP_PORT_ELIDED;
const SUCCESS = true;
const FAILURE = false;

function worker_script_redirect_test(origin, target_origin, expectation, description) {
  promise_test(async t => {
    const target_url = target_origin + "/connection-allowlist/tentative/resources/worker-fetch-script.js";
    const url = origin + "/common/redirect.py?status=302&location=" + encodeURIComponent(target_url);

    let worker;
    try {
      worker = new Worker(url);
    } catch (e) {
      // Some implementations might throw synchronously for some origins, but usually it's an error event.
      assert_equals(expectation, FAILURE, "Worker constructor threw unexpectedly");
      return;
    }

    const promise = new Promise((resolve, reject) => {
      worker.onmessage = () => resolve(SUCCESS);
      worker.onerror = (e) => {
        e.preventDefault();
        reject(new Error("Worker Load Error"));
      };
      // Send a message to the worker. If it loaded successfully, it will
      // respond and onmessage will fire. If it failed to load, onerror
      // should fire.
      worker.postMessage(`${get_host_info().HTTP_ORIGIN}/common/blank-with-cors.html`);
    });

    if (expectation === SUCCESS) {
      const result = await promise;
      assert_equals(result, expectation, description);
    } else {
      await promise_rejects_js(t, Error, promise, description);
    }
  }, description);
}

// 1. Same-origin worker script redirect:
// origin: http://{{hosts[][]}} (allowed by allowlist)
// target: http://{{hosts[][]}} (also allowed)
// This should FAIL because redirects are default-blocked for allowlisted fetches.
worker_script_redirect_test(
  get_host_info().HTTP_ORIGIN,
  get_host_info().HTTP_ORIGIN,
  FAILURE,
  "Same-origin dedicated worker main script fetch with redirect fails."
);

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

function worker_subresource_redirect_test(fetch_origin, fetch_target_origin, expectation, description) {
  promise_test(async t => {
    const worker = new Worker(dataUrl, { type: 'module' });

    const target_url = fetch_target_origin + "/common/blank-with-cors.html";
    const fetch_url = fetch_origin + "/common/redirect.py?status=302&location=" + encodeURIComponent(target_url);

    worker.postMessage(fetch_url);

    const msgEvent = await new Promise((resolve, reject) => {
      worker.onmessage = resolve;
      worker.onerror = (e) => reject(new Error("Worker Error"));
    });

    if (expectation === SUCCESS) {
      assert_true(msgEvent.data.success, `Fetch to ${fetch_url} should succeed.`);
    } else {
      assert_false(msgEvent.data.success, `Fetch to ${fetch_url} should be blocked.`);
    }
  }, description);
}

// 2. Same-origin subresource fetch from same-origin worker with same-origin redirect:
// worker: same-origin (data: URL inherits policy)
// fetch origin: same-origin
// fetch target: same-origin
// This should FAIL.
worker_subresource_redirect_test(
  get_host_info().HTTP_ORIGIN,
  get_host_info().HTTP_ORIGIN,
  FAILURE,
  "Same-origin subresource fetch from dedicated worker with same-origin redirect fails."
);

// 3. Same-origin subresource fetch from same-origin worker with cross-origin redirect:
// worker: same-origin (data: URL inherits policy)
// fetch origin: same-origin
// fetch target: cross-origin
// This should FAIL.
worker_subresource_redirect_test(
  get_host_info().HTTP_ORIGIN,
  get_host_info().HTTP_REMOTE_ORIGIN,
  FAILURE,
  "Same-origin subresource fetch from dedicated worker with cross-origin redirect fails."
);
