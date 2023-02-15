// META: script=/storage-access-api/helpers.js
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
'use strict';

// Note that this file follows the pattern in:
// storage-access-api/requestStorageAccess.sub.window.js
//
// Some tests are run at the top-level, and an iframe is added to validate API
// behavior in that context.

// Prefix each test case with an indicator so we know what context they are run
// in if they are used in multiple iframes.
let testPrefix = 'top-level-context';

// Keep track of if we run these tests in a nested context, we don't want to
// recurse forever.
let topLevelDocument = true;

// The query string allows derivation of test conditions, like whether the tests
// are running in a top-level context.
const queryParams = window.location.search.substring(1).split('&');
queryParams.forEach((param) => {
  if (param.toLowerCase() == 'rootdocument=false') {
    topLevelDocument = false;
  } else if (param.split('=')[0].toLowerCase() == 'testcase') {
    testPrefix = param.split('=')[1];
  }
});

// TODO(crbug.com/1351540): when/if requestStorageAccessForOrigin is standardized,
// upstream with the Storage Access API helpers file.
function RunRequestStorageAccessForOriginInDetachedFrame(site) {
  const nestedFrame = document.createElement('iframe');
  document.body.append(nestedFrame);
  const inner_doc = nestedFrame.contentDocument;
  nestedFrame.remove();
  return inner_doc.requestStorageAccessForOrigin(site);
}

function RunRequestStorageAccessForOriginViaDomParser(site) {
  const parser = new DOMParser();
  const doc = parser.parseFromString('<html></html>', 'text/html');
  return doc.requestStorageAccessForOrigin(site);
}

// Common tests to run in all frames.
test(
    () => {
      assert_not_equals(document.requestStorageAccessForOrigin, undefined);
    },
    '[' + testPrefix +
        '] document.requestStorageAccessForOrigin() should be supported on the document interface');

promise_test(
  t => {
    return promise_rejects_js(t, TypeError,
      document.requestStorageAccessForOrigin(),
      'document.requestStorageAccessForOrigin() call without origin argument');
  },
  '[' + testPrefix +
      '] document.requestStorageAccessForOrigin() should be rejected when called with no argument');

if (topLevelDocument) {
  promise_test(
      t => {
        return promise_rejects_dom(t, 'NotAllowedError',
          document.requestStorageAccessForOrigin('https://test.com'),
         'document.requestStorageAccessForOrigin() call without user gesture');
      },
      '[' + testPrefix +
          '] document.requestStorageAccessForOrigin() should be rejected by default with no user gesture');

  promise_test(async t => {
    const description =
        'document.requestStorageAccessForOrigin() call in a detached frame';
    // Can't use promise_rejects_dom here because the exception is from the wrong global.
    return RunRequestStorageAccessForOriginInDetachedFrame('https://foo.com')
        .then(t.unreached_func('Should have rejected: ' + description))
        .catch((e) => {
          assert_equals(e.name, 'InvalidStateError', description);
        });
  }, '[non-fully-active] document.requestStorageAccessForOrigin() should not resolve when run in a detached frame');

  promise_test(async t => {
    const description =
        'document.requestStorageAccessForOrigin() in a detached DOMParser result';
    return RunRequestStorageAccessForOriginViaDomParser('https://foo.com')
        .then(t.unreached_func('Should have rejected: ' + description))
        .catch((e) => {
          assert_equals(e.name, 'InvalidStateError', description);
        });
  }, '[non-fully-active] document.requestStorageAccessForOrigin() should not resolve when run in a detached DOMParser document');

  // Create a test with a single-child same-origin iframe.
  // This will validate that calls to requestStorageAccessForOrigin are rejected
  // in non-top-level contexts.
  RunTestsInIFrame(
      './resources/requestStorageAccessForOrigin-iframe.https.html?testCase=same-origin-frame&rootdocument=false');

  promise_test(
      async t => {
        await RunCallbackWithGesture(
          () => document.requestStorageAccessForOrigin(document.location.origin));
      },
      '[' + testPrefix +
          '] document.requestStorageAccessForOrigin() should be resolved when called properly with a user gesture and the same site');

  promise_test(
      async t => {
        await RunCallbackWithGesture(
          () => promise_rejects_dom(t, 'NotAllowedError', document.requestStorageAccessForOrigin('bogus-url'),
            'document.requestStorageAccessForOrigin() call with bogus URL'));
      },
      '[' + testPrefix +
          '] document.requestStorageAccessForOrigin() should be rejected when called with an invalid site');

  promise_test(
      async t => {
        await RunCallbackWithGesture(
          () => promise_rejects_dom(t, 'NotAllowedError', document.requestStorageAccessForOrigin('data:,Hello%2C%20World%21'),
            'document.requestStorageAccessForOrigin() call with data URL'));
      },
      '[' + testPrefix +
          '] document.requestStorageAccessForOrigin() should be rejected when called with an opaque origin');


  promise_test(
    async t => {
      await test_driver.set_permission(
        { name: 'top-level-storage-access', requestedOrigin: 'https://foo.com' }, 'granted');

      await RunCallbackWithGesture(
        () => document.requestStorageAccessForOrigin('https://foo.com'));
    },
    '[' + testPrefix +
    '] document.requestStorageAccessForOrigin() should be resolved when called properly with a user gesture');

} else {
  promise_test(
      async t => {
        await RunCallbackWithGesture(
          () => promise_rejects_dom(t, 'NotAllowedError', document.requestStorageAccessForOrigin(document.location.origin),
            'document.requestStorageAccessForOrigin() call in a non-top-level context'));
      },
      '[' + testPrefix +
          '] document.requestStorageAccessForOrigin() should be rejected when called in an iframe');
}
