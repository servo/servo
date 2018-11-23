// META: script=/service-workers/service-worker/resources/test-helpers.sub.js
// META: script=resources/utils.js
'use strict';

// Covers basic functionality provided by BackgroundFetchManager.fetch().
// Specifically, when `fetch` contains request uploads.
// https://wicg.github.io/background-fetch/#background-fetch-manager-fetch

backgroundFetchTest(async (test, backgroundFetch) => {
  const uploadData = 'Background Fetch!';
  const request =
    new Request('resources/upload.py', {method: 'POST', body: uploadData});

  await backgroundFetch.fetch(uniqueId(), request);
  const {type, eventRegistration, results} = await getMessageFromServiceWorker();

  assert_equals(type, 'backgroundfetchsuccess');
  assert_equals(results.length, 1);
  assert_equals(eventRegistration.result, 'success');
  assert_equals(eventRegistration.failureReason, '');
  assert_equals(results[0].text, uploadData);

}, 'Fetch with an upload should work');