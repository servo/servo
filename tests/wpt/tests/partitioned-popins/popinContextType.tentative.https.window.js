// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js

'use strict';

// Spec: https://explainers-by-googlers.github.io/partitioned-popins/
// Step 1 - Call `window.popinContextType()` and receive null.

async_test(t => {
  assert_equals(window.popinContextType(), null);
  t.done();
}, "Verify PopinContextType is null on top-level page");
