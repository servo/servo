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
let testPrefix = 'insecure-context';

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

// TODO(crbug.com/1410556): when/if requestStorageAccessFor is standardized,
// we should consider upstreaming these helpers.
function RunRequestStorageAccessForInDetachedFrame(site) {
  const nestedFrame = document.createElement('iframe');
  document.body.append(nestedFrame);
  const inner_doc = nestedFrame.contentDocument;
  nestedFrame.remove();
  return inner_doc.requestStorageAccessFor(site);
}

function RunRequestStorageAccessForViaDomParser(site) {
  const parser = new DOMParser();
  const doc = parser.parseFromString('<html></html>', 'text/html');
  return doc.requestStorageAccessFor(site);
}

// Common tests to run in all frames.
test(
    () => {
      assert_not_equals(document.requestStorageAccessFor, undefined);
    },
    '[' + testPrefix +
        '] document.requestStorageAccessFor() should be supported on the document interface');

if (topLevelDocument) {
  promise_test(
      t => {
        return promise_rejects_dom(t, 'NotAllowedError',
          document.requestStorageAccessFor('https://test.com'),
         'document.requestStorageAccessFor() call without user gesture');
      },
      '[' + testPrefix +
          '] document.requestStorageAccessFor() should be rejected by default with no user gesture');

  promise_test(async t => {
    const description =
        'document.requestStorageAccessFor() call in a detached frame';
    // Can't use promise_rejects_dom here because the exception is from the wrong global.
    return RunRequestStorageAccessForInDetachedFrame('https://foo.com')
        .then(t.unreached_func('Should have rejected: ' + description))
        .catch((e) => {
          assert_equals(e.name, 'InvalidStateError', description);
        });
  }, '[non-fully-active] document.requestStorageAccessFor() should not resolve when run in a detached frame');

  promise_test(async t => {
    const description =
        'document.requestStorageAccessFor() in a detached DOMParser result';
    return RunRequestStorageAccessForViaDomParser('https://foo.com')
        .then(t.unreached_func('Should have rejected: ' + description))
        .catch((e) => {
          assert_equals(e.name, 'InvalidStateError', description);
        });
  }, '[non-fully-active] document.requestStorageAccessFor() should not resolve when run in a detached DOMParser document');

  // Create a test with a single-child same-origin iframe.
  // This will validate that calls to requestStorageAccessFor are rejected
  // in non-top-level contexts.
  RunTestsInIFrame(
      './resources/requestStorageAccessFor-iframe.html?testCase=frame-on-insecure-page&rootdocument=false');

  promise_test(
      async t => {
        await RunCallbackWithGesture(
          () => promise_rejects_dom(t, 'NotAllowedError', document.requestStorageAccessFor(document.location.origin), 'document.requestStorageAccessFor() call in insecure context'));
      },
      '[' + testPrefix +
          '] document.requestStorageAccessFor() should be rejected when called in an insecure context');

} else {
  promise_test(
      async t => {
        await RunCallbackWithGesture(
          () => promise_rejects_dom(t, 'NotAllowedError', document.requestStorageAccessFor(document.location.origin),
            'document.requestStorageAccessFor() call in a non-top-level context'));
      },
      '[' + testPrefix +
          '] document.requestStorageAccessFor() should be rejected when called in an iframe');
}
