// META: script=helpers.js
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
'use strict';

(async function() {
  // Validate the nested-iframe scenario where the cross-origin frame
  // containing the tests is not the first child.
  RunTestsInNestedIFrame('https://{{domains[www]}}:{{ports[https][0]}}/storage-access-api/resources/requestStorageAccess-iframe.https.html?testCase=nested-cross-origin-frame');
})();
