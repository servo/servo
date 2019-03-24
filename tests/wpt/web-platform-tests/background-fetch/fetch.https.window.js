// META: script=/common/get-host-info.sub.js
// META: script=/service-workers/service-worker/resources/test-helpers.sub.js
// META: script=resources/utils.js

'use strict';

// Covers basic functionality provided by BackgroundFetchManager.fetch().
// https://wicg.github.io/background-fetch/#background-fetch-manager-fetch

const wait = milliseconds =>
  new Promise(resolve => step_timeout(resolve, milliseconds));

promise_test(async test => {
  // 6.3.1.9.2: If |registration|’s active worker is null, then reject promise
  //            with a TypeError and abort these steps.
  const script = 'service_workers/sw.js';
  const scope = 'service_workers/' + location.pathname;

  const serviceWorkerRegistration =
    await service_worker_unregister_and_register(test, script, scope);

  assert_equals(
    serviceWorkerRegistration.active, null,
    'There must not be an activated worker');

  await promise_rejects(
    test, new TypeError(),
    serviceWorkerRegistration.backgroundFetch.fetch(
      uniqueId(), ['resources/feature-name.txt']),
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
    const request =
      new Request('resources/feature-name.txt', {mode: 'no-cors'});

    await promise_rejects(
      test, new TypeError(), backgroundFetch.fetch(uniqueId(), request),
      'Requests must not be in no-cors mode');
  }

}, 'Argument verification is done for BackgroundFetchManager.fetch()');

backgroundFetchTest(async (test, backgroundFetch) => {
  // 6.3.1.9.2: If |bgFetchMap[id]| exists, reject |promise| with a TypeError
  //            and abort these steps.
  return promise_rejects(test, new TypeError(), Promise.all([
    backgroundFetch.fetch('my-id', 'resources/feature-name.txt?1'),
    backgroundFetch.fetch('my-id', 'resources/feature-name.txt?2')
  ]));

}, 'IDs must be unique among active Background Fetch registrations');

backgroundFetchTest(async (test, backgroundFetch) => {
  const registrationId = uniqueId();
  const registration =
    await backgroundFetch.fetch(registrationId, '');

  assert_equals(registration.id, registrationId);

  const {type, eventRegistration, results} = await getMessageFromServiceWorker();

  assert_equals('backgroundfetchsuccess', type);
  assert_equals(eventRegistration.result, 'success');
  assert_equals(eventRegistration.failureReason, '');

}, 'Empty URL is OK.');

backgroundFetchTest(async (test, backgroundFetch) => {
  const registrationId = uniqueId();
  const registration =
    await backgroundFetch.fetch(registrationId,
        new Request('https://example/com', {
        method: 'PUT',
      }));

  assert_equals(registration.id, registrationId);

  const {type, eventRegistration, results} = await getMessageFromServiceWorker();

  assert_equals(type, 'backgroundfetchsuccess');
  assert_equals(eventRegistration.result, 'success');
  assert_equals(eventRegistration.failureReason, '');
}, 'Requests with PUT method require CORS Preflight and succeed.');

backgroundFetchTest(async (test, backgroundFetch) => {
  const registrationId = uniqueId();
  const registration =
    await backgroundFetch.fetch(registrationId,
        new Request('https://example/com', {
        method: 'POST',
        headers: {'Content-Type': 'text/json'}
      }));

  assert_equals(registration.id, registrationId);

  const {type, eventRegistration, results} = await getMessageFromServiceWorker();

  assert_equals(type, 'backgroundfetchsuccess');
  assert_equals(eventRegistration.result, 'success');
  assert_equals(eventRegistration.failureReason, '');
}, 'Requests with text/json content type require CORS Preflight and succeed.');

backgroundFetchTest(async (test, backgroundFetch) => {
  const registrationId = uniqueId();
  const registration =
    await backgroundFetch.fetch(registrationId, 'resources/feature-name.txt');

  assert_equals(registration.id, registrationId);
  assert_equals(registration.uploadTotal, 0);
  assert_equals(registration.uploaded, 0);
  assert_equals(registration.downloadTotal, 0);
  assert_equals(registration.result, '');
  assert_equals(registration.failureReason, '');
  assert_true(registration.recordsAvailable);
  // Skip `downloaded`, as the transfer may have started already.

  const {type, eventRegistration, results} = await getMessageFromServiceWorker();
  assert_equals('backgroundfetchsuccess', type);
  assert_equals(results.length, 1);

  assert_equals(eventRegistration.id, registration.id);
  assert_equals(eventRegistration.result, 'success');
  assert_equals(eventRegistration.failureReason, '');

  assert_true(results[0].url.includes('resources/feature-name.txt'));
  assert_equals(results[0].status, 200);
  assert_equals(results[0].text, 'Background Fetch');

}, 'Using Background Fetch to successfully fetch a single resource');

