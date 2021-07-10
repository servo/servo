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

  const registration = await backgroundFetch.fetch(uniqueId(), request);
  assert_equals(registration.uploadTotal, uploadData.length);

  const {type, eventRegistration, results} = await getMessageFromServiceWorker();
  assert_equals(type, 'backgroundfetchsuccess');
  assert_equals(results.length, 1);
  assert_equals(eventRegistration.result, 'success');
  assert_equals(eventRegistration.failureReason, '');
  assert_equals(eventRegistration.uploaded, uploadData.length);
  assert_equals(results[0].text, uploadData);
}, 'Fetch with an upload should work');

backgroundFetchTest(async (test, backgroundFetch) => {
  const uploadData = 'Background Fetch!';
  const uploadRequest =
      new Request('resources/upload.py', {method: 'POST', body: uploadData});

  const registration = await backgroundFetch.fetch(
      uniqueId(),
      [uploadRequest, '/common/slow.py']);

    const uploaded = await new Promise(resolve => {
      registration.onprogress = event => {
        if (event.target.downloaded === 0)
          return;
        // If a progress event with downloaded bytes was received, then
        // everything was uploaded.
        resolve(event.target.uploaded);
      };
    });

  assert_equals(uploaded, uploadData.length);
}, 'Progress event includes uploaded bytes');

backgroundFetchTest(async (test, backgroundFetch) => {
  const uploadRequest1 =
      new Request('resources/upload.py', {method: 'POST', body: 'upload1'});
  const uploadRequest2 =
      new Request('resources/upload.py', {method: 'POST', body: 'upload2'});

  await backgroundFetch.fetch(uniqueId(), [uploadRequest1, uploadRequest2]);

  const {type, eventRegistration, results} = await getMessageFromServiceWorker();
  assert_equals(type, 'backgroundfetchsuccess');
  assert_equals(results.length, 2);
  assert_equals(eventRegistration.result, 'success');
  assert_equals(eventRegistration.failureReason, '');

  assert_array_equals([results[0].text, results[1].text].sort(),
                      ['upload1', 'upload2']);
}, 'Duplicate upload requests work and can be distinguished.');
