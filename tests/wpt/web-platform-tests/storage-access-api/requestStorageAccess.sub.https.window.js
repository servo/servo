// META: script=helpers.js
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
'use strict';

// Document-level test config flags:
//
// testPrefix: Prefix each test case with an indicator so we know what context
// they are run in if they are used in multiple iframes.
const {testPrefix} = processQueryParams();

if (window !== window.top) {
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

promise_test(t => {
  return promise_rejects_dom(t, "NotAllowedError", document.requestStorageAccess(),
    "document.requestStorageAccess() call without user gesture");
}, "[" + testPrefix + "] document.requestStorageAccess() should be rejected with a NotAllowedError by default with no user gesture");

promise_test(
    async () => {
      await test_driver.set_permission(
          {name: 'storage-access'}, 'granted');

      await RunCallbackWithGesture(() => document.requestStorageAccess());
    },
    '[' + testPrefix +
        '] document.requestStorageAccess() should be resolved when called properly with a user gesture');

if (testPrefix == 'cross-origin-frame' || testPrefix == 'nested-cross-origin-frame') {
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
}
