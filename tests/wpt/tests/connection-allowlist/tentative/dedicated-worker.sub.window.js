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

    const msgEvent = await new Promise((resolve, reject) => {
      worker.onmessage = resolve;
      worker.onerror = (e) => reject(new Error("Worker Error"));
    });

    if (expectation === SUCCESS) {
      assert_true(msgEvent.data.success, `Fetch to ${origin} should succeed.`);
    } else {
      assert_false(msgEvent.data.success, `Fetch to ${origin} should be blocked.`);
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
worker_fetch_test(
  "http://{{hosts[alt][]}}" + port,
  FAILURE,
  "Cross-origin fetch from a dedicated worker (data: URL) should be blocked by inherited policy."
);

function worker_script_fetch_test(origin, expectation, description) {
  promise_test(async t => {
    const script_url = `${origin}/connection-allowlist/tentative/resources/worker-fetch-script.js`;
    let worker;
    try {
      worker = new Worker(script_url);
    } catch (e) {
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

// 3. Same-origin main script fetch should succeed.
worker_script_fetch_test(
  get_host_info().HTTP_ORIGIN,
  SUCCESS,
  "Same-origin dedicated worker main script fetch succeeds."
);

// 4. Cross-origin main script fetch should be blocked by the creator's policy.
worker_script_fetch_test(
  get_host_info().HTTP_REMOTE_ORIGIN,
  FAILURE,
  "Cross-origin dedicated worker main script fetch should be blocked by creator policy."
);

// 5. Worker script with empty connection allowlist should not be able to fetch anything.
promise_test(async t => {
  const script_url = "resources/worker-fetch-script-empty.js";
  const worker = new Worker(script_url);

  const fetch_url = `${get_host_info().HTTP_ORIGIN}/common/blank-with-cors.html`;
  worker.postMessage(fetch_url);

  const msgEvent = await new Promise((resolve, reject) => {
    worker.onmessage = resolve;
    worker.onerror = (e) => reject(new Error("Worker Error"));
  });

  assert_false(msgEvent.data.success, "Fetch from worker with empty allowlist should be blocked.");
}, "Dedicated worker with empty connection allowlist cannot perform any fetches.");