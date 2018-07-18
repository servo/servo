// META: script=/service-workers/service-worker/resources/test-helpers.sub.js
// META: script=resources/utils.js
'use strict';

// Covers functionality provided by BackgroundFetchManager.getIds(), which
// exposes the keys of active background fetches.
//
// https://wicg.github.io/background-fetch/#background-fetch-manager-getIds

backgroundFetchTest(async (test, backgroundFetch) => {
  const registrationId = uniqueId();
  const registration = await backgroundFetch.fetch(
      registrationId, 'resources/feature-name.txt');

  assert_equals(registration.id, registrationId);

  assert_true((await backgroundFetch.getIds()).includes(registrationId));

}, 'The BackgroundFetchManager exposes active fetches');
