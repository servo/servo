// META: script=helpers.js
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
'use strict';

const {testPrefix, topLevelDocument} = processQueryParams();

// Common tests to run in all frames.
promise_test(async () => {
  assert_not_equals(document.hasStorageAccess, undefined);
}, "[" + testPrefix + "] document.hasStorageAccess() should exist on the document interface");

promise_test(async () => {
  await MaybeSetStorageAccess("*", "*", "blocked");
  const hasAccess = await document.hasStorageAccess();
  if (topLevelDocument || testPrefix.includes('same-origin')) {
    assert_true(hasAccess, "Access should be granted in top-level frame or iframe that is in first-party context by default.");
  } else if (testPrefix == 'ABA') {
    assert_false(hasAccess, "Access should not be granted in secure same-origin iframe that is in a third-party context by default.");
  } else {
    assert_false(hasAccess, "Access should not be granted in secure cross-origin iframes.");
  }
}, "[" + testPrefix + "] document.hasStorageAccess() should not be allowed by default unless in top-level frame or same-origin iframe.");

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
  RunTestsInIFrame("resources/hasStorageAccess-iframe.https.html?testCase=same-origin-frame");

  // Create a test with a single-child cross-site iframe.
  RunTestsInIFrame("https://{{hosts[alt][]}}:{{ports[https][0]}}/storage-access-api/resources/hasStorageAccess-iframe.https.html?testCase=cross-site-frame");

  // Validate the nested-iframe scenario where the same-origin frame containing
  // the tests is not the first child.
  RunTestsInNestedIFrame("resources/hasStorageAccess-iframe.https.html?testCase=nested-same-origin-frame");

  // Validate the nested-iframe scenario where the cross-site frame containing
  //  the tests is not the first child.
  RunTestsInNestedIFrame("https://{{hosts[alt][]}}:{{ports[https][0]}}/storage-access-api/resources/hasStorageAccess-iframe.https.html?testCase=nested-cross-site-frame");
}
