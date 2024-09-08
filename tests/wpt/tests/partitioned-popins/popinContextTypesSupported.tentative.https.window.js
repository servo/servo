// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js

'use strict';

// Spec: https://explainers-by-googlers.github.io/partitioned-popins/
// Step 1 - Call `window.popinContextTypesSupported()` and receive an empty array.

async_test(t => {
  assert_array_equals(window.popinContextTypesSupported(), ["partitioned"]);
  t.done();
}, "Verify 'partitioned' PopinContextType is supported on a secure page");
