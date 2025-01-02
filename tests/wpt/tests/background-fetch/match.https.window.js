// META: script=/common/get-host-info.sub.js
// META: script=/service-workers/service-worker/resources/test-helpers.sub.js
// META: script=resources/utils.js

'use strict';

// Covers basic functionality provided by BackgroundFetchRegistration.match(All)?.
// https://wicg.github.io/background-fetch/#dom-backgroundfetchregistration-match

backgroundFetchTest(async (test, backgroundFetch) => {
  const registrationId = 'matchexistingrequest';
  const registration =
      await backgroundFetch.fetch(registrationId, 'resources/feature-name.txt');

  assert_equals(registration.id, registrationId);

  const {type, eventRegistration, results} = await getMessageFromServiceWorker();
  assert_equals('backgroundfetchsuccess', type);
  assert_equals(results.length, 1);

  assert_equals(eventRegistration.id, registration.id);
  assert_equals(eventRegistration.result, 'success');
  assert_equals(eventRegistration.failureReason, '');

  assert_true(results[0].url.includes('resources/feature-name.txt'));
  assert_equals(results[0].status, 200);
  assert_equals(results[0].text, 'Background Fetch');

}, 'Matching to a single request should work');

backgroundFetchTest(async (test, backgroundFetch) => {
  const registrationId = 'matchmissingrequest';
  const registration =
      await backgroundFetch.fetch(registrationId, 'resources/feature-name.txt');

  assert_equals(registration.id, registrationId);

  const {type, eventRegistration, results} = await getMessageFromServiceWorker();
  assert_equals('backgroundfetchsuccess', type);
  assert_equals(results.length, 0);

  assert_equals(eventRegistration.id, registration.id);
  assert_equals(eventRegistration.result, 'success');
  assert_equals(eventRegistration.failureReason, '');

}, 'Matching to a non-existing request should work');

backgroundFetchTest(async (test, backgroundFetch) => {
  const registrationId = 'matchexistingrequesttwice';
  const registration =
      await backgroundFetch.fetch(registrationId, 'resources/feature-name.txt');

  assert_equals(registration.id, registrationId);

  const {type, eventRegistration, results} = await getMessageFromServiceWorker();
  assert_equals('backgroundfetchsuccess', type);
  assert_equals(results.length, 2);

  assert_equals(eventRegistration.id, registration.id);
  assert_equals(eventRegistration.result, 'success');
  assert_equals(eventRegistration.failureReason, '');

  assert_true(results[0].url.includes('resources/feature-name.txt'));
  assert_equals(results[0].status, 200);
  assert_equals(results[0].text, 'Background Fetch');

  assert_true(results[1].url.includes('resources/feature-name.txt'));
  assert_equals(results[1].status, 200);
  assert_equals(results[1].text, 'Background Fetch');

}, 'Matching multiple times on the same request works as expected.');

backgroundFetchTest(async (test, backgroundFetch) => {
  const registration = await backgroundFetch.fetch(
      uniqueId(), ['resources/feature-name.txt', '/common/slow.py']);

  const record = await registration.match('resources/feature-name.txt');
  const response = await record.responseReady;
  assert_true(response.url.includes('resources/feature-name.txt'));
  const completedResponseText = await response.text();
  assert_equals(completedResponseText, 'Background Fetch');

}, 'Access to active fetches is supported.');

backgroundFetchTest(async (test, backgroundFetch) => {
  const registration = await backgroundFetch.fetch(
      uniqueId(), [
          'resources/feature-name.txt',
          'resources/feature-name.txt',
          'resources/feature-name.txt?id=3',
          new Request('resources/feature-name.txt', {method: 'PUT'}),
          '/common/slow.py',
      ]);

  let matchedRecords = null;

  // We should match all the duplicates.
  matchedRecords = await registration.matchAll('resources/feature-name.txt');
  assert_equals(matchedRecords.length, 2);

  // We should match the request with the query param as well.
  matchedRecords = await registration.matchAll('resources/feature-name.txt', {ignoreSearch: true});
  assert_equals(matchedRecords.length, 3);

  // We should match the PUT request as well.
  matchedRecords = await registration.matchAll('resources/feature-name.txt', {ignoreMethod: true});
  assert_equals(matchedRecords.length, 3);

  // We should match all requests.
  matchedRecords = await registration.matchAll('resources/feature-name.txt', {ignoreSearch: true, ignoreMethod: true});
  assert_equals(matchedRecords.length, 4);

}, 'Match with query options.');
