// META: script=/common/get-host-info.sub.js
// META: script=/service-workers/service-worker/resources/test-helpers.sub.js
//
// The following tests assume the policy `Connection-Allowlist: (response-origin)` has been set.

const port = get_host_info().HTTPS_PORT_ELIDED;
const SUCCESS = true;
const FAILURE = false;

async function service_worker_fetch_test(t, script, origin, expectation, description) {
  const scope = 'resources/';
  const registration = await service_worker_unregister_and_register(t, script, scope);
  await wait_for_state(t, registration.installing, 'activated');

  const fetch_url = `${origin}/common/blank-with-cors.html`;
  const controller = registration.active;

  const result = await new Promise((resolve) => {
    navigator.serviceWorker.onmessage = (e) => resolve(e.data.success);
    controller.postMessage(fetch_url);
  });

  if (expectation === SUCCESS) {
    assert_true(result, `Fetch to ${origin} should succeed.`);
  } else {
    assert_false(result, `Fetch to ${origin} should be blocked.`);
  }

  await registration.unregister();
}

// 1. Same-origin fetch from the service worker should succeed as it is allowlisted via (response-origin).
promise_test(async t => {
  await service_worker_fetch_test(
    t,
    'resources/service-worker-fetch-script.js',
    "https://{{hosts[][]}}" + port,
    SUCCESS,
    "Same-origin fetch from a service worker succeeds."
  );
}, "Same-origin fetch from a service worker succeeds.");

// 2. Cross-origin fetch from the service worker should be blocked as it is not allowlisted.
promise_test(async t => {
  await service_worker_fetch_test(
    t,
    'resources/service-worker-fetch-script.js',
    "https://{{hosts[alt][]}}" + port,
    FAILURE,
    "Cross-origin fetch from a service worker should be blocked by its own policy."
  );
}, "Cross-origin fetch from a service worker should be blocked by its own policy.");

// 3. Service worker with empty connection allowlist cannot perform any fetches.
promise_test(async t => {
  await service_worker_fetch_test(
    t,
    'resources/service-worker-fetch-script-empty.js',
    get_host_info().HTTPS_ORIGIN,
    FAILURE,
    "Service worker with empty connection allowlist cannot perform any fetches."
  );
}, "Service worker with empty connection allowlist cannot perform any fetches.");

// Tests 4 to 7 are independent of connection allowlists since service workers
// with local schemes are expected to fail.

// 4. Service worker registration with blob: scheme should fail.
promise_test(async t => {
  const blob = new Blob(['self.addEventListener("fetch", (e) => {});'], { type: 'text/javascript' });
  const url = URL.createObjectURL(blob);
  await promise_rejects_js(t, TypeError,
    navigator.serviceWorker.register(url),
    "Service worker registration with blob: scheme should fail with TypeError.");
}, "Service worker registration with blob: scheme fails.");

// 5. Service worker registration with data: scheme should fail.
promise_test(async t => {
  const url = "data:text/javascript,self.addEventListener('fetch', (e) => {});";
  await promise_rejects_js(t, TypeError,
    navigator.serviceWorker.register(url),
    "Service worker registration with data: scheme should fail with TypeError.");
}, "Service worker registration with data: scheme fails.");

// 6. Service worker registration with filesystem: scheme should fail.
promise_test(async t => {
  const url = "filesystem:https://{{host}}/temporary/sw.js";
  await promise_rejects_js(t, TypeError,
    navigator.serviceWorker.register(url),
    "Service worker registration with filesystem: scheme should fail with TypeError.");
}, "Service worker registration with filesystem: scheme fails.");

// 7. Service worker registration with about:blank scheme should fail.
promise_test(async t => {
  const url = "about:blank";
  await promise_rejects_js(t, TypeError,
    navigator.serviceWorker.register(url),
    "Service worker registration with about:blank scheme should fail with TypeError.");
}, "Service worker registration with about:blank scheme fails.");

// 8. Service worker main script fetch succeeds when it is same-origin and allowlisted.
promise_test(async t => {
  const script = 'resources/service-worker-fetch-script.js';
  const scope = 'resources/same-origin-allowlisted-scope';
  const registration = await service_worker_unregister_and_register(t, script, scope);
  t.add_cleanup(_ => registration.unregister());
  assert_not_equals(registration.installing, null, 'worker is installing');
}, "Same-origin service worker main script fetch succeeds with (response-origin) allowlist.");

// 9. Service worker main script fetch is blocked by creator's empty allowlist.
promise_test(async t => {
  const iframe = await with_iframe('resources/blank-with-allowlist.html?pipe=header(Connection-Allowlist,\\(\\))');
  t.add_cleanup(() => iframe.remove());

  const script = 'service-worker-fetch-script.js';
  const scope = 'empty-allowlist-scope';

  // register() should fail with TypeError because the fetch of the script is blocked by the iframe's empty allowlist.
  await promise_rejects_js(t, iframe.contentWindow.TypeError,
    iframe.contentWindow.navigator.serviceWorker.register(script, { scope: scope }),
    "Service worker registration should fail because main script fetch is blocked by creator's empty allowlist.");
}, "Service worker main script fetch is blocked by creator's empty allowlist.");

// 10. Same-origin service worker main script fetch is blocked by creator's non-matching allowlist.
promise_test(async t => {
  const remote_origin = "https://{{hosts[alt][]}}" + port;
  const allowlist = encodeURIComponent(`\\("${remote_origin}/*"\\)`);
  const iframe = await with_iframe(`resources/blank-with-allowlist.html?pipe=header(Connection-Allowlist,${allowlist})`);
  t.add_cleanup(() => iframe.remove());

  const script = 'service-worker-fetch-script.js';
  const scope = 'non-matching-allowlist-scope';

  // register() should fail with TypeError because the fetch of the script (same-origin)
  // is blocked by the iframe's allowlist which only permits the remote origin.
  await promise_rejects_js(t, iframe.contentWindow.TypeError,
    iframe.contentWindow.navigator.serviceWorker.register(script, { scope: scope }),
    "Service worker registration should fail because main script fetch is blocked by creator's non-matching allowlist.");
}, "Same-origin service worker main script fetch is blocked by creator's non-matching allowlist.");




