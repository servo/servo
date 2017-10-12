// META: script=/service-workers/service-worker/resources/test-helpers.sub.js
// META: script=resources/utils.js
'use strict';

// Tests that Mixed Content requests are blocked.
// https://w3c.github.io/webappsec-mixed-content/#should-block-fetch
// https://w3c.github.io/webappsec-mixed-content/#a-priori-authenticated-url
// https://w3c.github.io/webappsec-secure-contexts/#is-origin-trustworthy

// With an additional restriction that only https:// and loopback http://
// requests are allowed. Hence the wss:, file:, data:, etc schemes are blocked.
// https://github.com/WICG/background-fetch/issues/44

// This is not a comprehensive test of mixed content blocking - it is just
// intended to check that blocking is enabled.

backgroundFetchTest((t, bgFetch) => {
  return bgFetch.fetch(uniqueTag(), 'https://example.com');
}, 'https: fetch should register ok');

backgroundFetchTest((t, bgFetch) => {
  return bgFetch.fetch(uniqueTag(), 'http://127.0.0.1');
}, 'loopback IPv4 http: fetch should register ok');

backgroundFetchTest((t, bgFetch) => {
  return bgFetch.fetch(uniqueTag(), 'http://[::1]');
}, 'loopback IPv6 http: fetch should register ok');

// http://localhost is not tested here since the correct behavior from
// https://w3c.github.io/webappsec-secure-contexts/#is-origin-trustworthy
// depends on whether the UA conforms to the name resolution rules in
// https://tools.ietf.org/html/draft-west-let-localhost-be-localhost

backgroundFetchTest((t, bgFetch) => {
  return promise_rejects(t, new TypeError(),
                         bgFetch.fetch(uniqueTag(), 'http://example.com'));
}, 'non-loopback http: fetch should reject');

backgroundFetchTest((t, bgFetch) => {
  return promise_rejects(t, new TypeError(),
                         bgFetch.fetch(uniqueTag(), 'http://192.0.2.0'));
}, 'non-loopback IPv4 http: fetch should reject');

backgroundFetchTest((t, bgFetch) => {
  return promise_rejects(t, new TypeError(),
                         bgFetch.fetch(uniqueTag(), 'http://[2001:db8::1]'));
}, 'non-loopback IPv6 http: fetch should reject');

backgroundFetchTest((t, bgFetch) => {
  return promise_rejects(t, new TypeError(),
                         bgFetch.fetch(uniqueTag(), ['https://example.com',
                                                     'http://example.com']));
}, 'https: and non-loopback http: fetch should reject');

backgroundFetchTest((t, bgFetch) => {
  return promise_rejects(t, new TypeError(),
                         bgFetch.fetch(uniqueTag(), ['http://example.com',
                                                     'https://example.com']));
}, 'non-loopback http: and https: fetch should reject');


backgroundFetchTest((t, bgFetch) => {
  return promise_rejects(t, new TypeError(),
                         bgFetch.fetch(uniqueTag(), 'wss:127.0.0.1'));
}, 'wss: fetch should reject');

backgroundFetchTest((t, bgFetch) => {
  return promise_rejects(t, new TypeError(),
                         bgFetch.fetch(uniqueTag(), 'file:///'));
}, 'file: fetch should reject');

backgroundFetchTest((t, bgFetch) => {
  return promise_rejects(t, new TypeError(),
                         bgFetch.fetch(uniqueTag(), 'data:text/plain,foo'));
}, 'data: fetch should reject');

backgroundFetchTest((t, bgFetch) => {
  return promise_rejects(t, new TypeError(),
                         bgFetch.fetch(uniqueTag(), 'foobar:bazqux'));
}, 'unknown scheme fetch should reject');
