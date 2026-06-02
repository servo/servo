// META: script=../helpers.js
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
'use strict';

// This expects to be run in an iframe that is cross-site to the top-level frame.
(async function() {
  // Create a test with a single-child iframe that is same-site to the top-level frame but cross-site to the iframe
  // that is being created here, for the purpose of testing hasStorageAccess in an A(B(A)) frame tree setting.
  RunTestsInIFrame("https://{{host}}:{{ports[https][0]}}/storage-access-api/resources/hasStorageAccess-iframe.https.html?testCase=ABA");
})();
