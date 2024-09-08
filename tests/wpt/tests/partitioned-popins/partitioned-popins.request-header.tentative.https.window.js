// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js

'use strict';

// Spec: https://explainers-by-googlers.github.io/partitioned-popins/
// Step 1 (window) Set up listener to resolve messages as they come in.
// Step 2 (window) Open popin.
// Step 3 (popin) Record header and do HTTP redirect.
// Step 4 (popin) Record header and do JS redirect.
// Step 5 (popin) Record header.
// Step 6 (popin) Do fetch.
// Step 7 (popin-fetch) Record header.
// Step 8 (popin) Open iframe.
// Step 9 (popin-iframe) Record header and send message.
// Step 10 (window) Cleanup.

async_test(t => {
  // Step 1
  window.addEventListener("message", t.step_func(e => {
    switch (e.data.type) {
      case 'popin':
        // Step 10
        assert_equals(e.data.message, "Initial(partitioned)-HTTP(partitioned)-JS(partitioned)-fetch(missing)-iframe(missing)-");
        t.done();
        break;
    }
  }));

  // Step 2
  window.open("/partitioned-popins/resources/partitioned-popins.request-header.initial.py", '_blank', 'popin');
}, "Verify Request Header seen on all popin navigations/redirects");
