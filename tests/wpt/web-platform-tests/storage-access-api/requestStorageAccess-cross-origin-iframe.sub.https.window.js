// META: script=helpers.js
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
'use strict';

(async function() {
  // Set up storage access rules
  try {
    await test_driver.set_storage_access("https://{{domains[www]}}:{{ports[https][0]}}/", "*", "blocked");
  } catch (e) {
    // Ignore, can be unimplemented if the platform blocks cross-site cookies
    // by default. If this failed without default blocking we'll notice it later
    // in the test.
  }

  // Create a test with a single-child cross-origin iframe.
  RunTestsInIFrame('https://{{domains[www]}}:{{ports[https][0]}}/storage-access-api/resources/requestStorageAccess-iframe.https.html?testCase=cross-origin-frame&rootdocument=false');
})();
