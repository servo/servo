'use strict';

// Depends on /service-workers/service-worker/resources/test-helpers.sub.js
async function registerAndActivateServiceWorker(test) {
  const script = 'resources/sw.js';
  const scope = 'resources/scope' + location.pathname;
  let serviceWorkerRegistration =
      await service_worker_unregister_and_register(test, script, scope);
  add_completion_callback(() => {
    serviceWorkerRegistration.unregister();
  });
  await wait_for_state(test, serviceWorkerRegistration.installing, 'activated');
  return serviceWorkerRegistration;
}

function backgroundFetchTest(func, description) {
  promise_test(async t => {
    const serviceWorkerRegistration = await registerAndActivateServiceWorker(t);
    return func(t, serviceWorkerRegistration.backgroundFetch);
  }, description);
}

let _nextBackgroundFetchTag = 0;
function uniqueTag() {
  return 'tag' + _nextBackgroundFetchTag++;
}