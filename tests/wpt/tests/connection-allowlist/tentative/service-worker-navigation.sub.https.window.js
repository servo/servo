// META: script=/common/get-host-info.sub.js
// META: script=/service-workers/service-worker/resources/test-helpers.sub.js

const { HTTPS_ORIGIN, HTTPS_NOTSAMESITE_ORIGIN } = get_host_info();

// Helper to run a navigation test.
// - sw_script: SW script URL.
// - iframe_url: iframe URL.
// - target_url: URL to navigate to.
// - expected_result: 'success' or 'failure' (from SW navigate() promise).
// - expected_load: true or false (whether target page actually loaded).
async function run_navigation_test(t, sw_script, iframe_url, target_url, expected_result, expected_load) {
  const scope = 'resources/';

  // Register SW.
  const registration = await service_worker_unregister_and_register(t, sw_script, scope);
  await wait_for_state(t, registration.installing, 'activated');
  t.add_cleanup(() => registration.unregister());

  // Create iframe in scope.
  const iframe = document.createElement('iframe');
  iframe.src = iframe_url;

  const iframe_load_promise = new Promise((resolve) => {
    iframe.onload = () => resolve();
    iframe.onerror = () => resolve();
  });
  document.body.appendChild(iframe);
  await iframe_load_promise;
  t.add_cleanup(() => iframe.remove());

  assert_not_equals(iframe.contentWindow.navigator.serviceWorker.controller, null, 'iframe should be controlled');

  // Set up message listener for 'loaded' message from post-message.html.
  let received_load_message = false;
  const message_handler = (e) => {
    if (e.data === 'loaded') {
      received_load_message = true;
    }
  };
  window.addEventListener('message', message_handler);
  t.add_cleanup(() => window.removeEventListener('message', message_handler));

  // Trigger navigation from SW.
  const sw_message_promise = new Promise((resolve, reject) => {
    const timeout_id = step_timeout(() => {
      reject(new Error("Timeout waiting for SW message."));
    }, 5000);

    const channel = new MessageChannel();
    channel.port1.onmessage = (e) => {
      clearTimeout(timeout_id);
      resolve(e.data);
    };

    iframe.contentWindow.navigator.serviceWorker.controller.postMessage(
      {url: target_url, port: channel.port2}, [channel.port2]);
  });

  let sw_result;
  try {
    sw_result = await sw_message_promise;
  } catch (err) {
    assert_unreached(err.message);
  }

  assert_equals(sw_result.result, expected_result, `SW navigate should report ${expected_result}. Logs:\n${sw_result.logs ? sw_result.logs.join("\n") : 'none'}`);

  // Wait a bit to see if 'loaded' message arrives.
  await new Promise(resolve => step_timeout(resolve, 100));

  if (expected_load) {
    assert_true(received_load_message, 'Target page should have loaded');
  } else {
    assert_false(received_load_message, 'Target page should not have loaded');
  }
}

// Test 1: ServiceWorkerWindowClientNavigateEnforcesServiceWorkerAndDocumentAllowlists
// SW has empty allowlist, Document has no allowlist.
// Navigation to same-origin should be blocked by SW allowlist.
promise_test(async t => {
  const sw_script = 'resources/sw-navigate-empty.js';
  const iframe_url = 'resources/blank.html';
  const target_url = `${HTTPS_ORIGIN}/connection-allowlist/tentative/resources/post-message.html`;
  await run_navigation_test(t, sw_script, iframe_url, target_url, 'failure', false);
}, "clients.navigate() is blocked by Service Worker's empty Connection-Allowlist");

// Test 2: ServiceWorkerWindowClientNavigateObeysDocumentAllowlist
// SW allows same-origin and cross-origin.
// Document allows only same-origin.
// Navigation to cross-origin should be blocked by Document allowlist.
promise_test(async t => {
  const sw_script = 'resources/sw-navigate-allow-both.js';
  const iframe_url = 'resources/blank-allow-same-origin.html';
  const target_url = `${HTTPS_NOTSAMESITE_ORIGIN}/connection-allowlist/tentative/resources/post-message.html`;
  await run_navigation_test(t, sw_script, iframe_url, target_url, 'failure', false);
}, "clients.navigate() is blocked by Document's Connection-Allowlist");
