// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js

'use strict';

// Step 1 (main window) Set up listener to resolve messages as they come in.
// Step 2 (main-window) Open window for other origin.
// Step 3 (other-window) Write first-party cookies and report success.
// Step 4 (main-window) Embed iframe for other origin.
// Step 5 (main-iframe) Write third-party cookies and report success.
// Step 6 (main-window) Open partitioned popin for other origin.
// Step 7 (main-popin) Check for first-/third-party cookies and report success.
// Step 8 (main-window) Cleanup.

async_test(t => {
  const id = String(Math.random());
  // Step 1
  window.addEventListener("message", t.step_func(e => {
    switch (e.data.type) {
      case 'window-set':
        // Step 4
        assert_equals(e.data.message, "Set first-party data");
        const iframe = document.createElement("iframe");
        iframe.src = "https://{{hosts[alt][]}}:{{ports[https][0]}}/partitioned-popins/resources/partitioned-popins.cookies-iframe.html?id="+id;
        document.body.appendChild(iframe);
        break;
      case 'iframe-set':
        // Step 6
        assert_equals(e.data.message, "Set third-party data");
        window.open("https://{{hosts[alt][]}}:{{ports[https][0]}}/partitioned-popins/resources/partitioned-popins.cookies-popin.py?id="+id, '_blank', 'popin');
        break;
      case 'popin-read':
        // Step 8
        // We want to see the same behavior a cross-site iframe would have, initially only SameSite=None, then first-party strict and lax come back for an inner fetch.
        assert_equals(e.data.message, "ReadOnLoad:FirstPartyNone-ThirdPartyNone-,ReadOnFetch:FirstPartyStrict-FirstPartyLax-FirstPartyNone-ThirdPartyNone-");
        t.done();
        break;
    }
  }));

  // Step 2
  window.open("https://{{hosts[alt][]}}:{{ports[https][0]}}/partitioned-popins/resources/partitioned-popins.cookies-window.html?id="+id, '_blank', 'popup');
}, "Verify Partitioned Popins only have access to third-party cookies");
