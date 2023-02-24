// META: script=helpers.js
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
'use strict';

(async function() {
  // Create a test with a single-child cross-site iframe.
  RunTestsInIFrame('https://{{hosts[alt][www]}}:{{ports[https][0]}}/storage-access-api/resources/requestStorageAccess-iframe.https.html?testCase=cross-site-frame');
})();
