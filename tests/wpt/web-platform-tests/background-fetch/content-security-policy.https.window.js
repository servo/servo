// META: script=/service-workers/service-worker/resources/test-helpers.sub.js
// META: script=resources/utils.js
'use strict';

// Tests that requests blocked by Content Security Policy are rejected.
// https://w3c.github.io/webappsec-csp/#should-block-request

// This is not a comprehensive test of Content Security Policy - it is just
// intended to check that CSP checks are enabled.

var meta = document.createElement('meta');
meta.setAttribute('http-equiv', 'Content-Security-Policy');
meta.setAttribute('content', "connect-src 'none'");
document.head.appendChild(meta);

backgroundFetchTest((t, bgFetch) => {
  return promise_rejects(
      t, new TypeError(),
      bgFetch.fetch(uniqueTag(), 'https://example.com'));
}, 'fetch blocked by CSP should reject');
