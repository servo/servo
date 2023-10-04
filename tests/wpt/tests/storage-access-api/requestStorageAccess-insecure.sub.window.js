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

// Common tests to run in all frames.
promise_test(async () => {
  assert_not_equals(document.requestStorageAccess, undefined);
}, "[" + testPrefix + "] document.requestStorageAccess() should exist on the document interface");

promise_test(t => {
  return promise_rejects_dom(t, "NotAllowedError", document.requestStorageAccess(),
    "document.requestStorageAccess() call without user gesture");
}, "[" + testPrefix + "] document.requestStorageAccess() should be rejected in insecure context");

// Logic to load test cases within combinations of iFrames.
if (topLevelDocument) {
  // This specific test will run only as a top level test (not as a worker).
  // Specific requestStorageAccess() scenarios will be tested within the context
  // of various iFrames
  promise_test(t => {
    const description = "document.requestStorageAccess() call in a detached frame";
    // Can't use `promise_rejects_dom` here, since the error comes from the wrong global.
    return CreateDetachedFrame().requestStorageAccess()
      .then(t.unreached_func("Should have rejected: " + description), (e) => {
        assert_equals(e.name, 'InvalidStateError', description);
        t.done();
      });
  }, "[non-fully-active] document.requestStorageAccess() should reject when run in a detached frame");

  promise_test(t => {
    return promise_rejects_dom(t, 'InvalidStateError', CreateDocumentViaDOMParser().requestStorageAccess(),
     "document.requestStorageAccess() in a detached DOMParser result");
  }, "[non-fully-active] document.requestStorageAccess() should reject when run in a detached DOMParser document");

  // Create a test with a single-child same-origin iframe.
  const sameOriginFramePromise = RunTestsInIFrame(
      'resources/requestStorageAccess-iframe.html?testCase=same-origin-frame');

  // Create a test with a single-child cross-origin iframe.
  const crossOriginFramePromise = RunTestsInIFrame(
      'http://{{domains[www]}}:{{ports[http][0]}}/storage-access-api/resources/requestStorageAccess-iframe.html?testCase=cross-origin-frame');

  // Validate the nested-iframe scenario where the same-origin frame
  // containing the tests is not the first child.
  const nestedSameOriginFramePromise = RunTestsInNestedIFrame(
      'resources/requestStorageAccess-iframe.html?testCase=nested-same-origin-frame');

  // Validate the nested-iframe scenario where the cross-origin frame
  // containing the tests is not the first child.
  const nestedCrossOriginFramePromise = RunTestsInNestedIFrame(
      'http://{{domains[www]}}:{{ports[http][0]}}/storage-access-api/resources/requestStorageAccess-iframe.html?testCase=nested-cross-origin-frame');

  // Because the iframe tests expect no user activation, and because they
  // load asynchronously, we want to first run those tests before simulating
  // clicks on the page.
  const testsWithoutUserActivation = [
    sameOriginFramePromise,
    crossOriginFramePromise,
    nestedSameOriginFramePromise,
    nestedCrossOriginFramePromise,
  ];

  promise_test(async t => {
    await Promise .all(testsWithoutUserActivation);
    await RunCallbackWithGesture(() => {
      return promise_rejects_dom(t, "NotAllowedError", document.requestStorageAccess(),
      "should reject in insecure context");
    });
  },
  '[' + testPrefix +
      '] document.requestStorageAccess() should be rejected when called with a user gesture in insecure context');
}
