// META: script=helpers.js
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
'use strict';

// Create a test with a nested iframe that is same-site to the top-level frame
// but has cross-site frame in between.
RunTestsInIFrame(
    'https://{{hosts[alt][]}}:{{ports[https][0]}}/storage-access-api/resources/requestStorageAccess-ABA-iframe.https.html');
