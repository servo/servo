// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js

'use strict';

// Spec: https://explainers-by-googlers.github.io/partitioned-popins/
// Step 1 - Open a popin and succeed.
// Step 2 - Open a second popin and fail.
// Step 3 - Cleanup.

async_test(t => {
  let message = "";

  // Step 1
  try {
    let popin_1 = window.open("/partitioned-popins/resources/partitioned-popins.wait.html", '_blank', 'popin');
    message += popin_1 ? "FirstPopinOpened-" : "";
  } catch (_) {}

  // Step 2
  try {
    let popin_2 = window.open("/partitioned-popins/resources/partitioned-popins.wait.html", '_blank', 'popin');
    message += popin_2 ? "SecondPopinOpened-" : "";
  } catch (_) {}

  // Step 3
  assert_equals(message, "FirstPopinOpened-");
  t.done();
}, "Verify only one partitioned popin can be open at a time.");
