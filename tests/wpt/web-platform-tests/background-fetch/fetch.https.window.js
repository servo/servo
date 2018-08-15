// META: script=/service-workers/service-worker/resources/test-helpers.sub.js
// META: script=resources/utils.js
'use strict';

// Covers basic functionality provided by BackgroundFetchManager.fetch().
// https://wicg.github.io/background-fetch/#background-fetch-manager-fetch

promise_test(async test => {
  // 6.3.1.9.2: If |registration|’s active worker is null, then reject promise
  //            with a TypeError and abort these steps.
  const script = 'resources/sw.js';
  const scope = 'resources/scope' + location.pathname;

  const serviceWorkerRegistration =
      await service_worker_unregister_and_register(test, script, scope);

  assert_equals(
      serviceWorkerRegistration.active, null,
      'There must not be an activated worker');

  await promise_rejects(
      test, new TypeError(),
      serviceWorkerRegistration.backgroundFetch.fetch(
          uniqueId(), ['resources/sw.js']),
      'fetch() must reject on pending and installing workers');

}, 'Background Fetch requires an activated Service Worker');

backgroundFetchTest(async (test, backgroundFetch) => {
  // 6.3.1.6: If |requests| is empty, then return a promise rejected with a
  //          TypeError.
  await promise_rejects(
      test, new TypeError(), backgroundFetch.fetch(uniqueId(), []),
      'Empty sequences are treated as NULL');

  // 6.3.1.7.1: Let |internalRequest| be the request of the result of invoking
  //            the Request constructor with |request|. If this throws an
  //            exception, return a promise rejected with the exception.
  await promise_rejects(
      test, new TypeError(),
      backgroundFetch.fetch(uniqueId(), 'https://user:pass@domain/secret.txt'),
      'Exceptions thrown in the Request constructor are rethrown');

  // 6.3.1.7.2: If |internalRequest|’s mode is "no-cors", then return a
  //            promise rejected with a TypeError.
  {
    const request = new Request('resources/sw.js', {mode: 'no-cors'});
    await promise_rejects(
        test, new TypeError(), backgroundFetch.fetch(uniqueId(), request),
        'Requests must not be in no-cors mode');
  }

}, 'Argument verification is done for BackgroundFetchManager.fetch()');

backgroundFetchTest(async (test, backgroundFetch) => {
  // 6.3.1.9.2: If |bgFetchMap[id]| exists, reject |promise| with a TypeError
  //            and abort these steps.
  return promise_rejects(test, new TypeError(), Promise.all([
    backgroundFetch.fetch('my-id', 'resources/sw.js'),
    backgroundFetch.fetch('my-id', 'resources/feature-name.txt')
  ]));

}, 'IDs must be unique among active Background Fetch registrations');

backgroundFetchTest(async (test, backgroundFetch) => {
  const registrationId = uniqueId();
  const registration =
      await backgroundFetch.fetch(registrationId, 'resources/feature-name.txt');

  assert_equals(registration.id, registrationId);
  assert_equals(registration.uploadTotal, 0);
  assert_equals(registration.uploaded, 0);
  assert_equals(registration.downloadTotal, 0);
  // Skip `downloaded`, as the transfer may have started already.

  const {type, results} = await getMessageFromServiceWorker();
  assert_equals('backgroundfetchsuccess', type);
  assert_equals(results.length, 1);

  assert_true(results[0].url.includes('resources/feature-name.txt'));
  assert_equals(results[0].status, 200);
  assert_equals(results[0].text, 'Background Fetch');

}, 'Using Background Fetch to successfully fetch a single resource');