backgroundFetchTest(async (test, backgroundFetch) => {
  const registrationId = uniqueId();
  const registration =
    await backgroundFetch.fetch(registrationId, 'resources/feature-name.txt');

  assert_equals(registration.result, '');
  assert_equals(registration.failureReason, '');

  const {type, eventRegistration, results} =
    await getMessageFromServiceWorker();
  assert_equals('backgroundfetchsuccess', type);

  assert_equals(eventRegistration.id, registration.id);
  assert_equals(registration.result, 'success');
  assert_equals(registration.failureReason, '');

}, 'Registration object gets updated values when a background fetch completes.');

backgroundFetchTest(async (test, backgroundFetch) => {
  const registrationId = uniqueId();

  // Very large download total that will definitely exceed the quota.
  const options = {downloadTotal: Number.MAX_SAFE_INTEGER};
  await promise_rejects(
    test, 'QUOTA_EXCEEDED_ERR',
    backgroundFetch.fetch(registrationId, 'resources/feature-name.txt', options),
    'This fetch should have thrown a quota exceeded error');

}, 'Background Fetch that exceeds the quota throws a QuotaExceededError');

backgroundFetchTest(async (test, backgroundFetch) => {
  const registration = await backgroundFetch.fetch(
    'my-id', ['resources/feature-name.txt', 'resources/feature-name.txt']);

  const {type, eventRegistration, results} = await getMessageFromServiceWorker();
  assert_equals('backgroundfetchsuccess', type);
  assert_equals(results.length, 2);

  assert_equals(eventRegistration.id, registration.id);
  assert_equals(eventRegistration.result, 'success');
  assert_equals(eventRegistration.failureReason, '');

  for (const result of results) {
    assert_true(result.url.includes('resources/feature-name.txt'));
    assert_equals(result.status, 200);
    assert_equals(result.text, 'Background Fetch');
  }

}, 'Fetches can have requests with duplicate URLs');

backgroundFetchTest(async (test, backgroundFetch) => {
  const registrationId = uniqueId();
  const registration =
    await backgroundFetch.fetch(registrationId, 'resources/feature-name.txt');
  assert_true(registration.recordsAvailable);

  const {type, eventRegistration, results} = await getMessageFromServiceWorker();
  assert_equals('backgroundfetchsuccess', type);
  assert_equals(results.length, 1);

  // Wait for up to 5 seconds for the |eventRegistration|'s recordsAvailable
  // flag to be set to false, which happens after the successful invocation
  // of the ServiceWorker event has finished.
  for (let i = 0; i < 50; ++i) {
    if (!registration.recordsAvailable)
      break;
    await wait(100);
  }

  assert_false(registration.recordsAvailable);
}, 'recordsAvailable is false after onbackgroundfetchsuccess finishes execution.');

backgroundFetchTest(async (test, backgroundFetch) => {
  const registrationId = uniqueId();
  const registration =
    await backgroundFetch.fetch(registrationId, 'resources/missing-cat.txt');

  assert_equals(registration.id, registrationId);
  assert_equals(registration.result, '');
  assert_equals(registration.failureReason, '');

  const {type, eventRegistration, results} = await getMessageFromServiceWorker();
  assert_equals(type, 'backgroundfetchfail');
  assert_equals(results.length, 1);
  assert_true(results[0].url.includes('resources/missing-cat.txt'));
  assert_equals(results[0].status, 404);
  assert_equals(results[0].text, '');

  assert_equals(eventRegistration.id, registration.id);
  assert_equals(eventRegistration.result, 'failure');
  assert_equals(eventRegistration.failureReason, 'bad-status');

  assert_equals(registration.result, 'failure');
  assert_equals(registration.failureReason, 'bad-status');

}, 'Using Background Fetch to fetch a non-existent resource should fail.');

backgroundFetchTest(async (test, backgroundFetch) => {
  const registration = await backgroundFetch.fetch(
      'my-id',
      [location.origin, location.origin.replace('https', 'http')]);

  const {type, eventRegistration, results} = await getMessageFromServiceWorker();

  assert_equals('backgroundfetchfail', type);
  assert_equals(eventRegistration.failureReason, 'fetch-error');

  assert_equals(results.length, 2);

  const validResponse = results[0] ? results[0] : results[1];
  const nullResponse = !results[0] ? results[0] : results[1];

  assert_true(validResponse.url.includes(location.origin));
  assert_equals(nullResponse, null);

}, 'Fetches with mixed content should fail.');

backgroundFetchTest(async (test, backgroundFetch) => {
  const filePath = '/background-fetch/resources/feature-name.txt';
  const registration = await backgroundFetch.fetch(
    uniqueId(),
    `https://${get_host_info().REMOTE_HOST}${filePath}`);

  const {type, eventRegistration, results} = await getMessageFromServiceWorker();
  assert_equals(type, 'backgroundfetchfail');
  assert_equals(results.length, 1);

  assert_equals(results[0], null);
  assert_equals(eventRegistration.id, registration.id);
  assert_equals(eventRegistration.downloaded, 0);
}, 'Responses failing CORS checks are not leaked');
