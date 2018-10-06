// META: script=/service-workers/service-worker/resources/test-helpers.sub.js
// META: script=resources/utils.js
'use strict';

// Covers basic functionality provided by BackgroundFetchManager.abort().
// https://wicg.github.io/background-fetch/#background-fetch-registration-abort

backgroundFetchTest(async (test, backgroundFetch) => {
  const registration = await backgroundFetch.fetch(
      uniqueId(),
      ['resources/feature-name.txt', '/serviceworker/resources/slow-response.php']);

  assert_true(await registration.abort());
  assert_false(await registration.abort());

}, 'Aborting the same registration twice fails');

backgroundFetchTest(async (test, backgroundFetch) => {
  const registration = await backgroundFetch.fetch(
      uniqueId(),
      ['resources/feature-name.txt', '/serviceworker/resources/slow-response.php']);
  const resultPromise = getMessageFromServiceWorker();

  await new Promise(resolve => {
    registration.onprogress = async (e) => {
      // The size of the first file.
      if (e.target.downloaded < 16)
        return;

      // At this point the first file is downloaded.

      assert_true(await registration.abort());

      const {type, eventRegistration, results} = await resultPromise;

      assert_equals(eventRegistration.result, 'failure');
      assert_equals(eventRegistration.failureReason, 'aborted');

      assert_equals(type, 'backgroundfetchabort');
      assert_equals(results.length, 1);

      assert_true(results[0].url.includes('resources/feature-name.txt'));
      assert_equals(results[0].status, 200);
      assert_equals(results[0].text, 'Background Fetch');

      resolve();
    };
  });

}, 'Calling BackgroundFetchRegistration.abort sets the correct fields and responses are still available');