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

  await new Promise(resolve => {
    let aborted = false;
    const expectedResultText = 'Background Fetch';

    registration.onprogress = async event => {
      if (event.target.downloaded < expectedResultText.length)
        return;

      if (aborted)
        return;

      // Abort after the first file has been downloaded and check the results.

      aborted = true;
      assert_true(await registration.abort());

      const {type, eventRegistration, results} = await getMessageFromServiceWorker();

      assert_equals(eventRegistration.result, 'failure');
      assert_equals(eventRegistration.failureReason, 'aborted');
      assert_equals(registration.result, 'failure');
      assert_equals(registration.failureReason, 'aborted');

      assert_equals(type, 'backgroundfetchabort');

      // The abort might have gone through before the first result was persisted.
      if (results.length === 1) {
        assert_true(results[0].url.includes('resources/feature-name.txt'));
        assert_equals(results[0].status, 200);
        assert_equals(results[0].text, expectedResultText);
      }

      resolve();
    };
  });

}, 'Calling BackgroundFetchRegistration.abort sets the correct fields and responses are still available');