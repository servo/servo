// META: script=/service-workers/service-worker/resources/test-helpers.sub.js
// META: script=resources/utils.js
'use strict';

// Covers functionality provided by BackgroundFetchManager.get(), which
// exposes the keys of active background fetches.
//
// https://wicg.github.io/background-fetch/#background-fetch-manager-get

promise_test(async test => {
  const script = 'service_workers/sw.js';
  const scope = 'service_workers/' + location.pathname;

  const serviceWorkerRegistration =
      await service_worker_unregister_and_register(test, script, scope);

  assert_equals(
      serviceWorkerRegistration.active, null,
      'There must not be an activated worker');

  const registration = await serviceWorkerRegistration.backgroundFetch.get('x');
  assert_equals(registration, undefined);

}, 'BackgroundFetchManager.get() does not require an activated worker');

backgroundFetchTest(async (test, backgroundFetch) => {
  // The |id| parameter to the BackgroundFetchManager.get() method is required.
  await promise_rejects(test, new TypeError(), backgroundFetch.get());
  await promise_rejects(test, new TypeError(), backgroundFetch.get(''));

  const registration = await backgroundFetch.get('my-id');
  assert_equals(registration, undefined);

}, 'Getting non-existing registrations yields `undefined`');

backgroundFetchTest(async (test, backgroundFetch) => {
  const registrationId = uniqueId();
  const registration = await backgroundFetch.fetch(
      registrationId, 'resources/feature-name.txt', {downloadTotal: 1234});

  assert_equals(registration.id, registrationId);
  assert_equals(registration.uploadTotal, 0);
  assert_equals(registration.uploaded, 0);
  assert_equals(registration.downloadTotal, 1234);
  assert_equals(registration.result, '');
  assert_equals(registration.failureReason, '');
  assert_true(registration.recordsAvailable);
  // Skip `downloaded`, as the transfer may have started already.

  const secondRegistration = await backgroundFetch.get(registrationId);
  assert_not_equals(secondRegistration, null);

  assert_equals(secondRegistration.id, registration.id);
  assert_equals(secondRegistration.uploadTotal, registration.uploadTotal);
  assert_equals(secondRegistration.uploaded, registration.uploaded);
  assert_equals(secondRegistration.downloadTotal, registration.downloadTotal);
  assert_equals(secondRegistration.failureReason, registration.failureReason);
  assert_equals(secondRegistration.recordsAvailable, registration.recordsAvailable);

  // While the transfer might have started, both BackgroundFetchRegistration
  // objects should have the latest progress values.
  assert_equals(secondRegistration.downloaded, registration.downloaded);

}, 'Getting an existing registration has the expected values');
