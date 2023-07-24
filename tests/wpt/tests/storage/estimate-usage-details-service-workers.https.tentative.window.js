// META: title=StorageManager: estimate() for service worker registrations
const wait_for_active = worker => new Promise(resolve =>{
  if (worker.active) { resolve(worker.active); }

  const listen_for_active = worker => e => {
    if (e.target.state === 'activated') { resolve(worker.active); }
  }

  if (worker.waiting) {
    worker.waiting
        .addEventListener('statechange', listen_for_active(worker.waiting));
  }
  if (worker.installing) {
    worker.installing
        .addEventListener('statechange', listen_for_active(worker.installing));
  }
});

promise_test(async t => {
  let estimate = await navigator.storage.estimate();
  const usageBeforeCreate = estimate.usageDetails.serviceWorkerRegistrations ||
      0;
  // Note: helpers.js is an arbitrary file; it could be any file that
  // exists, but this test does not depend on the contents of said file.
  const serviceWorkerRegistration = await
      navigator.serviceWorker.register('./helpers.js');

  t.add_cleanup(() => serviceWorkerRegistration.unregister());
  await wait_for_active(serviceWorkerRegistration);

  estimate = await navigator.storage.estimate();
  assert_true('serviceWorkerRegistrations' in estimate.usageDetails);

  const usageAfterCreate = estimate.usageDetails.serviceWorkerRegistrations;
  assert_greater_than(
      usageAfterCreate, usageBeforeCreate,
      'estimated usage should increase after service worker is registered');
}, 'estimate() shows usage increase after large value is stored');
