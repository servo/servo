// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js

'use strict';

// Spec: https://explainers-by-googlers.github.io/partitioned-popins/
// Step 1 - Call `window.popinContextTypesSupported()` and receive an empty array.

async_test(t => {
  // Step 1
  assert_array_equals(window.popinContextTypesSupported(), []);
  t.done();
}, "Verify no PopinContextType is supported on an insecure page");
