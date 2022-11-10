// META: script=helpers.js
'use strict';

const {expectAccessAllowed, testPrefix, topLevelDocument} = processQueryParams();

// Common tests to run in all frames.
test(() => {
  assert_not_equals(document.hasStorageAccess, undefined);
}, "[" + testPrefix + "] document.hasStorageAccess() should exist on the document interface");

promise_test(async () => {
  const hasAccess = await document.hasStorageAccess();
  assert_equals(hasAccess, expectAccessAllowed, "Access should be granted by default: " + expectAccessAllowed);
}, "[" + testPrefix + "] document.hasStorageAccess() should be allowed by default: " + expectAccessAllowed);

promise_test(async () => {
  const createdDocument = document.implementation.createDocument("", null);

  const hasAccess = await createdDocument.hasStorageAccess();
  assert_false(hasAccess, "Access should be denied to a generated document not part of the DOM.");
}, "[" + testPrefix + "] document.hasStorageAccess() should work on a document object.");

// Logic to load test cases within combinations of iFrames.
if (topLevelDocument) {
  // This specific test will run only as a top level test (not as a worker).
  // Specific hasStorageAccess() scenarios will be tested within the context
  // of various iFrames

  // Create a test with a single-child same-origin iframe.
  RunTestsInIFrame("resources/hasStorageAccess-iframe.https.html?testCase=same-origin-frame&rootdocument=false");

  // Create a test with a single-child cross-origin iframe.
  RunTestsInIFrame("https://{{domains[www]}}:{{ports[https][0]}}/storage-access-api/resources/hasStorageAccess-iframe.https.html?testCase=cross-origin-frame&rootdocument=false");

  // Validate the nested-iframe scenario where the same-origin frame containing
  // the tests is not the first child.
  RunTestsInNestedIFrame("resources/hasStorageAccess-iframe.https.html?testCase=nested-same-origin-frame&rootdocument=false");

  // Validate the nested-iframe scenario where the cross-origin frame containing
  //  the tests is not the first child.
  RunTestsInNestedIFrame("https://{{domains[www]}}:{{ports[https][0]}}/storage-access-api/resources/hasStorageAccess-iframe.https.html?testCase=nested-cross-origin-frame&rootdocument=false");
}
