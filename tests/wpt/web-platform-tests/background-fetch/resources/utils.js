'use strict';

let nextBackgroundFetchId = 0;

// Waits for a single message received from a registered Service Worker.
async function getMessageFromServiceWorker() {
  return new Promise(resolve => {
    function listener(event) {
      navigator.serviceWorker.removeEventListener('message', listener);
      resolve(event.data);
    }

    navigator.serviceWorker.addEventListener('message', listener);
  });
}

// Registers the instrumentation Service Worker located at "resources/sw.js"
// with a scope unique to the test page that's running, and waits for it to be
// activated. The Service Worker will be unregistered automatically.
//
// Depends on /service-workers/service-worker/resources/test-helpers.sub.js
async function registerAndActivateServiceWorker(test) {
  const script = 'resources/sw.js';
  const scope = 'resources/scope' + location.pathname;

  let serviceWorkerRegistration =
      await service_worker_unregister_and_register(test, script, scope);

  add_completion_callback(() => serviceWorkerRegistration.unregister());

  await wait_for_state(test, serviceWorkerRegistration.installing, 'activated');
  return serviceWorkerRegistration;
}

// Creates a Promise test for |func| given the |description|. The |func| will be
// executed with the `backgroundFetch` object of an activated Service Worker
// Registration.
function backgroundFetchTest(func, description) {
  promise_test(async t => {
    const serviceWorkerRegistration = await registerAndActivateServiceWorker(t);
    serviceWorkerRegistration.active.postMessage(null /* unused */);

    assert_equals(await getMessageFromServiceWorker(), 'ready');

    return func(t, serviceWorkerRegistration.backgroundFetch);
  }, description);
}

// Returns a Background Fetch ID that's unique for the current page.
function uniqueId() {
  return 'id' + nextBackgroundFetchId++;
}
