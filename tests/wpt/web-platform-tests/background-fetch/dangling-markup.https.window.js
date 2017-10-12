// META: script=/service-workers/service-worker/resources/test-helpers.sub.js
// META: script=resources/utils.js
'use strict';

// "If request's url's potentially-dangling-markup flag is set, and request's
// url's scheme is an HTTP(S) scheme, then set response to a network error."
// https://github.com/whatwg/fetch/pull/519
// https://github.com/whatwg/fetch/issues/546

// This is not a comprehensive test of dangling markup detection - it is just
// intended to check that detection is enabled.

backgroundFetchTest((t, bgFetch) => {
  return promise_rejects(
      t, new TypeError(),
      bgFetch.fetch(uniqueTag(), 'https://example.com/?\n<'));
}, 'fetch to URL containing \\n and < should reject');
