// META: script=/service-workers/service-worker/resources/test-helpers.sub.js
// META: script=resources/utils.js
'use strict';

// Tests that requests to bad ports are blocked.
// https://fetch.spec.whatwg.org/#port-blocking

// This is not a comprehensive test of blocked ports - it is just intended to
// check that blocking is enabled.

backgroundFetchTest((t, bgFetch) => {
  return bgFetch.fetch(uniqueId(), 'https://example.com');
}, 'fetch to default https port should register ok');

backgroundFetchTest((t, bgFetch) => {
  return bgFetch.fetch(uniqueId(), 'http://127.0.0.1');
}, 'fetch to default http port should register ok');

backgroundFetchTest((t, bgFetch) => {
  return bgFetch.fetch(uniqueId(), 'https://example.com:443');
}, 'fetch to port 443 should register ok');

backgroundFetchTest((t, bgFetch) => {
  return bgFetch.fetch(uniqueId(), 'https://example.com:80');
}, 'fetch to port 80 should register ok, even over https');

backgroundFetchTest((t, bgFetch) => {
  return bgFetch.fetch(uniqueId(), 'https://example.com:8080');
}, 'fetch to non-default non-bad port (8080) should register ok');

backgroundFetchTest((t, bgFetch) => {
  return promise_rejects(
      t, new TypeError(),
      bgFetch.fetch(uniqueId(), 'https://example.com:587'));
}, 'fetch to bad port (SMTP) should reject');
