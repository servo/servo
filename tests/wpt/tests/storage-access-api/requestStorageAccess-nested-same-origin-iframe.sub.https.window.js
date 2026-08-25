// META: script=helpers.js
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
'use strict';

// Validate the nested-iframe scenario where the same-origin frame
// containing the tests is not the first child.
RunTestsInNestedIFrame('resources/requestStorageAccess-iframe.https.html?testCase=nested-same-origin-frame');
