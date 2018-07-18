// META: script=/service-workers/service-worker/resources/test-helpers.sub.js
// META: script=resources/utils.js
'use strict';

// Covers basic functionality provided by BackgroundFetchManager.fetch().
// https://wicg.github.io/background-fetch/#background-fetch-manager-fetch

backgroundFetchTest(async (test, backgroundFetch) => {
  const registrationId = uniqueId();
  const registration = await backgroundFetch.fetch(
      registrationId, 'resources/feature-name.txt');

  assert_equals(registration.id, registrationId);
  assert_equals(registration.uploadTotal, 0);
  assert_equals(registration.uploaded, 0);
  assert_equals(registration.downloadTotal, 0);
  // Skip `downloaded`, as the transfer may have started already.

  const {type, results} = await getMessageFromServiceWorker();
  assert_equals('backgroundfetched', type);
  assert_equals(results.length, 1);

  assert_true(results[0].url.includes('resources/feature-name.txt'));
  assert_equals(results[0].status, 200);
  assert_equals(results[0].text, 'Background Fetch');

}, 'Using Background Fetch to successfully fetch a single resource');
