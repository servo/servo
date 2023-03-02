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
  return bgFetch.fetch(uniqueId(), 'https://example.com');
}, 'https: fetch should register ok');

backgroundFetchTest((t, bgFetch) => {
  return bgFetch.fetch(uniqueId(), 'http://127.0.0.1');
}, 'loopback IPv4 http: fetch should register ok');

backgroundFetchTest((t, bgFetch) => {
  return bgFetch.fetch(uniqueId(), 'http://[::1]');
}, 'loopback IPv6 http: fetch should register ok');

backgroundFetchTest((t, bgFetch) => {
  return bgFetch.fetch(uniqueId(), 'http://localhost');
}, 'localhost http: fetch should register ok');

function testBgFetch(bgFetch, url)
{
  return bgFetch.fetch(uniqueId(), url).then(fetch => {
    return fetch.match(url);
  }).then(match => match.responseReady);
}

backgroundFetchTest((t, bgFetch) => {
  return promise_rejects_js(t, TypeError,
                         testBgFetch(bgFetch, 'wss:127.0.0.1'));
}, 'wss: fetch should reject');

backgroundFetchTest((t, bgFetch) => {
  return promise_rejects_js(t, TypeError,
                         testBgFetch(bgFetch, 'file:///'));
}, 'file: fetch should reject');

backgroundFetchTest((t, bgFetch) => {
  return promise_rejects_js(t, TypeError,
                         testBgFetch(bgFetch, 'data:text/plain,foo'));
}, 'data: fetch should reject');

backgroundFetchTest((t, bgFetch) => {
  return promise_rejects_js(t, TypeError,
                         testBgFetch(bgFetch, 'foobar:bazqux'));
}, 'unknown scheme fetch should reject');
