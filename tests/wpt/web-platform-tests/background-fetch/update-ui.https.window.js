// META: script=/service-workers/service-worker/resources/test-helpers.sub.js
// META: script=resources/utils.js
'use strict';

// Covers functionality provided by BackgroundFetchUpdateEvent.updateUI().
//
// https://wicg.github.io/background-fetch/#backgroundfetchupdateuievent

const swName = 'sw-update-ui.js';

backgroundFetchTest(async (test, backgroundFetch) => {
  const registrationId = 'update-once';
  const registration =
      await backgroundFetch.fetch(registrationId, 'resources/feature-name.txt');
  assert_equals(registration.id, registrationId);

  const message = await getMessageFromServiceWorker();
  assert_equals(message.update, 'update success');

}, 'Background Fetch updateUI resolves', swName);


backgroundFetchTest(async (test, backgroundFetch) => {
  const registrationId = 'update-twice';
  const registration =
      await backgroundFetch.fetch(registrationId, 'resources/feature-name.txt');
  assert_equals(registration.id, registrationId);

  const message = await getMessageFromServiceWorker();
  assert_equals(message.update, 'updateUI may only be called once.');

}, 'Background Fetch updateUI called twice fails', swName);
