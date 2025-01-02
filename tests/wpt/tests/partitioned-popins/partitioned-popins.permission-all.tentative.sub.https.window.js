// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js

'use strict';

// Spec: https://explainers-by-googlers.github.io/partitioned-popins/
// Step 1 (window) Set up listener to resolve messages as they come in.
// Step 2 (window) Open iframe for other origin.
// Step 3 (iframe) Open partitioned popin.
// Step 4 (popin) Cleanup.
// Step 5 (iframe) Report success.
// Step 6 (window) Cleanup.

async_test(t => {
  const id = String(Date.now());
  // Step 1
  window.addEventListener("message", t.step_func(e => {
    switch (e.data.type) {
      case 'popin':
        // Step 6
        assert_equals(e.data.message, "Success");
        t.done();
        break;
    }
  }));

  // Step 2
  const iframe = document.createElement("iframe");
  iframe.allow = "popins";
  iframe.src = "https://{{hosts[alt][]}}:{{ports[https][0]}}/partitioned-popins/resources/partitioned-popins.permissions-iframe.html";
  document.body.appendChild(iframe);
}, "Verify Partitioned Popins in an iframe work when the policy is *");
