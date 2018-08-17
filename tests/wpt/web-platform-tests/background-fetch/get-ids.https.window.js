// META: script=/service-workers/service-worker/resources/test-helpers.sub.js
// META: script=resources/utils.js
'use strict';

// Covers functionality provided by BackgroundFetchManager.getIds(), which
// exposes the keys of active background fetches.
//
// https://wicg.github.io/background-fetch/#background-fetch-manager-getIds

promise_test(async test => {
  const script = 'service_workers/sw.js';
  const scope = 'service_workers/' + location.pathname;

  const serviceWorkerRegistration =
      await service_worker_unregister_and_register(test, script, scope);

  assert_equals(
      serviceWorkerRegistration.active, null,
      'There must not be an activated worker');

  const ids = await serviceWorkerRegistration.backgroundFetch.getIds();
  assert_equals(ids.length, 0);

}, 'BackgroundFetchManager.getIds() does not require an activated worker');

backgroundFetchTest(async (test, backgroundFetch) => {
  // There should not be any active background fetches at this point.
  {
    const ids = await backgroundFetch.getIds();
    assert_equals(ids.length, 0);
  }

  const registrationId = uniqueId();
  const registration =
      await backgroundFetch.fetch(registrationId, 'resources/feature-name.txt');
  assert_equals(registration.id, registrationId);

  // The |registrationId| should be active, and thus be included in getIds().
  {
    const ids = await backgroundFetch.getIds();
    assert_equals(ids.length, 1);
    assert_equals(ids[0], registrationId);
  }

}, 'The BackgroundFetchManager exposes active fetches');
