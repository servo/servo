// META: script=helpers.js
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
'use strict';

// Document-level test config flags:
//
// testPrefix: Prefix each test case with an indicator so we know what context
// they are run in if they are used in multiple iframes.
//
// topLevelDocument: Keep track of if we run these tests in a nested context, we
// don't want to recurse forever.
const {testPrefix, topLevelDocument} = processQueryParams();

if (!topLevelDocument) {
  // WPT synthesizes a top-level HTML test for this JS file, and in that case we
  // don't want to, or need to, call set_test_context.
  test_driver.set_test_context(window.top);
}

// Common tests to run in all frames.
test(() => {
  assert_not_equals(document.requestStorageAccess, undefined);
}, "[" + testPrefix + "] document.requestStorageAccess() should exist on the document interface");

// Promise tests should all start with the feature in "prompt" state.
promise_setup(async () => {
  await test_driver.set_permission(
    { name: 'storage-access' }, 'prompt');
});

promise_test(
    async t => {
      if (topLevelDocument || !testPrefix.includes('cross-site') ||
          testPrefix.includes('ABA')) {
        await document.requestStorageAccess().catch(t.unreached_func(
            'document.requestStorageAccess() call should resolve in top-level frame or same-site iframe.'));
      } else {
        return promise_rejects_dom(
            t, "NotAllowedError", document.requestStorageAccess(),
            "document.requestStorageAccess() call without user gesture.");
      }
    },
    '[' + testPrefix +
        '] document.requestStorageAccess() should resolve in top-level frame or same-site iframe, otherwise reject with a NotAllowedError with no user gesture.');

promise_test(
    async (t) => {
      await MaybeSetStorageAccess("*", "*", "blocked");
      await test_driver.set_permission({name: 'storage-access'}, 'granted');
      t.add_cleanup(async () => {
        await test_driver.delete_all_cookies();
      });

      await document.requestStorageAccess();

      await fetch(`${window.location.origin}/cookies/resources/set-cookie.py?name=cookie&path=/&samesite=None&secure=`)
          .then((resp) => resp.text());
      const httpCookies = await fetch(`${window.location.origin}/storage-access-api/resources/echo-cookie-header.py`)
          .then((resp) => resp.text());
      assert_true(httpCookies.includes('cookie=1'),
          'After obtaining storage access, subresource requests from the frame should send and set cookies.');
    },
    '[' + testPrefix +
        '] document.requestStorageAccess() should be resolved with no user gesture when a permission grant exists, and ' +
        'should allow cookie access');

if (testPrefix.includes('cross-site')) {
  promise_test(
      async t => {
        await RunCallbackWithGesture(() => {
          return promise_rejects_dom(t, "NotAllowedError", document.requestStorageAccess(),
            "document.requestStorageAccess() call without permission");
        });
      },
      '[' + testPrefix +
          '] document.requestStorageAccess() should be rejected with a NotAllowedError without permission grant');

  promise_test(
      async t => {
        await test_driver.set_permission(
            {name: 'storage-access'}, 'denied');

        await RunCallbackWithGesture(() => {
          return promise_rejects_dom(t, "NotAllowedError", document.requestStorageAccess(),
            "document.requestStorageAccess() call without permission");
        });
      },
      '[' + testPrefix +
          '] document.requestStorageAccess() should be rejected with a NotAllowedError with denied permission');
} else {
  promise_test(
      async () => {
        await document.requestStorageAccess();
      },
      `[${testPrefix}] document.requestStorageAccess() should resolve without permission grant or user gesture`);

  promise_test(
      async () => {
        await test_driver.set_permission(
            {name: 'storage-access'}, 'denied');

        await document.requestStorageAccess();
      },
      `[${testPrefix}] document.requestStorageAccess() should resolve with denied permission`);
}
