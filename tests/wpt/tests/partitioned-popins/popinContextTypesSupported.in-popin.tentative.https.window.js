// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js

'use strict';

// Spec: https://explainers-by-googlers.github.io/partitioned-popins/
// Step 1 (window) Set up listener to resolve messages as they come in.
// Step 2 (window) Open popin.
// Step 3 (popin) Call `window.popinContextTypesSupported()` and send result to opener.
// Step 4 (main-window) Assert and cleanup.

async_test(t => {
  // Step 1
  window.addEventListener("message", t.step_func(e => {
    switch (e.data.type) {
      case 'popin':
        // Step 4
        assert_array_equals(e.data.message, []);
        t.done();
        break;
    }
  }));

  // Step 2
  window.open("/partitioned-popins/resources/popinContextTypesSupported.html", '_blank', 'popin');
}, "Verify no PopinContextType is supported inside of a popin");
