// META: script=/common/get-host-info.sub.js
//
// The following tests assume the policy `Connection-Allowlist: (response-origin)` has been set.

const port = get_host_info().HTTP_PORT_ELIDED;
const SUCCESS = true;
const FAILURE = false;

// The worker content will attempt to fetch a URL and post the result back.
const worker_content = `
  importScripts("${get_host_info().HTTP_ORIGIN}/connection-allowlist/tentative/resources/shared-worker-onconnect.js");
`;
const dataUrl = "data:text/javascript," + encodeURIComponent(worker_content);

function worker_fetch_test(origin, expectation, description) {
  promise_test(async t => {
    const worker = new SharedWorker(dataUrl);
    const fetch_url = `${origin}/common/blank-with-cors.html`;

    worker.port.postMessage(fetch_url);

    const msgEvent = await new Promise((resolve, reject) => {
      worker.port.onmessage = resolve;
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
  "Same-origin fetch from a shared worker (data: URL) succeeds."
);

// 2. Cross-origin fetch from the worker should be blocked as it is not allowlisted.
worker_fetch_test(
  "http://{{hosts[alt][]}}" + port,
  FAILURE,
  "Cross-origin fetch from a shared worker (data: URL) should be blocked by inherited policy."
);

function worker_script_fetch_test(origin, expectation, description) {
  promise_test(async t => {
    const script_url = `${origin}/connection-allowlist/tentative/resources/shared-worker-fetch-script.js`;
    let worker;
    try {
      worker = new SharedWorker(script_url);
    } catch (e) {
      assert_equals(expectation, FAILURE, "SharedWorker constructor threw unexpectedly");
      return;
    }

    const promise = new Promise((resolve, reject) => {
      worker.port.onmessage = () => resolve(SUCCESS);
      worker.onerror = (e) => {
        e.preventDefault();
        reject(new Error("Worker Load Error"));
      };
      // Send a message to the worker. If it loaded successfully, it will
      // respond and onmessage will fire. If it failed to load, onerror
      // should fire.
      worker.port.postMessage(`${get_host_info().HTTP_ORIGIN}/common/blank-with-cors.html`);
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
  "Same-origin shared worker main script fetch succeeds."
);

// 4. Cross-origin main script fetch should be blocked by the creator's policy.
worker_script_fetch_test(
  get_host_info().HTTP_REMOTE_ORIGIN,
  FAILURE,
  "Cross-origin shared worker main script fetch should be blocked by creator policy."
);

// 5. Worker script with empty connection allowlist should not be able to fetch anything.
promise_test(async t => {
  const script_url = "resources/shared-worker-fetch-script-empty.js";
  const worker = new SharedWorker(script_url);

  const fetch_url = `${get_host_info().HTTP_ORIGIN}/common/blank-with-cors.html`;
  worker.port.postMessage(fetch_url);

  const msgEvent = await new Promise((resolve, reject) => {
    worker.port.onmessage = resolve;
    worker.onerror = (e) => reject(new Error("Worker Error"));
  });

  assert_false(msgEvent.data.success, "Fetch from worker with empty allowlist should be blocked.");
}, "Shared worker with empty connection allowlist cannot perform any fetches.");

// 6. Same-origin fetch from a shared worker (blob: URL) should succeed.
promise_test(async t => {
  const blob = new Blob([worker_content], {type: 'text/javascript'});
  const blobUrl = URL.createObjectURL(blob);
  t.add_cleanup(() => URL.revokeObjectURL(blobUrl));

  const worker = new SharedWorker(blobUrl);
  const fetch_url = `${get_host_info().HTTP_ORIGIN}/common/blank-with-cors.html`;

  worker.port.postMessage(fetch_url);

  const msgEvent = await new Promise((resolve, reject) => {
    worker.port.onmessage = resolve;
    worker.onerror = (e) => reject(new Error("Worker Error"));
  });

  assert_true(msgEvent.data.success, "Same-origin fetch from a shared worker (blob: URL) should succeed.");
}, "Same-origin fetch from a shared worker (blob: URL) succeeds.");

// 7. Shared worker (blob: URL) inherits creator's connection allowlist policy.
promise_test(async t => {
  const blob = new Blob([worker_content], {type: 'text/javascript'});
  const blobUrl = URL.createObjectURL(blob);
  t.add_cleanup(() => URL.revokeObjectURL(blobUrl));

  const worker = new SharedWorker(blobUrl);
  const cross_origin = "http://{{hosts[alt][]}}" + port;
  const fetch_url = `${cross_origin}/common/blank-with-cors.html`;

  worker.port.postMessage(fetch_url);

  const msgEvent = await new Promise((resolve, reject) => {
    worker.port.onmessage = resolve;
    worker.onerror = (e) => reject(new Error("Worker Error"));
  });

  assert_false(msgEvent.data.success, "Cross-origin fetch from a shared worker (blob: URL) should be blocked by inherited policy.");
}, "Shared worker (blob: URL) inherits creator's connection allowlist policy.");
