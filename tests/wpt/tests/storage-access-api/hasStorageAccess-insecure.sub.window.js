// META: script=helpers.js
'use strict';

const {testPrefix, topLevelDocument} = processQueryParams();

// Common tests to run in all frames.
promise_test(async () => {
  assert_not_equals(document.hasStorageAccess, undefined);
}, "[" + testPrefix + "] document.hasStorageAccess() should be supported on the document interface");

promise_test(async () => {
  const hasAccess = await document.hasStorageAccess();
  assert_false(hasAccess, "Access should be disallowed in insecure contexts");
}, "[" + testPrefix + "] document.hasStorageAccess() should be disallowed in insecure contexts");

promise_test(async (t) => {
  const description = "Promise should reject when called on a generated document not part of the DOM.";
  const createdDocument = document.implementation.createDocument("", null);

  // Can't use `promise_rejects_dom` here, since the error comes from the wrong global.
  await createdDocument.hasStorageAccess().then(
    t.unreached_func("Should have rejected: " + description), (e) => {
      assert_equals(e.name, 'InvalidStateError', description);
    });
}, "[" + testPrefix + "] document.hasStorageAccess() should reject in a document that isn't fully active.");

// Logic to load test cases within combinations of iFrames.
if (topLevelDocument) {
  // This specific test will run only as a top level test (not as a worker).
  // Specific hasStorageAccess() scenarios will be tested within the context
  // of various iFrames

  // Create a test with a single-child same-origin iframe.
  RunTestsInIFrame("resources/hasStorageAccess-iframe.html?testCase=same-origin-frame");

  // Create a test with a single-child cross-origin iframe.
  RunTestsInIFrame("http://{{domains[www]}}:{{ports[http][0]}}/storage-access-api/resources/hasStorageAccess-iframe.html?testCase=cross-origin-frame");

  // Validate the nested-iframe scenario where the same-origin frame containing
  // the tests is not the first child.
  RunTestsInNestedIFrame("resources/hasStorageAccess-iframe.html?testCase=nested-same-origin-frame");

  // Validate the nested-iframe scenario where the cross-origin frame containing
  //  the tests is not the first child.
  RunTestsInNestedIFrame("http://{{domains[www]}}:{{ports[http][0]}}/storage-access-api/resources/hasStorageAccess-iframe.html?testCase=nested-cross-origin-frame");
}
