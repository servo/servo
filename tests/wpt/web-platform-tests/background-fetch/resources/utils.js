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

// Registers the |name| instrumentation Service Worker located at "service_workers/"
// with a scope unique to the test page that's running, and waits for it to be
// activated. The Service Worker will be unregistered automatically.
//
// Depends on /service-workers/service-worker/resources/test-helpers.sub.js
async function registerAndActivateServiceWorker(test, name) {
  const script = `service_workers/${name}`;
  const scope = 'service_workers/scope' + location.pathname;

  let serviceWorkerRegistration =
      await service_worker_unregister_and_register(test, script, scope);

  add_completion_callback(() => serviceWorkerRegistration.unregister());

  await wait_for_state(test, serviceWorkerRegistration.installing, 'activated');
  return serviceWorkerRegistration;
}

// Creates a Promise test for |func| given the |description|. The |func| will be
// executed with the `backgroundFetch` object of an activated Service Worker
// Registration.
// |workerName| is the name of the service worker file in the service_workers
// directory to register.
function backgroundFetchTest(func, description, workerName = 'sw.js') {
  promise_test(async t => {
    const serviceWorkerRegistration =
        await registerAndActivateServiceWorker(t, workerName);
    serviceWorkerRegistration.active.postMessage(null);

    assert_equals(await getMessageFromServiceWorker(), 'ready');

    return func(t, serviceWorkerRegistration.backgroundFetch);
  }, description);
}

// Returns a Background Fetch ID that's unique for the current page.
function uniqueId() {
  return 'id' + nextBackgroundFetchId++;
}
