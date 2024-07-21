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

promise_test(async () => {
  assert_not_equals(document.requestStorageAccess, undefined);
}, `[${testPrefix}] document.requestStorageAccess() should exist on the document interface`);

// Skip these tests when we're in a top-level document; these should only
// execute inside the iframe test defined by
// requestStorageAccess-sandboxed-iframe-*.sub.https.window.js
if (!topLevelDocument) {
  if (testPrefix.includes('allow-storage-access-by-user-activation')) {
    // Ideally this would check whether the user-activation condition changes
    // the behavior; however, due to limitations in the test driver, the
    // 'prompt' permission state is effectively the same as 'denied' from the
    // perspective of platform tests.
    promise_test(async t => {
      await test_driver.set_permission({name: 'storage-access'}, 'granted');
      await MaybeSetStorageAccess('*', '*', 'blocked');
      await document.requestStorageAccess();

      assert_true(
          await CanAccessCookiesViaHTTP(),
          'After obtaining storage access, subresource requests from the frame should send and set cookies.');
      assert_true(
          CanAccessCookiesViaJS(),
          'After obtaining storage access, scripts in the frame should be able to access cookies.');
    }, `[${testPrefix}] document.requestStorageAccess() should resolve even without a user gesture when already granted.`);

    promise_test(async () => {
      await test_driver.set_permission({ name: 'storage-access' }, 'granted');
      await MaybeSetStorageAccess('*', '*', 'blocked');

      await RunCallbackWithGesture(async () => {
        await document.requestStorageAccess();
      });

      assert_true(
        await CanAccessCookiesViaHTTP(),
        'After obtaining storage access, subresource requests from the frame should send and set cookies.');
      assert_true(
        CanAccessCookiesViaJS(),
        'After obtaining storage access, scripts in the frame should be able to access cookies.');
    }, `[${testPrefix}] document.requestStorageAccess() should resolve with a user gesture`);
  } else {
    // For cases where allow-storage-access-by-user-activation is not set for
    // this iframe
    promise_test(
        async t => {
          await test_driver.set_permission({name: 'storage-access'}, 'granted');
          await MaybeSetStorageAccess('*', '*', 'blocked');
          return promise_rejects_dom(
              t, 'NotAllowedError', document.requestStorageAccess(),
              'document.requestStorageAccess() call without user gesture.');
        },
        '[' + testPrefix +
            '] document.requestStorageAccess() should reject with a NotAllowedError with no user gesture.');

    promise_test(async t => {
      await test_driver.set_permission({name: 'storage-access'}, 'granted');
      await MaybeSetStorageAccess('*', '*', 'blocked');

      await RunCallbackWithGesture(async () => {
        await promise_rejects_dom(
            t, 'NotAllowedError', document.requestStorageAccess(),
            'document.requestStorageAccess() call with user gesture.');
      });
    }, `[${testPrefix}] document.requestStorageAccess() should reject with a NotAllowedError, even with a user gesture`);
  }
}
