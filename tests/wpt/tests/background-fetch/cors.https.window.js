// META: script=/common/get-host-info.sub.js
// META: script=/service-workers/service-worker/resources/test-helpers.sub.js
// META: script=resources/utils.js

'use strict';

// Verifies Background Fetch enforcement of standard CORS security rules,
// including preflights, exposed headers, response types, and credentials mode.

const {HTTPS_REMOTE_ORIGIN} = get_host_info();
const kCorsResource =
    HTTPS_REMOTE_ORIGIN + '/background-fetch/resources/cors-resource.txt';
const kNoCorsResource =
    HTTPS_REMOTE_ORIGIN + '/background-fetch/resources/feature-name.txt';

backgroundFetchTest(
    async (test, bgFetch) => {
      await bgFetch.fetch(uniqueId(), kCorsResource);
      const event = await getMessageFromServiceWorker();

      assert_equals(event.type, 'backgroundfetchsuccess');
      assert_equals(event.results.length, 1);
      assert_equals(event.results[0].type, 'cors');
      assert_equals(event.results[0].secretHeader, null);
    },
    'Cross-origin response type is cors and non-CORS-exposed headers are filtered',
    'sw-cors.js');

backgroundFetchTest(async (test, bgFetch) => {
  await bgFetch.fetch(uniqueId(), kNoCorsResource);
  const event = await getMessageFromServiceWorker();

  assert_equals(event.type, 'backgroundfetchfail');
}, 'Cross-origin Background Fetch fails without CORS headers', 'sw-cors.js');

backgroundFetchTest(
    async (test, bgFetch) => {
      const request = new Request(
          kCorsResource, {headers: {'X-Custom-Header': 'SecretValue'}});
      await bgFetch.fetch(uniqueId(), request);
      const event = await getMessageFromServiceWorker();

      assert_equals(event.type, 'backgroundfetchfail');
    },
    'Cross-origin Background Fetch with custom headers fails without preflight approval',
    'sw-cors.js');

backgroundFetchTest(
    async (test, bgFetch) => {
      const request = new Request(kCorsResource, {credentials: 'include'});
      await bgFetch.fetch(uniqueId(), request);
      const event = await getMessageFromServiceWorker();

      assert_equals(event.type, 'backgroundfetchfail');
    },
    'Cross-origin Background Fetch with credentials mode include fails against wildcard Origin',
    'sw-cors.js');

backgroundFetchTest(
    async (test, bgFetch) => {
      const request = new Request(kCorsResource, {credentials: 'omit'});
      await bgFetch.fetch(uniqueId(), request);
      const event = await getMessageFromServiceWorker();

      assert_equals(event.type, 'backgroundfetchsuccess');
    },
    'Cross-origin Background Fetch with credentials mode omit succeeds against wildcard Origin',
    'sw-cors.js');
