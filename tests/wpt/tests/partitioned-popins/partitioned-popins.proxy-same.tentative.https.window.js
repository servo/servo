// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/partitioned-popins/resources/proxy-helpers.js

'use strict';

// Spec: https://explainers-by-googlers.github.io/partitioned-popins/
// Step 1 (window) Set up listener to resolve messages as they come in.
// Step 2 (window) Open same-origin popin.
// Step 3 (popin) Set up listener to resolve messages as they come in.
// Step 4 (popin) Test and report usable methods against window.
// Step 5 (window) Test and compare usable methods against popin.
// Step 6 (popin) Cleanup.
// Step 7 (window) Cleanup.

// TODO(crbug.com/340606651): Remove expectations file and secure same-origin popins.

async_test(t => {
  let popin_proxy;

  // Step 1
  window.addEventListener("message", t.step_func(e => {
    switch (e.data.type) {
      case 'ready':
        // Step 5
        assert_equals(e.data.message, "Closed,Then,");
        assert_equals(getUsableMethods(popin_proxy), "Closed,Then,");
        popin_proxy.postMessage({type: "cleanup"}, "*");
        break;
      case 'cleanup':
        // Step 7
        t.done();
        break;
    }
  }));

  // Step 2
  popin_proxy = window.open("/partitioned-popins/resources/partitioned-popins.proxy-popin.html", '_blank', 'popin');
}, "Verify same-origin Partitioned Popins proxies only have access to postMessage and closed methods.");
