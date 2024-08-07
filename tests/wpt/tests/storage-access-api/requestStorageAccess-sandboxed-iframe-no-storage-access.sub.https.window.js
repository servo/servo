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

if (topLevelDocument) {
  const frameSourceUrl =
      'https://{{hosts[alt][www]}}:{{ports[https][0]}}/storage-access-api/requestStorageAccess-sandboxed-iframe-no-storage-access.sub.https.window.html';

  let sandboxAttribute = 'allow-scripts allow-same-origin';
  let testCase = 'sandboxed-iframe';

  RunTestsInIFrame(frameSourceUrl + `?testCase=${testCase}`, sandboxAttribute);
} else {
  test(() => {
    let iframe = document.createElement('iframe');
    assert_true(
        iframe.sandbox.supports('allow-storage-access-by-user-activation'),
        '`allow-storage-access-by-user-activation`' +
            'sandbox attribute should be supported');
  }, '`allow-storage-access-by-user-activation` sandbox attribute is supported');
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
