// META: script=/service-workers/service-worker/resources/test-helpers.sub.js
// META: script=resources/utils.js
'use strict';

// "If parsedURL includes credentials, then throw a TypeError."
// https://fetch.spec.whatwg.org/#dom-request
// (Added by https://github.com/whatwg/fetch/issues/26).
// "A URL includes credentials if its username or password is not the empty
// string."
// https://url.spec.whatwg.org/#include-credentials

backgroundFetchTest((t, bgFetch) => {
  return bgFetch.fetch(uniqueTag(), 'https://example.com');
}, 'fetch without credentials in URL should register ok');

backgroundFetchTest((t, bgFetch) => {
  return promise_rejects(
      t, new TypeError(),
      bgFetch.fetch(uniqueTag(), 'https://username:password@example.com'));
}, 'fetch with username and password in URL should reject');

backgroundFetchTest((t, bgFetch) => {
  return promise_rejects(
      t, new TypeError(),
      bgFetch.fetch(uniqueTag(), 'https://username:@example.com'));
}, 'fetch with username and empty password in URL should reject');

backgroundFetchTest((t, bgFetch) => {
  return promise_rejects(
      t, new TypeError(),
      bgFetch.fetch(uniqueTag(), 'https://:password@example.com'));
}, 'fetch with empty username and password in URL should reject');
