// META: script=/service-workers/service-worker/resources/test-helpers.sub.js
// META: script=resources/utils.js
'use strict';

// Covers functionality provided by BackgroundFetchUpdateUIEvent.updateUI().
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
  assert_equals(message.update, 'InvalidStateError');

}, 'Background Fetch updateUI called twice fails', swName);

backgroundFetchTest(async (test, backgroundFetch) => {
  const registrationId = 'update-inactive';
  const registration =
      await backgroundFetch.fetch(registrationId, 'resources/feature-name.txt');
  assert_equals(registration.id, registrationId);

  const message = await getMessageFromServiceWorker();
  assert_equals(message.update, 'InvalidStateError');

}, 'Background Fetch updateUI fails when event is not active', swName);
