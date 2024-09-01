// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js

'use strict';

// Spec: https://explainers-by-googlers.github.io/partitioned-popins/
// Step 1 (window) Set up listener to resolve messages as they come in.
// Step 2 (window) Open popin.
// Step 3 (popin) Try to open popin and report failure.
// Step 4 (main-window) Cleanup.

async_test(t => {
  // Step 1
  window.addEventListener("message", t.step_func(e => {
    switch (e.data.type) {
      case 'popin':
        // Step 4
        assert_equals(e.data.message, "Could not open inner popin");
        t.done();
        break;
    }
  }));

  // Step 2
  window.open("/partitioned-popins/resources/partitioned-popins.recursive.html", '_blank', 'popin');
}, "Verify Partitioned Popins cannot open their own popin");
