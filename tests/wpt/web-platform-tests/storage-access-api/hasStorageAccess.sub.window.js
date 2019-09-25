// META: script=helpers.js
'use strict';

// Unless overridden by a query string we expect access to be granted. This lets
// us re-use this test file within various iframes of differing origin.
let expectAccessAllowed = true;

// Prefix each test case with an indicator so we know what context they are run in
// if they are used in multiple iframes.
let testPrefix = "top-level-context";

// Keep track of if we run these tests in a nested context, we don't want to
// recurse forever.
let topLevelDocument = true;

// Check if we were called with a query string of allowed=false. This would
// indicate we expect the access to be denied.
let queryParams = window.location.search.substring(1).split("&");
queryParams.forEach(function (param, index) {
  if (param.toLowerCase() == "allowed=false") {
    expectAccessAllowed = false;
  } else if (param.toLowerCase() == "rootdocument=false") {
    topLevelDocument = false;
  } else if (param.split("=")[0].toLowerCase() == "testcase") {
    testPrefix = param.split("=")[1];
  }
});

// Common tests to run in all frames.
test(() => {
  assert_not_equals(document.hasStorageAccess, undefined);
}, "[" + testPrefix + "] document.hasStorageAccess() should be supported on the document interface");

promise_test(() => {
  return document.hasStorageAccess().then(hasAccess => {
    assert_equals(hasAccess, expectAccessAllowed, "Access should be granted by default: " + expectAccessAllowed);
  });
}, "[" + testPrefix + "] document.hasStorageAccess() should be allowed by default: " + expectAccessAllowed);

// Logic to load test cases within combinations of iFrames.
if (topLevelDocument) {
  // This specific test will run only as a top level test (not as a worker).
  // Specific hasStorageAccess() scenarios will be tested within the context
  // of various iFrames

  // Create a test with a single-child same-origin iframe.
  RunTestsInIFrame("hasStorageAccess.sub.window.html?testCase=same-origin-frame&rootdocument=false");

  // Create a test with a single-child cross-origin iframe.
  RunTestsInIFrame("http://{{domains[www]}}:{{ports[http][0]}}/storage-access-api/hasStorageAccess.sub.window.html?testCase=cross-origin-frame&allowed=false&rootdocument=false");

  // Validate the nested-iframe scenario where the same-origin frame containing
  // the tests is not the first child.
  RunTestsInNestedIFrame("hasStorageAccess.sub.window.html?testCase=nested-same-origin-frame&rootdocument=false");

  // Validate the nested-iframe scenario where the cross-origin frame containing
  //  the tests is not the first child.
  RunTestsInNestedIFrame("http://{{domains[www]}}:{{ports[http][0]}}/storage-access-api/hasStorageAccess.sub.window.html?testCase=nested-cross-origin-frame&allowed=false&rootdocument=false");

  // Run tests specific to the top-level window only here. They won't get re-run inside of various iframes.
  promise_test(() => {
    let createdDocument = document.implementation.createDocument("", null);

    return createdDocument.hasStorageAccess().then(hasAccess => {
      assert_false(hasAccess, "Access should be denied to a generated document not part of the DOM.");
    });
  }, "[" + testPrefix + "] document.hasStorageAccess() should work on a document object.");
}
