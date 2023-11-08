// META: script=helpers.js
// META: script=/cookies/resources/cookie-helper.sub.js
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
promise_test(async () => {
  assert_not_equals(document.requestStorageAccess, undefined);
}, "[" + testPrefix + "] document.requestStorageAccess() should exist on the document interface");

// Most tests need to start with the feature in "prompt" state.
async function CommonSetup() {
  await test_driver.set_permission({ name: 'storage-access' }, 'prompt');
}

promise_test(
    async t => {
      await CommonSetup();
      if (topLevelDocument || !testPrefix.includes('cross-site') ||
          testPrefix.includes('ABA')) {
        await document.requestStorageAccess().catch(t.unreached_func(
            'document.requestStorageAccess() call should resolve in top-level frame or same-site iframe.'));

        assert_true(await CanAccessCookiesViaHTTP(), 'After obtaining storage access, subresource requests from the frame should send and set cookies.');
        assert_true(CanAccessCookiesViaJS(), 'After obtaining storage access, scripts in the frame should be able to access cookies.');
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
      await CommonSetup();
      await MaybeSetStorageAccess("*", "*", "blocked");
      await test_driver.set_permission({name: 'storage-access'}, 'granted');
      t.add_cleanup(async () => {
        await test_driver.delete_all_cookies();
      });

      await document.requestStorageAccess();

      assert_true(await CanAccessCookiesViaHTTP(), 'After obtaining storage access, subresource requests from the frame should send and set cookies.');
      assert_true(CanAccessCookiesViaJS(), 'After obtaining storage access, scripts in the frame should be able to access cookies.');
    },
    '[' + testPrefix +
        '] document.requestStorageAccess() should be resolved with no user gesture when a permission grant exists, and ' +
        'should allow cookie access');

if (testPrefix.includes('cross-site')) {
  promise_test(
      async t => {
        await test_driver.set_permission(
            {name: 'storage-access'}, 'denied');

        await RunCallbackWithGesture(() => {
          return promise_rejects_dom(t, "NotAllowedError", document.requestStorageAccess(),
            "document.requestStorageAccess() call with denied permission");
        });
      },
      '[' + testPrefix +
          '] document.requestStorageAccess() should be rejected with a NotAllowedError with denied permission');
} else {
  promise_test(
      async () => {
        await CommonSetup();
        await document.requestStorageAccess();

        assert_true(await CanAccessCookiesViaHTTP(), 'After obtaining storage access, subresource requests from the frame should send and set cookies.');
        assert_true(CanAccessCookiesViaJS(), 'After obtaining storage access, scripts in the frame should be able to access cookies.');
      },
      `[${testPrefix}] document.requestStorageAccess() should resolve without permission grant or user gesture`);

  promise_test(
      async () => {
        await test_driver.set_permission(
            {name: 'storage-access'}, 'denied');

        await document.requestStorageAccess();

        assert_true(await CanAccessCookiesViaHTTP(), 'After obtaining storage access, subresource requests from the frame should send and set cookies.');
        assert_true(CanAccessCookiesViaJS(), 'After obtaining storage access, scripts in the frame should be able to access cookies.');
      },
      `[${testPrefix}] document.requestStorageAccess() should resolve with denied permission`);
}
