// META: script=/common/get-host-info.sub.js
// META: script=/service-workers/service-worker/resources/test-helpers.sub.js

const SUCCESS = true;
const FAILURE = false;

async function run_test(t, sw_policy, dw_policy, expected_success, expected_intercepted, description) {
  const scope = 'resources/';
  const sw_script = `resources/sw-intercept-and-fetch.js?pipe=header(Connection-Allowlist,${encodeURIComponent(sw_policy)})`;
  const registration = await service_worker_unregister_and_register(t, sw_script, scope);
  await wait_for_state(t, registration.installing, 'activated');

  const iframe = await with_iframe('resources/blank-with-allowlist.html');

  const dw_script = `worker-fetch-script.js?pipe=header(Connection-Allowlist,${encodeURIComponent(dw_policy)})`;
  const worker = new iframe.contentWindow.Worker(dw_script);

  t.add_cleanup(async () => {
    worker.terminate();
    iframe.remove();
    await registration.unregister();
  });

  const target_url = get_host_info().HTTPS_ORIGIN + '/common/blank-with-cors.html';

  let sw_intercepted = false;
  const listener = (event) => {
    if (event.data && event.data.type === 'sw-intercepted') {
      sw_intercepted = true;
    }
  };
  iframe.contentWindow.navigator.serviceWorker.addEventListener('message', listener);

  worker.postMessage(target_url);

  const worker_result = await new Promise((resolve) => {
    worker.onmessage = (e) => {
      resolve(e.data);
    };
    worker.onerror = (e) => {
      resolve({ success: false, error: 'WorkerError' });
    };
  });

  iframe.contentWindow.navigator.serviceWorker.removeEventListener('message', listener);

  assert_equals(worker_result.success, expected_success,
    `Fetch success expectation: ${expected_success}. Result error: ${worker_result.error || 'none'}`);
  assert_equals(sw_intercepted, expected_intercepted,
    `Fetch interception expectation: ${expected_intercepted}`);
}

// 1. DW allows, SW allows -> Fetch succeeds, SW intercepts.
promise_test(async t => {
  await run_test(
    t,
    '\\(response-origin\\)', // sw_policy
    '\\(response-origin\\)', // dw_policy
    SUCCESS,             // expected_success
    true,                // expected_intercepted
    "Fetch succeeds when both Dedicated Worker and Service Worker allow it."
  );
}, "Fetch succeeds when both Dedicated Worker and Service Worker allow it.");

// 2. DW blocks, SW allows -> Fetch is blocked, SW does NOT intercept.
promise_test(async t => {
  await run_test(
    t,
    '\\(response-origin\\)', // sw_policy
    '\\(\\)',                // dw_policy (empty allowlist blocks all)
    FAILURE,             // expected_success
    false,               // expected_intercepted
    "Fetch is blocked by Dedicated Worker's CA and does not reach Service Worker."
  );
}, "Fetch is blocked by Dedicated Worker's CA and does not reach Service Worker.");

// 3. DW allows, SW blocks -> Fetch is blocked, SW intercepts.
promise_test(async t => {
  await run_test(
    t,
    '\\(\\)',                // sw_policy (empty allowlist blocks all)
    '\\(response-origin\\)', // dw_policy
    FAILURE,             // expected_success
    true,                // expected_intercepted
    "Fetch is intercepted by Service Worker but blocked by Service Worker's CA."
  );
}, "Fetch is intercepted by Service Worker but blocked by Service Worker's CA.");
