// META: script=../helpers.js
'use strict';

// This expects to be run in an iframe that is cross-site to the top-level
// frame.
(async function() {
  // Create a test with a single-child iframe that is same-site to the top-level
  // frame but cross-site to the iframe that is being created here, for the
  // purpose of testing requestStorageAccess in an A(B(A)) frame tree setting.
  RunTestsInIFrame(
      'https://{{domains[www]}}:{{ports[https][0]}}/storage-access-api/resources/requestStorageAccess-iframe.https.html?testCase=ABA');
})();
